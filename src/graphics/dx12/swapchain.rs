use std::num::NonZero;

use oxidx::dx::{self, IDevice, IFactory4, ISwapchain1};
use parking_lot::Mutex;

use crate::graphics::core::{
    commands::SyncPoint,
    resource::{TextureDesc, TextureType, TextureUsages},
    swapchain::{RenderSwapchain, SwapchainDesc},
    types::Format,
};

use super::{
    context::DxRenderContext,
    resources::{DxTexture, TextureState},
};

#[derive(Debug)]
pub struct SwapchainFrame {
    pub texture: DxTexture,
    pub last_access: SyncPoint,
}

#[derive(Debug)]
pub struct Swapchain {
    swapchain: dx::Swapchain1,
    hwnd: NonZero<isize>,
    resources: Vec<SwapchainFrame>,
    desc: SwapchainDesc,
}

impl RenderSwapchain for DxRenderContext {
    type Swapchain = Swapchain;
    type Wnd = NonZero<isize>;

    fn create_swapchain(&self, desc: SwapchainDesc, wnd: &Self::Wnd) -> Self::Swapchain {
        let width = desc.width;
        let height = desc.height;

        let d = dx::SwapchainDesc1::new(desc.width, desc.height)
            .with_format(dx::Format::Rgba8Unorm)
            .with_usage(dx::FrameBufferUsage::RenderTargetOutput)
            .with_buffer_count(desc.frames)
            .with_scaling(dx::Scaling::None)
            .with_swap_effect(dx::SwapEffect::FlipDiscard)
            .with_flags(dx::SwapchainFlags::AllowTearing);

        let swapchain = self
            .factory
            .create_swapchain_for_hwnd(
                &*self.gfx_queue.queue.lock(),
                *wnd,
                &d,
                None,
                dx::OUTPUT_NONE,
            )
            .expect("Failed to create swapchain");

        let mut swapchain = Self::Swapchain {
            swapchain,
            hwnd: *wnd,
            resources: vec![],
            desc,
        };
        swapchain.resize(self, width, height, 0);

        swapchain
    }
}

impl Swapchain {
    pub fn resize(
        &mut self,
        ctx: &DxRenderContext,
        width: u32,
        height: u32,
        sync_point: SyncPoint,
    ) {
        {
            std::mem::take(&mut self.resources);
        }

        self.swapchain
            .resize_buffers(
                self.desc.frames,
                width,
                height,
                dx::Format::Unknown,
                dx::SwapchainFlags::AllowTearing,
            )
            .expect("Failed to resize swapchain");

        for i in 0..self.desc.frames {
            let res: dx::Resource = self
                .swapchain
                .get_buffer(i)
                .expect("Failed to get swapchain buffer");

            let descriptor = ctx.descriptors.rtv_heap.lock().alloc(1);
            ctx.gpu
                .create_render_target_view(Some(&res), None, descriptor.cpu);
            let descriptor = Some(descriptor);

            let texture = DxTexture {
                desc: TextureDesc {
                    name: None,
                    ty: TextureType::D2,
                    width,
                    height,
                    depth: 1,
                    mip_levels: 1,
                    format: Format::Rgba8Unorm,
                    usage: TextureUsages::RenderTarget,
                },
                state: TextureState::Local {
                    raw: res,
                    state: Mutex::new(dx::ResourceStates::Common),
                },
                size: 0, // TODO: Calculate
                descriptor,
            };

            self.resources.push(SwapchainFrame {
                texture,
                last_access: sync_point,
            });
        }
    }
}
