/*use std::{
    collections::VecDeque,
    sync::{atomic::AtomicU64, Arc},
};

use bytemuck::{checked::cast_slice, Pod};
use parking_lot::Mutex;

use crate::allocators::{Handle, Pool};

use super::{
    traits::CommandQueue as _,
    types::{
        BindGroupDesc, BindGroupLayoutDesc, BufferDesc, BufferUsage, CommandQueueType,
        ComputePipelineDesc, CreateBufferInfo, CreateImageInfo, CreateImageViewInfo, ImageDesc,
        ImageViewDesc, MemoryType, PipelineLayoutDesc, RenderBackend as Backend,
        RenderPipelineDesc, SyncPoint,
    },
};

pub struct RenderBackend {
    devices: Vec<Arc<RenderDevice>>,
}

impl RenderBackend {
    pub(super) fn new() -> Self {
        Self {
            devices: vec![Arc::new(RenderDevice::new(0))],
        }
    }
}

impl super::traits::Api for RenderBackend {
    type Device = RenderDevice;
    type CommandQueue = CommandQueue;
    type CommandBuffer = CommandBuffer;

    type Buffer = Buffer;
    type Image = Image;
    type BindGroupLayout = BindGroupLayout;
    type PipelineLayout = PipelineLayout;
    type BindGroup = BindGroup;
    type TemporaryBindGroup = TemporaryBindGroup;
    type RenderPipeline = RenderPipeline;
    type ComputePipeline = ComputePipeline;

    fn get_all_devices<'a>(&'a self) -> impl Iterator<Item = &'a Self::Device> + 'a {
        self.devices.iter().map(|v| &**v)
    }

    fn get_device(&self, index: usize) -> Arc<Self::Device> {
        Arc::clone(&self.devices[index])
    }
}

pub struct RenderDevice {
    device_id: usize,
    buffers: Mutex<Pool<Buffer>>,
    images: Mutex<Pool<Image>>,
    bind_group_layouts: Mutex<Pool<BindGroupLayout>>,
    bind_groups: Mutex<Pool<BindGroup>>,
    pipeline_layouts: Mutex<Pool<PipelineLayout>>,
    render_pipeline: Mutex<Pool<RenderPipeline>>,
    compute_pipeline: Mutex<Pool<ComputePipeline>>,

    io_queue: CommandQueue,
}

impl RenderDevice {
    fn new(device_id: usize) -> Self {
        Self {
            device_id,
            buffers: Mutex::new(Pool::new(None)),
            images: Mutex::new(Pool::new(None)),
            bind_group_layouts: Mutex::new(Pool::new(None)),
            bind_groups: Mutex::new(Pool::new(None)),
            pipeline_layouts: Mutex::new(Pool::new(None)),
            render_pipeline: Mutex::new(Pool::new(None)),
            compute_pipeline: Mutex::new(Pool::new(None)),

            io_queue: CommandQueue::new(CommandQueueType::Io, None),
        }
    }
}

impl super::traits::Device<RenderBackend> for RenderDevice {
    fn get_backend(&self) -> Backend {
        Backend::Mock
    }

    fn get_device_id(&self) -> usize {
        self.device_id
    }

    fn create_command_queue(
        self: &Arc<Self>,
        ty: CommandQueueType,
        cb_count: Option<usize>,
    ) -> Arc<CommandQueue> {
        Arc::new(CommandQueue::new(ty, cb_count))
    }

    fn create_buffer<T: Pod>(&self, desc: &CreateBufferInfo<T>) -> Handle<Buffer> {
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
                let src_buffer = self.create_buffer(&CreateBufferInfo {
                    name: None,
                    usage: BufferUsage::Copy,
                    size: desc.size,
                    stride: desc.stride,
                    mem_ty: desc.mem_ty,
                    content: desc.content,
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

    fn destroy_buffer(&self, buffer: Handle<Buffer>) {
        self.buffers.lock().remove(buffer);
    }

    fn create_image(&self, desc: &CreateImageInfo) -> Handle<Image> {
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

            let src_buffer = self.create_buffer(&CreateBufferInfo {
                name: None,
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

    fn destroy_image(&self, image: Handle<Image>) {
        self.images.lock().remove(image);
    }

    fn create_image_view(&self, image: Handle<Image>, desc: CreateImageViewInfo) -> Handle<Image> {
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

    fn create_bind_group_layout(&self, desc: BindGroupLayoutDesc) -> Handle<BindGroupLayout> {
        self.bind_group_layouts.lock().push(BindGroupLayout {})
    }

    fn destroy_bind_group_layout(&self, handle: Handle<BindGroupLayout>) {
        self.bind_group_layouts.lock().remove(handle);
    }

    fn create_pipeline_layout(
        &self,
        desc: PipelineLayoutDesc<RenderBackend>,
    ) -> Handle<PipelineLayout> {
        self.pipeline_layouts.lock().push(PipelineLayout {})
    }

    fn destroy_pipeline_layout(&self, handle: Handle<PipelineLayout>) {
        self.pipeline_layouts.lock().remove(handle);
    }

    fn create_bind_group(&self, desc: BindGroupDesc<RenderBackend>) -> Handle<BindGroup> {
        self.bind_groups.lock().push(BindGroup {})
    }

    fn destroy_bind_group(&self, handle: Handle<BindGroup>) {
        self.bind_groups.lock().remove(handle);
    }

    fn create_temp_bind_group(&self, desc: BindGroupDesc<RenderBackend>) -> TemporaryBindGroup {
        TemporaryBindGroup {}
    }

    fn create_render_pipeline(&self, desc: &RenderPipelineDesc) -> Handle<RenderPipeline> {
        self.render_pipeline.lock().push(RenderPipeline {})
    }

    fn destroy_render_pipeline(&self, handle: Handle<RenderPipeline>) {
        self.render_pipeline.lock().remove(handle);
    }

    fn create_compute_pipeline(&self, desc: &ComputePipelineDesc) -> Handle<ComputePipeline> {
        self.compute_pipeline.lock().push(ComputePipeline {})
    }

    fn destroy_compute_pipeline(&self, handle: Handle<ComputePipeline>) {
        self.compute_pipeline.lock().remove(handle);
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

    fn signal(&self) -> SyncPoint {
        self.fence.inc_value()
    }

    fn is_complete(&self, value: SyncPoint) -> bool {
        self.fence.is_complete(value)
    }
}

impl super::traits::CommandQueue<RenderBackend> for CommandQueue {
    fn create_command_buffer(&self) -> CommandBuffer {
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

    fn stash_cmd_buffer(&self, cmd_buffer: CommandBuffer) {
        self.in_record.lock().push(cmd_buffer);
    }

    fn push_cmd_buffer(&self, cmd_buffer: CommandBuffer) {
        self.pending.lock().push(cmd_buffer);
    }

    fn commit(&self) -> SyncPoint {
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

#[derive(Debug)]
pub struct PipelineLayout {}
*/
