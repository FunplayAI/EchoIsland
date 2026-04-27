use tauri::AppHandle;

use crate::macos_shared_expanded_window;
use crate::native_panel_renderer::facade::host::{
    NativePanelHostDisplayReposition, NativePanelRuntimeHostController,
    hide_native_panel_via_host_controller,
    reposition_native_panel_host_from_input_descriptor_via_controller,
    set_native_panel_host_shared_body_height_via_controller,
};

use super::{
    panel_entry::create_native_island_panel,
    panel_host_commands::{
        current_native_panel_window_frame, order_out_native_panel_window_with_app,
        reposition_native_panel_window_with_app, sync_order_out_in_runtime_state,
        sync_reposition_in_runtime_state,
    },
    panel_runtime_input::native_panel_runtime_input_descriptor,
    panel_snapshot::set_shared_expanded_body_height as set_shared_expanded_body_height_via_snapshot,
};

pub(super) fn hide_native_panel_with_host_controller<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    with_macos_native_panel_host_controller(app, hide_native_panel_via_host_controller)
}

pub(super) fn reposition_native_panel_to_selected_display_with_host_controller<
    R: tauri::Runtime,
>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    let input = native_panel_runtime_input_descriptor();
    with_macos_native_panel_host_controller(app, |controller| {
        reposition_native_panel_host_from_input_descriptor_via_controller(controller, &input)
    })
}

pub(super) fn set_shared_expanded_body_height_with_host_controller<R: tauri::Runtime>(
    app: &AppHandle<R>,
    body_height: f64,
) -> Result<(), String> {
    with_macos_native_panel_host_controller(app, |controller| {
        set_native_panel_host_shared_body_height_via_controller(controller, body_height)
    })
}

fn with_macos_native_panel_host_controller<R, T>(
    app: &AppHandle<R>,
    f: impl FnOnce(&mut MacosNativePanelHostController<R>) -> Result<T, String>,
) -> Result<T, String>
where
    R: tauri::Runtime,
{
    let mut controller = MacosNativePanelHostController::new(app.clone());
    f(&mut controller)
}

struct MacosNativePanelHostController<R: tauri::Runtime> {
    app: AppHandle<R>,
}

impl<R: tauri::Runtime> MacosNativePanelHostController<R> {
    fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: tauri::Runtime> NativePanelRuntimeHostController for MacosNativePanelHostController<R> {
    type Error = String;

    fn runtime_host_create_panel(&mut self) -> Result<(), Self::Error> {
        create_native_island_panel()
    }

    fn runtime_host_hide_panel(&mut self) -> Result<(), Self::Error> {
        let _ = macos_shared_expanded_window::hide_shared_expanded_window(&self.app);
        order_out_native_panel_window_with_app(&self.app, || {
            sync_order_out_in_runtime_state(|_| {})
        })
    }

    fn runtime_host_reposition(
        &mut self,
        reposition: NativePanelHostDisplayReposition,
    ) -> Result<(), Self::Error> {
        reposition_native_panel_window_with_app(
            &self.app,
            reposition,
            || unsafe { current_native_panel_window_frame() },
            |reposition| sync_reposition_in_runtime_state(reposition, |_| {}),
        )
    }

    fn runtime_host_set_shared_body_height(&mut self, body_height: f64) -> Result<(), Self::Error> {
        set_shared_expanded_body_height_via_snapshot(&self.app, body_height)
    }
}
