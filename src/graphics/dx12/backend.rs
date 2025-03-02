use std::sync::Arc;

use crate::graphics::{
    backend::DynApi,
    core::backend::{Api, RenderDeviceId, RenderDeviceInfo},
    RenderContextEnum,
};

use super::context::DxRenderContext;

pub struct DxBackend {}

impl Api for DxBackend {
    type Device = DxRenderContext;

    fn enumerate_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDeviceInfo> + 'a {
        std::iter::empty()
    }

    fn create_device(&self, index: RenderDeviceId) -> Self::Device {
        todo!()
    }
}

impl DynApi for DxBackend {
    fn enumerate_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDeviceInfo> + 'a {
        Api::enumerate_devices(self)
    }

    fn create_device(&self, index: RenderDeviceId) -> Arc<RenderContextEnum> {
        Arc::new(RenderContextEnum::DxRenderContext(Api::create_device(
            self, index,
        )))
    }
}
