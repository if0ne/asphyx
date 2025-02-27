use super::handle::RenderHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComputePipeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderPipeline;

pub trait DynShaderDevice {
    fn create_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>, desc: ());
    fn destroy_compute_pipeline(&self, handle: RenderHandle<ComputePipeline>);

    fn create_render_pipeline(&self, handle: RenderHandle<RenderPipeline>, desc: ());
    fn destroy_render_pipeline(&self, handle: RenderHandle<RenderPipeline>);
}
