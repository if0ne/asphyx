use super::{commands::CommandDevice, resource::ResourceDevice};

pub type RenderDeviceId = usize;

#[derive(Clone, Debug)]
pub struct RenderDeviceInfo {}

pub trait Api: Sized {
    type Device: CommandDevice + ResourceDevice;

    fn enumerate_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDeviceInfo> + 'a;
    fn create_device(&self, index: RenderDeviceId) -> Self::Device;
}
