use oxidx::dx;

use crate::graphics::types::{BufferState, CommandQueueType};

impl From<CommandQueueType> for dx::CommandListType {
    #[inline]
    fn from(value: CommandQueueType) -> Self {
        match value {
            CommandQueueType::Graphics => dx::CommandListType::Direct,
            CommandQueueType::Compute => dx::CommandListType::Compute,
            CommandQueueType::Io => dx::CommandListType::Copy,
        }
    }
}

impl From<BufferState> for dx::ResourceStates {
    #[inline]
    fn from(value: BufferState) -> Self {
        match value {
            BufferState::Unknown => dx::ResourceStates::Common,
            BufferState::Generic => dx::ResourceStates::GenericRead,
            BufferState::CopyDst => dx::ResourceStates::CopyDest,
            BufferState::CopySrc => dx::ResourceStates::CopySource,
        }
    }
}
