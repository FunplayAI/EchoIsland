use std::{ptr::NonNull, sync::Once};

use block2::RcBlock;
use objc2_app_kit::{
    NSWorkspace, NSWorkspaceDidWakeNotification, NSWorkspaceScreensDidSleepNotification,
    NSWorkspaceScreensDidWakeNotification, NSWorkspaceSessionDidBecomeActiveNotification,
    NSWorkspaceSessionDidResignActiveNotification, NSWorkspaceWillSleepNotification,
};
use objc2_foundation::{NSNotification, NSNotificationName};

static INSTALL_WORKSPACE_OBSERVERS: Once = Once::new();

pub(crate) fn install_macos_lifecycle_diagnostics() {
    INSTALL_WORKSPACE_OBSERVERS.call_once(|| {
        crate::diagnostics::log_diagnostic_event("macos_lifecycle_observers_install_begin", &[]);
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let center = workspace.notificationCenter();
            register_workspace_notification(
                &center,
                NSWorkspaceWillSleepNotification,
                "workspace_will_sleep",
            );
            register_workspace_notification(
                &center,
                NSWorkspaceDidWakeNotification,
                "workspace_did_wake",
            );
            register_workspace_notification(
                &center,
                NSWorkspaceScreensDidSleepNotification,
                "workspace_screens_did_sleep",
            );
            register_workspace_notification(
                &center,
                NSWorkspaceScreensDidWakeNotification,
                "workspace_screens_did_wake",
            );
            register_workspace_notification(
                &center,
                NSWorkspaceSessionDidResignActiveNotification,
                "workspace_session_did_resign_active",
            );
            register_workspace_notification(
                &center,
                NSWorkspaceSessionDidBecomeActiveNotification,
                "workspace_session_did_become_active",
            );
        }
        crate::diagnostics::log_diagnostic_event("macos_lifecycle_observers_install_complete", &[]);
    });
}

unsafe fn register_workspace_notification(
    center: &objc2_foundation::NSNotificationCenter,
    notification_name: &'static NSNotificationName,
    event: &'static str,
) {
    let block = RcBlock::new(move |_notification: NonNull<NSNotification>| {
        let mut fields = vec![
            ("source", "nsworkspace".to_string()),
            ("notification", event.to_string()),
        ];
        fields.extend(crate::diagnostics::current_context_fields());
        crate::diagnostics::log_diagnostic_event("macos_lifecycle_notification", &fields);
    });
    let observer = unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(notification_name),
            None,
            None,
            &block,
        )
    };

    // The app needs these observers for the full process lifetime. Leaking the
    // tiny observer token avoids invalidating AppKit's copied notification block.
    std::mem::forget(observer);
}
