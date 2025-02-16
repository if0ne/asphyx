pub struct RenderBackend {}

impl RenderBackend {
    pub fn get_all_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDevice> + 'a {
        std::iter::empty()
    }
}

pub struct RenderDevice {}
