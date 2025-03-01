use enum_dispatch::enum_dispatch;

#[cfg(target_os = "windows")]
use super::dx12::commands::{
    DxCommandBuffer, DxComputeEncoder, DxRenderEncoder, DxTransferEncoder,
};

#[enum_dispatch]
pub trait DynCommandBuffer {
    fn render_encoder(&mut self) -> RenderEncoderEnum<'_>;
    fn compute_encoder(&mut self) -> ComputeEncoderEnum<'_>;
    fn transfer_encoder(&mut self) -> TransferEncoderEnum<'_>;
}

#[enum_dispatch(DynCommandBuffer)]
pub enum CommandBufferEnum {
    #[cfg(target_os = "windows")]
    DxCommandBuffer,
}

#[enum_dispatch]
pub trait DynRenderEncoder<'a> {}

#[enum_dispatch(DynRenderEncoder)]
pub enum RenderEncoderEnum<'a> {
    #[cfg(target_os = "windows")]
    DxRenderEncoder(DxRenderEncoder<'a>),
}

#[enum_dispatch]
pub trait DynComputeEncoder<'a> {}

#[enum_dispatch(DynComputeEncoder)]
pub enum ComputeEncoderEnum<'a> {
    #[cfg(target_os = "windows")]
    DxComputeEncoder(DxComputeEncoder<'a>),
}

#[enum_dispatch]
pub trait DynTransferEncoder<'a> {}

#[enum_dispatch(DynTransferEncoder)]
pub enum TransferEncoderEnum<'a> {
    #[cfg(target_os = "windows")]
    DxTransferEncoder(DxTransferEncoder<'a>),
}
