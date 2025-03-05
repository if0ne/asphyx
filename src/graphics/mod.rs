use core::{
    handle::{RenderHandle, RenderHandleAllocator},
    resource::{Buffer, Texture},
};
use std::sync::Arc;

use dx12::backend::DxBackend;
use parking_lot::Mutex;

pub mod context;
pub mod core;

#[cfg(target_os = "windows")]
mod dx12;

mod mock;

#[derive(Debug)]
pub struct RenderSystem {
    handles: HandleStorage,

    #[cfg(target_os = "windows")]
    dx_api: Option<Arc<DxBackend>>,
}

impl RenderSystem {
    pub fn new(backend_settings: &[RenderBackendSettings]) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                let dx_api = backend_settings
                    .iter()
                    .find(|b| b.api == RenderBackend::Dx12)
                    .and_then(|settings| Some(Arc::new(DxBackend::new(settings.debug))));

                Self {
                    handles: HandleStorage::new(),
                    dx_api,
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn dx_api(&self) -> Option<Arc<DxBackend>> {
        self.dx_api.clone()
    }

    #[inline]
    pub fn create_buffer_handle(&self) -> RenderHandle<Buffer> {
        self.handles.create_buffer_handle()
    }

    #[inline]
    pub fn free_buffer_handle(&self, handle: RenderHandle<Buffer>) {
        self.handles.free_buffer_handle(handle)
    }

    #[inline]
    pub fn create_texture_handle(&self) -> RenderHandle<Texture> {
        self.handles.create_texture_handle()
    }

    #[inline]
    pub fn free_texture_handle(&self, handle: RenderHandle<Texture>) {
        self.handles.free_texture_handle(handle)
    }
}

#[derive(Debug)]
pub struct HandleStorage {
    buffers: Mutex<RenderHandleAllocator<Buffer>>,
    textures: Mutex<RenderHandleAllocator<Texture>>,
}

impl HandleStorage {
    fn new() -> Self {
        Self {
            buffers: Mutex::new(RenderHandleAllocator::new()),
            textures: Mutex::new(RenderHandleAllocator::new()),
        }
    }

    #[inline]
    pub fn create_buffer_handle(&self) -> RenderHandle<Buffer> {
        self.buffers.lock().allocate()
    }

    #[inline]
    pub fn free_buffer_handle(&self, handle: RenderHandle<Buffer>) {
        self.buffers.lock().free(handle);
    }

    #[inline]
    pub fn create_texture_handle(&self) -> RenderHandle<Texture> {
        self.textures.lock().allocate()
    }

    #[inline]
    pub fn free_texture_handle(&self, handle: RenderHandle<Texture>) {
        self.textures.lock().free(handle);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderBackendSettings {
    pub api: RenderBackend,
    pub debug: DebugFlags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBackend {
    #[cfg(target_os = "windows")]
    Dx12,
    Mock,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DebugFlags: u32 {
        const CpuValidation = 0x1;
        const GpuValidation = 0x2;
        const RenderDoc = 0x4;
        const Pix = 0x8;
    }
}
