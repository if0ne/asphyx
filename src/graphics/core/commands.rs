pub type SyncPoint = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandBufferType {
    Graphics,
    Compute,
    Transfer,
}

pub trait CommandBuffer {
    type RenderEncoder: RenderEncoder;
    type ComputeEncoder: ComputeEncoder;
    type TransferEncoder: TransferEncoder;

    fn render_encoder(&mut self) -> Self::RenderEncoder;
    fn compute_encoder(&mut self) -> Self::ComputeEncoder;
    fn transfer_encoder(&mut self) -> Self::TransferEncoder;
}

pub trait CommandDevice {
    type CommandBuffer: CommandBuffer;

    fn create_command_buffer(&self, ty: CommandBufferType) -> Self::CommandBuffer;
    fn stash_cmd_buffer(&self, cmd_buffer: Self::CommandBuffer);
    fn push_cmd_buffer(&self, cmd_buffer: Self::CommandBuffer);
    fn commit(&self, ty: CommandBufferType) -> SyncPoint;
}

pub trait RenderEncoder {}

pub trait ComputeEncoder {}

pub trait TransferEncoder {}
