use crate::graphics::core::resource::{
    CreateBufferDesc, CreateSamplerDesc, CreateTextureDesc, CreateTextureViewDesc, ResourceDevice,
};

use super::context::DxRenderContext;

impl ResourceDevice for DxRenderContext {
    type Buffer = DxBuffer;
    type Texture = ();
    type Sampler = ();

    fn create_buffer(&self, desc: &CreateBufferDesc) -> Self::Buffer {
        DxBuffer {}
    }

    fn destroy_buffer(&self, buffer: Self::Buffer) {
        todo!()
    }

    fn open_buffer(&self, buffer: &Self::Buffer, other: &Self) -> Self::Buffer {
        todo!()
    }

    fn create_texture(&self, desc: &CreateTextureDesc) -> Self::Texture {
        todo!()
    }

    fn destroy_texture(&self, buffer: Self::Texture) {
        todo!()
    }

    fn create_texture_view(
        &self,
        texture: &Self::Texture,
        desc: &CreateTextureViewDesc,
    ) -> Self::Texture {
        todo!()
    }

    fn open_texture(&self, texture: &Self::Texture, other: &Self) -> Self::Texture {
        todo!()
    }

    fn create_sampler(&self, desc: &CreateSamplerDesc) -> Self::Sampler {
        todo!()
    }

    fn destroy_sampler(&self, buffer: Self::Sampler) {
        todo!()
    }
}

#[derive(Debug)]
pub struct DxBuffer {}
