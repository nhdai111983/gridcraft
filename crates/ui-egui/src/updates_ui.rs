//! One background check at startup (and on Help ▸ Check for Updates…) and the modal prompt that
//! offers the new release: Update and Restart (direct self-update), Download or Later. The flow
//! itself is `craft_update::updater::Updater`.

use craft_update::CheckResultResponse;
use craft_update::updater::{Action, Status, Updater};

use crate::SheetApp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) struct Runtime {
    updater: Updater,
    /// The check at start was asked for; `tick` starts it with the context.
    startup: bool,
    /// Help ▸ Check for Updates… was chosen; the next `tick` starts it.
    manual: bool,
    /// The result dialog of a requested check is showing.
    result_open: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { updater: Updater::new(&crate::UPDATE_APP, VERSION), startup: false, manual: false, result_open: false }
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
        rt.result_open = true;
    } else if rt.startup {
        rt.updater.check_at_start(ctx);
    }
    rt.updater.tick(ctx);
    // A requested check that found a release leaves the status idle and the prompt opens.
    if matches!(rt.updater.status(), Status::Idle) {
        rt.result_open = false;
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
    if app.updates_rt.result_open {
        let rt = &mut app.updates_rt;
        match craft_update::CheckResultDialog::new(&crate::UPDATE_APP, VERSION).show(ctx, rt.updater.status()) {
            CheckResultResponse::Close => {
                rt.updater.dismiss_status();
                rt.result_open = false;
            }
            CheckResultResponse::CheckAgain => rt.updater.check_now(ctx),
            CheckResultResponse::None => {}
        }
    }
}
