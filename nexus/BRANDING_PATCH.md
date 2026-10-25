# BRANDING_PATCH

Danh sách file mà nhánh `nexus` sửa so với upstream vì lý do thương hiệu.
Mỗi mục là đường dẫn file chính xác trong dấu backtick (không dùng glob/thư mục).

Giá trị: sản phẩm **Nexus Sheet**, bộ **NexusOffice**, nhà phát hành **SonnyTech**, app id
`vn.sonnytech.nexus.sheets`, tài sản phát hành `NexusSheet-*`, binary cài đặt
`nexus-sheet`/`nexus-sheet-cli` (MSI, zip portable, deb/rpm/AppImage/tar, zip CLI macOS; tên build
Cargo vẫn là `gridcraft`/`gridcraft-cli`, bundle macOS `Nexus Sheet.app` với executable `NexusSheet`).

Nguồn của bản build (About, metadata gói, README): `https://github.com/sonnytech-vn/gridcraft` (tổ chức sẽ được
tạo và repo chuyển sang; GitHub chuyển hướng URL cũ). Remote git, workflow đồng bộ upstream và URL
`GITHUB_REPOSITORY`/zsync của bản phát hành chưa đổi.

Giữ nguyên định danh máy (không hiện cho người dùng): ProgId `Sonnytech.NexusSheets.Workbook`/`.TextData`,
khoá registry `Software\Sonnytech\Nexus Sheets\…` (Capabilities, Installer) và giá trị
`RegisteredApplications\NexusSheets`, `UpgradeCode` và GUID component WiX, app id `vn.sonnytech.nexus.sheets`
(chữ thường), tên file icon theo app id `vn.sonnytech.nexus.sheets.*`, `serverInfo.name = "gridcraft"`, tên crate,
biến môi trường `GRIDCRAFT_*`, email maintainer `nhdai111983@users.noreply.github.com`.

Đổi thương hiệu 2026-10-23: Sonnytech → SonnyTech, NexusOffices → NexusOffice, Nexus Sheets → Nexus Sheet
(binary `nexus-sheet`/`nexus-sheet-cli`, tài sản `NexusSheet-*`, thư mục cài/Start Menu `Nexus Sheet`, icon nguồn
`nexus/assets/nexus-sheet.*`, thư mục cấu hình `Nexus Sheet`/`nexus-sheet`). Thư mục cấu hình cũ
(`Nexus Sheets`/`nexus-sheets`) vẫn được đọc khi thư mục mới chưa tồn tại; ghi luôn vào thư mục mới.

Không đổi (spec §4.4): tên crate `gridcraft-*`, biến `GRIDCRAFT_*`, URI `gridcraft://…`, tên máy MCP
`serverInfo.name = "gridcraft"` (test và cấu hình client dựa vào), khoá i18n tiếng Anh, khoá plist
`GridCraftVersion`/`GridCraftBuildCommit`, id DOM `gridcraft_canvas`, schema `gridcraft-check/1`.

Cơ chế chính:
- `crates/ui-egui/src/i18n/brand.rs` (file mới) — `pub fn brand(s) -> Cow<str>` thay
  `GridCraft → Nexus Sheet`, `ArtCraft → NexusOffice`; `Language::tr()` (hàm tra chuỗi giao diện
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
| `crates/ui-egui/src/lib.rs` | `pub mod nexus;`; tiêu đề hộp thoại lỗi = `nexus::APP_NAME`; thông báo VBA "Nexus Sheet keeps them intact" | Có — `APP_NAME` |
| `crates/ui-egui/src/dialogs.rs` | Tab About → `nexus::about_ui` (bỏ "Part of ArtCraft", link getartcraft/Discord); hộp Agent Control và Macros ghi Nexus Sheet, lệnh `nexus-sheet`/`nexus-sheet-cli` | Có |
| `crates/ui-egui/src/backstage.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` (4 chỗ) | Có — `APP_NAME` |
| `crates/ui-egui/src/formula_bar.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` | Có — `APP_NAME` |
| `crates/ui-egui/src/grid.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` (2 chỗ) | Có — `APP_NAME` |
| `crates/ui-egui/src/ribbon.rs` | Tiêu đề hộp thoại lỗi = `nexus::APP_NAME` | Có — `APP_NAME` |
| `crates/engine/src/lib.rs` | Tác giả mặc định `"Nexus Sheet User"` (ký ghi chú/bình luận ghi vào file) | Có — tên app qua biến build |
| `crates/engine/src/io.rs` | Thông báo mở file (mật khẩu, .xls lạ, JSON) nêu Nexus Sheet | Có |
| `crates/engine/src/cmd/solver.rs` | Tiêu đề báo cáo Solver ghi vào workbook "Nexus Sheet … Report" (+ test) | Có |
| `crates/engine/src/cmd/whatif.rs` | Chú thích dữ liệu Scenario Manager ghi vào workbook | Có |
| `crates/engine/src/cmd/spelling.rs` | Thông báo thiếu từ điển | Có |
| `crates/engine/src/cmd/extra.rs` | Mô tả lệnh `home.addIns` (hiện qua list_commands/MCP) và gợi ý `nexus-sheet-cli mcp` | Có |
| `crates/functions/src/info.rs` | `INFO("osversion")` trả "Nexus Sheet" (giá trị ghi vào ô) | Có |
| `crates/xlsx/src/write.rs` | `docProps/app.xml` `<Application>Nexus Sheet</Application>` | Có — tên app qua biến build |
| `crates/xlsx/src/pivot.rs` | `refreshedBy="Nexus Sheet"` trong pivot cache ghi vào file | Có |
| `crates/xlsx/src/xls/mod.rs` | Thông báo nhập .xls (mã hoá, không có sheet, cú pháp chưa hỗ trợ) | Có |
| `crates/xlsx/src/xls/formula.rs` | Lý do bỏ qua defined name khi nhập .xls | Có |
| `crates/pdf/src/lib.rs` | PDF `/Producer (Nexus Sheet PDF)`, `/Creator (Nexus Sheet)` mặc định | Có — tên app qua tham số |
| `crates/pdf/src/tests.rs` | Assert Producer mới | Đi cùng `lib.rs` |
| `crates/mcp/src/server.rs` | `serverInfo.title = "Nexus Sheet"`, INSTRUCTIONS; **`serverInfo.name` giữ `"gridcraft"`** (định danh máy) | Có |
| `crates/mcp/src/backend.rs` | Thông báo kết nối app | Có |
| `crates/mcp/src/tools.rs` | Mô tả tool execute_command/list_commands/screenshot (lệnh `nexus-sheet-cli mcp --connect`) | Có |
| `crates/mcp/src/headless.rs` | Thông báo `NO_UI` (tool UI ở chế độ headless) chỉ lệnh `nexus-sheet --control` / `nexus-sheet-cli mcp --connect` | Có — tên binary qua biến build |
| `apps/gridcraft/src/main.rs` | Tiêu đề cửa sổ, app id, tên eframe, `--version`, log, tiền tố `nexus-sheet:` của panic hook trên stderr, icon cửa sổ từ `nexus/assets/`, thư mục cấu hình `Nexus Sheet` (macOS/Windows) / `nexus-sheet` (XDG) — không đụng cấu hình GridCraft ; `load_prefs` đọc thư mục cấu hình cũ `Nexus Sheets`/`nexus-sheets` khi thư mục mới chưa có (`legacy_config_dir`) | Có |
| `apps/gridcraft/src/native_menu.rs` | Menu macOS: tên app/About Nexus Sheet, bỏ mục getartcraft.com và Discord ArtCraft trong Help | Có |
| `apps/gridcraft/tests/mac_menu_appkit.rs` | Assert tên menu app mới (chỉ chạy trên macOS) | Đi cùng `native_menu.rs` |
| `apps/gridcraft/src/print.rs` | Tên job in "Nexus Sheet document" | Có |
| `apps/gridcraft/build.rs` | Resource Windows: icon Nexus, ProductName, CompanyName SonnyTech, LegalCopyright (ghi công), OriginalFilename `nexus-sheet.exe` | Có |
| `apps/gridcraft-cli/src/main.rs` | USAGE, `--version` và tiền tố lỗi dùng tên cài đặt `nexus-sheet-cli`; lỗi kết nối app (gợi ý `nexus-sheet --control`) | Có — tên binary qua biến build |
| `apps/gridcraft-cli/tests/cli.rs` | Assert `--version` = `nexus-sheet-cli <ver>` | Đi cùng `main.rs` |
| `apps/gridcraft-cli/src/check.rs` | Dòng báo cáo "mismatch signatures (expected → Nexus Sheet)", tiền tố lỗi `nexus-sheet-cli:` | Có |
| `apps/gridcraft-web/index.html` | `<title>Nexus Sheet</title>`, favicon Nexus, "Loading Nexus Sheet…" | Có |
| `apps/gridcraft-web/src/web.rs` | Thông báo lỗi khởi động bản web | Có |
| `packaging/windows/gridcraft.wxs` | MSI: Name/Manufacturer SonnyTech, **UpgradeCode + GUID component mới**, cài `nexus-sheet.exe`/`nexus-sheet-cli.exe` vào `Program Files\Nexus Sheet`, ProgId `Sonnytech.NexusSheets.*`, khoá `Software\Sonnytech\Nexus Sheets\…`, `RegisteredApplications\NexusSheet`, App Paths `nexus-sheet.exe`, hộp thoại cài đặt, bỏ URL ARP getartcraft/Discord (About → repo fork); cài `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` cạnh exe (4 component mới, GUID tự sinh); vòng sửa 1: thư mục `licenses\` + ComponentGroup `GridcraftFontLicences` (`<Files Include="$(var.BinDir)\licenses\*">`) trong feature Main, có điều kiện `FontLicences` (mặc định `no`) | Một phần (biến WiX) |
| `packaging/windows/package.ps1` | Tài sản `NexusSheet-*.msi/.zip`, icon Nexus; zip portable chứa `nexus-sheet.exe`/`nexus-sheet-cli.exe` như MSI; chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào thư mục stage (MSI cài từ đó) và zip portable (Apache-2.0 §4(d)); vòng sửa 1: dựng `licenses\` = OFL craft-fonts (`OFL-<family>.txt`) + `assets\fonts\OFL-*.txt` (tên duy nhất không phân biệt hoa thường; ném lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL), truyền `-d FontLicences=yes|no` cho WiX, zip portable lấy OFL từ `licenses\` | Một phần |
| `packaging/windows/sign.ps1` | Mô tả chữ ký `/d Nexus Sheet`, `/du` repo fork (hiện trong hộp UAC khi có ký) | Một phần |
| `packaging/windows/test-desktop-shortcut.ps1` | Smoke test MSI theo bố cục mới (thư mục, shortcut, khoá registry, tên exe, icon); chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào thư mục bin giả để `wix build` có đủ file mà `gridcraft.wxs` cài; vòng sửa 1: build với `FontLicences=yes` và một file mẫu `licenses\OFL-smoke-fixture.txt` (sinh lúc chạy, không commit); khẳng định `NOTICE`, `ATTRIBUTION.md` và file đó được cài và bị gỡ khi uninstall | Đi cùng `gridcraft.wxs` |
| `packaging/windows/preview-installer-dialogs.ps1` | Tìm cửa sổ "Nexus Sheet Setup" | Đi cùng `gridcraft.wxs` |
| `packaging/macos/Info.plist.in` | Tên bundle, executable `NexusSheet`, id `vn.sonnytech.nexus.sheets`, icon, copyright (ghi công) | Một phần |
| `packaging/macos/package.sh` | `Nexus Sheet.app`, `NexusSheet-*.dmg`, `NexusSheet-cli-*.zip` chứa `nexus-sheet-cli`, icns Nexus, tên volume; không chép `dmg-layout.DS_Store` (gắn tên GridCraft); `copy_notices` chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào `Contents/Resources` của app (trước khi ký) | Một phần |
| `packaging/linux/package.sh` | `APP_ID=vn.sonnytech.nexus.sheets`, `/usr/bin/nexus-sheet(-cli)`, `NexusSheet-*`, icon `nexus/assets/hicolor`, maintainer SonnyTech, zsync trỏ fork; AppImage giữ `usr/share/doc/nexus-sheet` (giấy phép, `NOTICE`, `ATTRIBUTION.md`) thay vì bỏ ra ngoài | Một phần |
| `packaging/linux/nfpm.yaml` | Gói `nexus-sheet`, vendor/homepage, binary, ghi công | Một phần |
| `packaging/env.sh` | `copy_docs` chép thêm `NOTICE` (Apache-2.0 §4(d)) và `ATTRIBUTION.md` (README Nexus trỏ tới cả hai) qua hàm mới `copy_notices` — bắt buộc, thiếu file thì lỗi; dùng cho deb/AppImage/tar, zip CLI macOS, zip web (và FreeBSD của upstream); vòng sửa 1: chép thêm `assets/fonts/OFL-*.txt` (font của repo nhúng trong binary, nếu có) sau OFL craft-fonts và báo lỗi khi trùng tên; mọi `cp` có `|| return 1` (gọi qua `||` thì errexit tắt); `copy_font_licences` báo lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL | Nên PR upstream (gói upstream cũng thiếu NOTICE) |
| `packaging/web/package.sh` | Zip `NexusSheet-web-<ver>.zip` | Một phần |
| `packaging/web/README.md` | Được đóng gói thành `HOSTING.md` trong zip web → đổi tên sản phẩm/tên zip | Không (tài liệu đi kèm gói) |
| `omastore.toml` | Icon, tên tài sản `NexusSheet-*`, exec `bin/nexus-sheet` (bỏ `ai.storyteller`) | Không |
| `README.md` | README Nexus Sheet ngắn (vi trước, có English): sản phẩm, NexusOffice/SonnyTech, câu ghi công "Nexus Sheet được phát triển dựa trên GridCraft của ArtCraft (MIT/Apache-2.0)" chỉ bằng chữ (không logo/Discord/getartcraft), trỏ LICENSE-MIT/LICENSE-APACHE/NOTICE/ATTRIBUTION; được `copy_docs` và zip portable đóng gói | Không (README của fork) |
| `Cargo.toml` | `[workspace.package] repository` → `https://github.com/sonnytech-vn/gridcraft` (brand-check `brand-url`; metadata, không code nào đọc `CARGO_PKG_REPOSITORY`) | Có — metadata của fork |
| `ATTRIBUTION.md` | 14 dòng cho icon Nexus (`nexus/assets/**`, SonnyTech, CC0-1.0) | Không (bắt buộc theo luật asset) |
| `crates/ui-egui/src/nexus_toolbar.rs` | File mới (NX-UI, ROADMAP F2.2): bố cục NexusOffice kiểu WPS — thanh menu sinh từ registry lệnh (mỗi tab ribbon một menu, mỗi nhóm một menu con, vị trí cấp 3 thành menu con lồng; `Edit`/`Window`/`Table`/`Format`/PivotTable gộp vào Home/View/Table Design/Chart Design/PivotTable để không mất lệnh nào) + một hàng công cụ (clipboard, phông/cỡ, B/I/U, viền, màu nền/chữ, căn lề, xuống dòng, gộp ô, định dạng số, sắp xếp/lọc, chèn/xoá hàng-cột, AutoSum, tìm); cửa sổ hẹp thì cụm cuối dồn vào menu "More tools"; đường gradient SonnyTech dưới thanh menu; test (đủ lệnh, registry rỗng, kittest bấm menu/Bold, cửa sổ hẹp, `ui.accent`, prefs) | — (file mới) |
| `crates/ui-egui/src/lib.rs` (thêm) | `pub mod nexus_toolbar;`, trường `SheetApp::toolbar` (mặc định tắt = ribbon upstream), `ui()` vẽ `nexus_toolbar::show` thay `ribbon::show` khi bật; `UiState::accent` (`#RRGGBB`, đọc lỏng: giá trị rác = không có, không ghi khoá khi `None`); lệnh `ui.accent` (`{"value": "#RRGGBB"}` đặt, `""`/`null` về màu app, không có `value` thì đọc); `logic()` áp màu qua `theme::set_accent` | Có — cờ giao diện + màu chủ đạo qua prefs |
| `crates/ui-egui/src/theme.rs` | Bảng màu Nexus Sheet: Emerald `#10B981` (ink `#047857` cho chữ/nền có chữ trắng, 5.5:1), nền slate sáng `#F8FAFC`/`#F1F5F9`/`#E9EDF2`, viền `#E2E8F0`; tối `#0F172A`/`#1E293B`/`#020617`/`#334155`; token mới `on_accent`; màu chủ đạo người dùng chọn lưu trong bộ nhớ context egui (`set_accent`/`accent`, `with_accent` đẩy sáng/tối tới khi chữ đạt WCAG 4.5:1, viền vùng chọn 3:1), `ACCENT_PRESETS`, `parse_hex`/`to_hex`, `contrast_ratio`; `apply` tách `install_visuals`; bỏ xanh lá kiểu Excel `#107C41` | Có — bảng màu qua biến build |
| `crates/ui-egui/src/backstage.rs` (thêm) | File › Options › General: hàng "Accent colour/Màu chủ đạo" (Màu ứng dụng, 6 màu gợi sẵn, chọn màu tuỳ ý → `ui.accent`); thanh điều hướng File dùng `on_accent` thay trắng cố định | Có — giao diện NexusOffice |
| `crates/ui-egui/src/ribbon.rs` (thêm) | Chỉ đổi phạm vi: `contextual_tabs`, `active_style`, `theme_colors`, `act`, `menu_items`, `font_combo`, `size_combo` thành `pub(crate)` để hàng công cụ dùng lại | Có — nếu upstream mở các hàm này |
| `crates/ui-egui/src/control.rs` | `engine.commands` liệt kê `ui.accent` (tham số, kết quả) | Có |
| `crates/ui-egui/src/dialogs.rs` (thêm) | Nút OK/Solve: chữ `on_accent` thay trắng cố định (chế độ tối và màu chủ đạo sáng vẫn đọc được) | Nên PR upstream (trắng trên accent sáng chỉ ~2:1) |
| `crates/ui-egui/src/chartview.rs` | Ô giá trị slicer đang chọn: chữ `on_accent` thay trắng cố định | Nên PR upstream |
| `crates/ui-egui/src/icons.rs` | `GREEN` của icon = ink Emerald `#047857` thay `#107C41` (xanh kiểu Excel) | Có — bảng màu qua biến build |
| `crates/ui-egui/src/grid.rs` (thêm) | Ô checkbox trong ô: màu đánh dấu `theme::APP_INK` thay `#107C41` | Có — bảng màu |
| `crates/ui-egui/src/panes.rs` | Nhãn "Resolved" của bình luận: `theme::APP_INK` thay `#107C41` | Có — bảng màu |
| `crates/ui-egui/examples/snapshot.rs` | Cờ `--toolbar` (bố cục NexusOffice), `--accent '#RRGGBB'`, `--ui-cmd`, `--click`/`--hover` (mở menu trước khi chụp) để chụp màn hình | Có |
| `apps/gridcraft/src/main.rs` (thêm) | Bật `app.toolbar` (bố cục NexusOffice) | Có — cờ giao diện |
| `apps/gridcraft-web/src/web.rs` (thêm) | Bật `app.toolbar` cho bản web; NX-UI.6: bật `app.wps` thay `app.toolbar` | Có — cờ giao diện |
| `crates/ui-egui/src/nexus_wps.rs` | File mới (NX-UI.5, ROADMAP NexusOffice F1.3): bố cục NexusOffice kiểu WPS giống Nexus Docs — không thanh tiêu đề; hàng trên 36 pt (Tệp mở Backstage, Quick Access, thẻ ribbon căn trái có » cho thẻ thừa, trạng thái lưu có Tự động lưu/Lưu/Lưu thành, ô Tìm lệnh, Nhận xét, Chia sẻ, "⋯" giao diện/Tùy chọn/Giới thiệu); dải lệnh 56 pt hai hàng nút nhỏ, nút lớn = icon 20 px trên một dòng nhãn, nhóm không vừa gập vào bảng "…"; `compact_stack`, cổng nhóm (`group`), `fit_groups`/`fit_tabs`; test kittest | — (file mới) |
| `crates/ui-egui/src/file_dialogs.rs` | File mới (NX-UI.5): `FileDialogRequest`/`FileDialogAnswer`, hộp thoại Mở/Lưu thành/xuất mà host trả lời sau qua channel (không chặn luồng giao diện), `AfterPick` (mở, lưu rồi có thể đóng, xuất PDF, lưu bản sao), `save_types` | — (file mới) |
| `crates/ui-egui/src/i18n/vi.rs` | File mới (NX-UI.4): catalogue tiếng Việt 960 nhãn (mọi nhãn các ngôn ngữ khác dịch + nhãn bố cục NexusOffice) | — (file mới) |
| `crates/ui-egui/src/i18n.rs` (thêm) | NX-UI.4: `Language::Vi` ("vi", `vi-VN`), `mod vi`, test catalogue đòi tiếng Việt cho mọi nhãn | Nên PR upstream (ngôn ngữ mới) |
| `crates/ui-egui/src/lib.rs` (thêm NX-UI.5) | `pub mod nexus_wps; pub mod file_dialogs;`, `SheetApp::wps` (thắng `toolbar`) và `SheetApp::embedded` (không đặt tiêu đề cửa sổ), `ui()` vẽ `nexus_wps::show` thay thanh tiêu đề + ribbon; `Services::new_document` (File › Mới/Ctrl+N/mẫu → host mở thẻ mới), `Services::open_document` (sổ được chọn → host, trừ khi cửa sổ chỉ có sổ trống chưa đụng), `Services::file_dialog` (hộp thoại không chặn); `open_dialog("open"/"saveAs")`, xuất PDF hỏi host trước picker chặn; `logic()` nhận câu trả lời | Có — hook host (nên PR upstream dạng trung lập) |
| `crates/ui-egui/src/ribbon.rs` (thêm NX-UI.5) | Mỗi nhóm giữa hai `sep` của các thẻ đi qua `group()` (ribbon upstream vẽ tại chỗ, dải WPS lọc/gập), cột nút nhỏ qua `stack()`; `tab_groups`/`shown_tabs` tách khỏi `show`; Quick Access tách thành `quick_access()` (nhãn trợ năng cho Lưu/Hoàn tác/Làm lại, menu dịch); thanh tiêu đề không kéo cửa sổ khi `embedded`; `keytip`, `theme_choices` thành `pub(crate)` | Một phần — `group`/`stack` không đổi hình ribbon upstream |
| `crates/ui-egui/src/widgets.rs` (thêm) | `CONTENT_H`/`COMPACT_H`/`content_h`; nút lớn trên dải WPS: icon 20 px trên một dòng nhãn, cao 50; nút lớn có nhãn trợ năng | Có |
| `crates/ui-egui/src/paper_view.rs` | Vạch ngăn thẻ Header & Footer cao theo `content_h` (dải WPS) | Có |
| `crates/ui-egui/src/icons.rs` (thêm) | Icon vẽ bằng code `Hamburger` (nút Tệp) và `ChevronDoubleRight` (»), cho bố cục WPS | Có |
| `crates/ui-egui/src/dialogs.rs` (thêm NX-UI.5) | "Lưu thay đổi?" dịch được; Lưu trên sổ chưa có tệp chờ Lưu thành của host rồi đóng; Lưu bản sao hỏi hộp thoại host | Có |
| `crates/ui-egui/src/backstage.rs` (thêm NX-UI.5) | Xuất (PDF/CSV/HTML) hỏi hộp thoại host trước picker chặn | Có |
| `crates/ui-egui/tests/nexus_embed.rs` | File mới (NX-UI.5): test hook host (mới, mở, lưu không chặn, lưu rồi đóng, huỷ) | — (file mới) |
| `crates/ui-egui/examples/snapshot.rs` (thêm NX-UI.6) | Cờ `--wps`, `--embedded` | Có |
| `apps/gridcraft/src/main.rs` (thêm NX-UI.6) | Bật `app.wps` thay `app.toolbar` (bố cục menu vẫn sau cờ) | Có — cờ giao diện |
| `crates/ui-egui/src/nexus_save_prompt.rs` | File mới, giống hệt ở 4 fork: hộp «Lưu thay đổi?» chung của NexusOffice khi chạy nhúng (quyết định Tech Lead 2026-10-25) — tiêu đề «Lưu thay đổi?», câu «Lưu thay đổi của “<tên>” trước khi đóng?», nút Lưu (màu chủ đạo) · Không lưu · Hủy căn phải, Enter = Lưu (nút đang có tiêu điểm), Esc = Hủy, tiêu điểm bắt đầu ở Lưu, modal trên tab (làm mờ tab, chặn chuột/phím bên dưới); số đo theo DESIGN_SYSTEM.md §3 (bo góc hộp 12, nút 6, lề 20, nút tối thiểu 88×32, tiêu đề 22, chữ 13), màu thẻ theo §2.4 (nền, viền, chữ giống nhau ở mọi app; chỉ nút Lưu theo màu chủ đạo); Tab/Shift+Tab chỉ đi qua ba nút theo thứ tự đọc; AccessKit: thẻ là `Dialog` mang tên «Lưu thay đổi?» | — (file mới) |
| `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/lib.rs` (thêm) | NX-UI.12: khi `app.embedded`, hộp `saveChanges` vẽ bằng `nexus_save_prompt` (Lưu/Không lưu/Hủy làm đúng việc của Save/Don't Save/Cancel upstream, gồm ghi ô đang gõ trước khi lưu và chờ Lưu thành của chủ nhà); trường `SheetApp::drawn_in` (vùng tab). Không nhúng giữ nguyên hộp upstream. Test `crates/ui-egui/tests/nexus_save_prompt.rs` | Có — cờ nhúng |

### GUID WiX mới (Nexus Sheet riêng, không dùng chung với GridCraft hay app Nexus khác)

| Mục | GUID cũ (upstream GridCraft) | GUID mới (Nexus Sheet) |
|---|---|---|
| `UpgradeCode` | `5511327B-F305-41BE-92D0-EB84944E7C4D` | `AD43085E-E0C6-4FBD-A8FF-DA99DAC6F718` |
| Component `GridcraftApp` | `0507D4B6-12FF-4882-84E1-59083E74A4EB` | `B077AEA1-B862-4956-B495-84E7BBB90C49` |
| Component `GridcraftCli`, `DesktopShortcutPreference`, `DesktopShortcutComponent` | (tự sinh theo đường dẫn/khoá) | tự sinh — thư mục cài (`Nexus Sheet`), tên file và khoá registry (`Software\Sonnytech\Nexus Sheets\Installer`) đã khác |
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
`nexus-sheet.svg` + PNG/ICO/ICNS/hicolor sinh bằng `nexus/assets/render-icons.py`).

## File khác

| File | Lý do | Thay được bằng white-label? |
|---|---|---|
| `crates/core/src/locale.rs` | File mới (NX-VI.1): `Locale` (`EnUs`, `ViVn`) — dấu thập phân/nhóm/danh sách, tên tháng/thứ CLDR `vi`, SA/CH | — (file mới; nên PR upstream) |
| `crates/core/src/lib.rs` | NX-VI.1: `pub mod locale; pub use locale::Locale` | Nên PR upstream |
| `crates/core/src/parse.rs` | NX-VI.1: `parse_input_in`/`parse_number_text_locale` — nhập `1.234,5`, `24/10/2026`, `1.234 ₫` theo vi-VN; `VND_FORMAT` | Nên PR upstream |
| `crates/numfmt/src/lib.rs` | NX-VI.1: `format_value_in`, `format_general_fit_in`, `text_function_in`; định dạng hệ thống (14, 22, F800, F400) theo vùng | Nên PR upstream |
| `crates/numfmt/src/render.rs` | NX-VI.1: dấu tách, tên tháng/thứ, SA/CH theo vùng khi vẽ số/ngày | Nên PR upstream |
| `crates/numfmt/src/parse.rs` | NX-VI.1: `delocalize_code` (mã định dạng TEXT viết kiểu vi-VN) | Nên PR upstream |
| `crates/model/src/workbook.rs` | NX-VI.1: `Workbook::locale` (`serde(skip)`, không lưu vào file) | Nên PR upstream |
| `crates/formula/src/input.rs` | NX-VI.1: `FormulaLocale::Vi` — `;`, dấu phẩy thập phân, tên hàm tiếng Anh | Nên PR upstream |
| `crates/functions/src/lib.rs` | NX-VI.1: `Ctx::locale` | Nên PR upstream |
| `crates/functions/src/text.rs` | NX-VI.1: VALUE/NUMBERVALUE đọc theo vùng | Nên PR upstream |
| `crates/calc/src/eval.rs` | NX-VI.1: TEXT và `Ctx::locale` theo vùng của workbook | Nên PR upstream |
| `crates/xlsx/src/lib.rs` | NX-VI.1: `CsvOptions::locale` | Nên PR upstream |
| `crates/xlsx/src/csv.rs` | NX-VI.1: đọc CSV theo vùng, `sniff_delimiter_in` (vi-VN: `;`) | Nên PR upstream |
| `crates/engine/src/lib.rs` | NX-VI.1: `Session::locale`, `Session::set_locale` (tính lại, giữ trạng thái đã lưu), `add_document` nhận vùng | Nên PR upstream |
| `crates/engine/src/display.rs` | NX-VI.1: hiển thị và chữ khi sửa ô theo vùng của workbook | Nên PR upstream |
| `crates/engine/src/io.rs` | NX-VI.1: `OpenContext`/`open_bytes_in`; CSV vi-VN (`;`, dấu phẩy thập phân) | Nên PR upstream |
| `crates/engine/src/cmd/edit.rs` | NX-VI.1: `cell.set`/`range.fill` nhận `locale` | Nên PR upstream |
| `crates/engine/src/cmd/data.rs` | NX-VI.1: `check_validation_in` | Nên PR upstream |
| `crates/engine/src/cmd/file.rs` | NX-VI.1: `file.open` đọc CSV theo vùng | Nên PR upstream |
| `crates/engine/src/cmd/print.rs` | NX-VI.1: số General khi in theo vùng | Nên PR upstream |
| `crates/ui-egui/src/i18n.rs` | NX-VI.1: `Language::regional_format`, `regional_format_of`, `FormulaLocale::Vi` cho tiếng Việt | Một phần nên PR upstream |
| `crates/ui-egui/src/formula_locale.rs` | NX-VI.1: giá trị tiếng Việt chuyển nguyên cho engine đọc theo vi-VN | Nên PR upstream |
| `crates/ui-egui/src/lib.rs` | NX-VI.1: đặt vùng mỗi khung hình; `cell.set` gửi `locale`; Enter sau chuỗi Tab về cột đầu (Excel, `TabReturn`); `keep_keyboard` huỷ việc egui chuyển focus khi lưới/ô sửa vừa nhận bàn phím | Nên PR upstream |
| `crates/ui-egui/src/formula_bar.rs` | NX-VI.1: sửa lỗi gõ ký tự rồi Tab ở khung hình kế tiếp làm egui chuyển focus và mất phím; phím mũi tên kết thúc chuỗi Tab | Nên PR upstream (sửa lỗi) |
| `crates/ui-egui/tests/interaction.rs` | NX-VI.1: test điều hướng bàn phím đổi kỳ vọng — Enter sau Tab về cột bắt đầu (B5, như Excel) thay vì đi thẳng xuống (C5) | Nên PR upstream cùng thay đổi |
| `crates/ui-egui/tests/nexus_embed.rs` | NX-VI.1: test gõ một hàng bằng Tab/Enter kỳ vọng hàng sau bắt đầu ở cột A | — (test NexusOffice) |
| `crates/ui-egui/src/grid.rs` | NX-VI.1: số General trong lưới theo vùng; nhớ id widget lưới (`focus_id`) cho `keep_keyboard` | Nên PR upstream |
| `crates/ui-egui/src/ribbon.rs` | NX-VI.1: Currency trong danh sách Number Format = đồng khi tiếng Việt (`number_format_code`) | Có — tiền tệ mặc định theo vùng |
| `crates/ui-egui/src/nexus_toolbar.rs` | NX-VI.1: Currency = đồng khi tiếng Việt | Có |
| `crates/core/src/locale.rs` (thêm NX-VI.4) | `coercion_locale`/`coercion_scope`: vùng mà phép ép kiểu chữ ↔ số dùng trong lúc tính công thức (thread-local có guard, mặc định en-US) | — (file của fork) |
| `crates/core/src/value.rs` | NX-VI.4: `Value::to_number_in` đọc chữ theo `coercion_locale` (`="1,5"+1`), `Value::to_text` viết số theo nó (`=1.5&""` → `1,5`) | Nên PR upstream |
| `crates/core/src/parse.rs` (thêm NX-VI.4) | `parse_number_text` theo `coercion_locale` (tiêu chí COUNTIF…, D-functions, COUNT), `coerce_number_text` | Nên PR upstream |
| `crates/core/src/lib.rs` (thêm NX-VI.4) | xuất `coercion_locale`, `coercion_scope` | Nên PR upstream |
| `crates/functions/src/date.rs` | NX-VI.4: DATEVALUE/TIMEVALUE đọc chữ theo vùng của workbook (`25/10/2026`) | Nên PR upstream |
| `crates/calc/src/eval.rs` (thêm NX-VI.4) | `Evaluator::eval` ngoài cùng đặt `coercion_scope(workbook.locale)` | Nên PR upstream |
| `crates/engine/src/lib.rs` (thêm NX-VI.4) | `locale_sensitive` = sổ có công thức bất kỳ (ép kiểu/tiêu chí phụ thuộc vùng) → đổi vùng/mở file ở vi-VN tính lại mọi sổ có công thức | Nên PR upstream |
| `crates/numfmt/src/parse.rs` (thêm NX-VI.4) | `respell_code` chung cho `delocalize_code` và `localize_code_separators` (mới); số trong điều kiện `[<1000,5]` cũng đổi dấu | Nên PR upstream |
| `crates/numfmt/src/lib.rs` (thêm NX-VI.4) | `localize_code` (mã hiển thị trong Format Cells: dấu tách vi-VN, ngày hệ thống 14/22 → `dd/mm/yyyy`); xuất `localize_code_separators` | Nên PR upstream |
| `crates/chart/src/lib.rs` | NX-VI.4: `ChartData::locale` (`serde(skip)`) | Nên PR upstream |
| `crates/chart/src/resolve.rs` | NX-VI.4: `resolve` đặt `locale = wb.locale`; nhãn hạng mục ngày/số và tên chuỗi dạng số theo vùng | Nên PR upstream |
| `crates/chart/src/axis.rs` | NX-VI.4: `Fmt::new_in(code, locale)` — nhãn trục/nhãn dữ liệu theo vùng (General đổi dấu thập phân) | Nên PR upstream |
| `crates/chart/src/render.rs` | NX-VI.4: 14 chỗ `Fmt::new` → `Fmt::new_in(.., data.locale)`; `sanitize` giữ `locale`; khoảng histogram tách bằng dấu danh sách (`[1,5; 2,5]`) | Nên PR upstream |
| `crates/chart/src/tests.rs` | NX-VI.4: literal `ChartData {..}` thêm `locale: Default::default()` (trường mới) | Đi cùng `lib.rs` |
| `crates/xlsx/src/csv.rs` (thêm NX-VI.4) | `sniff_delimiter_in` bỏ qua dòng tiêu đề đầu không có tab/`;`/`,` (`Report\nName,Note\nA,x; y` → `,`) | Nên PR upstream |
| `crates/xlsx/src/tests/csv_tests.rs` | NX-VI.3/NX-VI.4: test CSV vi-VN (sniffer, dòng tiêu đề, `;` lạc trong ô chữ) | Đi cùng `csv.rs` |
| `crates/ui-egui/src/nexus_region.rs` | File mới (NX-VI.4): Vùng tách khỏi ngôn ngữ giao diện — `UiState::regional_format`/`formula_locale`, lệnh `ui.region`, hàng File › Tùy chọn › Vùng, lưu `region` trong `ui.json` (đọc lỏng), `CodeEdit` (mã định dạng hiển thị theo vùng, lưu en-US), `category_code` (Currency = đồng ở vi-VN) | — (file mới) |
| `crates/ui-egui/src/lib.rs` (thêm NX-VI.4) | `pub mod nexus_region;`, `UiState::region`, lệnh `ui.region`, mỗi khung hình `set_locale(ui.regional_format())` và `nexus_region::set_current` | Có — hook vùng |
| `crates/ui-egui/src/dialogs.rs` (thêm NX-VI.4) | Format Cells › Number: mã/nút mẫu hiển thị theo vùng (`CodeEdit`), lưu en-US; Currency theo vùng; `ui.regional_format()` thay `language.regional_format()` | Nên PR upstream |
| `crates/ui-egui/src/backstage.rs` (thêm NX-VI.4) | Hàng "Vùng (số và ngày)" trong File › Tùy chọn › Chung | Có |
| `crates/ui-egui/src/control.rs` (thêm NX-VI.4) | `engine.commands` liệt kê `ui.region` | Có |
| `crates/ui-egui/src/formula_bar.rs`, `crates/ui-egui/src/grid.rs` (thêm NX-VI.4) | Cú pháp công thức theo vùng (`ui.formula_locale()`, `nexus_region::current`) thay theo ngôn ngữ | Có |
| `crates/ui-egui/examples/snapshot.rs` (thêm NX-VI.4) | Cờ `--select B5` (chọn ô trước khi chụp) và `--region vi-VN|en-US` (vùng tách khỏi ngôn ngữ) — chụp được Format Cells/biểu đồ tiếng Việt mà không cần JSON | Có |
| `crates/ui-egui/src/ribbon.rs`, `crates/ui-egui/src/nexus_toolbar.rs` (thêm NX-VI.4) | Currency/nút kế toán đồng theo vùng thay theo ngôn ngữ (`number_format_code(region, …)`) | Có |

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
- Control channel (`ui.state`) báo `message.title` = "Nexus Sheet" nhưng tiêu đề dialog `About`/`start`
  trong JSON vẫn là khoá tiếng Anh ("About GridCraft"), chỉ phần hiển thị được brand.
- Gói FreeBSD (`packaging/freebsd/package.sh`) và Flatpak (`packaging/linux/flatpak*`) chưa đổi — chưa
  phát hành (workflow freebsd tắt theo spec §4.6).
- DMG macOS: `dmg-layout.DS_Store`/`background.tiff` gắn với volume "GridCraft" → DMG Nexus mở với
  layout mặc định của Finder; sinh lại trên macOS (Task 12).
- `xtask ico` test và `packaging/icons.sh` vẫn trỏ icon upstream `assets/app-icon/` (công cụ dev).
