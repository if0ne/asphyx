use std::hint::cold_path;
use std::marker::PhantomData;
use std::sync::Arc;

use oxidx::dx::{self, IGraphicsCommandList, IGraphicsCommandListExt};

use crate::graphics::core::commands::{
    CommandBuffer, CommandBufferType, CommandDevice, ComputeEncoder, DynTransferEncoder,
    RenderEncoder, SyncPoint, TransferEncoder,
};
use crate::graphics::core::handle::RenderHandle;
use crate::graphics::core::resource::{Buffer, Texture};
use crate::graphics::dx12::inner::commands::CommandAllocatorEntry;

use super::context::{DxRenderContext, HandleStorage};
use super::resources::{DxBuffer, DxTexture, TextureState};

#[derive(Debug)]
pub struct DxCommandBuffer {
    pub(super) handles: Arc<HandleStorage>,
    pub(super) ty: CommandBufferType,
    pub(super) list: dx::GraphicsCommandList,
    pub(super) allocator: CommandAllocatorEntry,
}

impl CommandDevice for DxRenderContext {
    type CommandBuffer = DxCommandBuffer;

    fn create_command_buffer(&self, ty: CommandBufferType) -> Self::CommandBuffer {
        match ty {
            CommandBufferType::Graphics => self
                .gfx_queue
                .create_command_buffer(Arc::clone(&self.handles)),
            CommandBufferType::Compute => self
                .compute_queue
                .create_command_buffer(Arc::clone(&self.handles)),
            CommandBufferType::Transfer => self
                .transfer_queue
                .create_command_buffer(Arc::clone(&self.handles)),
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

    fn commit(&self, ty: CommandBufferType) -> SyncPoint {
        match ty {
            CommandBufferType::Graphics => self.gfx_queue.commit(),
            CommandBufferType::Compute => self.compute_queue.commit(),
            CommandBufferType::Transfer => self.transfer_queue.commit(),
        }
    }

    fn wait_cpu(&self, ty: CommandBufferType, time: SyncPoint) {
        match ty {
            CommandBufferType::Graphics => {
                self.gfx_queue.wait_cpu(time);
            }
            CommandBufferType::Compute => {
                self.compute_queue.wait_cpu(time);
            }
            CommandBufferType::Transfer => {
                self.transfer_queue.wait_cpu(time);
            }
        }
    }
}

impl CommandBuffer for DxCommandBuffer {
    type BufferBarrier<'a> = (&'a DxBuffer, dx::ResourceStates);
    type TextureBarrier<'a> = (&'a DxTexture, dx::ResourceStates, bool);

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
        DxTransferEncoder { cmd_buffer: self }
    }

    fn set_buffer_barriers(&self, barriers: &[Self::BufferBarrier<'_>]) {
        let barriers = barriers
            .iter()
            .filter_map(|(b, s)| {
                let mut old_state = b.state.lock();

                if *old_state == *s {
                    return None;
                }

                let barrier = Some(dx::ResourceBarrier::transition(
                    &b.raw, *old_state, *s, None,
                ));
                *old_state = *s;

                barrier
            })
            .collect::<Vec<_>>();

        if !barriers.is_empty() {
            self.list.resource_barrier(&barriers);
        }
    }

    fn set_texture_barriers(&self, barriers: &[Self::TextureBarrier<'_>]) {
        let barriers = barriers
            .iter()
            .filter_map(|(t, s, b)| {
                let (raw, mut old_state) = match &t.state {
                    TextureState::Local { raw, state } => (raw, state.lock()),
                    TextureState::CrossAdapter { cross, state, .. } => (cross, state.lock()),
                    TextureState::Binded {
                        local,
                        cross,
                        local_state,
                        cross_state,
                        ..
                    } => {
                        if *b {
                            (cross, cross_state.lock())
                        } else {
                            (local, local_state.lock())
                        }
                    }
                };

                if *old_state == *s {
                    return None;
                }

                let barrier = Some(dx::ResourceBarrier::transition(raw, *old_state, *s, None));
                *old_state = *s;

                barrier
            })
            .collect::<Vec<_>>();

        if !barriers.is_empty() {
            self.list.resource_barrier(&barriers);
        }
    }
}

pub struct DxRenderEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> RenderEncoder for DxRenderEncoder<'a> {}

pub struct DxComputeEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> ComputeEncoder for DxComputeEncoder<'a> {}

pub struct DxTransferEncoder<'a> {
    cmd_buffer: &'a mut DxCommandBuffer,
}

impl<'a> TransferEncoder for DxTransferEncoder<'a> {
    type Buffer = DxBuffer;
    type Texture = DxTexture;

    fn copy_buffer_to_buffer(&self, dst: &Self::Buffer, src: &Self::Buffer) {
        self.cmd_buffer.list.copy_resource(&dst.raw, &src.raw);
    }

    fn copy_texture_to_texture(&self, dst: &Self::Texture, src: &Self::Texture) {
        match (&dst.state, &src.state) {
            (
                TextureState::Local { raw: dst_raw, .. },
                TextureState::Local { raw: src_raw, .. },
            ) => self.cmd_buffer.list.copy_resource(dst_raw, src_raw),
            _ => return,
        };
    }

    fn upload_to_texture(&self, dst: &Self::Texture, src: &Self::Buffer, data: &[u8]) {
        let dst_res = match &dst.state {
            TextureState::Local { raw, .. } => raw,
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

impl<'a> DynTransferEncoder for DxTransferEncoder<'a> {
    fn copy_buffer_to_buffer(&self, dst: RenderHandle<Buffer>, src: RenderHandle<Buffer>) {
        let guard = self.cmd_buffer.handles.buffers.lock();

        let Some(dst) = guard.get(dst) else {
            cold_path();
            return;
        };

        let Some(src) = guard.get(src) else {
            cold_path();
            return;
        };

        <Self as TransferEncoder>::copy_buffer_to_buffer(&self, dst, src);
    }

    fn copy_texture_to_texture(&self, dst: RenderHandle<Texture>, src: RenderHandle<Texture>) {
        let guard = self.cmd_buffer.handles.textures.lock();

        let Some(dst) = guard.get(dst) else {
            cold_path();
            return;
        };

        let Some(src) = guard.get(src) else {
            cold_path();
            return;
        };

        <Self as TransferEncoder>::copy_texture_to_texture(&self, dst, src);
    }

    fn upload_to_texture(
        &self,
        dst: RenderHandle<Texture>,
        src: RenderHandle<Buffer>,
        data: &[u8],
    ) {
        let bguard = self.cmd_buffer.handles.buffers.lock();
        let tguard = self.cmd_buffer.handles.textures.lock();

        let Some(dst) = tguard.get(dst) else {
            cold_path();
            return;
        };

        let Some(src) = bguard.get(src) else {
            cold_path();
            return;
        };

        <Self as TransferEncoder>::upload_to_texture(&self, dst, src, data);
    }
}
