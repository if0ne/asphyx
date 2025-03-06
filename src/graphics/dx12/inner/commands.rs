use std::{collections::VecDeque, sync::Arc};

use oxidx::dx::{self, ICommandAllocator, ICommandQueue, IDevice, IGraphicsCommandList, PSO_NONE};
use parking_lot::Mutex;

use crate::graphics::{
    core::commands::{CommandBufferType, SyncPoint},
    dx12::{commands::DxCommandBuffer, context::HandleStorage, conv::map_command_buffer_type},
};

use super::sync::DxFence;

#[derive(Debug)]
pub(crate) struct DxCommandQueue {
    device: dx::Device,
    queue: Mutex<dx::CommandQueue>,
    ty_raw: dx::CommandListType,
    ty: CommandBufferType,

    fence: DxFence,

    capacity: Option<usize>,
    cmd_allocators: Mutex<VecDeque<CommandAllocatorEntry>>,
    cmd_lists: Mutex<Vec<dx::GraphicsCommandList>>,

    in_record: Mutex<Vec<DxCommandBuffer>>,
    pending: Mutex<Vec<DxCommandBuffer>>,

    frequency: f64,
}

impl DxCommandQueue {
    pub(crate) fn new(device: &dx::Device, ty: CommandBufferType, capacity: Option<usize>) -> Self {
        let queue = device
            .create_command_queue(&dx::CommandQueueDesc::new(map_command_buffer_type(ty)))
            .expect("failed to create command queue");

        let fence = DxFence::new(device);

        let frequency = 1000.0
            / queue
                .get_timestamp_frequency()
                .expect("failed to fetch timestamp frequency") as f64;

        let cmd_allocators = (0..3)
            .map(|_| CommandAllocatorEntry {
                raw: device
                    .create_command_allocator(map_command_buffer_type(ty))
                    .expect("failed to create command allocator"),
                sync_point: 0,
            })
            .collect::<VecDeque<_>>();

        let cmd_list = device
            .create_command_list(
                0,
                map_command_buffer_type(ty),
                &cmd_allocators[0].raw,
                PSO_NONE,
            )
            .expect("failed to create command list");
        cmd_list.close().expect("failed to close list");

        Self {
            device: device.clone(),
            queue: Mutex::new(queue),
            ty: ty.clone(),
            ty_raw: map_command_buffer_type(ty),
            fence,
            frequency,

            capacity,
            cmd_allocators: Mutex::new(cmd_allocators),
            cmd_lists: Mutex::new(vec![cmd_list]),
            in_record: Default::default(),
            pending: Default::default(),
        }
    }

    pub(crate) fn signal(&self, fence: &DxFence) -> u64 {
        let value = fence.inc_value();
        self.queue
            .lock()
            .signal(&fence.fence, value)
            .expect("failed to signal");

        value
    }

    pub(crate) fn is_complete(&self, value: u64) -> bool {
        self.fence.get_completed_value() >= value
    }

    pub(crate) fn signal_queue(&self) -> u64 {
        self.signal(&self.fence)
    }

    pub(crate) fn create_command_buffer(&self, handles: Arc<HandleStorage>) -> DxCommandBuffer {
        if let Some(buffer) = self.in_record.lock().pop() {
            return buffer;
        };

        let allocator = if let Some(allocator) =
            self.cmd_allocators.lock().pop_front().and_then(|a| {
                if self.is_complete(a.sync_point) {
                    Some(a)
                } else {
                    None
                }
            }) {
            allocator
                .raw
                .reset()
                .expect("failed to reset command allocator");

            allocator
        } else {
            if self.capacity.is_some() {
                let entry = self.cmd_allocators.lock().pop_front().expect("unreachable");
                self.fence.wait(entry.sync_point);

                entry
            } else {
                CommandAllocatorEntry {
                    raw: self
                        .device
                        .create_command_allocator(self.ty_raw)
                        .expect("failed to create command allocator"),
                    sync_point: 0,
                }
            }
        };

        let list = if let Some(list) = self.cmd_lists.lock().pop() {
            list.reset(&allocator.raw, PSO_NONE)
                .expect("Failed to reset list");
            list
        } else {
            let list = self
                .device
                .create_command_list(0, self.ty_raw, &allocator.raw, PSO_NONE)
                .expect("failed to create command list");
            list.close().expect("failed to close list");
            list
        };

        DxCommandBuffer {
            handles,
            ty: self.ty,
            list,
            allocator,
        }
    }

    pub(crate) fn stash_cmd_buffer(&self, cmd_buffer: DxCommandBuffer) {
        self.in_record.lock().push(cmd_buffer);
    }

    pub(crate) fn push_cmd_buffer(&self, cmd_buffer: DxCommandBuffer) {
        cmd_buffer.list.close().expect("failed to close list");
        self.pending.lock().push(cmd_buffer);
    }

    pub(crate) fn commit(&self) -> SyncPoint {
        let cmd_buffers = self.pending.lock().drain(..).collect::<Vec<_>>();
        let lists = cmd_buffers
            .iter()
            .map(|b| Some(b.list.clone()))
            .collect::<Vec<_>>();

        self.queue.lock().execute_command_lists(&lists);
        let fence_value = self.signal_queue();

        let allocators = cmd_buffers.into_iter().map(|mut buffer| {
            buffer.allocator.sync_point = fence_value;
            buffer.allocator
        });
        self.cmd_allocators.lock().extend(allocators);

        let lists = lists.into_iter().map(|list| list.unwrap());
        self.cmd_lists.lock().extend(lists);

        fence_value
    }

    pub(crate) fn wait_cpu(&self, time: SyncPoint) {
        self.fence.wait(time);
    }
}

#[derive(Debug)]
pub(crate) struct CommandAllocatorEntry {
    raw: dx::CommandAllocator,
    sync_point: SyncPoint,
}
