use std::{
    collections::VecDeque,
    sync::{atomic::AtomicU64, Arc},
};

use bytemuck::{cast_slice_mut, checked::cast_slice, Pod};
use parking_lot::Mutex;

use super::{
    types::{
        BindGroupDesc, BindGroupLayoutDesc, BufferDesc, BufferUsage, CommandQueueType,
        ComputePipelineDesc, ImageDesc, ImageViewDesc, RenderPipelineDesc, SyncPoint,
    },
    Handle, Pool,
};

pub struct RenderBackend {
    devices: Vec<Arc<RenderDevice>>,
}

impl RenderBackend {
    pub fn new() -> Self {
        Self {
            devices: vec![Arc::new(RenderDevice::new())],
        }
    }

    pub fn get_all_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDevice> + 'a {
        self.devices.iter().map(|v| &**v)
    }

    pub fn get_device(self, index: usize) -> Arc<RenderDevice> {
        Arc::clone(&self.devices[index])
    }
}

pub struct RenderDevice {
    buffers: Mutex<Pool<Buffer>>,
    images: Mutex<Pool<Image>>,
    render_pipeline: Mutex<Pool<RenderPipeline>>,
    compute_pipeline: Mutex<Pool<ComputePipeline>>,
    bind_group_layout: Mutex<Pool<BindGroupLayout>>,
    bind_group: Mutex<Pool<BindGroup>>,
    temp_bind_group: Mutex<Pool<TemporaryBindGroup>>,

    io_queue: CommandQueue,
}

impl RenderDevice {
    pub fn new() -> Self {
        Self {
            buffers: Mutex::new(Pool::new(None)),
            images: Mutex::new(Pool::new(None)),
            render_pipeline: Mutex::new(Pool::new(None)),
            compute_pipeline: Mutex::new(Pool::new(None)),
            bind_group_layout: Mutex::new(Pool::new(None)),
            bind_group: Mutex::new(Pool::new(None)),
            temp_bind_group: Mutex::new(Pool::new(None)),

            io_queue: CommandQueue::new(CommandQueueType::Io, None),
        }
    }

    pub fn create_command_queue(
        self: &Arc<Self>,
        ty: CommandQueueType,
        cb_count: Option<usize>,
    ) -> Arc<CommandQueue> {
        Arc::new(CommandQueue::new(ty, cb_count))
    }

    pub fn create_buffer<T: Pod>(&self, desc: BufferDesc<T>) -> Handle<Buffer> {
        let mut buffer = Buffer {
            buf: vec![0; desc.size],
            size: desc.size,
            stride: desc.stride,
            usage: desc.usage,
        };

        let dst_buffer = if let Some(content) = desc.content {
            if !desc.usage.contains(BufferUsage::Copy) {
                let mut cmd = self.io_queue.create_command_buffer(self);

                let dst_buffer = self.buffers.lock().push(buffer);
                let src_buffer = self.create_buffer(BufferDesc {
                    usage: BufferUsage::Copy,
                    ..desc
                });

                {
                    let io_encoder = cmd.blit_encoder();
                    io_encoder.copy_buffer_to_buffer(&self, dst_buffer, src_buffer);
                }

                self.io_queue.push_cmd_buffer(cmd);
                self.io_queue.commit();

                dst_buffer
            } else {
                buffer.buf.clone_from_slice(cast_slice(&content));

                self.buffers.lock().push(buffer)
            }
        } else {
            self.buffers.lock().push(buffer)
        };

        dst_buffer
    }

    pub fn create_image(&mut self, desc: ImageDesc) -> Handle<Image> {
        todo!()
    }

    pub fn create_image_view(&mut self, desc: ImageViewDesc) -> Handle<Image> {
        todo!()
    }

    pub fn create_bind_group_layout(
        &mut self,
        desc: BindGroupLayoutDesc,
    ) -> Handle<BindGroupLayout> {
        todo!()
    }

    pub fn create_bind_group(&mut self, desc: BindGroupDesc) -> Handle<BindGroup> {
        todo!()
    }

    pub fn create_temp_bind_group(&mut self, desc: BindGroupDesc) -> Handle<TemporaryBindGroup> {
        todo!()
    }

    pub fn create_render_pipeline_state(
        &mut self,
        desc: &RenderPipelineDesc,
    ) -> Handle<RenderPipeline> {
        todo!()
    }

    pub fn create_compute_pipeline_state(
        &mut self,
        desc: &ComputePipelineDesc,
    ) -> Handle<ComputePipeline> {
        todo!()
    }
}

pub struct CommandQueue {
    ty: CommandQueueType,

    cmd_buffers: Mutex<VecDeque<CommandBufferEntry>>,
    capacity: Option<usize>,

    fence: LocalFence,

    in_record: Mutex<Vec<CommandBuffer>>,
    pending: Mutex<Vec<CommandBuffer>>,
}

impl CommandQueue {
    fn new(ty: CommandQueueType, capacity: Option<usize>) -> Self {
        let cmd_buffers = if let Some(capacity) = capacity {
            VecDeque::with_capacity(capacity)
        } else {
            VecDeque::new()
        };

        Self {
            ty,
            cmd_buffers: Mutex::new(cmd_buffers),
            capacity,
            fence: LocalFence::new(),

            in_record: Mutex::new(Vec::new()),
            pending: Mutex::new(Vec::new()),
        }
    }

    pub fn create_command_buffer(&self, _device: &RenderDevice) -> CommandBuffer {
        if let Some(buffer) = self.in_record.lock().pop() {
            return buffer;
        }

        let entry = if self
            .cmd_buffers
            .lock()
            .front()
            .is_some_and(|v| self.is_complete(v.value))
        {
            self.cmd_buffers.lock().pop_front().expect("unreachable")
        } else {
            if self.capacity.is_some() {
                let entry = self.cmd_buffers.lock().pop_front().expect("unreachable");
                self.fence.wait(entry.value);

                entry
            } else {
                CommandBufferEntry { value: 0 }
            }
        };

        CommandBuffer { entry }
    }

    pub fn stash_cmd_buffer(&self, cmd_buffer: CommandBuffer) {
        self.in_record.lock().push(cmd_buffer);
    }

    pub fn push_cmd_buffer(&self, cmd_buffer: CommandBuffer) {
        self.pending.lock().push(cmd_buffer);
    }

    pub fn commit(&self) -> SyncPoint {
        let cmd_buffers = self.pending.lock().drain(..).collect::<Vec<_>>();

        let fence_value = self.signal();
        self.cmd_buffers
            .lock()
            .extend(cmd_buffers.into_iter().map(|mut v| {
                v.entry.value = fence_value;
                v.entry
            }));

        fence_value
    }

    fn signal(&self) -> SyncPoint {
        self.fence.inc_value()
    }

    fn is_complete(&self, value: SyncPoint) -> bool {
        self.fence.is_complete(value)
    }
}

struct CommandBufferEntry {
    value: SyncPoint,
}

pub struct CommandBuffer {
    entry: CommandBufferEntry,
}

impl CommandBuffer {
    pub fn render_encoder(&mut self) -> RenderEncoder<'_> {
        RenderEncoder { cmd_buffer: self }
    }

    pub fn compute_encoder(&mut self) -> ComputeEncoder<'_> {
        ComputeEncoder { cmd_buffer: self }
    }

    pub fn blit_encoder(&mut self) -> BlitEncoder<'_> {
        BlitEncoder { cmd_buffer: self }
    }
}

pub struct RenderEncoder<'a> {
    cmd_buffer: &'a mut CommandBuffer,
}

pub struct ComputeEncoder<'a> {
    cmd_buffer: &'a mut CommandBuffer,
}

pub struct BlitEncoder<'a> {
    cmd_buffer: &'a mut CommandBuffer,
}

impl<'a> BlitEncoder<'a> {
    pub fn copy_buffer_to_buffer(
        &self,
        device: &RenderDevice,
        dst: Handle<Buffer>,
        src: Handle<Buffer>,
    ) {
        let mut guard = device.buffers.lock();

        let src = { guard.get(src).expect("failed to get buffer").buf.clone() };
        let dst = guard.get_mut(dst).expect("failed to get buffer");

        dst.buf.clone_from_slice(&src);
    }
}

pub struct RenderPipeline {}

pub struct ComputePipeline {}

#[derive(Debug)]
pub struct Buffer {
    buf: Vec<u8>,
    size: usize,
    stride: usize,
    usage: BufferUsage,
}

pub struct Image {}

pub struct BindGroupLayout {}

pub struct BindGroup {}

pub struct TemporaryBindGroup {}

pub struct LocalFence {
    value: AtomicU64,
}

impl LocalFence {
    pub fn new() -> Self {
        Self {
            value: Default::default(),
        }
    }

    pub fn wait(&self, value: SyncPoint) {
        while self.value.load(std::sync::atomic::Ordering::Acquire) < value {
            self.value
                .fetch_add(1, std::sync::atomic::Ordering::Release);
        }
    }

    pub fn is_complete(&self, value: SyncPoint) -> bool {
        self.value.load(std::sync::atomic::Ordering::Acquire) >= value
    }

    pub fn inc_value(&self) -> SyncPoint {
        self.value.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
    }
}

pub struct SharedFence {}
