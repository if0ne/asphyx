use std::sync::Arc;

use parking_lot::Mutex;
use traits::{Api, Device, DynDevice};
use types::{CreateBufferInfo, CreateImageInfo};

use crate::allocators::{Handle, Pool, UntypedHandle};

pub mod mock;
pub mod traits;
pub mod types;

pub struct Renderer {
    buffers: Mutex<Pool<Buffer>>,
    images: Mutex<Pool<Image>>,

    mock_api: Option<Arc<mock::RenderBackend>>,
}

impl Renderer {
    pub fn new(backends: &[Backend]) -> Self {
        let mock_api = backends
            .iter()
            .find(|b| **b == Backend::Mock)
            .map(|_| Arc::new(mock::RenderBackend::new()));

        Self {
            buffers: Mutex::new(Pool::new(None)),
            images: Mutex::new(Pool::new(None)),

            mock_api,
        }
    }

    pub fn mock(&self) -> Option<Arc<mock::RenderBackend>> {
        self.mock_api.clone()
    }

    pub fn create_buffer(
        &self,
        desc: &CreateBufferInfo,
        devices: &[&dyn DynDevice],
    ) -> Handle<Buffer> {
        let handles = devices
            .iter()
            .map(|d| SharedEntry {
                handle: d.create_buffer(desc),
                backend: d.get_backend(),
                device_id: d.get_device_id(),
            })
            .collect::<Vec<_>>();

        self.buffers.lock().push(Buffer { buffers: handles })
    }

    pub fn get_buffer_handle<A: Api>(
        &self,
        handle: Handle<Buffer>,
        device: &impl Device<A>,
    ) -> Option<Handle<A::Buffer>> {
        self.buffers
            .lock()
            .get(handle)
            .map(|b| {
                b.buffers
                    .iter()
                    .find(|e| {
                        e.backend == device.get_backend() && e.device_id == device.get_device_id()
                    })
                    .map(|e| e.handle.into())
            })
            .flatten()
    }

    pub fn create_image(
        &self,
        desc: &CreateImageInfo,
        devices: &[&dyn DynDevice],
    ) -> Handle<Image> {
        let handles = devices
            .iter()
            .map(|d| SharedEntry {
                handle: d.create_image(desc),
                backend: d.get_backend(),
                device_id: d.get_device_id(),
            })
            .collect::<Vec<_>>();

        self.images.lock().push(Image { images: handles })
    }

    pub fn get_image_handle<A: Api>(
        &self,
        handle: Handle<Image>,
        device: &impl Device<A>,
    ) -> Option<Handle<A::Image>> {
        self.images
            .lock()
            .get(handle)
            .map(|b| {
                b.images
                    .iter()
                    .find(|e| {
                        e.backend == device.get_backend() && e.device_id == device.get_device_id()
                    })
                    .map(|e| e.handle.into())
            })
            .flatten()
    }
}

pub struct Image {
    images: Vec<SharedEntry>,
}

pub struct Buffer {
    buffers: Vec<SharedEntry>,
}

struct SharedEntry {
    backend: Backend,
    device_id: usize,
    handle: UntypedHandle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Dx12,
    Vulkan,
    Mock,
}
