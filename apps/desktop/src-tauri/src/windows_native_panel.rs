#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use crate::{
    app_settings::current_app_settings,
    display_settings::{list_available_displays, resolve_preferred_display_index},
    native_panel_core::{
        ExpandedSurface, HoverTransition, PanelAnimationDescriptor, PanelGeometryMetrics,
        PanelLayout, PanelLayoutInput, PanelPoint, PanelRect, PanelRenderState,
        PanelRenderStateInput, PanelState, resolve_panel_layout, resolve_panel_render_state,
        sync_hover_expansion_state,
    },
    native_panel_renderer::{
        NativePanelHost, NativePanelHostWindowDescriptor, NativePanelHostWindowState,
        NativePanelPlatformEvent, NativePanelPlatformEventHandler, NativePanelPointerInput,
        NativePanelPointerInputOutcome, NativePanelPointerRegion, NativePanelRenderCommandBundle,
        NativePanelRenderer, NativePanelRuntimeInputDescriptor, NativePanelRuntimeSceneCache,
        NativePanelSceneHost, NativePanelTimelineDescriptor, dispatch_native_panel_platform_events,
        native_panel_platform_event_for_pointer_input,
        native_panel_platform_event_for_pointer_region, native_panel_pointer_inside_for_input,
        native_panel_pointer_state_at_point, native_panel_timeline_descriptor_for_animation,
        rerender_runtime_scene_cache_to_host_on_transition_with_input_descriptor,
        rerender_runtime_scene_cache_to_host_with_input_descriptor,
        resolve_native_panel_render_command_bundle,
        sync_and_apply_runtime_scene_from_input_descriptor,
        sync_runtime_scene_bundle_from_input_descriptor,
    },
    native_panel_scene::{PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelScene},
};

mod host_window;
pub(crate) mod runtime_backend;
mod runtime_input;

use host_window::{WindowsNativePanelHostWindow, decode_windows_native_panel_window_message};
use runtime_input::windows_runtime_input_descriptor;

const WINDOWS_FALLBACK_PANEL_SCREEN_FRAME: PanelRect = PanelRect {
    x: 0.0,
    y: 0.0,
    width: 1440.0,
    height: 900.0,
};

#[derive(Default)]
pub(crate) struct WindowsNativePanelRenderer {
    last_scene: Option<PanelScene>,
    last_runtime_render_state: Option<PanelRuntimeRenderState>,
    last_screen_frame: Option<PanelRect>,
    last_animation_descriptor: Option<PanelAnimationDescriptor>,
    last_timeline_descriptor: Option<NativePanelTimelineDescriptor>,
    last_host_window_descriptor: Option<NativePanelHostWindowDescriptor>,
    last_layout: Option<PanelLayout>,
    last_render_state: Option<PanelRenderState>,
    last_render_command_bundle: Option<NativePanelRenderCommandBundle>,
    last_window_state: Option<NativePanelHostWindowState>,
    last_pointer_regions: Vec<NativePanelPointerRegion>,
}

impl NativePanelRenderer for WindowsNativePanelRenderer {
    type Error = String;

    fn render_scene(
        &mut self,
        scene: &PanelScene,
        runtime: PanelRuntimeRenderState,
    ) -> Result<(), Self::Error> {
        self.last_scene = Some(scene.clone());
        self.last_runtime_render_state = Some(runtime);
        self.refresh_cached_render_inputs();
        Ok(())
    }

    fn apply_animation_descriptor(
        &mut self,
        descriptor: PanelAnimationDescriptor,
    ) -> Result<(), Self::Error> {
        self.last_animation_descriptor = Some(descriptor);
        self.refresh_cached_render_inputs();
        Ok(())
    }

    fn apply_timeline_descriptor(
        &mut self,
        descriptor: NativePanelTimelineDescriptor,
    ) -> Result<(), Self::Error> {
        self.last_timeline_descriptor = Some(descriptor);
        self.last_animation_descriptor = Some(descriptor.animation);
        self.refresh_cached_render_inputs();
        Ok(())
    }

    fn sync_host_window_state(
        &mut self,
        state: NativePanelHostWindowState,
    ) -> Result<(), Self::Error> {
        self.last_window_state = Some(state);
        Ok(())
    }

    fn sync_screen_frame(&mut self, screen_frame: Option<PanelRect>) -> Result<(), Self::Error> {
        self.update_screen_frame(screen_frame);
        Ok(())
    }

    fn sync_shared_body_height(
        &mut self,
        _shared_body_height: Option<f64>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn record_host_window_descriptor(
        &mut self,
        descriptor: NativePanelHostWindowDescriptor,
    ) -> Result<(), Self::Error> {
        self.last_host_window_descriptor = Some(descriptor);
        Ok(())
    }

    fn sync_pointer_regions(
        &mut self,
        regions: &[NativePanelPointerRegion],
    ) -> Result<(), Self::Error> {
        self.last_pointer_regions = regions.to_vec();
        Ok(())
    }

    fn apply_render_command_bundle(
        &mut self,
        bundle: &NativePanelRenderCommandBundle,
    ) -> Result<(), Self::Error> {
        self.last_scene = Some(bundle.scene.clone());
        self.last_runtime_render_state = Some(bundle.runtime);
        self.last_layout = Some(bundle.layout);
        self.last_render_state = Some(bundle.render_state);
        self.last_pointer_regions = bundle.pointer_regions.clone();
        self.last_render_command_bundle = Some(bundle.clone());
        Ok(())
    }

    fn set_visible(&mut self, _visible: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl WindowsNativePanelRenderer {
    fn update_screen_frame(&mut self, screen_frame: Option<PanelRect>) {
        self.last_screen_frame = screen_frame;
        self.refresh_cached_render_inputs();
    }

    fn refresh_cached_render_inputs(&mut self) {
        let Some(descriptor) = self.last_animation_descriptor else {
            return;
        };
        let screen_frame = self
            .last_screen_frame
            .unwrap_or(WINDOWS_FALLBACK_PANEL_SCREEN_FRAME);
        let layout = resolve_panel_layout(PanelLayoutInput {
            screen_frame,
            metrics: PanelGeometryMetrics {
                compact_height: crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT,
                compact_width: crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH,
                expanded_width: crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH,
                panel_width: crate::native_panel_core::DEFAULT_PANEL_CANVAS_WIDTH,
            },
            canvas_height: descriptor.canvas_height,
            visible_height: descriptor.visible_height,
            bar_progress: descriptor.width_progress,
            height_progress: descriptor.height_progress,
            drop_progress: descriptor.drop_progress,
            content_visibility: descriptor.cards_progress,
            collapsed_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            drop_distance: crate::native_panel_core::PANEL_DROP_DISTANCE,
            content_top_gap: crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP,
            content_bottom_inset: crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET,
            cards_side_inset: crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET,
            shoulder_size: crate::native_panel_core::COMPACT_SHOULDER_SIZE,
            separator_side_inset: crate::native_panel_core::EXPANDED_SEPARATOR_SIDE_INSET,
        });
        let runtime = self.last_runtime_render_state.unwrap_or_default();
        let status_surface_active = self
            .last_scene
            .as_ref()
            .is_some_and(|scene| scene.surface == ExpandedSurface::Status);
        let render_state = resolve_panel_render_state(PanelRenderStateInput {
            shared_expanded_enabled: false,
            shell_visible: layout.shell_visible,
            separator_visibility: layout.separator_visibility,
            bar_progress: descriptor.width_progress,
            height_progress: descriptor.height_progress,
            cards_height: layout.cards_frame.height,
            status_surface_active,
            content_visibility: descriptor.cards_progress,
            transitioning: runtime.transitioning,
            headline_emphasized: runtime.shell_scene.headline_emphasized,
            edge_actions_visible: runtime.shell_scene.edge_actions_visible,
        });

        let Some(scene) = self.last_scene.clone() else {
            self.last_layout = Some(layout);
            self.last_render_state = Some(render_state);
            self.last_pointer_regions = Vec::new();
            self.last_render_command_bundle = None;
            return;
        };
        let bundle =
            resolve_native_panel_render_command_bundle(layout, &scene, runtime, render_state, None);
        let _ = self.apply_render_command_bundle(&bundle);
    }
}

#[derive(Default)]
pub(crate) struct WindowsNativePanelHost {
    renderer: WindowsNativePanelRenderer,
    window: WindowsNativePanelHostWindow,
    pending_events: Vec<NativePanelPlatformEvent>,
}

impl NativePanelHost for WindowsNativePanelHost {
    type Error = String;
    type Renderer = WindowsNativePanelRenderer;

    fn renderer(&mut self) -> &mut Self::Renderer {
        &mut self.renderer
    }

    fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
        self.window.descriptor
    }

    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
        &mut self.window.descriptor
    }

    fn window_state(&self) -> NativePanelHostWindowState {
        self.window.window_state()
    }

    fn create(&mut self) -> Result<(), Self::Error> {
        self.window.create();
        self.sync_renderer_host_window_descriptor()
    }

    fn after_host_window_descriptor_updated(&mut self) -> Result<(), Self::Error> {
        self.window.refresh_frame_from_descriptor();
        Ok(())
    }

    fn show(&mut self) -> Result<(), Self::Error> {
        NativePanelHost::create(self)?;
        self.window.show();
        self.sync_renderer_host_window_descriptor()
    }

    fn hide(&mut self) -> Result<(), Self::Error> {
        self.window.hide();
        self.sync_renderer_host_window_descriptor()
    }

    fn take_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

impl WindowsNativePanelHost {
    fn create(&mut self) -> Result<(), String> {
        NativePanelHost::create(self)
    }

    fn update_window_descriptor_with(
        &mut self,
        update: impl FnOnce(&mut NativePanelHostWindowDescriptor),
    ) -> Result<(), String> {
        self.update_host_window_descriptor(update)
    }

    fn reposition_to_display(
        &mut self,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) -> Result<(), String> {
        NativePanelHost::reposition_to_display(self, preferred_display_index, screen_frame)
    }

    fn set_shared_body_height(&mut self, body_height: f64) -> Result<(), String> {
        NativePanelHost::set_shared_body_height(self, body_height)
    }

    fn apply_animation_descriptor(
        &mut self,
        descriptor: PanelAnimationDescriptor,
    ) -> Result<(), String> {
        let timeline = native_panel_timeline_descriptor_for_animation(descriptor);
        NativePanelHost::apply_timeline_descriptor(self, timeline)
    }

    fn queue_platform_event_for_pointer_region(&mut self, region: &NativePanelPointerRegion) {
        if let Some(event) = native_panel_platform_event_for_pointer_region(region) {
            self.pending_events.push(event);
        }
    }

    fn queue_platform_event_at_point(
        &mut self,
        point: PanelPoint,
    ) -> Option<NativePanelPlatformEvent> {
        let event = native_panel_pointer_state_at_point(&self.renderer.last_pointer_regions, point)
            .platform_event;
        if let Some(event) = event.clone() {
            self.pending_events.push(event);
        }
        event
    }

    fn queue_platform_event_for_pointer_input(
        &mut self,
        input: NativePanelPointerInput,
    ) -> Option<NativePanelPlatformEvent> {
        let event = native_panel_platform_event_for_pointer_input(
            &self.renderer.last_pointer_regions,
            input,
        );
        if let Some(event) = event.clone() {
            self.pending_events.push(event);
        }
        event
    }

    fn pointer_inside_at_point(&self, point: PanelPoint) -> bool {
        native_panel_pointer_state_at_point(&self.renderer.last_pointer_regions, point).inside
    }

    fn pointer_inside_for_input(&self, input: NativePanelPointerInput) -> Option<bool> {
        native_panel_pointer_inside_for_input(&self.renderer.last_pointer_regions, input)
    }

    fn dispatch_queued_platform_events_with_handler<H>(
        &mut self,
        handler: &mut H,
    ) -> Result<(), H::Error>
    where
        H: NativePanelPlatformEventHandler,
    {
        dispatch_native_panel_platform_events(handler, self.take_platform_events())
    }
}

impl NativePanelSceneHost for WindowsNativePanelHost {}

impl WindowsNativePanelRuntime {
    fn sync_hover_inside(&mut self, inside: bool, now: Instant) -> Option<HoverTransition> {
        sync_hover_expansion_state(
            &mut self.panel_state,
            inside,
            now,
            crate::native_panel_core::HOVER_DELAY_MS,
        )
    }

    fn sync_hover_at_point(&mut self, point: PanelPoint, now: Instant) -> Option<HoverTransition> {
        let inside = self.host.pointer_inside_at_point(point);
        self.sync_hover_inside(inside, now)
    }

    fn sync_hover_for_pointer_input(
        &mut self,
        input: NativePanelPointerInput,
        now: Instant,
    ) -> Option<HoverTransition> {
        self.host
            .pointer_inside_for_input(input)
            .and_then(|inside| self.sync_hover_inside(inside, now))
    }

    fn sync_snapshot_bundle(
        &mut self,
        snapshot: &RuntimeSnapshot,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<(), String> {
        sync_and_apply_runtime_scene_from_input_descriptor(
            &mut self.host,
            &mut self.scene_cache,
            &mut self.panel_state,
            snapshot,
            input,
            chrono::Utc::now(),
        )?;
        Ok(())
    }

    fn rebuild_scene_bundle_from_cached_snapshot_input(
        panel_state: &mut PanelState,
        snapshot: &RuntimeSnapshot,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> PanelRuntimeSceneBundle {
        sync_runtime_scene_bundle_from_input_descriptor(
            panel_state,
            snapshot,
            input,
            chrono::Utc::now(),
        )
        .bundle
    }

    fn rerender_from_last_snapshot_with_input(
        &mut self,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<bool, String> {
        rerender_runtime_scene_cache_to_host_with_input_descriptor(
            &mut self.host,
            &mut self.scene_cache,
            input,
            |snapshot| {
                Self::rebuild_scene_bundle_from_cached_snapshot_input(
                    &mut self.panel_state,
                    snapshot,
                    input,
                )
            },
        )
    }

    fn rerender_from_last_snapshot<R: tauri::Runtime>(
        &mut self,
        app: &AppHandle<R>,
    ) -> Result<bool, String> {
        let input = windows_runtime_input_descriptor(app);
        self.rerender_from_last_snapshot_with_input(&input)
    }

    fn sync_hover_and_refresh_at_point_with_input(
        &mut self,
        point: PanelPoint,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, String> {
        let transition = self.sync_hover_at_point(point, now);
        rerender_runtime_scene_cache_to_host_on_transition_with_input_descriptor(
            &mut self.host,
            &mut self.scene_cache,
            transition,
            input,
            |snapshot| {
                Self::rebuild_scene_bundle_from_cached_snapshot_input(
                    &mut self.panel_state,
                    snapshot,
                    input,
                )
            },
        )
    }

    fn sync_hover_and_refresh_inside_with_input(
        &mut self,
        inside: bool,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, String> {
        let transition = self.sync_hover_inside(inside, now);
        rerender_runtime_scene_cache_to_host_on_transition_with_input_descriptor(
            &mut self.host,
            &mut self.scene_cache,
            transition,
            input,
            |snapshot| {
                Self::rebuild_scene_bundle_from_cached_snapshot_input(
                    &mut self.panel_state,
                    snapshot,
                    input,
                )
            },
        )
    }

    fn sync_hover_and_refresh_for_pointer_input_with_input(
        &mut self,
        input_event: NativePanelPointerInput,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, String> {
        let transition = self.sync_hover_for_pointer_input(input_event, now);
        rerender_runtime_scene_cache_to_host_on_transition_with_input_descriptor(
            &mut self.host,
            &mut self.scene_cache,
            transition,
            input,
            |snapshot| {
                Self::rebuild_scene_bundle_from_cached_snapshot_input(
                    &mut self.panel_state,
                    snapshot,
                    input,
                )
            },
        )
    }

    fn sync_hover_and_refresh_at_point<R: tauri::Runtime>(
        &mut self,
        app: &AppHandle<R>,
        point: PanelPoint,
        now: Instant,
    ) -> Result<Option<HoverTransition>, String> {
        let input = windows_runtime_input_descriptor(app);
        self.sync_hover_and_refresh_at_point_with_input(point, now, &input)
    }

    fn sync_hover_and_refresh_inside<R: tauri::Runtime>(
        &mut self,
        app: &AppHandle<R>,
        inside: bool,
        now: Instant,
    ) -> Result<Option<HoverTransition>, String> {
        let input = windows_runtime_input_descriptor(app);
        self.sync_hover_and_refresh_inside_with_input(inside, now, &input)
    }

    fn dispatch_platform_event_at_point_with_handler<H>(
        &mut self,
        point: PanelPoint,
        handler: &mut H,
    ) -> Result<Option<NativePanelPlatformEvent>, H::Error>
    where
        H: NativePanelPlatformEventHandler,
    {
        let event = self.host.queue_platform_event_at_point(point);
        if event.is_some() {
            self.host
                .dispatch_queued_platform_events_with_handler(handler)?;
        }
        Ok(event)
    }

    fn dispatch_platform_event_for_pointer_input_with_handler<H>(
        &mut self,
        input: NativePanelPointerInput,
        handler: &mut H,
    ) -> Result<Option<NativePanelPlatformEvent>, H::Error>
    where
        H: NativePanelPlatformEventHandler,
    {
        let event = self.host.queue_platform_event_for_pointer_input(input);
        if event.is_some() {
            self.host
                .dispatch_queued_platform_events_with_handler(handler)?;
        }
        Ok(event)
    }

    fn dispatch_platform_event_at_point<R: tauri::Runtime + 'static>(
        &mut self,
        app: &AppHandle<R>,
        point: PanelPoint,
    ) -> Result<Option<NativePanelPlatformEvent>, String> {
        let mut handler = WindowsNativePanelPlatformEventHandler { app: app.clone() };
        self.dispatch_platform_event_at_point_with_handler(point, &mut handler)
    }

    fn handle_pointer_input_with_handler(
        &mut self,
        input_event: NativePanelPointerInput,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
        handler: &mut impl NativePanelPlatformEventHandler<Error = String>,
    ) -> Result<NativePanelPointerInputOutcome, String> {
        match input_event {
            NativePanelPointerInput::Move(_) | NativePanelPointerInput::Leave => {
                Ok(NativePanelPointerInputOutcome::Hover(
                    self.sync_hover_and_refresh_for_pointer_input_with_input(
                        input_event,
                        now,
                        input,
                    )?,
                ))
            }
            NativePanelPointerInput::Click(_) => Ok(NativePanelPointerInputOutcome::Click(
                self.dispatch_platform_event_for_pointer_input_with_handler(input_event, handler)?,
            )),
        }
    }
}

struct WindowsNativePanelPlatformEventHandler<R: tauri::Runtime + 'static> {
    app: AppHandle<R>,
}

impl<R: tauri::Runtime + 'static> NativePanelPlatformEventHandler
    for WindowsNativePanelPlatformEventHandler<R>
{
    type Error = String;

    fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error> {
        crate::native_panel_runtime::spawn_native_focus_session(self.app.clone(), session_id);
        Ok(())
    }

    fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn quit_application(&mut self) -> Result<(), Self::Error> {
        self.app.exit(0);
        Ok(())
    }

    fn cycle_display(&mut self) -> Result<(), Self::Error> {
        let displays = list_available_displays(&self.app)?;
        let total = displays.len().max(1);
        let settings = current_app_settings();
        let current =
            resolve_preferred_display_index(&displays, settings.preferred_display_key.as_deref());
        let next = (current + 1) % total;
        let selected = &displays[next];
        crate::app_settings::update_preferred_display_selection(next, Some(selected.key.clone()))
            .map_err(|error| error.to_string())?;
        reposition_native_panel_to_selected_display(&self.app)
    }

    fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
        let next_enabled = !current_app_settings().completion_sound_enabled;
        crate::app_settings::update_completion_sound_enabled(next_enabled)
            .map_err(|error| error.to_string())?;
        refresh_native_panel_from_last_snapshot(&self.app)
    }

    fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
        let next_enabled = !current_app_settings().mascot_enabled;
        crate::app_settings::update_mascot_enabled(next_enabled)
            .map_err(|error| error.to_string())?;
        refresh_native_panel_from_last_snapshot(&self.app)
    }

    fn open_release_page(&mut self) -> Result<(), Self::Error> {
        crate::commands::open_release_page()
    }
}

#[derive(Default)]
struct WindowsNativePanelRuntime {
    panel_state: PanelState,
    host: WindowsNativePanelHost,
    scene_cache: NativePanelRuntimeSceneCache,
    last_animation_descriptor: Option<PanelAnimationDescriptor>,
}

static WINDOWS_NATIVE_PANEL_RUNTIME: OnceLock<Mutex<WindowsNativePanelRuntime>> = OnceLock::new();

pub(crate) fn current_windows_native_panel_runtime_backend()
-> runtime_backend::WindowsNativePanelRuntimeBackendFacade {
    runtime_backend::current_windows_native_panel_runtime_backend()
}

pub(crate) fn native_ui_enabled() -> bool {
    false
}

pub(crate) fn create_native_panel() -> Result<(), String> {
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .host
        .create()
}

pub(crate) fn hide_main_webview_window<R: tauri::Runtime>(_: &AppHandle<R>) -> Result<(), String> {
    Ok(())
}

pub(crate) fn spawn_platform_loops<R: tauri::Runtime + 'static>(_: AppHandle<R>) {}

pub(crate) fn dispatch_queued_native_panel_platform_events<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
) -> Result<(), String> {
    let events = windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .host
        .take_platform_events();
    let mut handler = WindowsNativePanelPlatformEventHandler { app };
    dispatch_native_panel_platform_events(&mut handler, events)
}

pub(crate) fn update_native_panel_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), String> {
    let mut runtime = windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?;
    let input = windows_runtime_input_descriptor(app);
    runtime.sync_snapshot_bundle(snapshot, &input)
}

pub(crate) fn hide_native_panel<R: tauri::Runtime>(_: &AppHandle<R>) -> Result<(), String> {
    let mut runtime = windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?;
    runtime.host.hide()
}

pub(crate) fn refresh_native_panel_from_last_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .rerender_from_last_snapshot(app)
        .map(|_| ())
}

pub(crate) fn handle_native_panel_pointer_move<R: tauri::Runtime>(
    app: &AppHandle<R>,
    point: PanelPoint,
    now: Instant,
) -> Result<Option<HoverTransition>, String> {
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .sync_hover_and_refresh_at_point(app, point, now)
}

pub(crate) fn handle_native_panel_pointer_leave<R: tauri::Runtime>(
    app: &AppHandle<R>,
    now: Instant,
) -> Result<Option<HoverTransition>, String> {
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .sync_hover_and_refresh_inside(app, false, now)
}

pub(crate) fn handle_native_panel_pointer_click<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    point: PanelPoint,
) -> Result<Option<NativePanelPlatformEvent>, String> {
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .dispatch_platform_event_at_point(app, point)
}

pub(crate) fn handle_native_panel_window_message<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    message_id: u32,
    lparam: isize,
    now: Instant,
) -> Result<Option<NativePanelPointerInputOutcome>, String> {
    let Some(message) = decode_windows_native_panel_window_message(message_id, lparam) else {
        return Ok(None);
    };
    let runtime_input = windows_runtime_input_descriptor(app);
    let mut handler = WindowsNativePanelPlatformEventHandler { app: app.clone() };
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .handle_pointer_input_with_handler(message, now, &runtime_input, &mut handler)
        .map(Some)
}

pub(crate) fn reposition_native_panel_to_selected_display<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    let input = windows_runtime_input_descriptor(app);
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .host
        .reposition_to_display(input.selected_display_index(), input.screen_frame)
}

pub(crate) fn set_shared_expanded_body_height<R: tauri::Runtime>(
    _: &AppHandle<R>,
    body_height: f64,
) -> Result<(), String> {
    windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?
        .host
        .set_shared_body_height(body_height)
}

pub(crate) fn apply_native_panel_animation_descriptor(
    descriptor: PanelAnimationDescriptor,
) -> Result<(), String> {
    let mut runtime = windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?;
    runtime.host.apply_animation_descriptor(descriptor)?;
    runtime.last_animation_descriptor = Some(descriptor);
    Ok(())
}

fn windows_native_panel_runtime() -> &'static Mutex<WindowsNativePanelRuntime> {
    WINDOWS_NATIVE_PANEL_RUNTIME.get_or_init(|| Mutex::new(WindowsNativePanelRuntime::default()))
}

#[cfg(test)]
mod tests {
    use crate::{
        native_panel_core::{
            CompletionBadgeItem, HoverTransition, PanelAnimationDescriptor, PanelAnimationKind,
            PanelHitAction, PanelHitTarget, PanelPoint, PanelRect, PanelState,
        },
        native_panel_renderer::{
            NativePanelEdgeAction, NativePanelHost, NativePanelHostWindowState,
            NativePanelPlatformEvent, NativePanelPlatformEventHandler, NativePanelPointerInput,
            NativePanelPointerInputOutcome, NativePanelPointerRegion, NativePanelPointerRegionKind,
            NativePanelRenderer, NativePanelRuntimeInputDescriptor, NativePanelTimelineDescriptor,
            sync_runtime_scene_bundle_from_input_descriptor,
        },
        native_panel_scene::{
            PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelSceneBuildInput, SceneCard,
            SceneMascotPose, build_panel_scene,
        },
    };
    use chrono::Utc;
    use echoisland_runtime::{PendingPermissionView, RuntimeSnapshot};
    use std::time::Duration;

    fn snapshot() -> RuntimeSnapshot {
        RuntimeSnapshot {
            status: "idle".to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 1,
            total_session_count: 1,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![],
        }
    }

    fn runtime_input_descriptor() -> NativePanelRuntimeInputDescriptor {
        NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: Some(PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            }),
        }
    }

    fn test_runtime_scene_bundle(
        panel_state: &mut PanelState,
        raw_snapshot: &RuntimeSnapshot,
        input: &PanelSceneBuildInput,
    ) -> PanelRuntimeSceneBundle {
        sync_runtime_scene_bundle_from_input_descriptor(
            panel_state,
            raw_snapshot,
            &NativePanelRuntimeInputDescriptor {
                scene_input: input.clone(),
                screen_frame: None,
            },
            Utc::now(),
        )
        .bundle
    }

    fn pending_permission_snapshot(session_id: &str) -> RuntimeSnapshot {
        let pending = PendingPermissionView {
            request_id: "req-1".to_string(),
            session_id: session_id.to_string(),
            source: "claude".to_string(),
            tool_name: Some("Bash".to_string()),
            tool_description: Some("Run command".to_string()),
            requested_at: Utc::now(),
        };
        let mut snapshot = snapshot();
        snapshot.pending_permission_count = 1;
        snapshot.pending_permission = Some(pending.clone());
        snapshot.pending_permissions = vec![pending];
        snapshot
    }

    #[test]
    fn windows_scaffold_consumes_shared_scene_bundle() {
        let mut state = PanelState::default();
        let bundle =
            test_runtime_scene_bundle(&mut state, &snapshot(), &PanelSceneBuildInput::default());
        let scene = bundle.scene;
        let runtime_render_state = bundle.runtime_render_state;

        assert!(!scene.cards.is_empty());
        assert!(matches!(
            scene.mascot_pose,
            SceneMascotPose::Idle | SceneMascotPose::Running | SceneMascotPose::Hidden
        ));
        assert!(
            scene
                .cards
                .iter()
                .any(|card| matches!(card, SceneCard::Empty))
        );
        assert!(!runtime_render_state.transitioning);
    }

    #[test]
    fn windows_host_lifecycle_tracks_create_show_hide() {
        let mut host = super::WindowsNativePanelHost::default();

        assert_eq!(
            host.window.lifecycle,
            super::host_window::WindowsNativePanelWindowLifecycle::NotCreated
        );
        assert!(!host.window.descriptor.visible);

        host.show().expect("show host");
        assert_eq!(
            host.window.lifecycle,
            super::host_window::WindowsNativePanelWindowLifecycle::Created
        );
        assert!(host.window.descriptor.visible);
        assert_eq!(
            host.renderer.last_window_state,
            Some(NativePanelHostWindowState {
                frame: None,
                visible: true,
                preferred_display_index: 0,
            })
        );

        host.reposition_to_display(2, None)
            .expect("reposition host");
        assert_eq!(host.window.descriptor.preferred_display_index, 2);
        assert_eq!(
            host.renderer.last_window_state,
            Some(NativePanelHostWindowState {
                frame: None,
                visible: true,
                preferred_display_index: 2,
            })
        );

        host.set_shared_body_height(320.0)
            .expect("sync shared body height");
        assert_eq!(host.window.descriptor.shared_body_height, Some(320.0));
        assert_eq!(
            host.renderer.last_host_window_descriptor,
            Some(host.window.descriptor)
        );

        host.hide().expect("hide host");
        assert_eq!(
            host.window.lifecycle,
            super::host_window::WindowsNativePanelWindowLifecycle::Created
        );
        assert!(!host.window.descriptor.visible);
        assert_eq!(
            host.renderer.last_window_state,
            Some(NativePanelHostWindowState {
                frame: None,
                visible: false,
                preferred_display_index: 2,
            })
        );
    }

    #[test]
    fn windows_renderer_caches_shared_animation_descriptor() {
        let mut host = super::WindowsNativePanelHost::default();
        let descriptor = PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 120.0,
            width_progress: 0.5,
            height_progress: 0.0,
            shoulder_progress: 1.0,
            drop_progress: 0.0,
            cards_progress: 0.25,
        };

        host.apply_animation_descriptor(descriptor)
            .expect("apply descriptor");

        assert_eq!(host.renderer.last_animation_descriptor, Some(descriptor));
        assert_eq!(
            host.renderer.last_timeline_descriptor,
            Some(NativePanelTimelineDescriptor {
                animation: descriptor,
                cards_entering: true,
            })
        );
        assert_eq!(
            host.renderer.last_host_window_descriptor,
            Some(host.window.descriptor)
        );
        assert_eq!(
            host.window.descriptor.timeline,
            Some(NativePanelTimelineDescriptor {
                animation: descriptor,
                cards_entering: true,
            })
        );
        assert!(host.window.last_frame.is_some());
        assert_eq!(
            host.renderer.last_window_state,
            Some(host.window.window_state())
        );
        assert_eq!(
            host.window.lifecycle,
            super::host_window::WindowsNativePanelWindowLifecycle::Created
        );
    }

    #[test]
    fn windows_renderer_caches_pointer_regions_from_host_trait() {
        let mut host = super::WindowsNativePanelHost::default();
        let regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::Shell,
        }];

        host.sync_pointer_regions(&regions)
            .expect("sync pointer regions");

        assert_eq!(host.renderer.last_pointer_regions, regions);
    }

    #[test]
    fn windows_host_queues_platform_events_from_pointer_regions() {
        let mut host = super::WindowsNativePanelHost::default();
        let frame = PanelRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        };

        host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
            frame,
            kind: NativePanelPointerRegionKind::Shell,
        });
        host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
            frame,
            kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        });
        host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
            frame,
            kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings),
        });
        host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
            frame,
            kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Quit),
        });

        assert_eq!(
            host.take_platform_events(),
            vec![
                NativePanelPlatformEvent::FocusSession("session-1".to_string()),
                NativePanelPlatformEvent::ToggleSettingsSurface,
                NativePanelPlatformEvent::QuitApplication,
            ]
        );
        assert!(host.take_platform_events().is_empty());
    }

    #[test]
    fn windows_host_queues_platform_event_by_point_from_cached_regions() {
        let mut host = super::WindowsNativePanelHost::default();
        host.renderer.last_pointer_regions = vec![
            NativePanelPointerRegion {
                frame: PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 200.0,
                    height: 200.0,
                },
                kind: NativePanelPointerRegionKind::CardsContainer,
            },
            NativePanelPointerRegion {
                frame: PanelRect {
                    x: 20.0,
                    y: 20.0,
                    width: 80.0,
                    height: 40.0,
                },
                kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: "session-1".to_string(),
                }),
            },
            NativePanelPointerRegion {
                frame: PanelRect {
                    x: 140.0,
                    y: 140.0,
                    width: 40.0,
                    height: 40.0,
                },
                kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Quit),
            },
        ];

        assert_eq!(
            host.queue_platform_event_at_point(PanelPoint { x: 30.0, y: 30.0 }),
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            host.queue_platform_event_at_point(PanelPoint { x: 150.0, y: 150.0 }),
            Some(NativePanelPlatformEvent::QuitApplication)
        );
        assert_eq!(
            host.queue_platform_event_at_point(PanelPoint { x: 190.0, y: 190.0 }),
            None
        );
        assert_eq!(
            host.take_platform_events(),
            vec![
                NativePanelPlatformEvent::FocusSession("session-1".to_string()),
                NativePanelPlatformEvent::QuitApplication,
            ]
        );
    }

    #[test]
    fn windows_runtime_syncs_hover_expand_from_cached_regions() {
        let now = std::time::Instant::now();
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.panel_state.pointer_inside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
        runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        let transition = runtime.sync_hover_at_point(PanelPoint { x: 30.0, y: 30.0 }, now);

        assert_eq!(transition, Some(HoverTransition::Expand));
        assert!(runtime.panel_state.expanded);
        assert!(runtime.panel_state.pointer_outside_since.is_none());
    }

    #[test]
    fn windows_runtime_syncs_hover_collapse_outside_cached_regions() {
        let now = std::time::Instant::now();
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.panel_state.expanded = true;
        runtime.panel_state.pointer_outside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
        runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        let transition = runtime.sync_hover_at_point(PanelPoint { x: 180.0, y: 180.0 }, now);

        assert_eq!(transition, Some(HoverTransition::Collapse));
        assert!(!runtime.panel_state.expanded);
        assert!(runtime.panel_state.pointer_inside_since.is_none());
    }

    #[test]
    fn windows_runtime_hover_expand_refreshes_cached_scene_from_last_snapshot() {
        let now = std::time::Instant::now();
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.scene_cache.last_snapshot = Some(pending_permission_snapshot("session-1"));
        runtime.panel_state.pointer_inside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
        runtime
            .host
            .apply_animation_descriptor(PanelAnimationDescriptor {
                kind: PanelAnimationKind::Open,
                canvas_height: 180.0,
                visible_height: 140.0,
                width_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 1.0,
                drop_progress: 1.0,
                cards_progress: 1.0,
            })
            .expect("seed animation descriptor");
        runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        let transition = runtime
            .sync_hover_and_refresh_at_point_with_input(
                PanelPoint { x: 30.0, y: 30.0 },
                now,
                &runtime_input_descriptor(),
            )
            .expect("expand and refresh");

        assert_eq!(transition, Some(HoverTransition::Expand));
        assert!(runtime.panel_state.expanded);
        assert!(runtime.scene_cache.last_scene.is_some());
        assert!(runtime.scene_cache.last_runtime_render_state.is_some());
        assert!(runtime.host.renderer.last_scene.is_some());
        assert!(runtime.host.renderer.last_runtime_render_state.is_some());
        assert!(
            runtime
                .scene_cache
                .last_scene
                .as_ref()
                .is_some_and(|scene| {
                    scene.hit_targets.iter().any(|target| {
                        target.action == PanelHitAction::FocusSession && target.value == "session-1"
                    })
                })
        );
    }

    #[test]
    fn windows_runtime_hover_collapse_refreshes_cached_scene_from_last_snapshot() {
        let now = std::time::Instant::now();
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.scene_cache.last_snapshot = Some(snapshot());
        runtime.panel_state.expanded = true;
        runtime.panel_state.pointer_outside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
        runtime
            .host
            .apply_animation_descriptor(PanelAnimationDescriptor {
                kind: PanelAnimationKind::Close,
                canvas_height: 120.0,
                visible_height: 120.0,
                width_progress: 0.0,
                height_progress: 0.0,
                shoulder_progress: 0.0,
                drop_progress: 0.0,
                cards_progress: 0.0,
            })
            .expect("seed animation descriptor");
        runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        let transition = runtime
            .sync_hover_and_refresh_at_point_with_input(
                PanelPoint { x: 180.0, y: 180.0 },
                now,
                &runtime_input_descriptor(),
            )
            .expect("collapse and refresh");

        assert_eq!(transition, Some(HoverTransition::Collapse));
        assert!(!runtime.panel_state.expanded);
        assert!(runtime.scene_cache.last_scene.is_some());
        assert!(runtime.host.renderer.last_scene.is_some());
        assert!(
            runtime
                .scene_cache
                .last_scene
                .as_ref()
                .is_some_and(|scene| scene.hit_targets.is_empty())
        );
    }

    #[test]
    fn windows_runtime_hover_transition_without_snapshot_skips_refresh() {
        let now = std::time::Instant::now();
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.panel_state.pointer_inside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
        runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        let transition = runtime
            .sync_hover_and_refresh_at_point_with_input(
                PanelPoint { x: 30.0, y: 30.0 },
                now,
                &runtime_input_descriptor(),
            )
            .expect("expand without snapshot");

        assert_eq!(transition, Some(HoverTransition::Expand));
        assert!(runtime.panel_state.expanded);
        assert!(runtime.scene_cache.last_scene.is_none());
        assert!(runtime.host.renderer.last_scene.is_none());
    }

    #[test]
    fn windows_runtime_dispatches_platform_event_at_point_through_handler() {
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.host.renderer.last_pointer_regions = vec![
            NativePanelPointerRegion {
                frame: PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 200.0,
                    height: 200.0,
                },
                kind: NativePanelPointerRegionKind::CardsContainer,
            },
            NativePanelPointerRegion {
                frame: PanelRect {
                    x: 20.0,
                    y: 20.0,
                    width: 80.0,
                    height: 40.0,
                },
                kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: "session-1".to_string(),
                }),
            },
        ];
        let mut handler = RecordingEventHandler::default();

        let event = runtime
            .dispatch_platform_event_at_point_with_handler(
                PanelPoint { x: 30.0, y: 30.0 },
                &mut handler,
            )
            .expect("dispatch point event");

        assert_eq!(
            event,
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            handler.handled,
            vec![NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )]
        );
        assert!(runtime.host.pending_events.is_empty());
    }

    #[test]
    fn windows_runtime_pointer_event_dispatch_is_noop_when_point_has_no_target() {
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 120.0,
                height: 60.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];
        let mut handler = RecordingEventHandler::default();

        let event = runtime
            .dispatch_platform_event_at_point_with_handler(
                PanelPoint { x: 10.0, y: 10.0 },
                &mut handler,
            )
            .expect("dispatch empty point event");

        assert_eq!(event, None);
        assert!(handler.handled.is_empty());
        assert!(runtime.host.pending_events.is_empty());
    }

    #[test]
    fn windows_runtime_window_message_pointer_leave_collapses_and_refreshes() {
        let now = std::time::Instant::now();
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.scene_cache.last_snapshot = Some(snapshot());
        runtime.panel_state.expanded = true;
        runtime.panel_state.pointer_outside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
        runtime
            .host
            .apply_animation_descriptor(PanelAnimationDescriptor {
                kind: PanelAnimationKind::Close,
                canvas_height: 120.0,
                visible_height: 120.0,
                width_progress: 0.0,
                height_progress: 0.0,
                shoulder_progress: 0.0,
                drop_progress: 0.0,
                cards_progress: 0.0,
            })
            .expect("seed animation descriptor");
        let mut handler = RecordingEventHandler::default();

        let outcome = runtime
            .handle_pointer_input_with_handler(
                NativePanelPointerInput::Leave,
                now,
                &runtime_input_descriptor(),
                &mut handler,
            )
            .expect("handle pointer leave");

        assert_eq!(
            outcome,
            NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Collapse))
        );
        assert!(!runtime.panel_state.expanded);
        assert!(runtime.scene_cache.last_scene.is_some());
        assert!(runtime.host.renderer.last_scene.is_some());
        assert!(handler.handled.is_empty());
    }

    #[test]
    fn windows_runtime_window_message_click_dispatches_hit_target_event() {
        let mut runtime = super::WindowsNativePanelRuntime::default();
        runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        }];
        let mut handler = RecordingEventHandler::default();

        let outcome = runtime
            .handle_pointer_input_with_handler(
                NativePanelPointerInput::Click(PanelPoint { x: 30.0, y: 30.0 }),
                std::time::Instant::now(),
                &NativePanelRuntimeInputDescriptor {
                    scene_input: PanelSceneBuildInput::default(),
                    screen_frame: None,
                },
                &mut handler,
            )
            .expect("handle pointer click");

        assert_eq!(
            outcome,
            NativePanelPointerInputOutcome::Click(Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )))
        );
        assert_eq!(
            handler.handled,
            vec![NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )]
        );
    }

    #[derive(Default)]
    struct RecordingEventHandler {
        handled: Vec<NativePanelPlatformEvent>,
    }

    impl NativePanelPlatformEventHandler for RecordingEventHandler {
        type Error = String;

        fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::FocusSession(session_id));
            Ok(())
        }

        fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::ToggleSettingsSurface);
            Ok(())
        }

        fn quit_application(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::QuitApplication);
            Ok(())
        }

        fn cycle_display(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::CycleDisplay);
            Ok(())
        }

        fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::ToggleCompletionSound);
            Ok(())
        }

        fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::ToggleMascot);
            Ok(())
        }

        fn open_release_page(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::OpenReleasePage);
            Ok(())
        }
    }

    #[test]
    fn windows_host_dispatches_queued_platform_events_through_handler() {
        let mut host = super::WindowsNativePanelHost::default();
        host.pending_events = vec![
            NativePanelPlatformEvent::FocusSession("session-1".to_string()),
            NativePanelPlatformEvent::ToggleCompletionSound,
            NativePanelPlatformEvent::ToggleMascot,
            NativePanelPlatformEvent::OpenReleasePage,
        ];
        let mut handler = RecordingEventHandler::default();

        host.dispatch_queued_platform_events_with_handler(&mut handler)
            .expect("dispatch queued events");

        assert_eq!(
            handler.handled,
            vec![
                NativePanelPlatformEvent::FocusSession("session-1".to_string()),
                NativePanelPlatformEvent::ToggleCompletionSound,
                NativePanelPlatformEvent::ToggleMascot,
                NativePanelPlatformEvent::OpenReleasePage,
            ]
        );
        assert!(host.pending_events.is_empty());
    }

    #[test]
    fn windows_renderer_caches_scene_and_resolves_shared_render_inputs() {
        let mut panel_state = PanelState::default();
        let bundle = test_runtime_scene_bundle(
            &mut panel_state,
            &snapshot(),
            &PanelSceneBuildInput::default(),
        );
        let scene = bundle.scene;
        let runtime_render_state = bundle.runtime_render_state;
        let mut renderer = super::WindowsNativePanelRenderer::default();

        renderer.update_screen_frame(Some(PanelRect {
            x: 100.0,
            y: 50.0,
            width: 1000.0,
            height: 700.0,
        }));
        renderer
            .render_scene(&scene, runtime_render_state)
            .expect("render scene");
        renderer
            .apply_animation_descriptor(PanelAnimationDescriptor {
                kind: PanelAnimationKind::Open,
                canvas_height: 180.0,
                visible_height: 140.0,
                width_progress: 0.5,
                height_progress: 0.75,
                shoulder_progress: 1.0,
                drop_progress: 0.25,
                cards_progress: 0.8,
            })
            .expect("apply descriptor");

        assert_eq!(
            renderer.last_scene.as_ref().map(|cached| cached.surface),
            Some(scene.surface)
        );
        assert_eq!(
            renderer.last_runtime_render_state,
            Some(runtime_render_state)
        );
        assert_eq!(
            renderer.last_layout,
            Some(crate::native_panel_core::PanelLayout {
                panel_frame: PanelRect {
                    x: 390.0,
                    y: 570.0,
                    width: 420.0,
                    height: 180.0,
                },
                content_frame: PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 420.0,
                    height: 180.0,
                },
                pill_frame: PanelRect {
                    x: 76.0,
                    y: 141.875,
                    width: 268.0,
                    height: 37.0,
                },
                left_shoulder_frame: PanelRect {
                    x: 70.0,
                    y: 172.875,
                    width: 6.0,
                    height: 6.0,
                },
                right_shoulder_frame: PanelRect {
                    x: 344.0,
                    y: 172.875,
                    width: 6.0,
                    height: 6.0,
                },
                expanded_frame: PanelRect {
                    x: 76.0,
                    y: 65.46875,
                    width: 268.0,
                    height: 113.40625,
                },
                cards_frame: PanelRect {
                    x: 10.0,
                    y: 10.0,
                    width: 248.0,
                    height: 57.40625,
                },
                separator_frame: PanelRect {
                    x: 14.0,
                    y: 75.90625,
                    width: 240.0,
                    height: 1.0,
                },
                shared_content_frame: PanelRect {
                    x: 476.0,
                    y: 645.46875,
                    width: 248.0,
                    height: 57.40625,
                },
                shell_visible: true,
                separator_visibility: 0.66,
            })
        );

        let render_state = renderer.last_render_state.expect("cached render state");
        assert!(!render_state.shared.enabled);
        assert!(!render_state.shared.visible);
        assert_eq!(
            render_state.layer_style.headline_emphasized,
            runtime_render_state.shell_scene.headline_emphasized
        );
        assert_eq!(
            render_state.layer_style.edge_actions_visible,
            runtime_render_state.shell_scene.edge_actions_visible
        );
        assert!(
            renderer
                .last_pointer_regions
                .iter()
                .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CardsContainer))
        );
        let command_bundle = renderer
            .last_render_command_bundle
            .as_ref()
            .expect("cached render command bundle");
        assert_eq!(
            command_bundle.compact_bar.frame,
            command_bundle.layout.pill_frame
        );
        assert_eq!(
            command_bundle.compact_bar.headline.text,
            scene.compact_bar.headline.text
        );
        assert_eq!(
            command_bundle.card_stack.frame,
            command_bundle.layout.cards_frame
        );
        assert_eq!(command_bundle.card_stack.cards.len(), scene.cards.len());
        assert_eq!(command_bundle.mascot.pose, scene.mascot_pose);
    }

    #[test]
    fn windows_renderer_resolves_pointer_regions_from_shared_scene_and_layout() {
        let mut panel_state = PanelState::default();
        let bundle = test_runtime_scene_bundle(
            &mut panel_state,
            &pending_permission_snapshot("session-1"),
            &PanelSceneBuildInput::default(),
        );
        let scene = bundle.scene;
        let runtime_render_state = bundle.runtime_render_state;
        let mut renderer = super::WindowsNativePanelRenderer::default();

        renderer.update_screen_frame(Some(PanelRect {
            x: 100.0,
            y: 50.0,
            width: 1000.0,
            height: 700.0,
        }));
        renderer
            .render_scene(&scene, runtime_render_state)
            .expect("render scene");
        renderer
            .apply_animation_descriptor(PanelAnimationDescriptor {
                kind: PanelAnimationKind::Open,
                canvas_height: 180.0,
                visible_height: 140.0,
                width_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 1.0,
                drop_progress: 1.0,
                cards_progress: 1.0,
            })
            .expect("apply descriptor");

        assert!(
            renderer
                .last_pointer_regions
                .iter()
                .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CompactBar))
        );
        assert!(
            renderer
                .last_pointer_regions
                .iter()
                .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CardsContainer))
        );
        assert!(renderer.last_pointer_regions.iter().any(|region| matches!(
            &region.kind,
            NativePanelPointerRegionKind::HitTarget(target)
                if target.action == PanelHitAction::FocusSession
                    && target.value == "session-1"
        )));
    }

    #[test]
    fn windows_renderer_caches_complete_render_commands() {
        let mut expanded_state = PanelState {
            expanded: true,
            ..PanelState::default()
        };
        let expanded_bundle = test_runtime_scene_bundle(
            &mut expanded_state,
            &snapshot(),
            &PanelSceneBuildInput::default(),
        );
        let mut renderer = super::WindowsNativePanelRenderer::default();
        renderer
            .render_scene(&expanded_bundle.scene, expanded_bundle.runtime_render_state)
            .expect("render expanded scene");
        renderer
            .apply_animation_descriptor(PanelAnimationDescriptor {
                kind: PanelAnimationKind::Open,
                canvas_height: 180.0,
                visible_height: 180.0,
                width_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 1.0,
                drop_progress: 1.0,
                cards_progress: 1.0,
            })
            .expect("apply expanded descriptor");

        let expanded_command = renderer
            .last_render_command_bundle
            .as_ref()
            .expect("expanded render command");
        assert!(expanded_command.compact_bar.actions_visible);
        assert_eq!(
            expanded_command.card_stack.cards.len(),
            expanded_bundle.scene.cards.len()
        );
        assert_eq!(
            expanded_command.mascot.pose,
            expanded_bundle.scene.mascot_pose
        );
        assert_eq!(expanded_command.action_buttons.len(), 2);
        assert!(expanded_command.glow.is_none());

        let completion_state = PanelState {
            completion_badge_items: vec![CompletionBadgeItem {
                session_id: "session-1".to_string(),
                completed_at: Utc::now(),
                last_user_prompt: None,
                last_assistant_message: Some("Done".to_string()),
            }],
            ..PanelState::default()
        };
        let completion_scene = build_panel_scene(
            &completion_state,
            &snapshot(),
            &PanelSceneBuildInput::default(),
        );
        renderer
            .render_scene(&completion_scene, PanelRuntimeRenderState::default())
            .expect("render completion scene");
        renderer
            .apply_animation_descriptor(PanelAnimationDescriptor {
                kind: PanelAnimationKind::Close,
                canvas_height: 80.0,
                visible_height: 80.0,
                width_progress: 0.0,
                height_progress: 0.0,
                shoulder_progress: 0.0,
                drop_progress: 0.0,
                cards_progress: 0.0,
            })
            .expect("apply completion descriptor");

        let completion_command = renderer
            .last_render_command_bundle
            .as_ref()
            .expect("completion render command");
        assert!(completion_command.glow.is_some());
        assert_eq!(
            completion_command.compact_bar.completion_count,
            completion_scene.compact_bar.completion_count
        );
    }

    #[test]
    fn windows_host_recomputes_cached_frame_when_display_changes() {
        let mut host = super::WindowsNativePanelHost::default();
        let descriptor = PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 120.0,
            visible_height: 120.0,
            width_progress: 1.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        };

        host.apply_animation_descriptor(descriptor)
            .expect("apply descriptor");
        host.reposition_to_display(
            1,
            Some(PanelRect {
                x: 500.0,
                y: 100.0,
                width: 800.0,
                height: 600.0,
            }),
        )
        .expect("reposition host");

        assert_eq!(host.window.descriptor.preferred_display_index, 1);
        assert_eq!(
            host.window.last_frame,
            Some(PanelRect {
                x: 759.0,
                y: 580.0,
                width: 283.0,
                height: 120.0,
            })
        );
        assert_eq!(
            host.renderer.last_window_state,
            Some(host.window.window_state())
        );
    }

    #[test]
    fn windows_window_frame_interpolates_descriptor_width() {
        let descriptor = PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 120.0,
            visible_height: 160.0,
            width_progress: 0.25,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        };

        let frame = super::host_window::resolve_windows_panel_window_frame(
            descriptor,
            PanelRect {
                x: 100.0,
                y: 50.0,
                width: 1000.0,
                height: 700.0,
            },
            200.0,
            400.0,
        );

        assert_eq!(
            frame,
            PanelRect {
                x: 475.0,
                y: 590.0,
                width: 250.0,
                height: 160.0,
            }
        );
    }

    #[test]
    fn windows_native_ui_remains_disabled() {
        assert!(!super::native_ui_enabled());
    }
}
