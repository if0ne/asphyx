use std::sync::Arc;

use enum_dispatch::enum_dispatch;

use dx12::{backend::DxBackend, context::DxRenderContext};

pub mod backend;
pub mod commands;
pub mod context;
pub mod core;

#[cfg(target_os = "windows")]
mod dx12;

mod mock;

#[enum_dispatch(RenderContext)]
pub enum RenderContextEnum {
    #[cfg(target_os = "windows")]
    DxRenderContext(Arc<DxRenderContext>),
}

#[enum_dispatch(DynApi)]
pub enum ApiEnum {
    DxBackend,
}
