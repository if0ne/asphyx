use std::sync::Arc;

use super::core::{
    handle::RenderHandle,
    resource::{Buffer, BufferDesc, Sampler, SamplerDesc, Texture, TextureDesc, TextureViewDesc},
    shader::{ComputePipeline, RenderPipeline},
};

pub trait RenderContext {
    // Resources
    fn bind_buffer(
        self: &Arc<Self>,
        handle: RenderHandle<Buffer>,
        desc: BufferDesc,
        init_data: Option<&[u8]>,
    );
    fn unbind_buffer(&self, handle: RenderHandle<Buffer>);

    fn bind_texture(
        self: &Arc<Self>,
        handle: RenderHandle<Texture>,
        desc: TextureDesc,
        init_data: Option<&[u8]>,
    );
    fn unbind_texture(&self, handle: RenderHandle<Texture>);

    fn bind_texture_view(
        &self,
        handle: RenderHandle<Texture>,
        texture: RenderHandle<Texture>,
        desc: TextureViewDesc,
    );

    fn open_texture_handle(&self, handle: RenderHandle<Texture>, other: &Self);

    fn bind_sampler(&self, handle: RenderHandle<Sampler>, desc: SamplerDesc);
    fn unbind_sampler(&self, handle: RenderHandle<Sampler>);

    // Shader
    fn bind_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>, desc: ());
    fn unbind_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>);

    fn bind_render_pipeline(&self, handle: RenderHandle<RenderPipeline>, desc: ());
    fn unbind_render_pipeline(&self, handle: RenderHandle<RenderPipeline>);
}
