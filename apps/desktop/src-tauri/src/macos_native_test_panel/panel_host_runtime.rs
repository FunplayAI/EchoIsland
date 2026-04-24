use echoisland_runtime::RuntimeSnapshot;
use objc2_app_kit::NSPanel;
use objc2_foundation::NSRect;
use tauri::AppHandle;

use super::panel_host_adapter::MacosNativePanelHostAdapter;
use super::panel_host_descriptor::{
    sync_native_host_window_screen_frame, sync_native_host_window_shared_body_height,
    sync_native_host_window_timeline, sync_native_host_window_visibility,
};
use super::panel_refs::{native_panel_handles, native_panel_state, panel_from_ptr};
use super::panel_transition_entry::{
    begin_native_panel_surface_transition, begin_native_panel_transition,
};
use super::panel_types::{NativePanelHandles, NativePanelRenderPayload};
use crate::native_panel_renderer::{
    NativePanelHost, NativePanelHostWindowDescriptor, NativePanelHostWindowState,
    NativePanelPlatformEvent, NativePanelPointerRegion, NativePanelSceneHost,
    NativePanelTimelineDescriptor,
};

#[allow(dead_code)]
#[derive(Default)]
pub(super) struct MacosNativePanelRuntimeHost {
    pub(super) adapter: MacosNativePanelHostAdapter,
}

impl NativePanelHost for MacosNativePanelRuntimeHost {
    type Error = String;
    type Renderer = <MacosNativePanelHostAdapter as NativePanelHost>::Renderer;

    fn renderer(&mut self) -> &mut Self::Renderer {
        self.adapter.renderer()
    }

    fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
        self.adapter.host_window_descriptor()
    }

    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
        self.adapter.host_window_descriptor_mut()
    }

    fn window_state(&self) -> NativePanelHostWindowState {
        self.adapter.window_state()
    }

    fn show(&mut self) -> Result<(), Self::Error> {
        self.adapter.show()
    }

    fn hide(&mut self) -> Result<(), Self::Error> {
        self.adapter.hide()
    }

    fn sync_pointer_regions(
        &mut self,
        regions: &[NativePanelPointerRegion],
    ) -> Result<(), Self::Error> {
        self.adapter.sync_pointer_regions(regions)?;
        sync_runtime_pointer_regions(regions);
        Ok(())
    }

    fn take_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
        self.adapter.take_platform_events()
    }
}

impl NativePanelSceneHost for MacosNativePanelRuntimeHost {}

#[allow(dead_code)]
impl MacosNativePanelRuntimeHost {
    pub(super) fn capture() -> Option<Self> {
        let mut host = Self::default();
        host.refresh_from_runtime()?;
        Some(host)
    }

    pub(super) fn refresh_from_runtime(&mut self) -> Option<()> {
        let frame = current_native_panel_frame();
        let state = native_panel_state()?;
        let guard = state.lock().ok()?;
        self.sync_from_native_panel_state(&guard, frame);
        Some(())
    }

    pub(super) fn sync_from_native_panel_state(
        &mut self,
        state: &super::panel_types::NativePanelState,
        frame: Option<NSRect>,
    ) {
        self.adapter.sync_from_native_panel_state(state, frame);
    }

    pub(super) unsafe fn with_window<T>(
        f: impl FnOnce(NativePanelHandles, &'static NSPanel) -> T,
    ) -> Option<T> {
        let handles = current_runtime_panel_handles()?;
        Some(f(handles, unsafe { panel_from_ptr(handles.panel) }))
    }

    pub(super) unsafe fn order_out() -> Option<()> {
        unsafe {
            Self::with_window(|_, panel| {
                let _ = sync_runtime_host_visibility(false);
                panel.orderOut(None);
            })
        }
    }

    pub(super) unsafe fn reposition_to_screen(
        preferred_display_index: usize,
        screen_frame: NSRect,
        centered_frame: NSRect,
    ) -> Option<()> {
        unsafe {
            Self::with_window(|_, panel| {
                panel.setFrame_display(centered_frame, true);
                let _ = sync_runtime_host_visibility(true);
                let _ = sync_runtime_host_screen_frame(preferred_display_index, screen_frame);
            })
        }
    }

    pub(super) unsafe fn current_window_frame() -> Option<NSRect> {
        unsafe { Self::with_window(|_, panel| panel.frame()) }
    }

    pub(super) fn rerender_from_last_snapshot<R: tauri::Runtime>(
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        let Some(handles) = current_runtime_panel_handles() else {
            return Ok(());
        };
        let Some(payload) = current_runtime_panel_render_payload() else {
            return Ok(());
        };
        dispatch_runtime_panel_render_payload(app, handles, payload)
    }

    pub(super) fn apply_render_payload<R: tauri::Runtime>(
        app: &AppHandle<R>,
        payload: NativePanelRenderPayload,
    ) -> Result<(), String> {
        let Some(handles) = current_runtime_panel_handles() else {
            return Ok(());
        };
        dispatch_runtime_panel_render_payload(app, handles, payload)
    }

    pub(super) fn begin_transition<R: tauri::Runtime>(
        app: &AppHandle<R>,
        snapshot: RuntimeSnapshot,
        expanded: bool,
    ) -> Result<(), String> {
        let Some(handles) = current_runtime_panel_handles() else {
            return Ok(());
        };
        let app_for_transition = app.clone();
        app.run_on_main_thread(move || unsafe {
            begin_native_panel_transition(app_for_transition, handles, snapshot, expanded);
        })
        .map_err(|error| error.to_string())
    }

    pub(super) fn begin_surface_transition<R: tauri::Runtime>(
        app: &AppHandle<R>,
        snapshot: RuntimeSnapshot,
    ) -> Result<(), String> {
        let Some(handles) = current_runtime_panel_handles() else {
            return Ok(());
        };
        let app_for_transition = app.clone();
        app.run_on_main_thread(move || unsafe {
            begin_native_panel_surface_transition(app_for_transition, handles, snapshot);
        })
        .map_err(|error| error.to_string())
    }
}

pub(super) fn with_native_runtime_panel_state_mut<T>(
    f: impl FnOnce(&mut super::panel_types::NativePanelState) -> T,
) -> Option<T> {
    native_panel_state()
        .and_then(|state_mutex| state_mutex.lock().ok().map(|mut state| f(&mut state)))
}

pub(super) fn current_runtime_panel_handles() -> Option<NativePanelHandles> {
    native_panel_handles()
}

pub(super) fn current_runtime_panel_render_payload() -> Option<NativePanelRenderPayload> {
    let mut host = MacosNativePanelRuntimeHost::capture()?;
    native_panel_state().and_then(|state_mutex| {
        state_mutex.lock().ok().and_then(|state| {
            current_runtime_panel_render_payload_for_state_and_host(&state, &mut host)
        })
    })
}

pub(super) fn dispatch_runtime_panel_render_payload<R: tauri::Runtime>(
    app: &AppHandle<R>,
    handles: NativePanelHandles,
    payload: NativePanelRenderPayload,
) -> Result<(), String> {
    app.run_on_main_thread(move || unsafe {
        crate::macos_native_test_panel::panel_snapshot::apply_native_panel_render_payload(
            handles, payload,
        );
    })
    .map_err(|error| error.to_string())
}

pub(super) fn rerender_runtime_panel_from_last_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    MacosNativePanelRuntimeHost::rerender_from_last_snapshot(app)
}

pub(super) fn sync_runtime_host_visibility(visible: bool) -> Option<()> {
    with_native_runtime_panel_state_mut(|state| {
        sync_runtime_host_visibility_in_state(state, visible);
    })
}

pub(super) fn sync_runtime_host_visibility_in_state(
    state: &mut super::panel_types::NativePanelState,
    visible: bool,
) {
    sync_native_host_window_visibility(state, visible);
}

pub(super) fn sync_runtime_pointer_regions(regions: &[NativePanelPointerRegion]) -> Option<()> {
    with_native_runtime_panel_state_mut(|state| {
        sync_runtime_pointer_regions_in_state(state, regions);
    })
}

pub(super) fn sync_runtime_pointer_regions_in_state(
    state: &mut super::panel_types::NativePanelState,
    regions: &[NativePanelPointerRegion],
) {
    state.pointer_regions = regions.to_vec();
}

pub(super) fn sync_runtime_host_screen_frame(
    preferred_display_index: usize,
    screen_frame: NSRect,
) -> Option<()> {
    with_native_runtime_panel_state_mut(|state| {
        sync_runtime_host_screen_frame_in_state(state, preferred_display_index, screen_frame);
    })
}

pub(super) fn sync_runtime_host_screen_frame_in_state(
    state: &mut super::panel_types::NativePanelState,
    preferred_display_index: usize,
    screen_frame: NSRect,
) {
    sync_native_host_window_screen_frame(state, preferred_display_index, screen_frame);
}

#[allow(dead_code)]
pub(super) fn sync_runtime_host_shared_body_height(shared_body_height: Option<f64>) -> Option<()> {
    with_native_runtime_panel_state_mut(|state| {
        sync_runtime_host_shared_body_height_in_state(state, shared_body_height);
    })
}

pub(super) fn sync_runtime_host_shared_body_height_in_state(
    state: &mut super::panel_types::NativePanelState,
    shared_body_height: Option<f64>,
) {
    sync_native_host_window_shared_body_height(state, shared_body_height);
}

#[allow(dead_code)]
pub(super) fn sync_runtime_host_timeline(descriptor: NativePanelTimelineDescriptor) -> Option<()> {
    with_native_runtime_panel_state_mut(|state| {
        sync_runtime_host_timeline_in_state(state, descriptor);
    })
}

pub(super) fn sync_runtime_host_timeline_in_state(
    state: &mut super::panel_types::NativePanelState,
    descriptor: NativePanelTimelineDescriptor,
) {
    sync_native_host_window_timeline(state, descriptor);
}

fn current_native_panel_frame() -> Option<NSRect> {
    unsafe { MacosNativePanelRuntimeHost::current_window_frame() }
}

fn current_runtime_panel_render_payload_for_state_and_host(
    state: &super::panel_types::NativePanelState,
    host: &mut MacosNativePanelRuntimeHost,
) -> Option<NativePanelRenderPayload> {
    let snapshot = host.adapter.renderer.scene_cache.last_snapshot.clone()?;
    Some(NativePanelRenderPayload {
        snapshot,
        expanded: state.expanded,
        shared_body_height: state.shared_body_height,
        transitioning: state.transitioning,
        transition_cards_progress: state.transition_cards_progress,
        transition_cards_entering: state.transition_cards_entering,
    })
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::MacosNativePanelRuntimeHost;
    use crate::{
        macos_native_test_panel::{
            mascot::NativeMascotRuntime, panel_types::NativeExpandedSurface,
        },
        native_panel_renderer::{
            NativePanelHost, NativePanelHostWindowDescriptor, NativePanelTimelineDescriptor,
        },
    };

    fn panel_state() -> crate::macos_native_test_panel::panel_types::NativePanelState {
        crate::macos_native_test_panel::panel_types::NativePanelState {
            expanded: false,
            transitioning: false,
            transition_cards_progress: 0.0,
            transition_cards_entering: false,
            skip_next_close_card_exit: false,
            last_raw_snapshot: None,
            last_snapshot: Some(echoisland_runtime::RuntimeSnapshot {
                status: "idle".to_string(),
                primary_source: "codex".to_string(),
                active_session_count: 0,
                total_session_count: 0,
                pending_permission_count: 0,
                pending_question_count: 0,
                pending_permission: None,
                pending_question: None,
                pending_permissions: vec![],
                pending_questions: vec![],
                sessions: vec![],
            }),
            scene_cache: crate::native_panel_renderer::NativePanelRuntimeSceneCache::default(),
            status_queue: Vec::new(),
            completion_badge_items: Vec::new(),
            pending_permission_card: None,
            pending_question_card: None,
            status_auto_expanded: false,
            surface_mode: NativeExpandedSurface::Default,
            shared_body_height: None,
            host_window_descriptor: NativePanelHostWindowDescriptor {
                visible: true,
                preferred_display_index: 1,
                screen_frame: None,
                shared_body_height: Some(140.0),
                timeline: Some(NativePanelTimelineDescriptor {
                    animation: crate::native_panel_core::PanelAnimationDescriptor {
                        kind: crate::native_panel_core::PanelAnimationKind::Open,
                        canvas_height: 180.0,
                        visible_height: 150.0,
                        width_progress: 0.5,
                        height_progress: 0.75,
                        shoulder_progress: 1.0,
                        drop_progress: 0.3,
                        cards_progress: 0.9,
                    },
                    cards_entering: true,
                }),
            },
            pointer_inside_since: None,
            pointer_outside_since: None,
            primary_mouse_down: false,
            last_focus_click: None,
            pointer_regions: Vec::new(),
            mascot_runtime: NativeMascotRuntime::new(Instant::now()),
        }
    }

    #[test]
    fn runtime_host_projects_native_state_into_host_trait_shape() {
        let state = panel_state();
        let mut host = MacosNativePanelRuntimeHost::default();

        host.sync_from_native_panel_state(
            &state,
            Some(NSRect::new(
                NSPoint::new(20.0, 30.0),
                NSSize::new(320.0, 240.0),
            )),
        );

        assert_eq!(host.host_window_descriptor(), state.host_window_descriptor);
        assert_eq!(
            host.adapter
                .renderer
                .scene_cache
                .last_snapshot
                .as_ref()
                .map(|snapshot| snapshot.status.as_str()),
            Some("idle")
        );
        assert_eq!(
            host.window_state(),
            state
                .host_window_descriptor
                .window_state(Some(crate::native_panel_core::PanelRect {
                    x: 20.0,
                    y: 30.0,
                    width: 320.0,
                    height: 240.0,
                }))
        );
    }

    #[test]
    fn runtime_host_prefers_shared_descriptor_frame_when_available() {
        let mut state = panel_state();
        state.host_window_descriptor.screen_frame = Some(crate::native_panel_core::PanelRect {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        });
        let mut host = MacosNativePanelRuntimeHost::default();

        host.sync_from_native_panel_state(
            &state,
            Some(NSRect::new(
                NSPoint::new(20.0, 30.0),
                NSSize::new(320.0, 240.0),
            )),
        );

        assert_eq!(
            host.window_state(),
            state
                .host_window_descriptor
                .window_state(Some(crate::native_panel_core::PanelRect {
                    x: 586.0,
                    y: 720.0,
                    width: 268.0,
                    height: 180.0,
                }))
        );
    }

    #[test]
    fn runtime_host_syncs_descriptor_into_renderer_cache() {
        let state = panel_state();
        let mut host = MacosNativePanelRuntimeHost::default();
        host.sync_from_native_panel_state(&state, None);

        host.show().expect("show runtime host");

        assert_eq!(
            host.adapter.renderer.last_host_window_descriptor,
            Some(state.host_window_descriptor)
        );
        assert_eq!(
            host.adapter.renderer.last_host_window_state,
            Some(state.host_window_descriptor.window_state(None))
        );
        assert_eq!(
            host.adapter.renderer.last_timeline_descriptor,
            state.host_window_descriptor.timeline
        );
        assert!(host.adapter.renderer.visible);
    }

    #[test]
    fn runtime_host_syncs_pointer_regions_into_native_state() {
        let mut state = panel_state();
        let mut host = MacosNativePanelRuntimeHost::default();
        host.sync_from_native_panel_state(&state, None);
        let regions = vec![crate::native_panel_renderer::NativePanelPointerRegion {
            frame: crate::native_panel_core::PanelRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            },
            kind: crate::native_panel_renderer::NativePanelPointerRegionKind::CompactBar,
        }];

        host.sync_pointer_regions(&regions)
            .expect("sync pointer regions into runtime host");
        super::sync_runtime_pointer_regions_in_state(&mut state, &regions);

        assert_eq!(host.adapter.renderer.last_pointer_regions, regions);
        assert_eq!(
            host.adapter.renderer.last_pointer_regions,
            state.pointer_regions
        );
    }

    #[test]
    fn runtime_host_state_helper_updates_descriptor_fields() {
        let mut state = panel_state();

        super::sync_native_host_window_visibility(&mut state, false);
        super::sync_native_host_window_shared_body_height(&mut state, Some(222.0));
        super::sync_native_host_window_screen_frame(
            &mut state,
            3,
            NSRect::new(NSPoint::new(40.0, 50.0), NSSize::new(800.0, 600.0)),
        );
        super::sync_native_host_window_timeline(
            &mut state,
            crate::native_panel_renderer::native_panel_timeline_descriptor(
                crate::native_panel_core::PanelAnimationDescriptor {
                    kind: crate::native_panel_core::PanelAnimationKind::Close,
                    canvas_height: 160.0,
                    visible_height: 90.0,
                    width_progress: 0.2,
                    height_progress: 0.3,
                    shoulder_progress: 0.4,
                    drop_progress: 0.1,
                    cards_progress: 0.0,
                },
                false,
            ),
        );

        assert!(!state.host_window_descriptor.visible);
        assert_eq!(state.host_window_descriptor.shared_body_height, Some(222.0));
        assert_eq!(state.host_window_descriptor.preferred_display_index, 3);
        assert_eq!(
            state.host_window_descriptor.screen_frame,
            Some(crate::native_panel_core::PanelRect {
                x: 40.0,
                y: 50.0,
                width: 800.0,
                height: 600.0,
            })
        );
        assert_eq!(
            state
                .host_window_descriptor
                .timeline
                .map(|timeline| timeline.cards_entering),
            Some(false)
        );
    }

    #[test]
    fn runtime_host_builds_render_payload_from_shared_scene_cache_snapshot() {
        let state = panel_state();
        let mut host = MacosNativePanelRuntimeHost::default();
        host.sync_from_native_panel_state(&state, None);

        let payload =
            super::current_runtime_panel_render_payload_for_state_and_host(&state, &mut host)
                .expect("payload from shared scene cache snapshot");

        assert_eq!(payload.snapshot.status, "idle");
        assert_eq!(payload.expanded, state.expanded);
        assert_eq!(payload.shared_body_height, state.shared_body_height);
        assert_eq!(payload.transitioning, state.transitioning);
        assert_eq!(
            payload.transition_cards_progress,
            state.transition_cards_progress
        );
        assert_eq!(
            payload.transition_cards_entering,
            state.transition_cards_entering
        );
    }
}
