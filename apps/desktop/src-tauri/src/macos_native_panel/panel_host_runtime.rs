use objc2_foundation::NSRect;

use super::panel_host_adapter::MacosNativePanelHostAdapter;
use super::panel_host_commands::current_native_panel_window_frame;
use super::panel_refs::native_panel_state;
use crate::native_panel_renderer::facade::{
    command::NativePanelPlatformEvent,
    descriptor::{
        NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPointerRegion,
    },
    host::{NativePanelHost, NativePanelSceneHost, sync_runtime_pointer_regions_in_state},
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
        let _ = with_native_runtime_panel_state_mut(|state| {
            sync_runtime_pointer_regions_in_state(state, regions);
        });
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
}

pub(super) fn with_native_runtime_panel_state_mut<T>(
    f: impl FnOnce(&mut super::panel_types::NativePanelState) -> T,
) -> Option<T> {
    native_panel_state()
        .and_then(|state_mutex| state_mutex.lock().ok().map(|mut state| f(&mut state)))
}

fn current_native_panel_frame() -> Option<NSRect> {
    unsafe { current_native_panel_window_frame() }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::MacosNativePanelRuntimeHost;
    use crate::{
        macos_native_panel::{mascot::NativeMascotRuntime, panel_types::NativeExpandedSurface},
        native_panel_renderer::facade::{
            descriptor::{
                NativePanelHostWindowDescriptor, NativePanelPointerRegion,
                NativePanelPointerRegionKind, NativePanelTimelineDescriptor,
                native_panel_timeline_descriptor,
            },
            host::{
                NativePanelHost, sync_runtime_host_screen_frame_in_state,
                sync_runtime_host_shared_body_height_in_state, sync_runtime_host_timeline_in_state,
                sync_runtime_host_visibility_in_state, sync_runtime_pointer_regions_in_state,
            },
            renderer::NativePanelRuntimeSceneCache,
        },
    };

    fn panel_state() -> crate::macos_native_panel::panel_types::NativePanelState {
        crate::macos_native_panel::panel_types::NativePanelState {
            expanded: false,
            transitioning: false,
            transition_cards_progress: 0.0,
            transition_cards_entering: false,
            skip_next_close_card_exit: false,
            pending_transition: None,
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
            scene_cache: NativePanelRuntimeSceneCache::default(),
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
            ignores_mouse_events: true,
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
        let regions = vec![NativePanelPointerRegion {
            frame: crate::native_panel_core::PanelRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        host.sync_pointer_regions(&regions)
            .expect("sync pointer regions into runtime host");
        sync_runtime_pointer_regions_in_state(&mut state, &regions);

        assert_eq!(host.adapter.renderer.last_pointer_regions, regions);
        assert_eq!(
            host.adapter.renderer.last_pointer_regions,
            state.pointer_regions
        );
    }

    #[test]
    fn runtime_host_state_helper_updates_descriptor_fields() {
        let mut state = panel_state();

        sync_runtime_host_visibility_in_state(&mut state, false);
        sync_runtime_host_shared_body_height_in_state(&mut state, Some(222.0));
        sync_runtime_host_screen_frame_in_state(
            &mut state,
            3,
            crate::native_panel_core::PanelRect {
                x: 40.0,
                y: 50.0,
                width: 800.0,
                height: 600.0,
            },
        );
        sync_runtime_host_timeline_in_state(
            &mut state,
            native_panel_timeline_descriptor(
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
        let payload = state
            .render_payload()
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
