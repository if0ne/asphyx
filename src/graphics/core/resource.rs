#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buffer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Texture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampler;

pub trait ResourceDevice {
    type Buffer;
    type Texture;
    type Sampler;

    fn create_buffer(&self, desc: &CreateBufferDesc) -> Self::Buffer;
    fn destroy_buffer(&self, buffer: Self::Buffer);

    fn open_buffer(&self, buffer: &Self::Buffer, other: &Self) -> Self::Buffer;

    fn create_texture(&self, desc: &CreateTextureDesc) -> Self::Texture;
    fn destroy_texture(&self, buffer: Self::Texture);

    fn create_texture_view(
        &self,
        texture: &Self::Texture,
        desc: &CreateTextureViewDesc,
    ) -> Self::Texture;

    fn open_texture(&self, texture: &Self::Texture, other: &Self) -> Self::Texture;

    fn create_sampler(&self, desc: &CreateSamplerDesc) -> Self::Sampler;
    fn destroy_sampler(&self, buffer: Self::Sampler);
}

#[derive(Clone, Debug)]
pub struct CreateBufferDesc {}

#[derive(Clone, Debug)]
pub struct CreateTextureDesc {}

#[derive(Clone, Debug)]
pub struct CreateTextureViewDesc {}

#[derive(Clone, Debug)]
pub struct CreateSamplerDesc {}
