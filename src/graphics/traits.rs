use std::sync::Arc;

use bytemuck::Pod;

use crate::allocators::Handle;

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
    fn get_device(self, index: usize) -> Arc<Self::Device>;
}

pub trait Device<A: Api> {
    fn create_command_queue(
        self: &Arc<Self>,
        ty: CommandQueueType,
        cb_count: Option<usize>,
    ) -> Arc<A::CommandQueue>;

    fn create_buffer<T: Pod>(&self, desc: CreateBufferInfo<T>) -> Handle<A::Buffer>;

    fn create_image(&self, desc: CreateImageInfo) -> Handle<A::Image>;

    fn create_image_view(
        &self,
        image: Handle<A::Image>,
        desc: CreateImageViewInfo,
    ) -> Handle<A::Image>;

    fn create_bind_group_layout(&mut self, desc: BindGroupLayoutDesc)
        -> Handle<A::BindGroupLayout>;

    fn create_pipeline_layout(&mut self, desc: PipelineLayoutDesc<A>) -> Handle<A::PipelineLayout>;

    fn create_bind_group(&mut self, desc: BindGroupDesc<A>) -> Handle<A::BindGroup>;

    fn create_temp_bind_group(&mut self, desc: BindGroupDesc<A>) -> Handle<A::TemporaryBindGroup>;

    fn create_render_pipeline_state(
        &mut self,
        desc: &RenderPipelineDesc,
    ) -> Handle<A::RenderPipeline>;

    fn create_compute_pipeline_state(
        &mut self,
        desc: &ComputePipelineDesc,
    ) -> Handle<A::ComputePipeline>;
}

pub trait CommandQueue<A: Api> {
    fn create_command_buffer(&self) -> A::CommandBuffer;
    fn stash_cmd_buffer(&self, cmd_buffer: A::CommandBuffer);
    fn push_cmd_buffer(&self, cmd_buffer: A::CommandBuffer);
    fn commit(&self) -> SyncPoint;
}
