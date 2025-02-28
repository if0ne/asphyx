use crate::graphics::commands::CommandBuffer;

pub type SyncPoint = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandBufferType {
    Graphics,
    Compute,
    Transfer,
}

pub trait DynCommandDevice {
    fn create_command_buffer(&self, ty: CommandBufferType) -> CommandBuffer;
    fn stash_cmd_buffer(&self, cmd_buffer: CommandBuffer);
    fn push_cmd_buffer(&self, cmd_buffer: CommandBuffer);
    fn commit(&self, ty: CommandBufferType) -> SyncPoint;
}
