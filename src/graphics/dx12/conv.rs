use oxidx::dx;

use crate::graphics::core::commands::CommandBufferType;

impl From<CommandBufferType> for dx::CommandListType {
    fn from(value: CommandBufferType) -> Self {
        match value {
            CommandBufferType::Graphics => dx::CommandListType::Direct,
            CommandBufferType::Compute => dx::CommandListType::Compute,
            CommandBufferType::Transfer => dx::CommandListType::Copy,
        }
    }
}
