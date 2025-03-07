use std::{cmp::Reverse, collections::BinaryHeap, ops::Range};

use oxidx::dx::{self, IDescriptorHeap, IDevice};
use parking_lot::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
struct InnerRange(Range<usize>);

impl InnerRange {
    fn is_adjacent_to(&self, other: &Self) -> bool {
        self.0.end == other.0.start || other.0.end == self.0.start
    }

    fn merge(self, other: Self) -> Self {
        let start = self.0.start.min(other.0.start);
        let end = self.0.end.max(other.0.end);
        InnerRange(start..end)
    }
}

impl Ord for InnerRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.len().cmp(&other.0.len())
    }
}

impl PartialOrd for InnerRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub(crate) struct Descriptor {
    pub(super) ty: dx::DescriptorHeapType,
    pub(super) cpu: dx::CpuDescriptorHandle,
    pub(super) gpu: dx::GpuDescriptorHandle,
    heap_index: InnerRange,
}

#[derive(Debug)]
pub(crate) struct Descriptors {
    pub(crate) rtv_heap: Mutex<DescriptorHeap>,
    pub(crate) dsv_heap: Mutex<DescriptorHeap>,
    pub(crate) shader_heap: Mutex<DescriptorHeap>,
    pub(crate) sampler_heap: Mutex<DescriptorHeap>,
}

impl Descriptors {
    pub fn new(device: &dx::Device) -> Self {
        let rtv_heap = DescriptorHeap::new(&device, dx::DescriptorHeapType::Rtv, 128);
        let dsv_heap = DescriptorHeap::new(&device, dx::DescriptorHeapType::Dsv, 128);
        let shader_heap = DescriptorHeap::new(&device, dx::DescriptorHeapType::CbvSrvUav, 1024);
        let sampler_heap = DescriptorHeap::new(&device, dx::DescriptorHeapType::Sampler, 32);

        Self {
            rtv_heap: Mutex::new(rtv_heap),
            dsv_heap: Mutex::new(dsv_heap),
            shader_heap: Mutex::new(shader_heap),
            sampler_heap: Mutex::new(sampler_heap),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DescriptorHeap {
    pub(crate) heap: dx::DescriptorHeap,
    pub(crate) ty: dx::DescriptorHeapType,
    pub(crate) size: usize,
    pub(crate) inc_size: usize,
    free_ranges: BinaryHeap<Reverse<InnerRange>>,
}

impl DescriptorHeap {
    pub(crate) fn new(device: &dx::Device, ty: dx::DescriptorHeapType, size: usize) -> Self {
        let (_, flags) =
            if ty == dx::DescriptorHeapType::CbvSrvUav || ty == dx::DescriptorHeapType::Sampler {
                (true, dx::DescriptorHeapFlags::ShaderVisible)
            } else {
                (false, dx::DescriptorHeapFlags::empty())
            };

        let inc_size = device.get_descriptor_handle_increment_size(ty);

        let heap = device
            .create_descriptor_heap(&dx::DescriptorHeapDesc::new(ty, size).with_flags(flags))
            .expect("Failed to create descriptor heap");

        let mut free_ranges = BinaryHeap::new();
        free_ranges.push(Reverse(InnerRange(0..size)));

        Self {
            heap,
            ty,
            size,
            inc_size,
            free_ranges,
        }
    }

    pub(super) fn alloc(&mut self, size: usize) -> Descriptor {
        let mut temp = Vec::new();
        let mut allocated = None;

        while let Some(Reverse(range)) = self.free_ranges.pop() {
            if range.0.len() >= size {
                let allocated_start = range.0.start;
                let allocated_end = range.0.start + size;

                if allocated_end < range.0.end {
                    let remaining = Range {
                        start: allocated_end,
                        end: range.0.end,
                    };
                    self.free_ranges.push(Reverse(InnerRange(remaining)));
                }

                allocated = Some(Range {
                    start: allocated_start,
                    end: allocated_end,
                });
                break;
            } else {
                temp.push(range);
            }
        }

        for range in temp {
            self.free_ranges.push(Reverse(range));
        }

        let allocated = allocated.expect("Out of memory");

        let cpu = self
            .heap
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(allocated.start, self.inc_size);
        let gpu = self
            .heap
            .get_gpu_descriptor_handle_for_heap_start()
            .advance(allocated.start, self.inc_size);

        Descriptor {
            ty: self.ty,
            cpu,
            gpu,
            heap_index: InnerRange(allocated),
        }
    }

    pub(super) fn free(&mut self, descriptor: Descriptor) {
        let mut merged = descriptor.heap_index;
        let mut temp = Vec::new();

        while let Some(Reverse(current)) = self.free_ranges.pop() {
            if merged.is_adjacent_to(&current) {
                merged = merged.merge(current);
            } else {
                temp.push(current);
            }
        }

        self.free_ranges.push(Reverse(merged));

        for current in temp {
            self.free_ranges.push(Reverse(current));
        }
    }
}
