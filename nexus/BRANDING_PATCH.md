# BRANDING_PATCH

Danh sách file mà nhánh `nexus` sửa so với upstream vì lý do thương hiệu.
Mỗi mục là đường dẫn file chính xác trong dấu backtick (không dùng glob/thư mục).

Giá trị: sản phẩm **Nexus Sheets**, bộ **NexusOffices**, nhà phát hành **Sonnytech**, app id
`vn.sonnytech.nexus.sheets`, tài sản phát hành `NexusSheets-*`, binary cài đặt
`nexus-sheets`/`nexus-sheets-cli` (MSI, zip portable, deb/rpm/AppImage/tar, zip CLI macOS; tên build
Cargo vẫn là `gridcraft`/`gridcraft-cli`, bundle macOS `Nexus Sheets.app` với executable `NexusSheets`).

Không đổi (spec §4.4): tên crate `gridcraft-*`, biến `GRIDCRAFT_*`, URI `gridcraft://…`, tên máy MCP
`serverInfo.name = "gridcraft"` (test và cấu hình client dựa vào), khoá i18n tiếng Anh, khoá plist
`GridCraftVersion`/`GridCraftBuildCommit`, id DOM `gridcraft_canvas`, schema `gridcraft-check/1`.

Cơ chế chính:
- `crates/ui-egui/src/i18n/brand.rs` (file mới) — `pub fn brand(s) -> Cow<str>` thay
  `GridCraft → Nexus Sheets`, `ArtCraft → NexusOffices`; `Language::tr()` (hàm tra chuỗi giao diện
  duy nhất, nhận nhãn giao diện, không nhận nội dung workbook) gọi `branded()` **một lần** cho kết quả,
  nên mọi ngôn ngữ (cả `en`) hiện tên Nexus mà không sửa key/catalog. Không áp vào văn bản người dùng:
  tiêu đề hộp thoại thông báo và chuỗi ghi vào file được đổi tại nguồn.
- `crates/ui-egui/src/nexus.rs` (file mới) — hằng `APP_NAME`, `APP_ID`, `SETTINGS_DIR(_XDG)`,
  `SOURCE_URL`, câu ghi công và trang About (en/vi qua `i18n::nexus_text`, hiện chỉ có `en` vì gridcraft
  chưa có tiếng Việt).

## File thương hiệu

| File | Lý do | Thay được bằng white-label? |
|---|---|---|
| `crates/ui-egui/src/i18n.rs` | `mod brand; pub use brand::{brand, nexus_text};` + `tr()` trả kết quả qua `brand::branded`; test catalog upstream so sánh với `brand(...)` | Có — hook "display name" trong i18n |
| `crates/ui-egui/src/lib.rs` | `pub mod nexus;`; tiêu đề hộp thoại lỗi = `nexus::APP_NAME`; thông báo VBA "Nexus Sheets keeps them intact" | Có — `APP_NAME` |
| `crates/ui-egui/src/dialogs.rs` | Tab About → `nexus::about_ui` (bỏ "Part of ArtCraft", link getartcraft/Discord); hộp Agent Control và Macros ghi Nexus Sheets, lệnh `nexus-sheets`/`nexus-sheets-cli` | Có |
| `crates/ui-egui/src/backstage.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` (4 chỗ) | Có — `APP_NAME` |
| `crates/ui-egui/src/formula_bar.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` | Có — `APP_NAME` |
| `crates/ui-egui/src/grid.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` (2 chỗ) | Có — `APP_NAME` |
| `crates/ui-egui/src/ribbon.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` | Có — `APP_NAME` |
| `crates/engine/src/lib.rs` | Tác giả mặc định `"Nexus Sheets User"` (ký ghi chú/bình luận ghi vào file) | Có — tên app qua biến build |
| `crates/engine/src/io.rs` | Thông báo mở file (mật khẩu, .xls lạ, JSON) nêu Nexus Sheets | Có |
| `crates/engine/src/cmd/solver.rs` | Tiêu đề báo cáo Solver ghi vào workbook "Nexus Sheets … Report" (+ test) | Có |
| `crates/engine/src/cmd/whatif.rs` | Chú thích dữ liệu Scenario Manager ghi vào workbook | Có |
| `crates/engine/src/cmd/spelling.rs` | Thông báo thiếu từ điển | Có |
| `crates/engine/src/cmd/extra.rs` | Mô tả lệnh `home.addIns` (hiện qua list_commands/MCP) và gợi ý `nexus-sheets-cli mcp` | Có |
| `crates/functions/src/info.rs` | `INFO("osversion")` trả "Nexus Sheets" (giá trị ghi vào ô) | Có |
| `crates/xlsx/src/write.rs` | `docProps/app.xml` `<Application>Nexus Sheets</Application>` | Có — tên app qua biến build |
| `crates/xlsx/src/pivot.rs` | `refreshedBy="Nexus Sheets"` trong pivot cache ghi vào file | Có |
| `crates/xlsx/src/xls/mod.rs` | Thông báo nhập .xls (mã hoá, không có sheet, cú pháp chưa hỗ trợ) | Có |
| `crates/xlsx/src/xls/formula.rs` | Lý do bỏ qua defined name khi nhập .xls | Có |
| `crates/pdf/src/lib.rs` | PDF `/Producer (Nexus Sheets PDF)`, `/Creator (Nexus Sheets)` mặc định | Có — tên app qua tham số |
| `crates/pdf/src/tests.rs` | Assert Producer mới | Đi cùng `lib.rs` |
| `crates/mcp/src/server.rs` | `serverInfo.title = "Nexus Sheets"`, INSTRUCTIONS; **`serverInfo.name` giữ `"gridcraft"`** (định danh máy) | Có |
| `crates/mcp/src/backend.rs` | Thông báo kết nối app | Có |
| `crates/mcp/src/tools.rs` | Mô tả tool execute_command/list_commands/screenshot (lệnh `nexus-sheets-cli mcp --connect`) | Có |
| `crates/mcp/src/headless.rs` | Thông báo `NO_UI` (tool UI ở chế độ headless) chỉ lệnh `nexus-sheets --control` / `nexus-sheets-cli mcp --connect` | Có — tên binary qua biến build |
| `apps/gridcraft/src/main.rs` | Tiêu đề cửa sổ, app id, tên eframe, `--version`, log, tiền tố `nexus-sheets:` của panic hook trên stderr, icon cửa sổ từ `nexus/assets/`, thư mục cấu hình `Nexus Sheets` (macOS/Windows) / `nexus-sheets` (XDG) — không đụng cấu hình GridCraft | Có |
| `apps/gridcraft/src/native_menu.rs` | Menu macOS: tên app/About Nexus Sheets, bỏ mục getartcraft.com và Discord ArtCraft trong Help | Có |
| `apps/gridcraft/tests/mac_menu_appkit.rs` | Assert tên menu app mới (chỉ chạy trên macOS) | Đi cùng `native_menu.rs` |
| `apps/gridcraft/src/print.rs` | Tên job in "Nexus Sheets document" | Có |
| `apps/gridcraft/build.rs` | Resource Windows: icon Nexus, ProductName, CompanyName Sonnytech, LegalCopyright (ghi công), OriginalFilename `nexus-sheets.exe` | Có |
| `apps/gridcraft-cli/src/main.rs` | USAGE, `--version` và tiền tố lỗi dùng tên cài đặt `nexus-sheets-cli`; lỗi kết nối app (gợi ý `nexus-sheets --control`) | Có — tên binary qua biến build |
| `apps/gridcraft-cli/tests/cli.rs` | Assert `--version` = `nexus-sheets-cli <ver>` | Đi cùng `main.rs` |
| `apps/gridcraft-cli/src/check.rs` | Dòng báo cáo "mismatch signatures (expected → Nexus Sheets)", tiền tố lỗi `nexus-sheets-cli:` | Có |
| `apps/gridcraft-web/index.html` | `<title>Nexus Sheets</title>`, favicon Nexus, "Loading Nexus Sheets…" | Có |
| `apps/gridcraft-web/src/web.rs` | Thông báo lỗi khởi động bản web | Có |
| `packaging/windows/gridcraft.wxs` | MSI: Name/Manufacturer Sonnytech, **UpgradeCode + GUID component mới**, cài `nexus-sheets.exe`/`nexus-sheets-cli.exe` vào `Program Files\Nexus Sheets`, ProgId `Sonnytech.NexusSheets.*`, khoá `Software\Sonnytech\Nexus Sheets\…`, `RegisteredApplications\NexusSheets`, App Paths `nexus-sheets.exe`, hộp thoại cài đặt, bỏ URL ARP getartcraft/Discord (About → repo fork); cài `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` cạnh exe (4 component mới, GUID tự sinh); vòng sửa 1: thư mục `licenses\` + ComponentGroup `GridcraftFontLicences` (`<Files Include="$(var.BinDir)\licenses\*">`) trong feature Main, có điều kiện `FontLicences` (mặc định `no`) | Một phần (biến WiX) |
| `packaging/windows/package.ps1` | Tài sản `NexusSheets-*.msi/.zip`, icon Nexus; zip portable chứa `nexus-sheets.exe`/`nexus-sheets-cli.exe` như MSI; chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào thư mục stage (MSI cài từ đó) và zip portable (Apache-2.0 §4(d)); vòng sửa 1: dựng `licenses\` = OFL craft-fonts (`OFL-<family>.txt`) + `assets\fonts\OFL-*.txt` (tên duy nhất không phân biệt hoa thường; ném lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL), truyền `-d FontLicences=yes|no` cho WiX, zip portable lấy OFL từ `licenses\` | Một phần |
| `packaging/windows/sign.ps1` | Mô tả chữ ký `/d Nexus Sheets`, `/du` repo fork (hiện trong hộp UAC khi có ký) | Một phần |
| `packaging/windows/test-desktop-shortcut.ps1` | Smoke test MSI theo bố cục mới (thư mục, shortcut, khoá registry, tên exe, icon); chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào thư mục bin giả để `wix build` có đủ file mà `gridcraft.wxs` cài; vòng sửa 1: build với `FontLicences=yes` và một file mẫu `licenses\OFL-smoke-fixture.txt` (sinh lúc chạy, không commit); khẳng định `NOTICE`, `ATTRIBUTION.md` và file đó được cài và bị gỡ khi uninstall | Đi cùng `gridcraft.wxs` |
| `packaging/windows/preview-installer-dialogs.ps1` | Tìm cửa sổ "Nexus Sheets Setup" | Đi cùng `gridcraft.wxs` |
| `packaging/macos/Info.plist.in` | Tên bundle, executable `NexusSheets`, id `vn.sonnytech.nexus.sheets`, icon, copyright (ghi công) | Một phần |
| `packaging/macos/package.sh` | `Nexus Sheets.app`, `NexusSheets-*.dmg`, `NexusSheets-cli-*.zip` chứa `nexus-sheets-cli`, icns Nexus, tên volume; không chép `dmg-layout.DS_Store` (gắn tên GridCraft); `copy_notices` chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào `Contents/Resources` của app (trước khi ký) | Một phần |
| `packaging/linux/package.sh` | `APP_ID=vn.sonnytech.nexus.sheets`, `/usr/bin/nexus-sheets(-cli)`, `NexusSheets-*`, icon `nexus/assets/hicolor`, maintainer Sonnytech, zsync trỏ fork; AppImage giữ `usr/share/doc/nexus-sheets` (giấy phép, `NOTICE`, `ATTRIBUTION.md`) thay vì bỏ ra ngoài | Một phần |
| `packaging/linux/nfpm.yaml` | Gói `nexus-sheets`, vendor/homepage, binary, ghi công | Một phần |
| `packaging/env.sh` | `copy_docs` chép thêm `NOTICE` (Apache-2.0 §4(d)) và `ATTRIBUTION.md` (README Nexus trỏ tới cả hai) qua hàm mới `copy_notices` — bắt buộc, thiếu file thì lỗi; dùng cho deb/AppImage/tar, zip CLI macOS, zip web (và FreeBSD của upstream); vòng sửa 1: chép thêm `assets/fonts/OFL-*.txt` (font của repo nhúng trong binary, nếu có) sau OFL craft-fonts và báo lỗi khi trùng tên; mọi `cp` có `|| return 1` (gọi qua `||` thì errexit tắt); `copy_font_licences` báo lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL | Nên PR upstream (gói upstream cũng thiếu NOTICE) |
| `packaging/web/package.sh` | Zip `NexusSheets-web-<ver>.zip` | Một phần |
| `packaging/web/README.md` | Được đóng gói thành `HOSTING.md` trong zip web → đổi tên sản phẩm/tên zip | Không (tài liệu đi kèm gói) |
| `omastore.toml` | Icon, tên tài sản `NexusSheets-*`, exec `bin/nexus-sheets` (bỏ `ai.storyteller`) | Không |
| `README.md` | README Nexus Sheets ngắn (vi trước, có English): sản phẩm, NexusOffices/Sonnytech, câu ghi công "Nexus Sheets được phát triển dựa trên GridCraft của ArtCraft (MIT/Apache-2.0)" chỉ bằng chữ (không logo/Discord/getartcraft), trỏ LICENSE-MIT/LICENSE-APACHE/NOTICE/ATTRIBUTION; được `copy_docs` và zip portable đóng gói | Không (README của fork) |
| `Cargo.toml` | `[workspace.package] repository` → `https://github.com/nhdai111983/gridcraft` (brand-check `brand-url`; metadata, không code nào đọc `CARGO_PKG_REPOSITORY`) | Có — metadata của fork |
| `ATTRIBUTION.md` | 14 dòng cho icon Nexus (`nexus/assets/**`, Sonnytech, CC0-1.0) | Không (bắt buộc theo luật asset) |

### GUID WiX mới (Nexus Sheets riêng, không dùng chung với GridCraft hay app Nexus khác)

| Mục | GUID cũ (upstream GridCraft) | GUID mới (Nexus Sheets) |
|---|---|---|
| `UpgradeCode` | `5511327B-F305-41BE-92D0-EB84944E7C4D` | `AD43085E-E0C6-4FBD-A8FF-DA99DAC6F718` |
| Component `GridcraftApp` | `0507D4B6-12FF-4882-84E1-59083E74A4EB` | `B077AEA1-B862-4956-B495-84E7BBB90C49` |
| Component `GridcraftCli`, `DesktopShortcutPreference`, `DesktopShortcutComponent` | (tự sinh theo đường dẫn/khoá) | tự sinh — thư mục cài (`Nexus Sheets`), tên file và khoá registry (`Software\Sonnytech\Nexus Sheets\Installer`) đã khác |
| Component `GridcraftMitLicense`, `GridcraftApacheLicense`, `GridcraftNotice`, `GridcraftAttribution` | (không có ở upstream) | tự sinh — file giấy phép/NOTICE/ATTRIBUTION.md cài cạnh exe |
| ComponentGroup `GridcraftFontLicences` (mỗi file trong `licenses\` một component) | (không có ở upstream) | tự sinh — OFL của font nhúng, cài vào `licenses\` |

Sinh bằng `python3 -c "import uuid; print(uuid.uuid4())"` ngày 2026-10-16.

### File mới (không tính là patch)
`crates/ui-egui/src/i18n/brand.rs`, `crates/ui-egui/src/nexus.rs`, `crates/engine/tests/nexus_brand.rs`
(test `default_author_is_nexus`, PDF Producer, `INFO("osversion")`), `crates/mcp/tests/nexus_brand.rs`
(serverInfo, tên lệnh trong tool/NO_UI), `apps/gridcraft-cli/tests/nexus_brand.rs` (USAGE/`--version`/tiền tố lỗi),
`packaging/linux/vn.sonnytech.nexus.sheets.desktop`,
`packaging/linux/vn.sonnytech.nexus.sheets.metainfo.xml.in` (MIME dùng lại file upstream
`ai.storyteller.gridcraft.mime.xml`, cài dưới tên APP_ID), `nexus/assets/*` (logo tạm
`nexus-sheets.svg` + PNG/ICO/ICNS/hicolor sinh bằng `nexus/assets/render-icons.py`).

## File khác

| File | Lý do | Thay được bằng white-label? |
|---|---|---|

## Rà soát gọi mạng (guard §5)
`rg -n -i "artcraft|storytold|storyteller|discord|telemetry|update|crash|sentry|posthog|https?://"`:
không có telemetry/crash report/cập nhật tự động trong app. Link ArtCraft app tự mở: About
(getartcraft, Discord) và menu Help macOS (getartcraft, Discord) — đã bỏ. MSI: URL ARP
getartcraft/Discord đã bỏ (ARPURLINFOABOUT → repo fork). AppImage zsync trỏ GitHub Releases của fork
(`GITHUB_REPOSITORY` hoặc `nhdai111983/gridcraft`). Dependency git `storytold/craft-libs` (craft-ui,
craft-print) chỉ là nguồn build (T5/§4.8 mirror), không phải gọi mạng lúc chạy.

## Việc còn nợ (đã allowlist có lý do, hoặc không bị brand-check bắt)
- Đề xuất PR upstream (white-label): hằng `APP_NAME`/`SETTINGS_DIR`/`APP_ID` lấy từ biến build
  (`GRIDCRAFT_APP_NAME`…) → fork chỉ còn đặt biến, bỏ phần lớn patch ở trên.
- Control channel (`ui.state`) báo `message.title` = "Nexus Sheets" nhưng tiêu đề dialog `About`/`start`
  trong JSON vẫn là khoá tiếng Anh ("About GridCraft"), chỉ phần hiển thị được brand.
- Gói FreeBSD (`packaging/freebsd/package.sh`) và Flatpak (`packaging/linux/flatpak*`) chưa đổi — chưa
  phát hành (workflow freebsd tắt theo spec §4.6).
- DMG macOS: `dmg-layout.DS_Store`/`background.tiff` gắn với volume "GridCraft" → DMG Nexus mở với
  layout mặc định của Finder; sinh lại trên macOS (Task 12).
- `xtask ico` test và `packaging/icons.sh` vẫn trỏ icon upstream `assets/app-icon/` (công cụ dev).
