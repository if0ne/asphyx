use std::sync::Arc;

use super::{commands::CommandDevice, resource::ResourceDevice};

pub type RenderDeviceId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceType {
    Discrete,
    Integrated,
    Cpu,
}

#[derive(Clone, Debug)]
pub struct RenderDeviceInfo {
    pub name: String,
    pub is_cross_adapter_texture_supported: bool,
    pub is_uma: bool,
    pub ty: DeviceType,
}

pub trait Api: Sized {
    type Device: CommandDevice + ResourceDevice;

    fn enumerate_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDeviceInfo> + 'a;
    fn create_device(&self, index: RenderDeviceId) -> Self::Device;
}

#[derive(Clone, Debug)]
pub struct RenderDeviceGroup<D: CommandDevice + ResourceDevice> {
    pub primary: Arc<D>,
    pub secondaries: Vec<Arc<D>>,
}

impl<D: CommandDevice + ResourceDevice> RenderDeviceGroup<D> {
    pub fn new(primary: Arc<D>, secondaries: Vec<Arc<D>>) -> Self {
        Self {
            primary,
            secondaries,
        }
    }

    pub fn call(&self, func: impl Fn(&Arc<D>)) {
        func(&self.primary);

        for device in self.secondaries.iter() {
            func(device);
        }
    }
}
