use crate::graphics::core::resource::ResourceDevice;

use super::context::DxRenderContext;

impl ResourceDevice for DxRenderContext {
    type Buffer = ();
    type Texture = ();
    type Sampler = ();

    fn create_buffer(
        &self,
        desc: &crate::graphics::core::resource::CreateBufferDesc,
    ) -> Self::Buffer {
        todo!()
    }

    fn destroy_buffer(&self, buffer: Self::Buffer) {
        todo!()
    }

    fn open_buffer(&self, buffer: &Self::Buffer, other: &Self) -> Self::Buffer {
        todo!()
    }

    fn create_texture(
        &self,
        desc: &crate::graphics::core::resource::CreateTextureDesc,
    ) -> Self::Texture {
        todo!()
    }

    fn destroy_texture(&self, buffer: Self::Texture) {
        todo!()
    }

    fn create_texture_view(
        &self,
        texture: &Self::Texture,
        desc: &crate::graphics::core::resource::CreateTextureViewDesc,
    ) -> Self::Texture {
        todo!()
    }

    fn open_texture(&self, texture: &Self::Texture, other: &Self) -> Self::Texture {
        todo!()
    }

    fn create_sampler(
        &self,
        desc: &crate::graphics::core::resource::CreateSamplerDesc,
    ) -> Self::Sampler {
        todo!()
    }

    fn destroy_sampler(&self, buffer: Self::Sampler) {
        todo!()
    }
}
