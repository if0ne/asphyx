use std::sync::Arc;

use super::{
    handle::RenderHandle,
    resource::{Buffer, Texture},
};

pub type SyncPoint = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandBufferType {
    Graphics,
    Compute,
    Transfer,
}

pub trait CommandBuffer {
    type RenderEncoder<'a>: RenderEncoder
    where
        Self: 'a;

    type ComputeEncoder<'a>: ComputeEncoder
    where
        Self: 'a;

    type TransferEncoder<'a>: TransferEncoder
    where
        Self: 'a;

    fn render_encoder(&mut self) -> Self::RenderEncoder<'_>;
    fn compute_encoder(&mut self) -> Self::ComputeEncoder<'_>;
    fn transfer_encoder(&mut self) -> Self::TransferEncoder<'_>;
}

pub trait CommandDevice {
    type CommandBuffer: CommandBuffer;

    fn create_command_buffer(self: &Arc<Self>, ty: CommandBufferType) -> Self::CommandBuffer;
    fn stash_cmd_buffer(&self, cmd_buffer: Self::CommandBuffer);
    fn push_cmd_buffer(&self, cmd_buffer: Self::CommandBuffer);
    fn commit(&self, ty: CommandBufferType) -> SyncPoint;
}

pub trait RenderEncoder {}

pub trait ComputeEncoder {}

pub trait TransferEncoder {
    fn copy_buffer_to_buffer(&self, dst: RenderHandle<Buffer>, src: RenderHandle<Buffer>);
    fn copy_texture_to_texture(&self, dst: RenderHandle<Texture>, src: RenderHandle<Texture>);
    fn upload_to_texture(&self, dst: RenderHandle<Texture>, src: RenderHandle<Buffer>, data: &[u8]);
}
