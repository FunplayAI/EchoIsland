use echoisland_runtime::SessionSnapshotView;

#[cfg(test)]
use echoisland_runtime::{PendingPermissionView, PendingQuestionView, RuntimeSnapshot};

#[cfg(test)]
pub(super) fn displayed_pending_permissions(
    snapshot: &RuntimeSnapshot,
) -> Vec<PendingPermissionView> {
    crate::native_panel_core::displayed_pending_permissions(snapshot)
}

#[cfg(test)]
pub(super) fn displayed_pending_questions(snapshot: &RuntimeSnapshot) -> Vec<PendingQuestionView> {
    crate::native_panel_core::displayed_pending_questions(snapshot)
}

pub(super) fn normalize_status(status: &str) -> String {
    crate::native_panel_core::normalize_status(status)
}

pub(super) fn format_source(source: &str) -> String {
    crate::native_panel_core::format_source(source)
}

pub(super) fn format_status(status: &str) -> String {
    crate::native_panel_core::format_status(status)
}

pub(super) fn session_title(session: &SessionSnapshotView) -> String {
    crate::native_panel_core::session_title(session)
}

pub(super) fn compact_title(value: &str, max_length: usize) -> String {
    crate::native_panel_core::compact_title(value, max_length)
}

pub(super) fn session_meta_line(session: &SessionSnapshotView) -> String {
    crate::native_panel_core::session_meta_line(session)
}
