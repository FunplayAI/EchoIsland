#[cfg(target_os = "macos")]
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Position, Size, WebviewUrl};

#[cfg(target_os = "macos")]
use tauri::window::Color;

#[cfg(target_os = "macos")]
use tauri_nspanel::{CollectionBehavior, PanelBuilder, PanelLevel, StyleMask, tauri_panel};

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(CodeIslandPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

#[cfg(target_os = "macos")]
pub fn create_main_panel(app_handle: &AppHandle) -> tauri::Result<()> {
    if app_handle
        .get_webview_window(crate::constants::MAIN_WINDOW_LABEL)
        .is_some()
    {
        return Ok(());
    }

    let panel =
        PanelBuilder::<_, CodeIslandPanel>::new(app_handle, crate::constants::MAIN_WINDOW_LABEL)
            .url(WebviewUrl::App("index.html".into()))
            .title("CodeIsland")
            .position(Position::Logical(LogicalPosition::new(546.0, 0.0)))
            .size(Size::Logical(LogicalSize::new(420.0, 80.0)))
            .level(PanelLevel::Custom(26))
            .floating(true)
            .has_shadow(false)
            .opaque(false)
            .transparent(true)
            .hides_on_deactivate(false)
            .becomes_key_only_if_needed(false)
            .accepts_mouse_moved_events(true)
            .movable_by_window_background(false)
            .works_when_modal(true)
            .collection_behavior(
                CollectionBehavior::new()
                    .full_screen_auxiliary()
                    .can_join_all_spaces()
                    .stationary()
                    .ignores_cycle(),
            )
            .style_mask(
                StyleMask::empty()
                    .borderless()
                    .full_size_content_view()
                    .nonactivating_panel(),
            )
            .no_activate(true)
            .with_window(|window| {
                window
                    .decorations(false)
                    .hidden_title(true)
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .transparent(true)
                    .resizable(false)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .shadow(false)
                    .background_color(Color(0, 0, 0, 0))
            })
            .build()?;

    let native_panel = panel.as_panel();
    if let Some(content_view) = native_panel.contentView() {
        content_view.layoutSubtreeIfNeeded();
        content_view.displayIfNeeded();
    }
    native_panel.displayIfNeeded();

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn create_main_panel(_app_handle: &AppHandle) -> tauri::Result<()> {
    Ok(())
}
