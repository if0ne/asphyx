use std::sync::Arc;

use oxidx::dx;
use parking_lot::Mutex;
use tracing::info;

use crate::graphics::{
    commands::CommandBufferEnum,
    context::RenderContext,
    core::{
        backend::RenderDeviceInfo,
        commands::{CommandBufferType, CommandDevice, SyncPoint},
        handle::{RenderHandle, SparseArray},
        resource::{
            Buffer, BufferDesc, ResourceDevice, Sampler, SamplerDesc, Texture, TextureDesc,
            TextureViewDesc,
        },
        shader::{ComputePipeline, RenderPipeline},
    },
    HandleStorage,
};

use super::{
    inner::commands::DxCommandQueue,
    resources::{DxBuffer, DxTexture},
};

#[derive(Debug)]
pub struct DxRenderContext {
    pub(super) gpu: dx::Device,
    adapter: dx::Adapter3,

    pub(super) gfx_queue: DxCommandQueue,
    pub(super) compute_queue: DxCommandQueue,
    pub(super) transfer_queue: DxCommandQueue,

    pub(super) desc: RenderDeviceInfo,

    handles: Arc<HandleStorage>,
    pub(super) buffers: Mutex<SparseArray<Buffer, DxBuffer>>,
    pub(super) textures: Mutex<SparseArray<Texture, DxTexture>>,
}

impl DxRenderContext {
    pub(super) fn new(
        adapter: dx::Adapter3,
        desc: RenderDeviceInfo,
        handles: Arc<HandleStorage>,
    ) -> Self {
        info!(
            "Creating device with adapter {} and id {}",
            desc.name, desc.id
        );

        let device = dx::create_device(Some(&adapter), dx::FeatureLevel::Level11)
            .expect("failed to create device");

        if desc.is_cross_adapter_texture_supported {
            info!("Cross Adapter Row Major Texture is supported");
        } else {
            info!("Cross Adapter Row Major Texture is NOT supported");
        }

        let gfx_queue = DxCommandQueue::new(&device, CommandBufferType::Graphics, None);
        let compute_queue = DxCommandQueue::new(&device, CommandBufferType::Compute, None);
        let transfer_queue = DxCommandQueue::new(&device, CommandBufferType::Transfer, None);

        Self {
            gpu: device,
            adapter,
            gfx_queue,
            compute_queue,
            transfer_queue,
            desc,
            handles,
            buffers: Mutex::new(SparseArray::new(128)),
            textures: Mutex::new(SparseArray::new(128)),
        }
    }
}

impl RenderContext for DxRenderContext {
    fn bind_buffer(
        &self,
        handle: RenderHandle<Buffer>,
        desc: BufferDesc,
        init_data: Option<&[u8]>,
    ) {
        let buffer = self.create_buffer(desc, init_data);
        self.buffers.lock().set(handle, buffer);
    }

    fn unbind_buffer(&self, handle: RenderHandle<Buffer>) {
        self.buffers.lock().remove(handle);
    }

    fn bind_texture(
        &self,
        handle: RenderHandle<Texture>,
        desc: TextureDesc,
        init_data: Option<&[u8]>,
    ) {
        let texture = self.create_texture(desc, init_data);
        self.textures.lock().set(handle, texture);
    }

    fn unbind_texture(&self, handle: RenderHandle<Texture>) {
        self.textures.lock().remove(handle);
    }

    fn bind_texture_view(
        &self,
        handle: RenderHandle<Texture>,
        texture: RenderHandle<Texture>,
        desc: TextureViewDesc,
    ) {
        todo!()
    }

    fn open_texture_handle(&self, handle: RenderHandle<Texture>, other: &Self) {
        let texture = {
            let guard = other.textures.lock();
            let texture = guard.get(handle).expect("Wrong handle");
            self.open_texture(texture, other)
        };
        self.textures.lock().set(handle, texture);
    }

    fn bind_sampler(&self, handle: RenderHandle<Sampler>, desc: SamplerDesc) {
        todo!()
    }

    fn unbind_sampler(&self, handle: RenderHandle<Sampler>) {
        todo!()
    }

    fn bind_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>, desc: ()) {
        todo!()
    }

    fn unbind_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>) {
        todo!()
    }

    fn bind_render_pipeline(&self, handle: RenderHandle<RenderPipeline>, desc: ()) {
        todo!()
    }

    fn unbind_render_pipeline(&self, handle: RenderHandle<RenderPipeline>) {
        todo!()
    }

    fn create_command_buffer(self: &Arc<Self>, ty: CommandBufferType) -> CommandBufferEnum {
        CommandDevice::create_command_buffer(self, ty).into()
    }

    fn stash_cmd_buffer(&self, cmd_buffer: CommandBufferEnum) {
        if let CommandBufferEnum::DxCommandBuffer(cmd) = cmd_buffer {
            CommandDevice::stash_cmd_buffer(self, cmd);
        } else {
            todo!("log")
        }
    }

    fn push_cmd_buffer(&self, cmd_buffer: CommandBufferEnum) {
        if let CommandBufferEnum::DxCommandBuffer(cmd) = cmd_buffer {
            CommandDevice::push_cmd_buffer(self, cmd);
        } else {
            todo!("log")
        }
    }

    fn commit(&self, ty: CommandBufferType) -> SyncPoint {
        CommandDevice::commit(self, ty)
    }
}
