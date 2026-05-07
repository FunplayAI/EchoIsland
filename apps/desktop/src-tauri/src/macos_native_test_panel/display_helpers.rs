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

pub(super) fn compact_title(value: &str, max_length: usize) -> String {
    crate::native_panel_core::compact_title(value, max_length)
}
