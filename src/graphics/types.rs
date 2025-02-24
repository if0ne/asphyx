use std::{borrow::Cow, ops::Range};

use bytemuck::Pod;
use oxidx::dx;

use crate::allocators::Handle;

use super::traits::Api;

pub type SyncPoint = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandQueueType {
    Graphics,
    Compute,
    Io,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageViewType {
    RenderTarget,
    DepthTarget,
    ShaderResource,
    Storage,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct BufferUsage: u32 {
        const Storage = 0x1;
        const Uniform = 0x2;
        const Vertex = 0x4;
        const Index = 0x8;
        const Copy = 0x10;
        const Readback = 0x20;
        const Shared = 0x40;
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ImageUsage: u32 {
        const RenderTarget = 0x1;
        const DepthStencil = 0x2;
        const Storage = 0x4;
        const ShaderVisible = 0x8;
        const Shared = 0x10;
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ImageState: u32 {
        const Unknown = 0x0;
        const Generic = 0x1;
        const CopyDst = 0x2;
        const CopySrc = 0x4;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferState {
    Unknown,
    Generic,
    CopyDst,
    CopySrc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryType {
    Upload,
    Readback,
    Device,
    Shared,
}

pub struct RenderPipelineDesc {}

pub struct ComputePipelineDesc {}

#[derive(Clone, Debug)]
pub struct CreateBufferInfo<'a, T: Pod = u8> {
    pub name: Option<Cow<'static, str>>,
    pub size: usize,
    pub stride: usize,
    pub usage: BufferUsage,
    pub mem_ty: Option<MemoryType>,
    pub content: Option<&'a [T]>,
}

#[derive(Clone, Debug)]
pub struct BufferDesc {
    pub size: usize,
    pub stride: usize,
    pub usage: BufferUsage,
    pub mem_ty: MemoryType,
}

impl<'a, T: Pod> Default for CreateBufferInfo<'a, T> {
    fn default() -> Self {
        Self {
            name: None,
            size: 0,
            stride: 0,
            usage: BufferUsage::empty(),
            mem_ty: None,
            content: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreateImageInfo<'a> {
    pub width: u32,
    pub height: u32,
    pub array: u32,
    pub levels: u32,
    pub format: dx::Format,
    pub usage: ImageUsage,
    pub state: ImageState,
    pub mem_ty: Option<MemoryType>,
    pub content: Option<&'a [u8]>,
}

#[derive(Clone, Debug)]
pub struct ImageDesc {
    pub width: u32,
    pub height: u32,
    pub array: u32,
    pub levels: u32,
    pub format: dx::Format,
    pub usage: ImageUsage,
    pub mem_ty: MemoryType,
}

impl<'a> Default for CreateImageInfo<'a> {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            array: 1,
            levels: 1,
            format: dx::Format::Unknown,
            usage: ImageUsage::empty(),
            state: ImageState::Unknown,
            mem_ty: None,
            content: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreateImageViewInfo {
    pub mip_slice: u16,
    pub plane_slice: u16,
    pub array: Range<u32>,
}

#[derive(Clone, Debug)]
pub struct ImageViewDesc {
    pub mip_slice: u16,
    pub plane_slice: u16,
    pub array: Range<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingType {
    Uniform,
    Texture,
    Sampler,
    Storage,
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub slot: u32,
    pub count: u32,
    pub ty: BindingType,
}

pub struct BindGroupLayoutDesc<'a> {
    pub bindings: &'a [Binding],
}

pub struct PipelineLayoutDesc<'a, A: Api> {
    pub groups: &'a [Handle<A::BindGroupLayout>],
}

#[derive(Debug)]
pub struct BufferDescriptor<A: Api> {
    pub buffer: Handle<A::Buffer>,
    pub slot: u32,
}

#[derive(Debug)]
pub struct ImageDescriptor<A: Api> {
    pub image: Handle<A::Image>,
    pub slot: u32,
}

pub struct BindGroupDesc<'a, A: Api> {
    pub space: u32,
    pub buffers: &'a [BufferDescriptor<A>],
    pub images: &'a [ImageDescriptor<A>],
}
