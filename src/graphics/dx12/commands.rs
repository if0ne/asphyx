use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use oxidx::dx;

use crate::graphics::commands::{
    ComputeEncoderEnum, DynCommandBuffer, DynComputeEncoder, DynRenderEncoder, DynTransferEncoder,
    RenderEncoderEnum, TransferEncoderEnum,
};

use crate::graphics::core::commands::{
    CommandBuffer, CommandBufferType, CommandDevice, ComputeEncoder, RenderEncoder, TransferEncoder,
};
use crate::graphics::dx12::inner::commands::CommandAllocatorEntry;

use super::context::DxRenderContext;

#[derive(Debug)]
pub struct DxCommandBuffer {
    pub(super) ctx: Weak<DxRenderContext>,
    pub(super) ty: CommandBufferType,
    pub(super) list: dx::GraphicsCommandList,
    pub(super) allocator: CommandAllocatorEntry,
}

impl CommandDevice for DxRenderContext {
    type CommandBuffer = DxCommandBuffer;

    fn create_command_buffer(self: &Arc<Self>, ty: CommandBufferType) -> Self::CommandBuffer {
        match ty {
            CommandBufferType::Graphics => self.gfx_queue.create_command_buffer(self),
            CommandBufferType::Compute => self.compute_queue.create_command_buffer(self),
            CommandBufferType::Transfer => self.transfer_queue.create_command_buffer(self),
        }
    }

    fn stash_cmd_buffer(&self, cmd: Self::CommandBuffer) {
        match cmd.ty {
            CommandBufferType::Graphics => self.gfx_queue.stash_cmd_buffer(cmd),
            CommandBufferType::Compute => self.compute_queue.stash_cmd_buffer(cmd),
            CommandBufferType::Transfer => self.transfer_queue.stash_cmd_buffer(cmd),
        }
    }

    fn push_cmd_buffer(&self, cmd: Self::CommandBuffer) {
        match cmd.ty {
            CommandBufferType::Graphics => self.gfx_queue.push_cmd_buffer(cmd),
            CommandBufferType::Compute => self.compute_queue.push_cmd_buffer(cmd),
            CommandBufferType::Transfer => self.transfer_queue.push_cmd_buffer(cmd),
        }
    }

    fn commit(&self, ty: CommandBufferType) -> crate::graphics::core::commands::SyncPoint {
        match ty {
            CommandBufferType::Graphics => self.gfx_queue.commit(),
            CommandBufferType::Compute => self.compute_queue.commit(),
            CommandBufferType::Transfer => self.transfer_queue.commit(),
        }
    }
}

impl CommandBuffer for DxCommandBuffer {
    type RenderEncoder<'a> = DxRenderEncoder<'a>;
    type ComputeEncoder<'a> = DxComputeEncoder<'a>;
    type TransferEncoder<'a> = DxTransferEncoder<'a>;

    fn render_encoder(&mut self) -> Self::RenderEncoder<'_> {
        todo!()
    }

    fn compute_encoder(&mut self) -> Self::ComputeEncoder<'_> {
        todo!()
    }

    fn transfer_encoder(&mut self) -> Self::TransferEncoder<'_> {
        todo!()
    }
}

impl DynCommandBuffer for DxCommandBuffer {
    fn render_encoder(&mut self) -> RenderEncoderEnum<'_> {
        todo!()
    }

    fn compute_encoder(&mut self) -> ComputeEncoderEnum<'_> {
        todo!()
    }

    fn transfer_encoder(&mut self) -> TransferEncoderEnum<'_> {
        todo!()
    }
}

pub struct DxRenderEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> RenderEncoder for DxRenderEncoder<'a> {}

impl<'a> DynRenderEncoder<'a> for DxRenderEncoder<'a> {}

pub struct DxComputeEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> ComputeEncoder for DxComputeEncoder<'a> {}

impl<'a> DynComputeEncoder<'a> for DxComputeEncoder<'a> {}

pub struct DxTransferEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> DynTransferEncoder<'a> for DxTransferEncoder<'a> {}

impl<'a> TransferEncoder for DxTransferEncoder<'a> {}
