use std::hint::cold_path;
use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use oxidx::dx::{self, IGraphicsCommandList, IGraphicsCommandListExt};

use crate::graphics::commands::{
    ComputeEncoderEnum, DynCommandBuffer, DynComputeEncoder, DynRenderEncoder, DynTransferEncoder,
    RenderEncoderEnum, TransferEncoderEnum,
};

use crate::graphics::core::commands::{
    CommandBuffer, CommandBufferType, CommandDevice, ComputeEncoder, RenderEncoder, TransferEncoder,
};
use crate::graphics::core::handle::RenderHandle;
use crate::graphics::core::resource::{Buffer, Texture};
use crate::graphics::dx12::inner::commands::CommandAllocatorEntry;

use super::context::DxRenderContext;
use super::resources::TextureState;

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
    cmd_buffer: &'a mut DxCommandBuffer,
}

impl<'a> DynTransferEncoder<'a> for DxTransferEncoder<'a> {}

impl<'a> TransferEncoder for DxTransferEncoder<'a> {
    fn copy_buffer_to_buffer(&self, dst: RenderHandle<Buffer>, src: RenderHandle<Buffer>) {
        let Some(ctx) = self.cmd_buffer.ctx.upgrade() else {
            cold_path();
            return;
        };
        let guard = ctx.buffers.lock();

        let Some(dst) = guard.get(dst) else {
            cold_path();
            return;
        };

        let Some(src) = guard.get(src) else {
            cold_path();
            return;
        };

        self.cmd_buffer.list.copy_resource(&dst.raw, &src.raw);
    }

    fn copy_texture_to_texture(&self, dst: RenderHandle<Texture>, src: RenderHandle<Texture>) {
        let Some(ctx) = self.cmd_buffer.ctx.upgrade() else {
            cold_path();
            return;
        };
        let guard = ctx.textures.lock();

        let Some(dst) = guard.get(dst) else {
            cold_path();
            return;
        };

        let Some(src) = guard.get(src) else {
            cold_path();
            return;
        };

        match (&dst.state, &src.state) {
            (TextureState::Local { raw: dst_raw }, TextureState::Local { raw: src_raw }) => {
                self.cmd_buffer.list.copy_resource(dst_raw, src_raw)
            }
            _ => return,
        };
    }

    fn upload_to_texture(
        &self,
        dst: RenderHandle<Texture>,
        src: RenderHandle<Buffer>,
        data: &[u8],
    ) {
        let Some(ctx) = self.cmd_buffer.ctx.upgrade() else {
            cold_path();
            return;
        };
        let bguard = ctx.buffers.lock();
        let tguard = ctx.textures.lock();

        let Some(dst) = tguard.get(dst) else {
            cold_path();
            return;
        };

        let Some(src) = bguard.get(src) else {
            cold_path();
            return;
        };

        let dst_res = match &dst.state {
            TextureState::Local { raw } => raw,
            _ => return,
        };

        let copied = self.cmd_buffer.list.update_subresources_fixed::<1, _, _>(
            dst_res,
            &src.raw,
            0,
            0..1,
            &[dx::SubresourceData::new(data).with_row_pitch(4 * dst.desc.width as usize)],
        );

        debug_assert!(copied > 0);
    }
}
