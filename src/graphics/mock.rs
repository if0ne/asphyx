use std::{
    collections::VecDeque,
    sync::{atomic::AtomicU64, Arc},
};

use bytemuck::{checked::cast_slice, Pod};
use parking_lot::Mutex;

use super::{
    types::{
        BindGroupLayoutDesc, BufferDesc, BufferUsage, CommandQueueType, ComputePipelineDesc,
        CreateBufferInfo, CreateImageInfo, CreateImageViewInfo, ImageDesc, ImageViewDesc,
        MemoryType, RenderPipelineDesc, SyncPoint,
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
    bind_group_layouts: Mutex<Pool<BindGroupLayout>>,
    bind_groups: Mutex<Pool<BindGroup>>,
    pipeline_layouts: Mutex<Pool<PipelineLayout>>,

    io_queue: CommandQueue,
}

impl RenderDevice {
    pub fn new() -> Self {
        Self {
            buffers: Mutex::new(Pool::new(None)),
            images: Mutex::new(Pool::new(None)),
            bind_group_layouts: Mutex::new(Pool::new(None)),
            bind_groups: Mutex::new(Pool::new(None)),
            pipeline_layouts: Mutex::new(Pool::new(None)),

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

    pub fn create_buffer<T: Pod>(&self, desc: CreateBufferInfo<T>) -> Handle<Buffer> {
        let mut buffer = Buffer {
            buf: vec![0; desc.size],
            desc: BufferDesc {
                size: desc.size,
                stride: desc.stride,
                usage: desc.usage,
                mem_ty: desc.mem_ty.unwrap_or(MemoryType::Upload), // TODO: Define mem type by usage
            },
        };

        let handle = if let Some(content) = desc.content {
            if !desc.usage.contains(BufferUsage::Copy) {
                let mut cmd = self.io_queue.create_command_buffer();

                let dst_buffer = self.buffers.lock().push(buffer);
                let src_buffer = self.create_buffer(CreateBufferInfo {
                    usage: BufferUsage::Copy,
                    ..desc
                });

                {
                    let io_encoder = cmd.blit_encoder();
                    io_encoder.copy_buffer_to_buffer(self, dst_buffer, src_buffer);
                }

                self.io_queue.push_cmd_buffer(cmd);
                self.io_queue.commit();
                self.buffers.lock().remove(src_buffer);

                dst_buffer
            } else {
                buffer.buf.clone_from_slice(cast_slice(&content));

                self.buffers.lock().push(buffer)
            }
        } else {
            self.buffers.lock().push(buffer)
        };

        handle
    }

    pub fn create_image(&self, desc: CreateImageInfo) -> Handle<Image> {
        let image = self.images.lock().push(Image {
            buf: Arc::new(Mutex::new(vec![0; 128])), // TODO: Calculate size
            desc: ImageDesc {
                width: desc.width,
                height: desc.height,
                array: desc.array,
                levels: desc.levels,
                format: desc.format,
                usage: desc.usage,
                mem_ty: desc.mem_ty.unwrap_or(MemoryType::Device), // TODO: Define mem type by usage
            },
            view: ImageViewDesc {
                mip_slice: 0,
                plane_slice: 0,
                array: 0..desc.array,
            },
            is_view: false,
        });

        if let Some(content) = desc.content {
            let mut cmd = self.io_queue.create_command_buffer();

            let src_buffer = self.create_buffer(CreateBufferInfo {
                usage: BufferUsage::Copy,
                size: (desc.width * desc.height * desc.array * 4) as usize, // TODO: Calculate size
                stride: 0,
                mem_ty: None,
                content: Some(content),
            });

            {
                let io_encoder = cmd.blit_encoder();
                io_encoder.copy_buffer_to_image(self, image, src_buffer);
            }

            self.io_queue.push_cmd_buffer(cmd);
            self.io_queue.commit();
            self.buffers.lock().remove(src_buffer);
        };

        image
    }

    pub fn create_image_view(
        &self,
        image: Handle<Image>,
        desc: CreateImageViewInfo,
    ) -> Handle<Image> {
        let (buf, img_desc) = {
            let guard = self.images.lock();
            let image = guard.get(image).expect("failed to get image");

            (Arc::clone(&image.buf), image.desc.clone())
        };

        self.images.lock().push(Image {
            buf, // TODO: Calculate size
            desc: img_desc,
            view: ImageViewDesc {
                mip_slice: desc.mip_slice,
                plane_slice: desc.plane_slice,
                array: desc.array,
            },
            is_view: true,
        })
    }

    pub fn create_bind_group_layout(
        &mut self,
        desc: BindGroupLayoutDesc,
    ) -> Handle<BindGroupLayout> {
        self.bind_group_layouts.lock().push(BindGroupLayout {})
    }

    pub fn create_pipeline_layout(&mut self, desc: PipelineLayoutDesc) -> Handle<PipelineLayout> {
        self.pipeline_layouts.lock().push(PipelineLayout {})
    }

    pub fn create_bind_group(&mut self, desc: BindGroupDesc) -> Handle<BindGroup> {
        self.bind_groups.lock().push(BindGroup {})
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

    pub fn create_command_buffer(&self) -> CommandBuffer {
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
        rd: &RenderDevice,
        dst: Handle<Buffer>,
        src: Handle<Buffer>,
    ) {
        let mut guard = rd.buffers.lock();
        let [dst, src] = guard.get_many([dst, src]).expect("failed to get buffers");

        dst.buf.clone_from_slice(&src.buf);
    }

    pub fn copy_buffer_to_image(&self, rd: &RenderDevice, dst: Handle<Image>, src: Handle<Buffer>) {
        let bguard = rd.buffers.lock();
        let iguard = rd.images.lock();

        let src = bguard.get(src).expect("failed to get buffer");
        let dst = iguard.get(dst).expect("failed to get image");

        dst.buf.lock().clone_from_slice(&src.buf);
    }
}

pub struct RenderPipeline {}

pub struct ComputePipeline {}

#[derive(Debug)]
pub struct Buffer {
    buf: Vec<u8>,
    desc: BufferDesc,
}

#[derive(Debug)]
pub struct Image {
    buf: Arc<Mutex<Vec<u8>>>,
    desc: ImageDesc,
    view: ImageViewDesc,
    is_view: bool,
}

#[derive(Debug)]
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

pub struct PipelineLayoutDesc<'a> {
    pub groups: &'a [Handle<BindGroupLayout>],
}

#[derive(Debug)]
pub struct PipelineLayout {}

#[derive(Debug)]
pub struct BufferDescriptor {
    pub buffer: Handle<Buffer>,
    pub slot: u32,
}

#[derive(Debug)]
pub struct ImageDescriptor {
    pub image: Handle<Image>,
    pub slot: u32,
}

pub struct BindGroupDesc<'a> {
    pub space: u32,
    pub buffers: &'a [BufferDescriptor],
    pub images: &'a [ImageDescriptor],
}
