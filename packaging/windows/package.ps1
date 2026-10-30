<#
.SYNOPSIS
  Build, sign and package Nexus Sheet (NexusOffice, by SonnyTech) for Windows.

.DESCRIPTION
  Produces, in $env:DIST (default: dist/release):
    NexusSheet-<version>-windows-<arch>.msi            per-machine installer (WiX v5)
    NexusSheet-<version>-windows-<arch>-portable.zip   nexus-sheet.exe + nexus-sheet-cli.exe

  The binaries link the C runtime statically (+crt-static), so neither the MSI nor the portable
  zip needs the Visual C++ redistributable. Signing is delegated to sign.ps1 (skipped with a
  warning when no signing secrets are set).

  Needs: Rust (MSVC toolchain + the target), the Windows SDK (rc.exe, signtool.exe),
  and WiX v5: dotnet tool install --global wix --version 5.0.2

.EXAMPLE
  pwsh packaging/windows/package.ps1 -Arch x64
  pwsh packaging/windows/package.ps1 -Arch x86 -SkipBuild
  pwsh packaging/windows/package.ps1 -Arch arm64     # cross-compiled; needs the MSVC ARM64 build tools
#>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The version lives in one place: [workspace.package] version in the root Cargo.toml.
$Version = $env:GRIDCRAFT_VERSION
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }
# MSI ProductVersion is numeric (major.minor.build); pre-release tags are dropped there.
$MsiVersion = ($Version -split '-')[0]

$Target = switch ($Arch) { 'x64' { 'x86_64-pc-windows-msvc' } 'x86' { 'i686-pc-windows-msvc' } 'arm64' { 'aarch64-pc-windows-msvc' } }
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (-not $env:GRIDCRAFT_BUILD_SHA) { $env:GRIDCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD 2>$null) }
if (-not $env:GRIDCRAFT_BUILD_DATE) { $env:GRIDCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }

Write-Output "Nexus Sheet $Version for Windows $Arch ($Target)"

if (-not $SkipBuild) {
  # Static CRT: no VC++ redistributable needed. Scoped to the target so host build scripts and
  # proc-macros are unaffected.
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  # Fail the build (rather than warn) if the icon/VERSIONINFO can't be embedded.
  $env:GRIDCRAFT_REQUIRE_WINRES = '1'
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p gridcraft -p gridcraft-cli --target $Target }
}

$Bin = Join-Path $TargetDir "$Target\release"

# Check both binaries' PE headers before packaging. Machine (COFF header) must match -Arch, so an
# x64 build can never ship labelled arm64. Subsystem (optional header): 2 = Windows GUI, 3 = console.
# The app must be GUI (no console window opens with it); the CLI must stay console so its output
# reaches the terminal.
function Get-PeHeader([string] $Path) {
  $bytes = [System.IO.File]::ReadAllBytes($Path)
  $pe = [BitConverter]::ToInt32($bytes, 0x3C)
  return @{ Machine = [BitConverter]::ToUInt16($bytes, $pe + 4); Subsystem = [BitConverter]::ToUInt16($bytes, $pe + 0x5C) }
}
$Machine = switch ($Arch) { 'x64' { 0x8664 } 'x86' { 0x14C } 'arm64' { 0xAA64 } }
foreach ($check in @(@('gridcraft.exe', 2), @('gridcraft-cli.exe', 3))) {
  $h = Get-PeHeader (Join-Path $Bin $check[0])
  if ($h.Machine -ne $Machine) { throw "$($check[0]) is for machine 0x$('{0:X}' -f $h.Machine), expected 0x$('{0:X}' -f $Machine) ($Arch)" }
  if ($h.Subsystem -ne $check[1]) { throw "$($check[0]) has PE subsystem $($h.Subsystem), expected $($check[1])" }
  Write-Output "ok $($check[0]): $Arch, PE subsystem $($h.Subsystem)"
}
$Stage = Join-Path $TargetDir "windows-package\$Arch"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item (Join-Path $Bin 'gridcraft.exe'), (Join-Path $Bin 'gridcraft-cli.exe') $Stage
# NexusOffice: the licences, NOTICE (Apache-2.0 §4(d)) and ATTRIBUTION.md travel beside the
# binaries: the MSI installs them from here and the portable zip copies them below.
foreach ($f in 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md') {
  Copy-Item (Join-Path $Root $f) $Stage
}
# NexusOffice: the OFL texts of every font the binaries embed go in licenses\ (the MSI installs it
# as licenses\, the portable zip puts its files beside the exes): craft-fonts' fonts\<family>\OFL.txt
# as OFL-<family>.txt, the name the Linux and macOS packages use, and the repo's own
# assets\fonts\OFL-*.txt under their own names. Names must be unique ignoring case (Windows).
$Licenses = Join-Path $Stage 'licenses'
New-Item -ItemType Directory -Force -Path $Licenses | Out-Null
$LicenceSources = @{}  # file name -> source; PowerShell hashtables ignore case
function Add-Licence([string] $Source, [string] $Name) {
  if ($LicenceSources.ContainsKey($Name)) { throw "licence file name clash: $Name from $Source and $($LicenceSources[$Name])" }
  $LicenceSources[$Name] = $Source
  Copy-Item -LiteralPath $Source (Join-Path $Licenses $Name)
}
if ($env:CRAFT_FONTS_DIR) {
  $families = Get-ChildItem -LiteralPath (Join-Path $env:CRAFT_FONTS_DIR 'fonts') -Directory -ErrorAction SilentlyContinue
  foreach ($family in $families | Sort-Object Name) {
    $ofl = Join-Path $family.FullName 'OFL.txt'
    if (Test-Path -LiteralPath $ofl) { Add-Licence $ofl "OFL-$($family.Name).txt" }
  }
}
if ($env:CRAFT_FONTS_REQUIRED -eq '1' -and $LicenceSources.Count -eq 0) {
  throw "CRAFT_FONTS_REQUIRED=1 but no fonts\<family>\OFL.txt in CRAFT_FONTS_DIR ('$env:CRAFT_FONTS_DIR')"
}
$RepoFonts = Join-Path (Join-Path $Root 'assets') 'fonts'
if (Test-Path -LiteralPath $RepoFonts) {
  foreach ($ofl in Get-ChildItem -LiteralPath $RepoFonts -Filter 'OFL-*.txt' -File | Sort-Object Name) {
    Add-Licence $ofl.FullName $ofl.Name
  }
}
# The MSI's licenses\ group is built only when there is something to install (WiX define).
$FontLicences = if ($LicenceSources.Count) { 'yes' } else { 'no' }

& (Join-Path $PSScriptRoot 'sign.ps1') (Join-Path $Stage 'gridcraft.exe') (Join-Path $Stage 'gridcraft-cli.exe')

# ---- MSI ---------------------------------------------------------------------------------------
$Msi = Join-Path $Dist "NexusSheet-$Version-windows-$Arch.msi"
Invoke-Native 'wix build' {
  wix build (Join-Path $PSScriptRoot 'gridcraft.wxs') -arch $Arch `
    -d "Version=$MsiVersion" -d "BinDir=$Stage" -d "IconPath=$(Join-Path $Root 'nexus\assets\nexus-sheet.ico')" `
    -d "FontLicences=$FontLicences" `
    -o $Msi
}
# wix writes its debug symbols (.wixpdb) next to the MSI; keep them out of the release assets.
Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Msi, '.wixpdb'))
& (Join-Path $PSScriptRoot 'sign.ps1') $Msi

# ---- portable zip ------------------------------------------------------------------------------
$Portable = Join-Path $TargetDir "windows-package\NexusSheet-$Version-windows-$Arch-portable"
Remove-Item -Recurse -Force $Portable -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Portable | Out-Null
# NexusOffice: the same binary names the MSI installs.
Copy-Item (Join-Path $Stage 'gridcraft.exe') (Join-Path $Portable 'nexus-sheet.exe')
Copy-Item (Join-Path $Stage 'gridcraft-cli.exe') (Join-Path $Portable 'nexus-sheet-cli.exe')
foreach ($f in 'README.md', 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md') {
  $p = Join-Path $Root $f
  if (Test-Path $p) { Copy-Item $p $Portable }
}
# NexusOffice: the font licences staged in licenses\ (craft-fonts and the repo's own), as in the MSI.
Copy-Item (Join-Path $Licenses '*') $Portable
$Zip = Join-Path $Dist "NexusSheet-$Version-windows-$Arch-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
Compress-Archive -Path $Portable -DestinationPath $Zip

# Smoke-test the CLI when this machine can run it. An ARM64 build made on an x64 runner can't run
# here; .github/workflows/windows-arm64.yml installs and runs it on ARM64 instead.
$HostArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
if ($Arch -ne 'arm64' -or $HostArch -eq 'arm64') {
  Invoke-Native 'gridcraft-cli --version' { & (Join-Path $Stage 'gridcraft-cli.exe') --version }
} else {
  Write-Output "skipping gridcraft-cli --version: an $Arch build doesn't run on this $HostArch machine"
}
Get-Item $Msi, $Zip | Format-Table Name, Length
