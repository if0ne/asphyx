use super::{
    core::backend::{RenderDeviceId, RenderDeviceInfo},
    RenderContextEnum,
};

pub trait DynApi {
    fn enumerate_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDeviceInfo> + 'a;
    fn create_device(&self, index: RenderDeviceId) -> RenderContextEnum;
}
