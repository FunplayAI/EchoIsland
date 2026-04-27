use tauri::AppHandle;

pub(crate) trait NativePanelPlatformThreadAdapter<R: tauri::Runtime> {
    fn dispatch_on_platform_thread(
        &self,
        work: impl FnOnce() + Send + 'static,
    ) -> Result<(), String>;
}

impl<R: tauri::Runtime> NativePanelPlatformThreadAdapter<R> for AppHandle<R> {
    fn dispatch_on_platform_thread(
        &self,
        work: impl FnOnce() + Send + 'static,
    ) -> Result<(), String> {
        self.run_on_main_thread(work)
            .map_err(|error| error.to_string())
    }
}

pub(crate) fn dispatch_native_panel_on_platform_thread<R: tauri::Runtime>(
    app: &impl NativePanelPlatformThreadAdapter<R>,
    work: impl FnOnce() + Send + 'static,
) -> Result<(), String> {
    app.dispatch_on_platform_thread(work)
}

pub(crate) trait NativePanelPlatformWindowHandleAdapter {
    type RawHandle: Copy + PartialEq + Eq;

    fn raw_window_handle(&self) -> Option<Self::RawHandle>;

    fn set_raw_window_handle(&mut self, handle: Option<Self::RawHandle>);
}

pub(crate) fn sync_native_panel_raw_window_handle<T>(target: &mut T, handle: Option<T::RawHandle>)
where
    T: NativePanelPlatformWindowHandleAdapter,
{
    target.set_raw_window_handle(handle);
}

pub(crate) fn native_panel_has_raw_window_handle<T>(target: &T) -> bool
where
    T: NativePanelPlatformWindowHandleAdapter,
{
    target.raw_window_handle().is_some()
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelPlatformWindowHandleAdapter, native_panel_has_raw_window_handle,
        sync_native_panel_raw_window_handle,
    };

    #[derive(Default)]
    struct TestWindowHandle {
        handle: Option<isize>,
    }

    impl NativePanelPlatformWindowHandleAdapter for TestWindowHandle {
        type RawHandle = isize;

        fn raw_window_handle(&self) -> Option<Self::RawHandle> {
            self.handle
        }

        fn set_raw_window_handle(&mut self, handle: Option<Self::RawHandle>) {
            self.handle = handle;
        }
    }

    #[test]
    fn window_handle_helpers_sync_and_detect_presence() {
        let mut handle = TestWindowHandle::default();

        assert!(!native_panel_has_raw_window_handle(&handle));

        sync_native_panel_raw_window_handle(&mut handle, Some(42));

        assert_eq!(handle.raw_window_handle(), Some(42));
        assert!(native_panel_has_raw_window_handle(&handle));
    }
}
