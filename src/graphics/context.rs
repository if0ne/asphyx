use std::sync::Arc;

use super::{
    commands::CommandBufferEnum,
    core::{
        commands::{CommandBufferType, SyncPoint},
        handle::RenderHandle,
        resource::{
            Buffer, CreateBufferDesc, CreateSamplerDesc, CreateTextureDesc, CreateTextureViewDesc,
            Sampler, Texture,
        },
        shader::{ComputePipeline, RenderPipeline},
    },
    RenderContextEnum,
};

pub trait RenderContext {
    // Resources
    fn bind_buffer(&self, handle: RenderHandle<Buffer>, desc: CreateBufferDesc);
    fn unbind_buffer(&self, handle: RenderHandle<Buffer>);

    fn open_buffer_handle(&self, handle: RenderHandle<Buffer>, other: &Self);

    fn bind_texture(&self, handle: RenderHandle<Texture>, desc: CreateTextureDesc);
    fn unbind_texture(&self, handle: RenderHandle<Texture>);

    fn bind_texture_view(
        &self,
        handle: RenderHandle<Texture>,
        texture: RenderHandle<Texture>,
        desc: CreateTextureViewDesc,
    );

    fn open_texture_handle(&self, handle: RenderHandle<Texture>, other: &Self);

    fn bind_sampler(&self, handle: RenderHandle<Sampler>, desc: CreateSamplerDesc);
    fn unbind_sampler(&self, handle: RenderHandle<Sampler>);

    // Shader
    fn bind_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>, desc: ());
    fn unbind_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>);

    fn bind_render_pipeline(&self, handle: RenderHandle<RenderPipeline>, desc: ());
    fn unbind_render_pipeline(&self, handle: RenderHandle<RenderPipeline>);

    // Commands
    fn create_command_buffer(self: &Arc<Self>, ty: CommandBufferType) -> CommandBufferEnum;
    fn stash_cmd_buffer(&self, cmd_buffer: CommandBufferEnum);
    fn push_cmd_buffer(&self, cmd_buffer: CommandBufferEnum);
    fn commit(&self, ty: CommandBufferType) -> SyncPoint;
}

#[derive(Clone, Debug)]
pub struct DynRenderDeviceGroup {
    pub primary: RenderContextEnum,
    pub secondaries: Vec<RenderContextEnum>,
}

impl DynRenderDeviceGroup {
    pub fn new(primary: RenderContextEnum, secondaries: Vec<RenderContextEnum>) -> Self {
        Self {
            primary,
            secondaries,
        }
    }

    pub fn call(&self, func: impl Fn(&RenderContextEnum)) {
        func(&self.primary);

        for device in self.secondaries.iter() {
            func(device);
        }
    }
}
