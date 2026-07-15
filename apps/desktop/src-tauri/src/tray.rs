use network_core::{load_protect_settings, HealthReport};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};

use crate::state::data_dir;

pub struct TrayState {
    pub status_item: MenuItem<tauri::Wry>,
    pub dnd_item: CheckMenuItem<tauri::Wry>,
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let initial_dnd = load_protect_settings(&data_dir())
        .map(|settings| settings.do_not_disturb)
        .unwrap_or(false);

    let status_item = MenuItem::with_id(app, "status", "Status: checking…", false, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Open dashboard", true, None::<&str>)?;
    let check_item = MenuItem::with_id(app, "check_now", "Run check now", true, None::<&str>)?;
    let dnd_item = CheckMenuItem::with_id(
        app,
        "toggle_dnd",
        "Do Not Disturb",
        true,
        initial_dnd,
        None::<&str>,
    )?;
    let quit_item = PredefinedMenuItem::quit(app, Some("Quit"))?;
    let menu = Menu::with_items(app, &[
        &status_item,
        &show_item,
        &check_item,
        &dnd_item,
        &quit_item,
    ])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("missing default window icon")?;

    TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(icon)
        .tooltip("KnotTrace")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => crate::monitor::show_main_window(app),
            "check_now" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::monitor::perform_check(&handle, "manual_tray").await;
                });
            }
            "toggle_dnd" => {
                if let Err(error) = crate::protect::toggle_do_not_disturb(app) {
                    tracing::warn!(target = "network_desktop::tray", "{error}");
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::monitor::show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(TrayState {
        status_item,
        dnd_item,
    });
    Ok(())
}

pub fn sync_tray_do_not_disturb(app: &AppHandle, enabled: bool) {
    if let Some(tray_state) = app.try_state::<TrayState>() {
        let _ = tray_state.dnd_item.set_checked(enabled);
    }
}

pub fn update_tray_status(app: &AppHandle, report: &HealthReport) {
    let grade = format!("{:?}", report.score.grade).to_uppercase();
    let do_not_disturb = load_protect_settings(&data_dir())
        .map(|settings| settings.do_not_disturb)
        .unwrap_or(false);
    let tooltip = format_tray_tooltip(do_not_disturb, &report.score.summary, report.score.score);

    if let Some(tray_state) = app.try_state::<TrayState>() {
        let _ = tray_state
            .status_item
            .set_text(format!("Status: {grade} ({}/100)", report.score.score));
        let _ = tray_state.dnd_item.set_checked(do_not_disturb);
    }

    if let Some(tray_icon) = app.tray_by_id("main") {
        let _ = tray_icon.set_tooltip(Some(tooltip));
    }
}

fn format_tray_tooltip(do_not_disturb: bool, summary: &str, score: u8) -> String {
    if do_not_disturb {
        format!("KnotTrace — DND · {summary} ({score}/100)")
    } else {
        format!("KnotTrace — {summary} ({score}/100)")
    }
}
