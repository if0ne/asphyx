use bytemuck::Pod;
use oxidx::dx::{self, IDevice, IResource};

use crate::graphics::core::resource::{
    BufferDesc, BufferUsages, ResourceDevice, SamplerDesc, TextureDesc, TextureType, TextureUsages,
    TextureViewDesc,
};

use super::{
    context::DxRenderContext,
    conv::{map_format, map_texture_flags},
};

impl ResourceDevice for DxRenderContext {
    type Buffer = DxBuffer;
    type Texture = DxTexture;
    type Sampler = ();

    fn create_buffer<T: Pod>(&self, desc: BufferDesc, init_data: Option<&[T]>) -> Self::Buffer {
        DxBuffer::new(self, desc)
    }

    fn destroy_buffer(&self, _buffer: Self::Buffer) {}

    fn create_texture<T: Pod>(&self, desc: TextureDesc, init_data: Option<&[T]>) -> Self::Texture {
        DxTexture::new(self, desc)
    }

    fn destroy_texture(&self, _texture: Self::Texture) {}

    fn open_texture(&self, texture: &Self::Texture, other: &Self) -> Self::Texture {
        let (heap, desc) = match &texture.state {
            TextureState::Local { .. } => panic!("Texture is local, can not open handle"),
            TextureState::CrossAdapter { heap, cross } => (heap, cross.get_desc()),
            TextureState::Binded { heap, cross, .. } => (heap, cross.get_desc()),
        };

        let handle = other
            .gpu
            .create_shared_handle(heap, None)
            .expect("Failed to open handle");
        let open_heap: dx::Heap = self
            .gpu
            .open_shared_handle(handle)
            .expect("Failed to open heap");
        handle.close().expect("Failed to close handle");

        let flags = map_texture_flags(
            texture.desc.usage,
            self.desc.is_cross_adapter_texture_supported,
        );

        let cross_desc = if flags.contains(dx::ResourceFlags::AllowCrossAdapter) {
            desc
        } else {
            desc.with_flags(dx::ResourceFlags::AllowCrossAdapter)
        };

        let cross_res = self
            .gpu
            .create_placed_resource(&open_heap, 0, &cross_desc, dx::ResourceStates::Common, None)
            .expect("Failed to create cross texture");

        if flags.contains(dx::ResourceFlags::AllowCrossAdapter) {
            DxTexture {
                desc: texture.desc.clone(),
                state: TextureState::CrossAdapter {
                    heap: open_heap,
                    cross: cross_res,
                },
            }
        } else {
            let d = match texture.desc.ty {
                TextureType::D1 => dx::ResourceDesc::texture_1d(texture.desc.width)
                    .with_array_size(texture.desc.depth),
                TextureType::D2 => {
                    dx::ResourceDesc::texture_2d(texture.desc.width, texture.desc.height)
                        .with_array_size(texture.desc.depth)
                }
                TextureType::D3 => dx::ResourceDesc::texture_3d(
                    texture.desc.width,
                    texture.desc.height,
                    texture.desc.depth,
                ),
            };

            let d = d
                .with_alignment(dx::HeapAlignment::ResourcePlacement)
                .with_format(map_format(texture.desc.format))
                .with_mip_levels(texture.desc.mip_levels)
                .with_layout(dx::TextureLayout::Unknown)
                .with_flags(flags);

            let local_res = self
                .gpu
                .create_committed_resource(
                    &dx::HeapProperties::default(),
                    dx::HeapFlags::empty(),
                    &d,
                    dx::ResourceStates::Common,
                    None,
                )
                .expect("failed to create texture");

            DxTexture {
                desc: texture.desc.clone(),
                state: TextureState::Binded {
                    heap: open_heap,
                    cross: cross_res,
                    local: local_res,
                },
            }
        }
    }

    fn create_texture_view(&self, texture: &Self::Texture, desc: TextureViewDesc) -> Self::Texture {
        todo!()
    }

    fn create_sampler(&self, desc: SamplerDesc) -> Self::Sampler {
        todo!()
    }

    fn destroy_sampler(&self, buffer: Self::Sampler) {
        todo!()
    }
}

#[derive(Debug)]
pub struct DxBuffer {
    pub(super) raw: dx::Resource,
    pub(super) desc: BufferDesc,
}

impl DxBuffer {
    fn new(device: &DxRenderContext, desc: BufferDesc) -> Self {
        let heap_props = if desc.usage.contains(BufferUsages::Uniform)
            | desc.usage.contains(BufferUsages::Copy)
        {
            dx::HeapProperties::upload()
        } else if desc.usage.contains(BufferUsages::QueryResolve) {
            dx::HeapProperties::readback()
        } else {
            dx::HeapProperties::default()
        };

        let d = dx::ResourceDesc::buffer(desc.size).with_layout(dx::TextureLayout::RowMajor);

        let initial_state = if desc.usage.contains(BufferUsages::Uniform)
            | desc.usage.contains(BufferUsages::Copy)
        {
            dx::ResourceStates::GenericRead
        } else if desc.usage.contains(BufferUsages::QueryResolve) {
            dx::ResourceStates::CopyDest
        } else {
            dx::ResourceStates::Common
        };

        let raw = device
            .gpu
            .create_committed_resource(&heap_props, dx::HeapFlags::empty(), &d, initial_state, None)
            .expect("Failed to create buffer");

        Self { raw, desc }
    }
}

#[derive(Debug)]
pub struct DxTexture {
    pub(super) desc: TextureDesc,
    pub(super) state: TextureState,
}

impl DxTexture {
    fn new(device: &DxRenderContext, desc: TextureDesc) -> Self {
        let d = match desc.ty {
            TextureType::D1 => dx::ResourceDesc::texture_1d(desc.width).with_array_size(desc.depth),
            TextureType::D2 => {
                dx::ResourceDesc::texture_2d(desc.width, desc.height).with_array_size(desc.depth)
            }
            TextureType::D3 => dx::ResourceDesc::texture_3d(desc.width, desc.height, desc.depth),
        };

        let d = d
            .with_alignment(dx::HeapAlignment::ResourcePlacement)
            .with_format(map_format(desc.format))
            .with_mip_levels(desc.mip_levels)
            .with_layout(dx::TextureLayout::Unknown)
            .with_flags(map_texture_flags(
                desc.usage,
                device.desc.is_cross_adapter_texture_supported,
            ));

        if desc.usage.contains(TextureUsages::Shared) {
            let cross_desc = if d.flags().contains(dx::ResourceFlags::AllowCrossAdapter) {
                d.clone().with_layout(dx::TextureLayout::RowMajor)
            } else {
                d.clone()
                    .with_flags(dx::ResourceFlags::AllowCrossAdapter)
                    .with_layout(dx::TextureLayout::RowMajor)
            };

            let size = device
                .gpu
                .get_copyable_footprints(&cross_desc, 0..1, 0, None, None, None)
                * 2; // FIX: Textures of arbitrary size require more memory than get_copyable_footprints returns

            let heap = device
                .gpu
                .create_heap(
                    &dx::HeapDesc::new(size, dx::HeapProperties::default())
                        .with_flags(dx::HeapFlags::SharedCrossAdapter | dx::HeapFlags::Shared),
                )
                .expect("Failed to create shared heap");

            let cross_res = device
                .gpu
                .create_placed_resource(&heap, 0, &cross_desc, dx::ResourceStates::Common, None)
                .expect("failed to create cross texture");

            if d.flags().contains(dx::ResourceFlags::AllowCrossAdapter) {
                Self {
                    desc,
                    state: TextureState::CrossAdapter {
                        heap,
                        cross: cross_res,
                    },
                }
            } else {
                let local_res = device
                    .gpu
                    .create_committed_resource(
                        &dx::HeapProperties::default(),
                        dx::HeapFlags::empty(),
                        &d,
                        dx::ResourceStates::Common,
                        None,
                    )
                    .expect("failed to create texture");

                Self {
                    desc,
                    state: TextureState::Binded {
                        heap,
                        cross: cross_res,
                        local: local_res,
                    },
                }
            }
        } else {
            let raw = device
                .gpu
                .create_committed_resource(
                    &dx::HeapProperties::default(),
                    dx::HeapFlags::empty(),
                    &d,
                    dx::ResourceStates::Common,
                    None,
                )
                .expect("failed to create texture");

            Self {
                desc,
                state: TextureState::Local { raw },
            }
        }
    }
}

#[derive(Debug)]
pub enum TextureState {
    Local {
        raw: dx::Resource,
    },
    CrossAdapter {
        heap: dx::Heap,
        cross: dx::Resource,
    },
    Binded {
        heap: dx::Heap,
        cross: dx::Resource,
        local: dx::Resource,
    },
}
