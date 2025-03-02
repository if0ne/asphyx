use std::sync::Arc;

use oxidx::dx;

use crate::graphics::{
    commands::CommandBufferEnum,
    context::RenderContext,
    core::{
        commands::{CommandBufferType, CommandDevice, SyncPoint},
        handle::RenderHandle,
        resource::{
            Buffer, CreateBufferDesc, CreateSamplerDesc, CreateTextureDesc, CreateTextureViewDesc,
            Sampler, Texture,
        },
        shader::{ComputePipeline, RenderPipeline},
    },
};

use super::inner::commands::DxCommandQueue;

#[derive(Debug)]
pub struct DxRenderContext {
    pub(super) gpu: dx::Device,

    pub(super) gfx_queue: DxCommandQueue,
    pub(super) compute_queue: DxCommandQueue,
    pub(super) transfer_queue: DxCommandQueue,
}

impl RenderContext for DxRenderContext {
    fn bind_buffer(&self, handle: RenderHandle<Buffer>, desc: CreateBufferDesc) {
        todo!()
    }

    fn unbind_buffer(&self, handle: RenderHandle<Buffer>) {
        todo!()
    }

    fn open_buffer_handle(&self, handle: RenderHandle<Buffer>, other: &Self) {
        todo!()
    }

    fn bind_texture(&self, handle: RenderHandle<Texture>, desc: CreateTextureDesc) {
        todo!()
    }

    fn unbind_texture(&self, handle: RenderHandle<Texture>) {
        todo!()
    }

    fn bind_texture_view(
        &self,
        handle: RenderHandle<Texture>,
        texture: RenderHandle<Texture>,
        desc: CreateTextureViewDesc,
    ) {
        todo!()
    }

    fn open_texture_handle(&self, handle: RenderHandle<Texture>, other: &Self) {
        todo!()
    }

    fn bind_sampler(&self, handle: RenderHandle<Sampler>, desc: CreateSamplerDesc) {
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
