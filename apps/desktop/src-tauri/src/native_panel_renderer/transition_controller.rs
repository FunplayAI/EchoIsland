use echoisland_runtime::RuntimeSnapshot;

use crate::native_panel_core::{HoverTransition, PanelAnimationTimeline, PanelSnapshotSyncResult};

use super::descriptors::{
    NativePanelTimelineDescriptor, native_panel_timeline_descriptor_for_animation,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelTransitionRequest {
    Open,
    Close,
    SurfaceSwitch,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelPendingTransition {
    pub(crate) request: NativePanelTransitionRequest,
    pub(crate) snapshot: RuntimeSnapshot,
}

pub(crate) trait NativePanelTransitionRequestDispatcher {
    type Error;

    fn dispatch_panel_transition(
        &mut self,
        snapshot: RuntimeSnapshot,
        expanded: bool,
    ) -> Result<(), Self::Error>;

    fn dispatch_surface_transition(&mut self, snapshot: RuntimeSnapshot)
    -> Result<(), Self::Error>;
}

impl NativePanelTransitionRequest {
    pub(crate) fn panel_expanded(self) -> Option<bool> {
        match self {
            Self::Open => Some(true),
            Self::Close => Some(false),
            Self::SurfaceSwitch => None,
        }
    }

    pub(crate) fn is_surface_switch(self) -> bool {
        matches!(self, Self::SurfaceSwitch)
    }
}

pub(crate) fn native_panel_transition_request_for_hover_transition(
    transition: Option<HoverTransition>,
) -> Option<NativePanelTransitionRequest> {
    match transition {
        Some(HoverTransition::Expand) => Some(NativePanelTransitionRequest::Open),
        Some(HoverTransition::Collapse) => Some(NativePanelTransitionRequest::Close),
        None => None,
    }
}

pub(crate) fn native_panel_transition_request_for_snapshot_sync(
    sync: &PanelSnapshotSyncResult,
) -> Option<NativePanelTransitionRequest> {
    match sync.panel_transition {
        Some(true) => Some(NativePanelTransitionRequest::Open),
        Some(false) => Some(NativePanelTransitionRequest::Close),
        None if sync.surface_transition => Some(NativePanelTransitionRequest::SurfaceSwitch),
        None => None,
    }
}

pub(crate) fn native_panel_transition_request_for_surface_change(
    changed: bool,
    expanded: bool,
    transitioning: bool,
) -> Option<NativePanelTransitionRequest> {
    (changed && expanded && !transitioning).then_some(NativePanelTransitionRequest::SurfaceSwitch)
}

pub(crate) fn dispatch_native_panel_transition_request_with_snapshot<D>(
    dispatcher: &mut D,
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
) -> Result<bool, D::Error>
where
    D: NativePanelTransitionRequestDispatcher,
{
    dispatch_native_panel_transition_request_with_snapshot_via(
        request,
        snapshot,
        |request, snapshot| match request {
            NativePanelTransitionRequest::Open => {
                dispatcher.dispatch_panel_transition(snapshot, true)
            }
            NativePanelTransitionRequest::Close => {
                dispatcher.dispatch_panel_transition(snapshot, false)
            }
            NativePanelTransitionRequest::SurfaceSwitch => {
                dispatcher.dispatch_surface_transition(snapshot)
            }
        },
    )
}

pub(crate) fn dispatch_native_panel_transition_request_with_snapshot_via<E>(
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
    dispatch: impl FnOnce(NativePanelTransitionRequest, RuntimeSnapshot) -> Result<(), E>,
) -> Result<bool, E> {
    let (Some(request), Some(snapshot)) = (request, snapshot) else {
        return Ok(false);
    };

    dispatch(request, snapshot)?;

    Ok(true)
}

pub(crate) fn pending_native_panel_transition_if_active(
    transitioning: bool,
    request: NativePanelTransitionRequest,
    snapshot: RuntimeSnapshot,
) -> Option<NativePanelPendingTransition> {
    transitioning.then_some(NativePanelPendingTransition { request, snapshot })
}

pub(crate) fn take_pending_native_panel_transition_after_completed(
    pending: &mut Option<NativePanelPendingTransition>,
    completed_request: NativePanelTransitionRequest,
) -> Option<NativePanelPendingTransition> {
    let pending_transition = pending.take()?;
    (pending_transition.request != completed_request).then_some(pending_transition)
}

pub(crate) fn clear_pending_native_panel_transition_request(
    pending: &mut Option<NativePanelPendingTransition>,
    request: NativePanelTransitionRequest,
) -> bool {
    if pending
        .as_ref()
        .is_some_and(|pending| pending.request == request)
    {
        pending.take();
        true
    } else {
        false
    }
}

pub(crate) fn dispatch_native_panel_transition_request_or_fallback<D>(
    dispatcher: &mut D,
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
    fallback: impl FnOnce() -> Result<(), D::Error>,
) -> Result<(), D::Error>
where
    D: NativePanelTransitionRequestDispatcher,
{
    dispatch_native_panel_transition_request_or_fallback_via(
        request,
        snapshot,
        |request, snapshot| match request {
            NativePanelTransitionRequest::Open => {
                dispatcher.dispatch_panel_transition(snapshot, true)
            }
            NativePanelTransitionRequest::Close => {
                dispatcher.dispatch_panel_transition(snapshot, false)
            }
            NativePanelTransitionRequest::SurfaceSwitch => {
                dispatcher.dispatch_surface_transition(snapshot)
            }
        },
        fallback,
    )
}

pub(crate) fn dispatch_native_panel_transition_request_or_fallback_via<E>(
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
    dispatch: impl FnOnce(NativePanelTransitionRequest, RuntimeSnapshot) -> Result<(), E>,
    fallback: impl FnOnce() -> Result<(), E>,
) -> Result<(), E> {
    if dispatch_native_panel_transition_request_with_snapshot_via(request, snapshot, dispatch)? {
        return Ok(());
    }

    fallback()
}

pub(crate) fn resolve_native_panel_animation_timeline(
    request: NativePanelTransitionRequest,
    start_height: f64,
    target_height: f64,
    card_count: usize,
) -> PanelAnimationTimeline {
    match request {
        NativePanelTransitionRequest::Open => {
            PanelAnimationTimeline::open(start_height, target_height, card_count)
        }
        NativePanelTransitionRequest::Close => {
            PanelAnimationTimeline::close(start_height, card_count)
        }
        NativePanelTransitionRequest::SurfaceSwitch => {
            PanelAnimationTimeline::surface_switch(start_height, target_height, card_count)
        }
    }
}

pub(crate) fn resolve_native_panel_terminal_timeline_descriptor(
    request: NativePanelTransitionRequest,
    start_height: f64,
    target_height: f64,
    card_count: usize,
) -> NativePanelTimelineDescriptor {
    let timeline =
        resolve_native_panel_animation_timeline(request, start_height, target_height, card_count);
    native_panel_timeline_descriptor_for_animation(timeline.sample(timeline.total_ms()))
}

#[cfg(test)]
mod tests {
    use echoisland_runtime::RuntimeSnapshot;

    use crate::native_panel_core::{
        PanelAnimationKind, PanelSnapshotSyncResult, card_transition_total_ms,
    };

    use super::{
        NativePanelTransitionRequest, NativePanelTransitionRequestDispatcher,
        clear_pending_native_panel_transition_request,
        dispatch_native_panel_transition_request_or_fallback,
        dispatch_native_panel_transition_request_or_fallback_via,
        dispatch_native_panel_transition_request_with_snapshot,
        dispatch_native_panel_transition_request_with_snapshot_via,
        native_panel_transition_request_for_hover_transition,
        native_panel_transition_request_for_snapshot_sync,
        native_panel_transition_request_for_surface_change,
        pending_native_panel_transition_if_active, resolve_native_panel_animation_timeline,
        resolve_native_panel_terminal_timeline_descriptor,
        take_pending_native_panel_transition_after_completed,
    };

    #[derive(Default)]
    struct RecordingDispatcher {
        calls: Vec<(String, bool)>,
    }

    impl NativePanelTransitionRequestDispatcher for RecordingDispatcher {
        type Error = String;

        fn dispatch_panel_transition(
            &mut self,
            snapshot: RuntimeSnapshot,
            expanded: bool,
        ) -> Result<(), Self::Error> {
            self.calls.push((snapshot.status, expanded));
            Ok(())
        }

        fn dispatch_surface_transition(
            &mut self,
            snapshot: RuntimeSnapshot,
        ) -> Result<(), Self::Error> {
            self.calls.push((snapshot.status, false));
            Ok(())
        }
    }

    fn snapshot(status: &str) -> RuntimeSnapshot {
        RuntimeSnapshot {
            status: status.to_string(),
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
        }
    }

    #[test]
    fn hover_transition_maps_to_panel_transition_request() {
        assert_eq!(
            native_panel_transition_request_for_hover_transition(Some(
                crate::native_panel_core::HoverTransition::Expand
            )),
            Some(NativePanelTransitionRequest::Open)
        );
        assert_eq!(
            native_panel_transition_request_for_hover_transition(Some(
                crate::native_panel_core::HoverTransition::Collapse
            )),
            Some(NativePanelTransitionRequest::Close)
        );
        assert_eq!(
            native_panel_transition_request_for_hover_transition(None),
            None
        );
    }

    #[test]
    fn snapshot_sync_prefers_panel_transition_before_surface_switch() {
        let open_sync = PanelSnapshotSyncResult {
            displayed_snapshot: echoisland_runtime::RuntimeSnapshot {
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
            },
            reminder: crate::native_panel_core::PanelReminderState::default(),
            panel_transition: Some(true),
            surface_transition: true,
        };
        let surface_sync = PanelSnapshotSyncResult {
            panel_transition: None,
            surface_transition: true,
            ..open_sync.clone()
        };

        assert_eq!(
            native_panel_transition_request_for_snapshot_sync(&open_sync),
            Some(NativePanelTransitionRequest::Open)
        );
        assert_eq!(
            native_panel_transition_request_for_snapshot_sync(&surface_sync),
            Some(NativePanelTransitionRequest::SurfaceSwitch)
        );
    }

    #[test]
    fn surface_change_only_requests_switch_when_runtime_can_animate() {
        assert_eq!(
            native_panel_transition_request_for_surface_change(true, true, false),
            Some(NativePanelTransitionRequest::SurfaceSwitch)
        );
        assert_eq!(
            native_panel_transition_request_for_surface_change(true, false, false),
            None
        );
        assert_eq!(
            native_panel_transition_request_for_surface_change(true, true, true),
            None
        );
    }

    #[test]
    fn transition_request_resolves_shared_timeline_and_terminal_descriptor() {
        let timeline = resolve_native_panel_animation_timeline(
            NativePanelTransitionRequest::Open,
            80.0,
            164.0,
            3,
        );
        assert_eq!(
            timeline.total_ms(),
            crate::native_panel_core::PANEL_OPEN_TOTAL_MS
                + card_transition_total_ms(
                    3,
                    crate::native_panel_core::PANEL_CARD_REVEAL_MS,
                    crate::native_panel_core::PANEL_CARD_REVEAL_STAGGER_MS
                )
        );

        let descriptor = resolve_native_panel_terminal_timeline_descriptor(
            NativePanelTransitionRequest::SurfaceSwitch,
            120.0,
            164.0,
            2,
        );

        assert_eq!(descriptor.animation.kind, PanelAnimationKind::SurfaceSwitch);
        assert_eq!(descriptor.animation.visible_height, 164.0);
        assert!(descriptor.cards_entering);
    }

    #[test]
    fn transition_request_dispatch_helper_routes_request_when_bundle_is_complete() {
        let mut dispatcher = RecordingDispatcher::default();

        let dispatched = dispatch_native_panel_transition_request_with_snapshot(
            &mut dispatcher,
            Some(NativePanelTransitionRequest::Open),
            Some(snapshot("idle")),
        )
        .expect("dispatch transition bundle");

        assert!(dispatched);
        assert_eq!(dispatcher.calls, vec![("idle".to_string(), true)]);
    }

    #[test]
    fn transition_request_dispatch_helper_skips_when_bundle_is_incomplete() {
        let mut dispatcher = RecordingDispatcher::default();

        let dispatched = dispatch_native_panel_transition_request_with_snapshot(
            &mut dispatcher,
            Some(NativePanelTransitionRequest::Open),
            None,
        )
        .expect("skip incomplete bundle");

        assert!(!dispatched);
        assert!(dispatcher.calls.is_empty());
    }

    #[test]
    fn closure_dispatch_helper_routes_request_when_bundle_is_complete() {
        let calls = std::cell::RefCell::new(Vec::new());

        let dispatched = dispatch_native_panel_transition_request_with_snapshot_via(
            Some(NativePanelTransitionRequest::Open),
            Some(snapshot("idle")),
            |request, snapshot| {
                calls.borrow_mut().push(match request {
                    NativePanelTransitionRequest::Open => (snapshot.status, true),
                    NativePanelTransitionRequest::Close => (snapshot.status, false),
                    NativePanelTransitionRequest::SurfaceSwitch => (snapshot.status, false),
                });
                Ok::<_, String>(())
            },
        )
        .expect("dispatch transition bundle via closures");

        assert!(dispatched);
        assert_eq!(calls.into_inner(), vec![("idle".to_string(), true)]);
    }

    #[test]
    fn transition_request_or_fallback_runs_fallback_when_bundle_is_incomplete() {
        let mut dispatcher = RecordingDispatcher::default();
        let mut fallback_called = false;

        dispatch_native_panel_transition_request_or_fallback(
            &mut dispatcher,
            Some(NativePanelTransitionRequest::Open),
            None,
            || {
                fallback_called = true;
                Ok(())
            },
        )
        .expect("run fallback for incomplete transition bundle");

        assert!(fallback_called);
        assert!(dispatcher.calls.is_empty());
    }

    #[test]
    fn closure_transition_request_or_fallback_runs_fallback_when_bundle_is_incomplete() {
        let mut fallback_called = false;

        dispatch_native_panel_transition_request_or_fallback_via(
            Some(NativePanelTransitionRequest::Open),
            None,
            |_request, _snapshot| Ok::<_, String>(()),
            || {
                fallback_called = true;
                Ok::<_, String>(())
            },
        )
        .expect("run closure fallback for incomplete transition bundle");

        assert!(fallback_called);
    }

    #[test]
    fn transition_request_or_fallback_skips_fallback_when_transition_dispatches() {
        let mut dispatcher = RecordingDispatcher::default();

        dispatch_native_panel_transition_request_or_fallback(
            &mut dispatcher,
            Some(NativePanelTransitionRequest::Open),
            Some(snapshot("idle")),
            || panic!("fallback should not run when transition dispatches"),
        )
        .expect("dispatch transition without fallback");

        assert_eq!(dispatcher.calls, vec![("idle".to_string(), true)]);
    }

    #[test]
    fn pending_transition_is_created_only_while_active() {
        assert!(
            pending_native_panel_transition_if_active(
                false,
                NativePanelTransitionRequest::Close,
                snapshot("idle")
            )
            .is_none()
        );

        let pending = pending_native_panel_transition_if_active(
            true,
            NativePanelTransitionRequest::Close,
            snapshot("idle"),
        )
        .expect("pending transition");

        assert_eq!(pending.request, NativePanelTransitionRequest::Close);
        assert_eq!(pending.snapshot.status, "idle");
    }

    #[test]
    fn pending_transition_after_completion_uses_last_distinct_request() {
        let mut pending = pending_native_panel_transition_if_active(
            true,
            NativePanelTransitionRequest::Close,
            snapshot("idle"),
        );

        let next = take_pending_native_panel_transition_after_completed(
            &mut pending,
            NativePanelTransitionRequest::Open,
        )
        .expect("distinct pending request");

        assert_eq!(next.request, NativePanelTransitionRequest::Close);
        assert!(pending.is_none());
    }

    #[test]
    fn pending_transition_after_completion_discards_duplicate_request() {
        let mut pending = pending_native_panel_transition_if_active(
            true,
            NativePanelTransitionRequest::Open,
            snapshot("running"),
        );

        assert!(
            take_pending_native_panel_transition_after_completed(
                &mut pending,
                NativePanelTransitionRequest::Open,
            )
            .is_none()
        );
        assert!(pending.is_none());
    }

    #[test]
    fn pending_transition_clear_only_matches_requested_kind() {
        let mut pending = pending_native_panel_transition_if_active(
            true,
            NativePanelTransitionRequest::Close,
            snapshot("idle"),
        );

        assert!(!clear_pending_native_panel_transition_request(
            &mut pending,
            NativePanelTransitionRequest::Open
        ));
        assert!(pending.is_some());
        assert!(clear_pending_native_panel_transition_request(
            &mut pending,
            NativePanelTransitionRequest::Close
        ));
        assert!(pending.is_none());
    }
}
