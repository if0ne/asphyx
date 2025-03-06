#[derive(Clone, Debug)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
}

#[derive(Clone, Debug)]
pub struct SwapchainDesc {
    pub width: u32,
    pub height: u32,
    pub present_mode: PresentMode,
    pub frames: usize,
}

pub trait RenderSwapchain {
    type Swapchain;
    type Wnd;

    fn create_swapchain(&self, desc: SwapchainDesc, wnd: &Self::Wnd) -> Self::Swapchain;
}
