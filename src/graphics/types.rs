use bytemuck::Pod;

pub type SyncPoint = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandQueueType {
    Graphics,
    Compute,
    Io,
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

pub struct RenderPipelineDesc {}

pub struct ComputePipelineDesc {}

#[derive(Clone, Debug)]
pub struct BufferDesc<'a, T: Pod = u8> {
    pub size: usize,
    pub stride: usize,
    pub usage: BufferUsage,
    pub content: Option<&'a [T]>,
}

pub struct ImageDesc {}

pub struct ImageViewDesc {}

pub struct BindGroupLayoutDesc {}

pub struct BindGroupDesc {}
