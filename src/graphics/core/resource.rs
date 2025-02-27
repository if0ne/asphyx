use super::handle::RenderHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buffer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Texture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampler;

pub trait DynResourceDevice {
    fn create_buffer(&self, handle: RenderHandle<Buffer>, desc: ());
    fn destroy_buffer(&self, handle: RenderHandle<Buffer>);

    fn create_texture(&self, handle: RenderHandle<Texture>, desc: ());
    fn destroy_texture(&self, handle: RenderHandle<Texture>);

    fn create_sampler(&self, handle: RenderHandle<Sampler>, desc: ());
    fn destroy_sampler(&self, handle: RenderHandle<Sampler>);
}
