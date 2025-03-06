#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Unknown,

    Rgba8Unorm,

    R32,
    Rg32,
    Rgb32,
    Rgba32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceState {
    Common,
    RenderTarget,
    Present,
}
