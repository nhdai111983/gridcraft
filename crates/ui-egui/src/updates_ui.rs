//! One background check at startup (and on Help ▸ Check for Updates…) and the modal prompt that
//! offers the new release: Update and Restart (direct self-update), Download or Later. The flow
//! itself is `craft_update::updater::Updater`.

use craft_update::suite::GRIDCRAFT;
use craft_update::updater::{Action, Status, Updater};

use crate::SheetApp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) struct Runtime {
    updater: Updater,
    /// The check at start was asked for; `tick` starts it with the context.
    startup: bool,
    /// Help ▸ Check for Updates… was chosen; the next `tick` starts it.
    manual: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { updater: Updater::new(&GRIDCRAFT, VERSION), startup: false, manual: false }
    }
}

/// NexusOffice: Nexus Sheet is updated by the NexusOffice installer, never on its own, so it
/// never asks GridCraft's releases (no network call, no self-update into GridCraft). Help ▸ Check
/// for Updates… says where updates come from instead.
pub(crate) fn start(app: &mut SheetApp, manual: bool) {
    if manual {
        app.message = Some((crate::nexus::APP_NAME.into(), crate::nexus::updates_text(app.ui.language).into()));
    }
}

pub(crate) fn tick(app: &mut SheetApp, ctx: &egui::Context) {
    let rt = &mut app.updates_rt;
    if std::mem::take(&mut rt.manual) {
        rt.updater.check_now(ctx);
    } else if rt.startup {
        rt.updater.check_at_start(ctx);
    }
    rt.updater.tick(ctx);
    let lang = app.ui.language;
    let outcome = match rt.updater.status() {
        Status::UpToDate => Some(lang.tr("GridCraft {version} is up to date.").replace("{version}", VERSION)),
        Status::Failed(e) => Some(lang.tr("Could not check for updates: {error}").replace("{error}", e)),
        Status::Idle | Status::Checking => None,
    };
    if let Some(message) = outcome {
        rt.updater.dismiss_status();
        app.message = Some((crate::nexus::APP_NAME.into(), message));
    }
}

pub(crate) fn show(app: &mut SheetApp, ctx: &egui::Context) {
    // The restart helper is scheduled before the app closes, so no update starts over unsaved work.
    let clean = !app.session.documents().iter().any(|d| d.is_dirty());
    match app.updates_rt.updater.show_if(ctx, clean) {
        Action::OpenUrl(url) => {
            if let Some(open) = &app.services.open_url {
                open(&url);
            }
        }
        // The guarded close path asks about unsaved work; the restart helper relaunches once the
        // process has exited.
        Action::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        Action::None => {}
    }
}
