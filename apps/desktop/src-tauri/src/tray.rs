use network_core::HealthReport;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};

pub struct TrayState {
    pub status_item: MenuItem<tauri::Wry>,
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let status_item = MenuItem::with_id(app, "status", "Status: checking…", false, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Open dashboard", true, None::<&str>)?;
    let check_item = MenuItem::with_id(app, "check_now", "Run check now", true, None::<&str>)?;
    let quit_item = PredefinedMenuItem::quit(app, Some("Quit"))?;
    let menu = Menu::with_items(app, &[&status_item, &show_item, &check_item, &quit_item])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("missing default window icon")?;

    TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(icon)
        .tooltip("Network Companion")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => crate::monitor::show_main_window(app),
            "check_now" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::monitor::perform_check(&handle, "manual_tray").await;
                });
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

    app.manage(TrayState { status_item });
    Ok(())
}

pub fn update_tray_status(app: &AppHandle, report: &HealthReport) {
    let grade = format!("{:?}", report.score.grade).to_uppercase();
    let tooltip = format!(
        "Network Companion — {} ({}/100)",
        report.score.summary, report.score.score
    );

    if let Some(tray_state) = app.try_state::<TrayState>() {
        let _ = tray_state
            .status_item
            .set_text(format!("Status: {grade} ({}/100)", report.score.score));
    }

    if let Some(tray_icon) = app.tray_by_id("main") {
        let _ = tray_icon.set_tooltip(Some(tooltip));
    }
}
