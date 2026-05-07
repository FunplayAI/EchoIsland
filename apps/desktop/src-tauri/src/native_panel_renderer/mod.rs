#![allow(dead_code, unused_imports)]

mod action_button_visual_spec;
mod animation_plan;
mod animation_scheduler;
mod card_visual_spec;
mod completion_glow_visual_spec;
mod descriptors;
mod env_flags;
mod host_runtime_facade;
mod mascot_visual_spec;
mod platform_adapter;
mod presentation_model;
mod render_commands;
mod renderer_backend;
mod runtime_backend;
mod runtime_click;
mod runtime_commands;
mod runtime_hover;
mod runtime_interaction;
mod runtime_pointer_input;
mod runtime_polling;
mod runtime_render_payload;
mod runtime_scene_cache;
mod runtime_scene_sync;
mod runtime_settings_surface;
mod runtime_transition_slots;
mod shell_command;
mod shell_pump;
mod shell_state;
mod traits;
mod transition_controller;
mod visual_plan;
mod visual_primitives;
mod window_message_pump;

#[cfg(test)]
mod runtime_tests;

pub(crate) mod facade {
    pub(crate) mod command {
        pub(crate) use super::super::descriptors::{
            NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerInputOutcome,
            NativePanelRuntimeCommandCapability, NativePanelRuntimeCommandHandler,
        };
        pub(crate) use super::super::host_runtime_facade::NativePanelRuntimeDispatchMode;
        pub(crate) use super::super::runtime_click::dispatch_queued_native_panel_platform_events_with_handler;
        pub(crate) use super::super::runtime_commands::{
            dispatch_drained_native_panel_platform_events_with_app_handle,
            dispatch_native_panel_click_command_with_app_handle,
            dispatch_native_panel_platform_events_with_app_handle,
            execute_native_panel_settings_surface_command,
            run_native_panel_pointer_input_with_queued_command_dispatch,
            run_native_panel_runtime_with_queued_command_dispatch,
            spawn_native_panel_platform_event_dispatch_loop,
            spawn_native_panel_platform_loops_with_event_dispatch,
        };
    }

    pub(crate) mod descriptor {
        pub(crate) use super::super::descriptors::{
            NativePanelEdgeAction, NativePanelEdgeActionFrames, NativePanelHostWindowDescriptor,
            NativePanelHostWindowState, NativePanelInteractionPlan, NativePanelPointerInput,
            NativePanelPointerPointState, NativePanelPointerRegion, NativePanelPointerRegionInput,
            NativePanelPointerRegionKind, NativePanelRuntimeInputContext,
            NativePanelRuntimeInputDescriptor, NativePanelTimelineDescriptor,
            native_panel_host_window_descriptor, native_panel_host_window_frame,
            native_panel_pointer_inside_for_input, native_panel_pointer_inside_regions,
            native_panel_pointer_state_at_point, native_panel_timeline_descriptor,
            native_panel_timeline_descriptor_for_animation, resolve_native_panel_interaction_plan,
            resolve_native_panel_pointer_regions,
        };
        pub(crate) use super::super::host_runtime_facade::NativePanelComputedHostWindow;
    }

    pub(crate) mod env {
        pub(crate) use super::super::env_flags::{
            native_panel_enabled_from_env_value, native_panel_enabled_from_webview_env_value,
            native_panel_feature_enabled_from_env_value,
        };
    }

    pub(crate) mod host {
        pub(crate) use super::super::host_runtime_facade::{
            NativePanelHostDisplayReposition, NativePanelRuntimeHostController,
            NativePanelRuntimeHostState, create_native_panel_via_host_controller,
            hide_native_panel_via_host_controller, native_panel_host_display_reposition,
            reposition_native_panel_host_from_input_descriptor_via_controller,
            set_native_panel_host_shared_body_height_via_controller,
            sync_runtime_host_display_reposition_in_state, sync_runtime_host_screen_frame_in_state,
            sync_runtime_host_shared_body_height_in_state, sync_runtime_host_timeline_in_state,
            sync_runtime_host_visibility_in_state, sync_runtime_pointer_regions_in_state,
        };
        pub(crate) use super::super::renderer_backend::native_panel_presentation_cards_visible;
        pub(crate) use super::super::runtime_backend::hide_main_webview_window_when_native_ui_enabled;
        pub(crate) use super::super::traits::{NativePanelHost, NativePanelSceneHost};
    }

    pub(crate) mod interaction {
        pub(crate) use super::super::descriptors::NativePanelRuntimeCommandHandler;
        pub(crate) use super::super::runtime_click::dispatch_native_panel_click_command_at_point_with_handler;
        pub(crate) use super::super::runtime_hover::{
            sync_native_panel_hover_expansion_state_for_state,
            sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor,
            sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor,
            sync_native_panel_hover_interaction_and_rerender_for_pointer_input_with_input_descriptor,
            sync_native_panel_hover_interaction_at_point_for_state,
            sync_native_panel_hover_interaction_for_pointer_input_for_state,
            sync_native_panel_hover_interaction_for_state,
        };
        pub(crate) use super::super::runtime_interaction::{
            NativePanelClickStateBridge, NativePanelCoreStateBridge,
            NativePanelHostBehaviorCommand, NativePanelHostBehaviorPlan,
            NativePanelHostInteractionStateBridge, NativePanelHostPollingInteractionResult,
            NativePanelHoverFallbackFrames, NativePanelHoverSyncResult,
            NativePanelPointerInputRuntimeBridge, NativePanelPointerRegionInteractionBridge,
            NativePanelPollingHostFacts, NativePanelPrimaryPointerStateBridge,
            NativePanelQueuedPlatformEventBridge, NativePanelSettingsSurfaceSnapshotUpdate,
            NativePanelSettingsSurfaceToggleResult, native_panel_click_state_slots,
            record_native_panel_focus_click_session, resolve_native_panel_last_focus_click,
        };
        pub(crate) use super::super::runtime_pointer_input::{
            handle_native_panel_pointer_input_with_handler,
            handle_optional_native_panel_pointer_input_with_handler,
        };
        pub(crate) use super::super::runtime_polling::{
            native_panel_interactive_inside_from_host_facts,
            native_panel_polling_interaction_input_from_host_facts,
            resolve_native_panel_host_behavior_plan, resolve_native_panel_hover_fallback_frames,
            sync_native_panel_host_behavior_for_interactive_inside,
            sync_native_panel_host_polling_interaction_for_state,
            sync_native_panel_host_polling_interaction_from_host_facts_for_state,
            sync_native_panel_mouse_passthrough_for_interactive_inside,
        };
        pub(crate) use super::super::runtime_settings_surface::{
            resolve_native_panel_settings_surface_snapshot_update_for_state,
            toggle_native_panel_settings_surface_for_state,
        };
        pub(crate) use super::super::runtime_transition_slots::sync_native_panel_hover_and_refresh_for_runtime;
    }

    pub(crate) mod presentation {
        pub(crate) use super::super::action_button_visual_spec::{
            ActionButtonVisibilitySpecInput, action_button_transition_progress_from_compact_width,
            action_button_visual_frame_for_phase, resolve_action_button_visibility_spec,
        };
        pub(crate) use super::super::card_visual_spec::{
            CardVisualAnimationSpec, CardVisualBadgeRole, CardVisualBadgeSpec, CardVisualBodyRole,
            CardVisualBodySpec, CardVisualColorSpec, CardVisualRowSpec, CardVisualShellSpec,
            CardVisualSpec, CardVisualStyle, card_visual_action_hint_layout,
            card_visual_badge_layout, card_visual_body_layout, card_visual_body_line_paint_spec,
            card_visual_content_layout, card_visual_content_visibility_phase,
            card_visual_header_text_paint_spec, card_visual_settings_row_layout,
            card_visual_shell_border_color, card_visual_shell_fill_color,
            card_visual_spec_from_scene_card_with_height, card_visual_staggered_phase,
            card_visual_tool_pill_layout,
        };
        pub(crate) use super::super::completion_glow_visual_spec::{
            COMPLETION_GLOW_IMAGE_HEIGHT, COMPLETION_GLOW_IMAGE_RADIUS,
            COMPLETION_GLOW_IMAGE_WIDTH, COMPLETION_GLOW_SLICE_LEFT, COMPLETION_GLOW_SLICE_RIGHT,
            COMPLETION_GLOW_VISIBLE_THRESHOLD, CompletionGlowImageSliceSpec,
            CompletionGlowVisualSpecInput, resolve_completion_glow_image_slices,
            resolve_completion_glow_visual_spec,
        };
        pub(crate) use super::super::mascot_visual_spec::{
            MascotCompletionBadgeVisualSpec, MascotEllipseVisualSpec,
            MascotMessageBubbleVisualSpec, MascotTextVisualSpec, MascotVisualSpec,
            MascotVisualSpecInput, resolve_mascot_visual_spec,
        };
        pub(crate) use super::super::presentation_model::{
            NativePanelActionButtonsPresentation, NativePanelCardStackPresentation,
            NativePanelCompactBarPresentation, NativePanelGlowPresentation,
            NativePanelMascotPresentation, NativePanelPresentationMetrics,
            NativePanelPresentationModel, NativePanelResolvedPresentation,
            NativePanelShellPresentation, NativePanelSnapshotRenderPlan,
            estimated_scene_card_height, native_panel_visual_display_mode_from_presentation,
            native_panel_visual_plan_input_from_presentation, resolve_native_panel_presentation,
            resolve_native_panel_snapshot_render_plan_for_scene,
        };
        pub(crate) use super::super::render_commands::{
            NativePanelActionButtonCommand, NativePanelCardStackCommand,
            NativePanelCompactBarCommand,
        };
        pub(crate) use super::super::visual_plan::{
            NativePanelVisualActionButtonInput, NativePanelVisualCardBadgeInput,
            NativePanelVisualCardBodyLineInput, NativePanelVisualCardBodyRole,
            NativePanelVisualCardInput, NativePanelVisualCardRowInput, NativePanelVisualCardStyle,
            NativePanelVisualDisplayMode, NativePanelVisualPlanInput,
            native_panel_visual_card_input_from_scene_card,
        };
        pub(crate) use super::super::visual_primitives::NativePanelVisualColor;
    }

    pub(crate) mod renderer {
        pub(crate) use super::super::animation_plan::{
            NativePanelAnimationPlan, NativePanelCardStackAnimationPlan,
            NativePanelStatusClosePreservationInput, NativePanelStatusClosePreservationPlan,
            NativePanelTransitionCardPhase, NativePanelTransitionLifecyclePlan,
            resolve_native_panel_animation_plan,
            resolve_native_panel_status_close_preservation_plan,
            resolve_native_panel_transition_lifecycle_plan,
        };
        pub(crate) use super::super::animation_scheduler::{
            NativePanelAnimationFrame, NativePanelAnimationFrameScheduler,
            NativePanelAnimationTarget,
        };
        pub(crate) use super::super::render_commands::{
            NativePanelRenderCommandBundle, resolve_native_panel_render_command_bundle,
        };
        pub(crate) use super::super::renderer_backend::{
            NativePanelCachedRendererBackend, cache_host_window_descriptor_on_renderer,
            cache_host_window_state_on_renderer, cache_pointer_regions_on_renderer,
            cache_render_command_bundle_on_renderer, cache_scene_runtime_on_renderer,
            cache_timeline_descriptor_on_renderer,
            resolve_and_cache_presentation_from_scene_cache_on_renderer,
            resolve_cached_presentation_model, sync_cached_presentation_model_slot,
            sync_cached_visibility_on_renderer,
        };
        pub(crate) use super::super::runtime_backend::cache_runtime_scene_sync_result;
        pub(crate) use super::super::runtime_interaction::NativePanelSceneRuntimeBridge;
        pub(crate) use super::super::runtime_scene_cache::{
            NativePanelRuntimeSceneCache, NativePanelRuntimeSceneMutableStateBridge,
            NativePanelRuntimeSceneStateBridge,
            build_native_panel_scene_for_state_bridge_with_input, cache_render_command_bundle,
            cache_render_command_bundle_for_state_bridge_with_input,
            cache_render_command_bundle_with_key, cache_scene_runtime_with_key,
            cached_runtime_render_state, cached_scene, native_panel_runtime_scene_cache_key,
            native_panel_runtime_scene_cache_key_for_state_bridge,
            resolve_and_cache_native_panel_presentation_for_state_bridge_with_input,
            resolve_current_native_panel_presentation_model_for_state_bridge_with_input,
            resolve_current_native_panel_render_command_bundle_for_state_bridge_with_input,
            resolve_native_panel_presentation_model_for_state_bridge_and_snapshot_with_input,
            resolve_native_panel_render_command_bundle_for_state_bridge_and_snapshot_with_input,
            resolve_native_panel_runtime_render_state_for_state_bridge_with_input,
            resolve_native_panel_scene_for_state_bridge_and_snapshot_with_input,
            resolve_native_panel_scene_for_state_bridge_with_input,
            resolve_native_panel_snapshot_render_plan_for_state_bridge_snapshot_with_input,
        };
        pub(crate) use super::super::runtime_scene_sync::sync_runtime_scene_bundle_from_state_input;
        pub(crate) use super::super::traits::NativePanelRenderer;
    }

    pub(crate) mod runtime {
        pub(crate) use super::super::host_runtime_facade::dispatch_native_panel_runtime_payload_with_handles;
        pub(crate) use super::super::runtime_backend::{
            NativePanelPlatformRuntimeBackendFacade, NativePanelPlatformRuntimeFacadeApi,
            NativePanelRuntimeBackend, NativePanelRuntimeSceneSyncResult,
            current_native_panel_runtime_backend,
            reposition_native_panel_to_selected_display_then_refresh,
            sync_runtime_scene_bundle_from_input_descriptor,
        };
        pub(crate) use super::super::runtime_render_payload::{
            NativePanelRuntimeRenderPayloadState, NativePanelRuntimeRenderPayloadStateBridge,
            dispatch_native_panel_runtime_render_payload_if_available,
            native_panel_runtime_render_payload_state_from_animation_plan,
            resolve_native_panel_runtime_render_payload_for_state,
        };
        pub(crate) use super::super::runtime_scene_sync::{
            rerender_runtime_scene_sync_result_to_host_for_runtime_with_input_descriptor,
            sync_runtime_scene_bundle_for_runtime_with_input,
            toggle_native_panel_settings_surface_and_rerender_for_runtime_with_input_descriptor,
        };
        pub(crate) use super::super::runtime_transition_slots::{
            apply_native_panel_hover_sync_result_for_runtime,
            apply_native_panel_runtime_scene_sync_result_for_runtime,
            apply_native_panel_settings_surface_toggle_result_for_runtime,
        };
    }

    pub(crate) mod shell {
        pub(crate) use super::super::platform_adapter::{
            NativePanelPlatformThreadAdapter, NativePanelPlatformWindowHandleAdapter,
            dispatch_native_panel_on_platform_thread, native_panel_has_raw_window_handle,
            sync_native_panel_raw_window_handle,
        };
        pub(crate) use super::super::shell_command::{
            NativePanelHostShellCommand, NativePanelHostShellCommandBackend,
            apply_native_panel_host_shell_command,
        };
        pub(crate) use super::super::shell_pump::{
            NativePanelHostShellRuntimePump, pump_native_panel_host_shell_runtime,
        };
        pub(crate) use super::super::shell_state::{
            NativePanelHostShellLifecycle, NativePanelHostShellState,
        };
        pub(crate) use super::super::window_message_pump::{
            NativePanelPlatformWindowMessage, NativePanelPlatformWindowMessagePump,
            pump_native_panel_platform_window_messages,
        };
    }

    pub(crate) mod transition {
        pub(crate) use super::super::transition_controller::{
            NativePanelPendingTransition, NativePanelTransitionRequest,
            clear_pending_native_panel_transition_request,
            dispatch_native_panel_transition_request_or_fallback_via,
            dispatch_native_panel_transition_request_with_snapshot_via,
            native_panel_transition_request_for_snapshot_sync,
            pending_native_panel_transition_if_active, resolve_native_panel_animation_timeline,
            take_pending_native_panel_transition_after_completed,
        };
    }

    pub(crate) mod visual {
        pub(crate) use super::super::visual_plan::resolve_native_panel_visual_plan;
        pub(crate) use super::super::visual_primitives::{
            NativePanelVisualColor, NativePanelVisualMascotEllipseRole,
            NativePanelVisualMascotRoundRectRole, NativePanelVisualMascotTextRole,
            NativePanelVisualPlan, NativePanelVisualPrimitive, NativePanelVisualShoulderSide,
            NativePanelVisualTextAlignment, NativePanelVisualTextRole, NativePanelVisualTextWeight,
        };
    }
}
