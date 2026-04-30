#[cfg(windows)]
use windows::Win32::Graphics::Direct2D::{
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1CreateFactory, ID2D1Factory,
};

#[cfg(windows)]
use std::sync::OnceLock;

#[derive(Clone, Debug, Default)]
pub(super) struct WindowsDirect2DFactory {
    #[cfg(windows)]
    factory: Option<ID2D1Factory>,
}

impl WindowsDirect2DFactory {
    pub(super) fn empty() -> Self {
        Self {
            #[cfg(windows)]
            factory: None,
        }
    }

    #[cfg(windows)]
    pub(super) fn create() -> Result<Self, String> {
        let factory =
            unsafe { D2D1CreateFactory::<ID2D1Factory>(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) }
                .map_err(|error| error.to_string())?;
        Ok(Self {
            factory: Some(factory),
        })
    }

    #[cfg(not(windows))]
    pub(super) fn create() -> Result<Self, String> {
        Ok(Self::empty())
    }

    #[cfg(windows)]
    pub(super) fn shared() -> Result<Self, String> {
        static FACTORY: OnceLock<Result<WindowsDirect2DFactory, String>> = OnceLock::new();
        FACTORY.get_or_init(Self::create).clone()
    }

    #[cfg(not(windows))]
    pub(super) fn shared() -> Result<Self, String> {
        Self::create()
    }

    pub(super) fn is_initialized(&self) -> bool {
        #[cfg(windows)]
        {
            self.factory.is_some()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    #[cfg(windows)]
    pub(super) fn factory(&self) -> Option<&ID2D1Factory> {
        self.factory.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::WindowsDirect2DFactory;

    #[test]
    fn direct2d_factory_wrapper_can_represent_uninitialized_factory() {
        let factory = WindowsDirect2DFactory::empty();

        assert!(!factory.is_initialized());
    }

    #[cfg(windows)]
    #[test]
    fn direct2d_factory_can_initialize_on_windows() {
        let factory = WindowsDirect2DFactory::create().expect("create Direct2D factory");

        assert!(factory.is_initialized());
    }
}
