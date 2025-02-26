use std::sync::Arc;

use bytemuck::Pod;

use crate::allocators::{Handle, UntypedHandle};

use super::types::*;

pub trait Api: Sized {
    type Device: Device<Self>;
    type CommandQueue: CommandQueue<Self>;
    type CommandBuffer;

    type Buffer;
    type Image;
    type BindGroupLayout;
    type PipelineLayout;
    type BindGroup;
    type TemporaryBindGroup;
    type RenderPipeline;
    type ComputePipeline;

    fn get_all_devices<'a>(&'a self) -> impl Iterator<Item = &'a Self::Device> + 'a;
    fn get_device(&self, index: usize) -> Arc<Self::Device>;
}

pub trait Device<A: Api> {
    fn get_backend(&self) -> RenderBackend;
    fn get_device_id(&self) -> usize;

    fn create_command_queue(
        self: &Arc<Self>,
        ty: CommandQueueType,
        cb_count: Option<usize>,
    ) -> Arc<A::CommandQueue>;

    fn create_buffer<T: Pod>(&self, desc: &CreateBufferInfo<T>) -> Handle<A::Buffer>;

    fn destroy_buffer(&self, buffer: Handle<A::Buffer>);

    fn create_image(&self, desc: &CreateImageInfo) -> Handle<A::Image>;

    fn destroy_image(&self, image: Handle<A::Image>);

    fn create_image_view(
        &self,
        image: Handle<A::Image>,
        desc: CreateImageViewInfo,
    ) -> Handle<A::Image>;

    fn create_bind_group_layout(&self, desc: BindGroupLayoutDesc) -> Handle<A::BindGroupLayout>;

    fn destroy_bind_group_layout(&self, handle: Handle<A::BindGroupLayout>);

    fn create_pipeline_layout(&self, desc: PipelineLayoutDesc<A>) -> Handle<A::PipelineLayout>;

    fn destroy_pipeline_layout(&self, handle: Handle<A::PipelineLayout>);

    fn create_bind_group(&self, desc: BindGroupDesc<A>) -> Handle<A::BindGroup>;

    fn destroy_bind_group(&self, handle: Handle<A::BindGroup>);

    fn create_temp_bind_group(&self, desc: BindGroupDesc<A>) -> A::TemporaryBindGroup;

    fn create_render_pipeline(&self, desc: &RenderPipelineDesc) -> Handle<A::RenderPipeline>;

    fn destroy_render_pipeline(&self, handle: Handle<A::RenderPipeline>);

    fn create_compute_pipeline(&self, desc: &ComputePipelineDesc) -> Handle<A::ComputePipeline>;

    fn destroy_compute_pipeline(&self, handle: Handle<A::ComputePipeline>);
}

pub trait DynApi {}

pub trait DynDevice {
    fn create_buffer(&self, desc: &CreateBufferInfo) -> UntypedHandle;
    fn create_image(&self, desc: &CreateImageInfo) -> UntypedHandle;
    fn get_backend(&self) -> RenderBackend;
    fn get_device_id(&self) -> usize;
}

pub trait CommandQueue<A: Api> {
    fn create_command_buffer(&self) -> A::CommandBuffer;
    fn stash_cmd_buffer(&self, cmd_buffer: A::CommandBuffer);
    fn push_cmd_buffer(&self, cmd_buffer: A::CommandBuffer);
    fn commit(&self) -> SyncPoint;
}
