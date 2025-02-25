mod conv;
/*
use std::{
    collections::VecDeque,
    ffi::CString,
    sync::{atomic::AtomicU64, Arc, Weak},
};

use bytemuck::Pod;
use oxidx::dx::{self, *};
use parking_lot::Mutex;
use tracing::{debug, error, info, warn};

use crate::{
    allocators::{Handle, Pool, UntypedHandle},
    graphics::{
        traits::{self, Device},
        types,
    },
};

use super::traits::CommandQueue;

#[derive(Debug)]
pub struct DxBackend {
    factory: dx::Factory4,
    debug: Option<dx::Debug1>,
    devices: Vec<Arc<DxDevice>>,
}

impl DxBackend {
    pub fn new(debug_flags: types::DebugFlags) -> Self {
        let flags = if !debug_flags.is_empty() {
            dx::FactoryCreationFlags::Debug
        } else {
            dx::FactoryCreationFlags::empty()
        };

        let factory = dx::create_factory4(flags).expect("failed to create DXGI factory");

        let debug = if !debug_flags.is_empty() {
            let debug: dx::Debug1 = dx::create_debug()
                .expect("failed to create debug")
                .try_into()
                .expect("failed to fetch debug1");

            debug.enable_debug_layer();
            debug.set_enable_gpu_based_validation(true);
            debug.set_callback(Box::new(|_, severity, _, msg| match severity {
                dx::MessageSeverity::Corruption => error!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Error => error!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Warning => warn!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Info => info!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Message => debug!("[D3D12 Validation] {}", msg),
            }));

            Some(debug)
        } else {
            None
        };

        let gpu_warp = factory
            .enum_warp_adapters()
            .expect("failed to get warp device");
        let mut gpus = vec![];

        if let Ok(factory) = TryInto::<dx::Factory7>::try_into(factory.clone()) {
            debug!("Factory7 is supported");

            let mut i = 0;

            while let Ok(adapter) =
                factory.enum_adapters_by_gpu_preference(i, dx::GpuPreference::HighPerformance)
            {
                let Ok(desc) = adapter.get_desc1() else {
                    i += 1;
                    continue;
                };

                if desc.flags().contains(dx::AdapterFlags::Sofware) {
                    i += 1;
                    continue;
                }

                info!("found adapter: {}", desc.description());

                if dx::create_device(Some(&adapter), dx::FeatureLevel::Level11).is_ok() {
                    gpus.push(adapter);
                }

                i += 1;
            }
        } else {
            let mut i = 0;
            while let Ok(adapter) = factory.enum_adapters(i) {
                let Ok(desc) = adapter.get_desc1() else {
                    i += 1;
                    continue;
                };

                if desc.flags().contains(dx::AdapterFlags::Sofware) {
                    i += 1;
                    continue;
                }

                info!("found adapter: {}", desc.description());

                if dx::create_device(Some(&adapter), dx::FeatureLevel::Level11).is_ok() {
                    gpus.push(adapter);
                }

                i += 1;
            }

            gpus.sort_by(|l, r| {
                let descs = (
                    l.get_desc1().map(|d| d.vendor_id()),
                    r.get_desc1().map(|d| d.vendor_id()),
                );

                match descs {
                    (Ok(0x8086), Ok(0x8086)) => std::cmp::Ordering::Equal,
                    (Ok(0x8086), Ok(_)) => std::cmp::Ordering::Less,
                    (Ok(_), Ok(0x8086)) => std::cmp::Ordering::Greater,
                    (_, _) => std::cmp::Ordering::Equal,
                }
            });
        }

        gpus.push(gpu_warp);

        let devices = gpus
            .into_iter()
            .enumerate()
            .map(|(id, a)| DxDevice::new(a, id))
            .collect();

        Self {
            factory,
            debug,
            devices,
        }
    }
}

impl traits::Api for DxBackend {
    type Device = DxDevice;
    type CommandQueue = DxCommandQueue;
    type CommandBuffer = DxCommandBuffer;
    type Buffer = DxBuffer;
    type Image = DxTexture;
    type BindGroupLayout = DxBindGroupLayout;
    type PipelineLayout = DxPipelineLayout;
    type BindGroup = DxBindGroup;
    type TemporaryBindGroup = DxTemporaryBindGroup;
    type RenderPipeline = DxRenderPipeline;
    type ComputePipeline = DxComputePipeline;

    fn get_all_devices<'a>(&'a self) -> impl Iterator<Item = &'a Self::Device> + 'a {
        self.devices.iter().map(|d| &**d)
    }

    fn get_device(&self, index: usize) -> Arc<Self::Device> {
        Arc::clone(&self.devices[index])
    }
}

impl traits::DynApi for DxBackend {}

#[derive(Debug)]
pub struct DxDevice {
    id: usize,
    adapter: dx::Adapter3,
    gpu: dx::Device,

    is_cross_adapter_texture_supported: bool,

    io_queue: Mutex<Option<DxCommandQueue>>,
    buffers: Mutex<Pool<DxBuffer>>,
}

impl DxDevice {
    fn new_inner(adapter: dx::Adapter3, id: usize) -> Self {
        info!(
            "creating device with adapter {} and id {:?}",
            adapter.get_desc1().unwrap().description(),
            id
        );

        let device = dx::create_device(Some(&adapter), dx::FeatureLevel::Level11)
            .expect("failed to create device");

        let mut feature = dx::features::OptionsFeature::default();
        device
            .check_feature_support(&mut feature)
            .expect("failed to fetch options feature");

        if feature.cross_adapter_row_major_texture_supported() {
            info!("Cross Adapter Row Major Texture is supported");
        } else {
            info!("Cross Adapter Row Major Texture is NOT supported");
        }

        Self {
            id,
            adapter,
            gpu: device,

            is_cross_adapter_texture_supported: feature.cross_adapter_row_major_texture_supported(),

            io_queue: Mutex::new(None),
            buffers: Mutex::new(Pool::new(None)),
        }
    }

    fn new(adapter: dx::Adapter3, id: usize) -> Arc<Self> {
        let device = Arc::new(Self::new_inner(adapter, id));

        *device.io_queue.lock() = Some(DxCommandQueue::new(
            &device,
            types::CommandQueueType::Io,
            None,
        ));

        device
    }
}

impl traits::Device<DxBackend> for DxDevice {
    fn get_backend(&self) -> super::types::RenderBackend {
        super::types::RenderBackend::Dx12
    }

    fn get_device_id(&self) -> usize {
        self.id
    }

    fn create_command_queue(
        self: &Arc<Self>,
        ty: types::CommandQueueType,
        cb_count: Option<usize>,
    ) -> Arc<DxCommandQueue> {
        Arc::new(DxCommandQueue::new(self, ty, cb_count))
    }

    fn create_buffer<T: bytemuck::Pod>(
        &self,
        desc: &types::CreateBufferInfo<T>,
    ) -> Handle<DxBuffer> {
        let dst_buffer = self.buffers.lock().push(DxBuffer::new(self, desc.clone()));

        if desc.content.is_some() {
            if self
                .buffers
                .lock()
                .get(dst_buffer)
                .is_some_and(|b| b.desc.mem_ty != types::MemoryType::Upload)
            {
                if let Some(queue) = &*self.io_queue.lock() {
                    let src_buffer = self.buffers.lock().push(DxBuffer::new(
                        self,
                        types::CreateBufferInfo {
                            name: None,
                            usage: types::BufferUsage::Copy,
                            size: desc.size,
                            stride: desc.stride,
                            mem_ty: desc.mem_ty,
                            content: desc.content,
                        },
                    ));

                    let mut cmd = queue.create_command_buffer();

                    {
                        let io_encoder = cmd.blit_encoder();
                        io_encoder.copy_buffer_to_buffer(self, dst_buffer, src_buffer);
                    }

                    queue.push_cmd_buffer(cmd);
                    queue.commit();

                    self.buffers.lock().remove(src_buffer);
                }
            }
        }

        dst_buffer
    }

    fn destroy_buffer(&self, buffer: Handle<DxBuffer>) {
        todo!()
    }

    fn create_image(&self, desc: &types::CreateImageInfo) -> Handle<DxTexture> {
        todo!()
    }

    fn destroy_image(&self, image: Handle<DxTexture>) {
        todo!()
    }

    fn create_image_view(
        &self,
        image: Handle<DxTexture>,
        desc: types::CreateImageViewInfo,
    ) -> Handle<DxTexture> {
        todo!()
    }

    fn create_bind_group_layout(
        &self,
        desc: types::BindGroupLayoutDesc,
    ) -> Handle<DxBindGroupLayout> {
        todo!()
    }

    fn destroy_bind_group_layout(&self, handle: Handle<DxBindGroupLayout>) {
        todo!()
    }

    fn create_pipeline_layout(
        &self,
        desc: types::PipelineLayoutDesc<DxBackend>,
    ) -> Handle<DxPipelineLayout> {
        todo!()
    }

    fn destroy_pipeline_layout(&self, handle: Handle<DxPipelineLayout>) {
        todo!()
    }

    fn create_bind_group(&self, desc: types::BindGroupDesc<DxBackend>) -> Handle<DxBindGroup> {
        todo!()
    }

    fn destroy_bind_group(&self, handle: Handle<DxBindGroup>) {
        todo!()
    }

    fn create_temp_bind_group(
        &self,
        desc: types::BindGroupDesc<DxBackend>,
    ) -> DxTemporaryBindGroup {
        todo!()
    }

    fn create_render_pipeline(&self, desc: &types::RenderPipelineDesc) -> Handle<DxRenderPipeline> {
        todo!()
    }

    fn destroy_render_pipeline(&self, handle: Handle<DxRenderPipeline>) {
        todo!()
    }

    fn create_compute_pipeline(
        &self,
        desc: &types::ComputePipelineDesc,
    ) -> Handle<DxComputePipeline> {
        todo!()
    }

    fn destroy_compute_pipeline(&self, handle: Handle<DxComputePipeline>) {
        todo!()
    }
}

impl traits::DynDevice for DxDevice {
    fn get_backend(&self) -> super::types::RenderBackend {
        <Self as traits::Device<DxBackend>>::get_backend(&self)
    }

    fn get_device_id(&self) -> usize {
        <Self as traits::Device<DxBackend>>::get_device_id(&self)
    }

    fn create_buffer(&self, desc: &types::CreateBufferInfo) -> UntypedHandle {
        <Self as traits::Device<DxBackend>>::create_buffer(&self, desc).into()
    }

    fn create_image(&self, desc: &types::CreateImageInfo) -> UntypedHandle {
        <Self as traits::Device<DxBackend>>::create_image(&self, desc).into()
    }
}

#[derive(Debug)]
pub struct DxCommandQueue {
    device: Weak<DxDevice>,
    queue: Mutex<dx::CommandQueue>,
    ty: dx::CommandListType,

    fence: DxFence,

    capacity: Option<usize>,
    cmd_allocators: Mutex<VecDeque<CommandAllocatorEntry>>,
    cmd_lists: Mutex<Vec<dx::GraphicsCommandList>>,

    in_record: Mutex<Vec<DxCommandBuffer>>,
    pending: Mutex<Vec<DxCommandBuffer>>,

    frequency: f64,
}

impl DxCommandQueue {
    fn new(device: &Arc<DxDevice>, ty: types::CommandQueueType, capacity: Option<usize>) -> Self {
        let queue = device
            .gpu
            .create_command_queue(&dx::CommandQueueDesc::new(ty.into()))
            .expect("failed to create command queue");

        let fence = DxFence::new(device);

        let frequency = 1000.0
            / queue
                .get_timestamp_frequency()
                .expect("failed to fetch timestamp frequency") as f64;

        let cmd_allocators = (0..3)
            .map(|_| CommandAllocatorEntry {
                raw: device
                    .gpu
                    .create_command_allocator(ty.into())
                    .expect("failed to create command allocator"),
                sync_point: 0,
            })
            .collect::<VecDeque<_>>();

        let cmd_list = device
            .gpu
            .create_command_list(0, ty.into(), &cmd_allocators[0].raw, PSO_NONE)
            .expect("failed to create command list");
        cmd_list.close().expect("failed to close list");

        Self {
            device: Arc::downgrade(&device),
            queue: Mutex::new(queue),
            ty: ty.into(),
            fence,
            frequency,

            capacity,
            cmd_allocators: Mutex::new(cmd_allocators),
            cmd_lists: Mutex::new(vec![cmd_list]),
            in_record: Default::default(),
            pending: Default::default(),
        }
    }

    fn signal(&self, fence: &DxFence) -> u64 {
        let value = fence.inc_value();
        self.queue
            .lock()
            .signal(&fence.fence, value)
            .expect("failed to signal");

        value
    }

    fn is_complete(&self, value: u64) -> bool {
        self.fence.get_completed_value() >= value
    }

    fn signal_queue(&self) -> u64 {
        self.signal(&self.fence)
    }
}

impl traits::CommandQueue<DxBackend> for DxCommandQueue {
    fn create_command_buffer(&self) -> DxCommandBuffer {
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
                        .upgrade()
                        .expect("device lost")
                        .gpu
                        .create_command_allocator(self.ty)
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
                .upgrade()
                .expect("device lost")
                .gpu
                .create_command_list(0, self.ty, &allocator.raw, PSO_NONE)
                .expect("failed to create command list");
            list.close().expect("failed to close list");
            list
        };

        DxCommandBuffer {
            device_id: self.device.upgrade().expect("device lost").id,
            ty: self.ty,
            list,
            allocator,
        }
    }

    fn stash_cmd_buffer(&self, cmd_buffer: DxCommandBuffer) {
        self.in_record.lock().push(cmd_buffer);
    }

    fn push_cmd_buffer(&self, cmd_buffer: DxCommandBuffer) {
        cmd_buffer.list.close().expect("failed to close list");
        self.pending.lock().push(cmd_buffer);
    }

    fn commit(&self) -> types::SyncPoint {
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
}

#[derive(Debug)]
struct CommandAllocatorEntry {
    raw: dx::CommandAllocator,
    sync_point: types::SyncPoint,
}

#[derive(Debug)]
pub struct DxCommandBuffer {
    device_id: usize,
    ty: dx::CommandListType,
    list: dx::GraphicsCommandList,
    allocator: CommandAllocatorEntry,
}

impl DxCommandBuffer {
    pub fn blit_encoder(&mut self) -> DxBlitEncoder<'_> {
        DxBlitEncoder { cmd_buffer: self }
    }
}

pub struct DxBlitEncoder<'a> {
    cmd_buffer: &'a mut DxCommandBuffer,
}

impl<'a> DxBlitEncoder<'a> {
    pub fn copy_buffer_to_buffer(
        &self,
        rd: &DxDevice,
        dst: Handle<DxBuffer>,
        src: Handle<DxBuffer>,
    ) {
        let mut guard = rd.buffers.lock();
        let [dst, src] = guard.get_many([dst, src]).expect("failed to get buffers");

        self.cmd_buffer.list.copy_resource(&dst.res, &src.res);
    }
}

#[derive(Debug)]
pub struct DxFence {
    pub fence: dx::Fence,
    pub value: AtomicU64,
}

impl DxFence {
    fn new(device: &DxDevice) -> Self {
        let fence = device
            .gpu
            .create_fence(0, dx::FenceFlags::empty())
            .expect("failed to create fence");

        Self {
            fence,
            value: Default::default(),
        }
    }

    pub fn wait(&self, value: u64) -> bool {
        if self.get_completed_value() < value {
            let event = dx::Event::create(false, false).expect("failed to create event");
            self.fence
                .set_event_on_completion(value, event)
                .expect("failed to bind fence to event");
            if event.wait(10_000_000) == 0x00000102 {
                panic!("device lost")
            }

            true
        } else {
            false
        }
    }

    pub fn inc_value(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    pub fn get_completed_value(&self) -> u64 {
        self.fence.get_completed_value()
    }

    pub fn get_current_value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct DxBuffer {
    res: dx::Resource,
    desc: super::types::BufferDesc,
    state: super::types::BufferState,
}

impl DxBuffer {
    fn new<T: Pod>(device: &DxDevice, desc: super::types::CreateBufferInfo<T>) -> Self {
        info!(
            "creating buffer for {:?}{:?}, size: {}, stride: {}, usage: {:?}, mem type: {:?}, with content: {}",
            device.get_backend(),
            device.id,
            desc.size * size_of::<T>(),
            desc.stride,
            desc.usage,
            desc.mem_ty,
            desc.content.is_some()
        );

        let (mem_ty, heap_props) = if let Some(ty) = desc.mem_ty {
            (
                ty,
                match ty {
                    types::MemoryType::Upload => dx::HeapProperties::upload(),
                    types::MemoryType::Readback => dx::HeapProperties::readback(),
                    types::MemoryType::Device => dx::HeapProperties::default(),
                    types::MemoryType::Shared => dx::HeapProperties::default(),
                },
            )
        } else {
            if desc.usage.intersects(types::BufferUsage::Uniform) {
                (types::MemoryType::Upload, dx::HeapProperties::upload())
            } else if desc.usage.intersects(types::BufferUsage::Readback) {
                (types::MemoryType::Readback, dx::HeapProperties::readback())
            } else {
                if desc.usage.intersects(types::BufferUsage::Shared) {
                    (types::MemoryType::Device, dx::HeapProperties::default())
                } else {
                    (types::MemoryType::Shared, dx::HeapProperties::default())
                }
            }
        };

        let state = if desc.usage.intersects(types::BufferUsage::Readback) {
            types::BufferState::CopyDst
        } else if desc.usage.intersects(types::BufferUsage::Copy)
            | desc.usage.intersects(types::BufferUsage::Uniform)
        {
            types::BufferState::Generic
        } else {
            types::BufferState::Unknown
        };

        let flags = if desc.usage.intersects(types::BufferUsage::Storage) {
            dx::ResourceFlags::AllowUnorderedAccess
        } else {
            dx::ResourceFlags::empty()
        };

        let flags = if desc.usage.intersects(types::BufferUsage::Shared) {
            flags | dx::ResourceFlags::AllowCrossAdapter
        } else {
            flags
        };

        let rdesc = dx::ResourceDesc::buffer(desc.size * size_of::<T>())
            .with_layout(dx::TextureLayout::RowMajor)
            .with_flags(flags);

        let res = device
            .gpu
            .create_committed_resource(
                &heap_props,
                dx::HeapFlags::empty(),
                &rdesc,
                state.into(),
                None,
            )
            .expect("failed to create buffer");

        if let Some(name) = desc.name {
            let debug_name = CString::new(name.as_bytes()).expect("failed to create resource name");
            res.set_debug_object_name(&debug_name)
                .expect("failed to set resource name");
        }

        if let Some(content) = desc.content {
            if mem_ty == types::MemoryType::Upload
                && desc.usage.intersects(types::BufferUsage::Copy)
            {
                let ptr = res.map::<T>(0, None).expect("failed to map resource");

                unsafe {
                    let pointer = std::slice::from_raw_parts_mut(ptr.as_ptr(), desc.size);
                    pointer.clone_from_slice(content);
                }
            }
        }

        Self {
            res,
            desc: types::BufferDesc {
                size: desc.size,
                stride: desc.stride,
                usage: desc.usage,
                mem_ty,
            },
            state,
        }
    }
}

#[derive(Debug)]
pub struct DxTexture {}

#[derive(Debug)]
pub struct DxBindGroupLayout {}

#[derive(Debug)]
pub struct DxPipelineLayout {}

#[derive(Debug)]
pub struct DxBindGroup {}

#[derive(Debug)]
pub struct DxTemporaryBindGroup {}

#[derive(Debug)]
pub struct DxRenderPipeline {}

#[derive(Debug)]
pub struct DxComputePipeline {}
*/
