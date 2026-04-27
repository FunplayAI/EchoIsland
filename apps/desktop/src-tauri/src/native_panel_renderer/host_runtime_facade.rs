use tauri::AppHandle;

use crate::native_panel_core::PanelRect;

use super::descriptors::{
    NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPointerRegion,
    NativePanelRuntimeInputDescriptor, NativePanelTimelineDescriptor,
    native_panel_host_window_frame, sync_native_panel_host_window_screen_frame,
    sync_native_panel_host_window_shared_body_height, sync_native_panel_host_window_timeline,
    sync_native_panel_host_window_visibility,
};

pub(crate) trait NativePanelRuntimeHostState {
    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor;

    fn pointer_regions_mut(&mut self) -> &mut Vec<NativePanelPointerRegion>;
}

pub(crate) trait NativePanelRuntimeHostController {
    type Error;

    fn runtime_host_create_panel(&mut self) -> Result<(), Self::Error>;

    fn runtime_host_hide_panel(&mut self) -> Result<(), Self::Error>;

    fn runtime_host_reposition(
        &mut self,
        reposition: NativePanelHostDisplayReposition,
    ) -> Result<(), Self::Error>;

    fn runtime_host_set_shared_body_height(&mut self, body_height: f64) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelHostDisplayReposition {
    pub(crate) preferred_display_index: usize,
    pub(crate) screen_frame: Option<PanelRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativePanelRuntimeHostCommand {
    Create,
    Hide,
    Reposition(NativePanelHostDisplayReposition),
    SetSharedBodyHeight(f64),
}

pub(crate) fn native_panel_host_display_reposition(
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
) -> NativePanelHostDisplayReposition {
    NativePanelHostDisplayReposition {
        preferred_display_index,
        screen_frame,
    }
}

pub(crate) fn native_panel_host_display_reposition_from_input_descriptor(
    input: &NativePanelRuntimeInputDescriptor,
) -> NativePanelHostDisplayReposition {
    native_panel_host_display_reposition(input.selected_display_index(), input.screen_frame)
}

pub(crate) fn native_panel_runtime_host_reposition_command_from_input_descriptor(
    input: &NativePanelRuntimeInputDescriptor,
) -> NativePanelRuntimeHostCommand {
    NativePanelRuntimeHostCommand::Reposition(
        native_panel_host_display_reposition_from_input_descriptor(input),
    )
}

pub(crate) fn execute_native_panel_runtime_host_command<H>(
    host: &mut H,
    command: NativePanelRuntimeHostCommand,
) -> Result<(), H::Error>
where
    H: NativePanelRuntimeHostController,
{
    match command {
        NativePanelRuntimeHostCommand::Create => host.runtime_host_create_panel(),
        NativePanelRuntimeHostCommand::Hide => host.runtime_host_hide_panel(),
        NativePanelRuntimeHostCommand::Reposition(reposition) => {
            host.runtime_host_reposition(reposition)
        }
        NativePanelRuntimeHostCommand::SetSharedBodyHeight(body_height) => {
            host.runtime_host_set_shared_body_height(body_height)
        }
    }
}

pub(crate) fn create_native_panel_via_host_controller<H>(host: &mut H) -> Result<(), H::Error>
where
    H: NativePanelRuntimeHostController,
{
    execute_native_panel_runtime_host_command(host, NativePanelRuntimeHostCommand::Create)
}

pub(crate) fn hide_native_panel_via_host_controller<H>(host: &mut H) -> Result<(), H::Error>
where
    H: NativePanelRuntimeHostController,
{
    execute_native_panel_runtime_host_command(host, NativePanelRuntimeHostCommand::Hide)
}

pub(crate) fn reposition_native_panel_host_from_input_descriptor_via_controller<H>(
    host: &mut H,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<(), H::Error>
where
    H: NativePanelRuntimeHostController,
{
    execute_native_panel_runtime_host_command(
        host,
        native_panel_runtime_host_reposition_command_from_input_descriptor(input),
    )
}

pub(crate) fn set_native_panel_host_shared_body_height_via_controller<H>(
    host: &mut H,
    body_height: f64,
) -> Result<(), H::Error>
where
    H: NativePanelRuntimeHostController,
{
    execute_native_panel_runtime_host_command(
        host,
        NativePanelRuntimeHostCommand::SetSharedBodyHeight(body_height),
    )
}

pub(crate) fn sync_native_panel_host_display_reposition(
    descriptor: &mut NativePanelHostWindowDescriptor,
    reposition: NativePanelHostDisplayReposition,
) {
    sync_native_panel_host_window_screen_frame(
        descriptor,
        reposition.preferred_display_index,
        reposition.screen_frame,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelRuntimeDispatchMode {
    Scheduled,
    Immediate,
}

pub(crate) fn dispatch_native_panel_runtime_with_handles<R, H, P, W>(
    app: &AppHandle<R>,
    handles: Option<H>,
    mode: NativePanelRuntimeDispatchMode,
    payload: P,
    work: W,
    dispatch_scheduled: impl FnOnce(&AppHandle<R>, H, P, W) -> Result<(), String>,
    dispatch_immediate: impl FnOnce(AppHandle<R>, H, P, W),
) -> Result<(), String>
where
    R: tauri::Runtime,
    H: Copy,
{
    let Some(handles) = handles else {
        return Ok(());
    };

    match mode {
        NativePanelRuntimeDispatchMode::Scheduled => {
            dispatch_scheduled(app, handles, payload, work)
        }
        NativePanelRuntimeDispatchMode::Immediate => {
            dispatch_immediate(app.clone(), handles, payload, work);
            Ok(())
        }
    }
}

pub(crate) fn dispatch_native_panel_runtime_payload_with_handles<R, H, P>(
    app: &AppHandle<R>,
    handles: Option<H>,
    mode: NativePanelRuntimeDispatchMode,
    payload: P,
    dispatch_scheduled: impl FnOnce(&AppHandle<R>, H, P) -> Result<(), String>,
    dispatch_immediate: impl FnOnce(AppHandle<R>, H, P),
) -> Result<(), String>
where
    R: tauri::Runtime,
    H: Copy,
{
    let Some(handles) = handles else {
        return Ok(());
    };

    match mode {
        NativePanelRuntimeDispatchMode::Scheduled => dispatch_scheduled(app, handles, payload),
        NativePanelRuntimeDispatchMode::Immediate => {
            dispatch_immediate(app.clone(), handles, payload);
            Ok(())
        }
    }
}

pub(crate) fn sync_runtime_host_visibility_in_state<S>(state: &mut S, visible: bool)
where
    S: NativePanelRuntimeHostState,
{
    sync_native_panel_host_window_visibility(state.host_window_descriptor_mut(), visible);
}

pub(crate) fn sync_runtime_pointer_regions_in_state<S>(
    state: &mut S,
    regions: &[NativePanelPointerRegion],
) where
    S: NativePanelRuntimeHostState,
{
    *state.pointer_regions_mut() = regions.to_vec();
}

pub(crate) fn sync_runtime_host_screen_frame_in_state<S>(
    state: &mut S,
    preferred_display_index: usize,
    screen_frame: PanelRect,
) where
    S: NativePanelRuntimeHostState,
{
    sync_native_panel_host_display_reposition(
        state.host_window_descriptor_mut(),
        native_panel_host_display_reposition(preferred_display_index, Some(screen_frame)),
    );
}

pub(crate) fn sync_runtime_host_display_reposition_in_state<S>(
    state: &mut S,
    reposition: NativePanelHostDisplayReposition,
) where
    S: NativePanelRuntimeHostState,
{
    sync_native_panel_host_display_reposition(state.host_window_descriptor_mut(), reposition);
}

pub(crate) fn sync_runtime_host_shared_body_height_in_state<S>(
    state: &mut S,
    shared_body_height: Option<f64>,
) where
    S: NativePanelRuntimeHostState,
{
    sync_native_panel_host_window_shared_body_height(
        state.host_window_descriptor_mut(),
        shared_body_height,
    );
}

pub(crate) fn sync_runtime_host_timeline_in_state<S>(
    state: &mut S,
    descriptor: NativePanelTimelineDescriptor,
) where
    S: NativePanelRuntimeHostState,
{
    sync_native_panel_host_window_timeline(state.host_window_descriptor_mut(), Some(descriptor));
}

pub(crate) trait NativePanelComputedHostWindow {
    fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor;

    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor;

    fn host_window_last_frame_mut(&mut self) -> &mut Option<PanelRect>;

    fn host_window_frame(&self) -> Option<PanelRect>;

    fn set_host_window_created(&mut self);

    fn fallback_screen_frame(&self) -> PanelRect;

    fn compact_width(&self) -> f64;

    fn expanded_width(&self) -> f64;

    fn host_window_create(&mut self) {
        self.set_host_window_created();
    }

    fn host_window_show(&mut self) {
        self.host_window_create();
        sync_native_panel_host_window_visibility(self.host_window_descriptor_mut(), true);
    }

    fn host_window_hide(&mut self) {
        sync_native_panel_host_window_visibility(self.host_window_descriptor_mut(), false);
    }

    fn host_window_reposition_to_display(
        &mut self,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) {
        self.host_window_create();
        sync_native_panel_host_display_reposition(
            self.host_window_descriptor_mut(),
            native_panel_host_display_reposition(preferred_display_index, screen_frame),
        );
        self.refresh_host_window_frame_from_descriptor();
    }

    fn host_window_set_shared_body_height(&mut self, body_height: f64) {
        sync_native_panel_host_window_shared_body_height(
            self.host_window_descriptor_mut(),
            Some(body_height),
        );
    }

    fn host_window_apply_timeline_descriptor(&mut self, descriptor: NativePanelTimelineDescriptor) {
        self.host_window_create();
        sync_native_panel_host_window_timeline(self.host_window_descriptor_mut(), Some(descriptor));
        self.refresh_host_window_frame_from_descriptor();
    }

    fn refresh_host_window_frame_from_descriptor(&mut self) {
        let descriptor = self.host_window_descriptor();
        if descriptor.animation_descriptor().is_none() {
            return;
        }
        *self.host_window_last_frame_mut() = native_panel_host_window_frame(
            descriptor,
            descriptor
                .screen_frame
                .unwrap_or_else(|| self.fallback_screen_frame()),
            self.compact_width(),
            self.expanded_width(),
        );
    }

    fn computed_host_window_state(&self) -> NativePanelHostWindowState {
        self.host_window_descriptor()
            .window_state(self.host_window_frame())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelComputedHostWindow, NativePanelHostDisplayReposition,
        NativePanelRuntimeHostCommand, NativePanelRuntimeHostController,
        NativePanelRuntimeHostState, create_native_panel_via_host_controller,
        execute_native_panel_runtime_host_command, hide_native_panel_via_host_controller,
        native_panel_host_display_reposition,
        native_panel_host_display_reposition_from_input_descriptor,
        native_panel_runtime_host_reposition_command_from_input_descriptor,
        reposition_native_panel_host_from_input_descriptor_via_controller,
        set_native_panel_host_shared_body_height_via_controller,
        sync_runtime_host_display_reposition_in_state,
    };
    use crate::{
        native_panel_core::PanelRect,
        native_panel_renderer::descriptors::{
            NativePanelHostWindowDescriptor, NativePanelPointerRegion,
            NativePanelRuntimeInputDescriptor,
        },
        native_panel_scene::PanelSceneBuildInput,
    };

    #[derive(Default)]
    struct TestRuntimeHostState {
        descriptor: NativePanelHostWindowDescriptor,
        pointer_regions: Vec<NativePanelPointerRegion>,
    }

    impl NativePanelRuntimeHostState for TestRuntimeHostState {
        fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
            &mut self.descriptor
        }

        fn pointer_regions_mut(&mut self) -> &mut Vec<NativePanelPointerRegion> {
            &mut self.pointer_regions
        }
    }

    #[derive(Default)]
    struct TestComputedHostWindow {
        descriptor: NativePanelHostWindowDescriptor,
        last_frame: Option<PanelRect>,
        created: bool,
    }

    #[derive(Default)]
    struct TestHostController {
        create_count: usize,
        hide_count: usize,
        last_reposition: Option<NativePanelHostDisplayReposition>,
        last_shared_body_height: Option<f64>,
    }

    impl NativePanelComputedHostWindow for TestComputedHostWindow {
        fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
            self.descriptor
        }

        fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
            &mut self.descriptor
        }

        fn host_window_last_frame_mut(&mut self) -> &mut Option<PanelRect> {
            &mut self.last_frame
        }

        fn host_window_frame(&self) -> Option<PanelRect> {
            self.last_frame
        }

        fn set_host_window_created(&mut self) {
            self.created = true;
        }

        fn fallback_screen_frame(&self) -> PanelRect {
            PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            }
        }

        fn compact_width(&self) -> f64 {
            crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH
        }

        fn expanded_width(&self) -> f64 {
            crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH
        }
    }

    impl NativePanelRuntimeHostController for TestHostController {
        type Error = String;

        fn runtime_host_create_panel(&mut self) -> Result<(), Self::Error> {
            self.create_count += 1;
            Ok(())
        }

        fn runtime_host_hide_panel(&mut self) -> Result<(), Self::Error> {
            self.hide_count += 1;
            Ok(())
        }

        fn runtime_host_reposition(
            &mut self,
            reposition: NativePanelHostDisplayReposition,
        ) -> Result<(), Self::Error> {
            self.last_reposition = Some(reposition);
            Ok(())
        }

        fn runtime_host_set_shared_body_height(
            &mut self,
            body_height: f64,
        ) -> Result<(), Self::Error> {
            self.last_shared_body_height = Some(body_height);
            Ok(())
        }
    }

    #[test]
    fn display_reposition_payload_can_be_built_from_runtime_input_descriptor() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput {
                settings: crate::native_panel_core::PanelSettingsState {
                    selected_display_index: 2,
                    ..Default::default()
                },
                ..Default::default()
            },
            screen_frame: Some(PanelRect {
                x: 40.0,
                y: 50.0,
                width: 800.0,
                height: 600.0,
            }),
        };

        assert_eq!(
            native_panel_host_display_reposition_from_input_descriptor(&input),
            NativePanelHostDisplayReposition {
                preferred_display_index: 2,
                screen_frame: Some(PanelRect {
                    x: 40.0,
                    y: 50.0,
                    width: 800.0,
                    height: 600.0,
                }),
            }
        );
    }

    #[test]
    fn runtime_host_reposition_command_can_be_built_from_input_descriptor() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput {
                settings: crate::native_panel_core::PanelSettingsState {
                    selected_display_index: 4,
                    ..Default::default()
                },
                ..Default::default()
            },
            screen_frame: Some(PanelRect {
                x: 100.0,
                y: 120.0,
                width: 1024.0,
                height: 768.0,
            }),
        };

        assert_eq!(
            native_panel_runtime_host_reposition_command_from_input_descriptor(&input),
            NativePanelRuntimeHostCommand::Reposition(NativePanelHostDisplayReposition {
                preferred_display_index: 4,
                screen_frame: Some(PanelRect {
                    x: 100.0,
                    y: 120.0,
                    width: 1024.0,
                    height: 768.0,
                }),
            })
        );
    }

    #[test]
    fn runtime_host_state_can_sync_display_reposition_payload() {
        let mut state = TestRuntimeHostState::default();

        sync_runtime_host_display_reposition_in_state(
            &mut state,
            native_panel_host_display_reposition(
                3,
                Some(PanelRect {
                    x: 40.0,
                    y: 50.0,
                    width: 800.0,
                    height: 600.0,
                }),
            ),
        );

        assert_eq!(state.descriptor.preferred_display_index, 3);
        assert_eq!(
            state.descriptor.screen_frame,
            Some(PanelRect {
                x: 40.0,
                y: 50.0,
                width: 800.0,
                height: 600.0,
            })
        );
    }

    #[test]
    fn computed_host_window_reposition_consumes_shared_display_payload() {
        let mut window = TestComputedHostWindow::default();

        window.host_window_reposition_to_display(
            1,
            Some(PanelRect {
                x: 100.0,
                y: 100.0,
                width: 1440.0,
                height: 900.0,
            }),
        );

        assert!(window.created);
        assert_eq!(window.descriptor.preferred_display_index, 1);
        assert_eq!(
            window.descriptor.screen_frame,
            Some(PanelRect {
                x: 100.0,
                y: 100.0,
                width: 1440.0,
                height: 900.0,
            })
        );
    }

    #[test]
    fn host_controller_helpers_route_create_hide_reposition_and_height() {
        let mut controller = TestHostController::default();
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput {
                settings: crate::native_panel_core::PanelSettingsState {
                    selected_display_index: 2,
                    ..Default::default()
                },
                ..Default::default()
            },
            screen_frame: Some(PanelRect {
                x: 10.0,
                y: 20.0,
                width: 800.0,
                height: 600.0,
            }),
        };

        create_native_panel_via_host_controller(&mut controller).expect("create via helper");
        hide_native_panel_via_host_controller(&mut controller).expect("hide via helper");
        reposition_native_panel_host_from_input_descriptor_via_controller(&mut controller, &input)
            .expect("reposition via helper");
        set_native_panel_host_shared_body_height_via_controller(&mut controller, 240.0)
            .expect("shared body height via helper");

        assert_eq!(controller.create_count, 1);
        assert_eq!(controller.hide_count, 1);
        assert_eq!(
            controller.last_reposition,
            Some(NativePanelHostDisplayReposition {
                preferred_display_index: 2,
                screen_frame: Some(PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 800.0,
                    height: 600.0,
                }),
            })
        );
        assert_eq!(controller.last_shared_body_height, Some(240.0));
    }

    #[test]
    fn runtime_host_command_executor_routes_all_host_commands() {
        let mut controller = TestHostController::default();
        let reposition = NativePanelHostDisplayReposition {
            preferred_display_index: 5,
            screen_frame: Some(PanelRect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }),
        };

        execute_native_panel_runtime_host_command(
            &mut controller,
            NativePanelRuntimeHostCommand::Create,
        )
        .expect("create command");
        execute_native_panel_runtime_host_command(
            &mut controller,
            NativePanelRuntimeHostCommand::Hide,
        )
        .expect("hide command");
        execute_native_panel_runtime_host_command(
            &mut controller,
            NativePanelRuntimeHostCommand::Reposition(reposition),
        )
        .expect("reposition command");
        execute_native_panel_runtime_host_command(
            &mut controller,
            NativePanelRuntimeHostCommand::SetSharedBodyHeight(280.0),
        )
        .expect("height command");

        assert_eq!(controller.create_count, 1);
        assert_eq!(controller.hide_count, 1);
        assert_eq!(controller.last_reposition, Some(reposition));
        assert_eq!(controller.last_shared_body_height, Some(280.0));
    }
}
