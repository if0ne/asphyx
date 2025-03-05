use std::{borrow::Cow, sync::Arc};

use bytemuck::Pod;

use super::types::Format;

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

    fn create_buffer<T: Pod>(&self, desc: BufferDesc, init_data: Option<&[T]>) -> Self::Buffer;
    fn destroy_buffer(&self, buffer: Self::Buffer);

    fn create_texture<T: Pod>(&self, desc: TextureDesc, init_data: Option<&[T]>) -> Self::Texture;
    fn destroy_texture(&self, buffer: Self::Texture);

    fn create_texture_view(&self, texture: &Self::Texture, desc: TextureViewDesc) -> Self::Texture;

    fn open_texture(&self, texture: &Self::Texture, other: &Self) -> Self::Texture;

    fn create_sampler(&self, desc: SamplerDesc) -> Self::Sampler;
    fn destroy_sampler(&self, buffer: Self::Sampler);
}

#[derive(Clone, Debug)]
pub struct BufferDesc {
    pub name: Option<Cow<'static, str>>,
    pub size: usize,
    pub stride: usize,
    pub usage: BufferUsages,
}

#[derive(Clone, Debug)]
pub struct TextureDesc {
    pub name: Option<Cow<'static, str>>,
    pub ty: TextureType,
    pub width: u32,
    pub height: u32,
    pub depth: u16,
    pub mip_levels: u32,
    pub format: Format,
    pub usage: TextureUsages,
}

#[derive(Clone, Debug)]
pub struct TextureViewDesc {}

#[derive(Clone, Debug)]
pub struct SamplerDesc {}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct BufferUsages: u32 {
        const Copy = 1 << 0;
        const Uniform = 1 << 1;
        const Vertex = 1 << 2;
        const Index = 1 << 3;
        const Storage = 1 << 4;
        const QueryResolve = 1 << 5;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureType {
    D1,
    D2,
    D3,
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct TextureUsages: u32 {
        const Copy = 1 << 0;
        const Resource = 1 << 1;
        const RenderTarget = 1 << 2;
        const DepthTarget = 1 << 3;
        const Storage = 1 << 4;
        const Shared = 1 << 5;
    }
}
