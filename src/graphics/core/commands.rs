use std::sync::Arc;

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

    fn create_command_buffer(&self, ty: CommandBufferType) -> Self::CommandBuffer;
    fn stash_cmd_buffer(&self, cmd_buffer: Self::CommandBuffer);
    fn push_cmd_buffer(&self, cmd_buffer: Self::CommandBuffer);
    fn commit(&self, ty: CommandBufferType) -> SyncPoint;
    fn wait_cpu(&self, ty: CommandBufferType, time: SyncPoint);
}

pub trait RenderEncoder {}

pub trait ComputeEncoder {}

pub trait TransferEncoder {
    type Buffer;
    type Texture;

    fn copy_buffer_to_buffer(&self, dst: &Self::Buffer, src: &Self::Buffer);
    fn copy_texture_to_texture(&self, dst: &Self::Texture, src: &Self::Texture);
    fn upload_to_texture(&self, dst: &Self::Texture, src: &Self::Buffer, data: &[u8]);
}
