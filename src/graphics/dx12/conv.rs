use oxidx::dx;

use crate::graphics::core::{commands::CommandBufferType, resource::TextureUsages, types::Format};

pub(super) fn map_command_buffer_type(ty: CommandBufferType) -> dx::CommandListType {
    match ty {
        CommandBufferType::Graphics => dx::CommandListType::Direct,
        CommandBufferType::Compute => dx::CommandListType::Compute,
        CommandBufferType::Transfer => dx::CommandListType::Copy,
    }
}

pub(super) fn map_format(format: Format) -> dx::Format {
    match format {
        Format::Unknown => dx::Format::Unknown,
        Format::R32 => dx::Format::R32Float,
        Format::Rg32 => dx::Format::Rg32Float,
        Format::Rgb32 => dx::Format::Rgb32Float,
        Format::Rgba32 => dx::Format::Rgba32Float,
    }
}

pub(super) fn map_texture_flags(
    flags: TextureUsages,
    is_cross_adapter_texture_supported: bool,
) -> dx::ResourceFlags {
    let mut f = dx::ResourceFlags::empty();

    if flags.contains(TextureUsages::RenderTarget) && !flags.contains(TextureUsages::DepthTarget) {
        f |= dx::ResourceFlags::AllowRenderTarget;
    }

    if flags.contains(TextureUsages::DepthTarget) {
        f |= dx::ResourceFlags::AllowDepthStencil;

        if !flags.contains(TextureUsages::Resource) {
            f |= dx::ResourceFlags::DenyShaderResource;
        }
    }

    if flags.contains(TextureUsages::Storage) {
        f |= dx::ResourceFlags::AllowUnorderedAccess;
    }

    if flags.contains(TextureUsages::Shared)
        && !flags.contains(TextureUsages::DepthTarget)
        && is_cross_adapter_texture_supported
    {
        f |= dx::ResourceFlags::AllowCrossAdapter;
    }

    f
}
