use std::sync::atomic::AtomicU64;

use oxidx::dx::{self, IDevice, IFence};

#[derive(Debug)]
pub struct DxFence {
    pub(super) fence: dx::Fence,
    value: AtomicU64,
}

impl DxFence {
    pub(super) fn new(device: &dx::Device) -> Self {
        let fence = device
            .create_fence(0, dx::FenceFlags::empty())
            .expect("failed to create fence");

        Self {
            fence,
            value: Default::default(),
        }
    }

    pub fn wait(&self, value: u64) -> bool {
        if self.get_completed_value() < value {
            let event = dx::Event::create(false, false).expect("failed to create event");
            self.fence
                .set_event_on_completion(value, event)
                .expect("failed to bind fence to event");
            if event.wait(10_000_000) == 0x00000102 {
                panic!("device lost")
            }

            true
        } else {
            false
        }
    }

    pub fn inc_value(&self) -> u64 {
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    pub fn get_completed_value(&self) -> u64 {
        self.fence.get_completed_value()
    }

    pub fn get_current_value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}
