use std::marker::PhantomData;

use crate::graphics::commands::{
    ComputeEncoderEnum, DynCommandBuffer, DynComputeEncoder, DynRenderEncoder, DynTransferEncoder,
    RenderEncoderEnum, TransferEncoderEnum,
};

pub struct DxCommandBuffer {}

impl DynCommandBuffer for DxCommandBuffer {
    fn render_encoder(&mut self) -> RenderEncoderEnum<'_> {
        todo!()
    }

    fn compute_encoder(&mut self) -> ComputeEncoderEnum<'_> {
        todo!()
    }

    fn transfer_encoder(&mut self) -> TransferEncoderEnum<'_> {
        todo!()
    }
}

pub struct DxRenderEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> DynRenderEncoder<'a> for DxRenderEncoder<'a> {}

pub struct DxComputeEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> DynComputeEncoder<'a> for DxComputeEncoder<'a> {}

pub struct DxTransferEncoder<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> DynTransferEncoder<'a> for DxTransferEncoder<'a> {}
