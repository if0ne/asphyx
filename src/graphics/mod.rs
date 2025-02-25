use std::sync::Arc;

use parking_lot::Mutex;
use traits::{Api, Device, DynApi, DynDevice};
use types::{CreateBufferInfo, CreateImageInfo, RenderBackend, RenderBackendSettings};

use crate::allocators::{Handle, Pool, UntypedHandle};

pub mod dx12;
pub mod mock;
pub mod traits;
pub mod types;

pub type RenderHandle = UntypedHandle;

pub struct RenderSystem {
    buffers: Mutex<Pool<()>>,
    images: Mutex<Pool<()>>,

    backends: Mutex<Vec<Arc<dyn DynApi>>>,
}

impl RenderSystem {
    pub fn new(backends: &[RenderBackendSettings]) -> Self {
        let backends = backends
            .iter()
            .map(|b| {
                let b: Arc<dyn DynApi> = match b.api {
                    RenderBackend::Dx12 => todo!(), //Arc::new(DxBackend::new(b.debug)),
                    RenderBackend::Vulkan => todo!(),
                    RenderBackend::Mock => todo!(),
                };

                b
            })
            .collect();

        Self {
            buffers: Mutex::new(Pool::new(None)),
            images: Mutex::new(Pool::new(None)),
            backends: Mutex::new(backends),
        }
    }

    pub fn create_buffer(
        &self,
        desc: &CreateBufferInfo,
        devices: &[&dyn DynDevice],
    ) -> RenderHandle {
        todo!()
    }

    pub fn get_buffer_handle<A: Api>(
        &self,
        handle: Handle<Buffer>,
        device: &impl Device<A>,
    ) -> Option<Handle<A::Buffer>> {
        todo!()
    }

    pub fn create_image(
        &self,
        desc: &CreateImageInfo,
        devices: &[&dyn DynDevice],
    ) -> Handle<Image> {
        todo!()
    }

    pub fn get_image_handle<A: Api>(
        &self,
        handle: Handle<Image>,
        device: &impl Device<A>,
    ) -> Option<Handle<A::Image>> {
        todo!()
    }
}

pub struct Image {
    images: Vec<SharedEntry>,
}

pub struct Buffer {
    buffers: Vec<SharedEntry>,
}

struct SharedEntry {
    backend: RenderBackend,
    device_id: usize,
    handle: UntypedHandle,
}
