use oxidx::dx::{
    self,
    features::{Architecture1Feature, OptionsFeature},
    IAdapter3, IDebug, IDebug1, IDebugExt, IDevice, IFactory4, IFactory6,
};
use tracing::{debug, error, info, warn};

use crate::graphics::{
    core::backend::{Api, DeviceType, RenderDeviceId, RenderDeviceInfo},
    DebugFlags,
};

use super::context::DxRenderContext;

#[derive(Debug)]
pub struct DxBackend {
    factory: dx::Factory4,
    debug: Option<dx::Debug1>,

    adapters: Vec<dx::Adapter3>,
    adapter_infos: Vec<RenderDeviceInfo>,
}

impl DxBackend {
    pub fn new(debug_flags: DebugFlags) -> Self {
        let flags = if !debug_flags.is_empty() {
            dx::FactoryCreationFlags::Debug
        } else {
            dx::FactoryCreationFlags::empty()
        };

        let factory = dx::create_factory4(flags).expect("failed to create DXGI factory");

        let debug = if !debug_flags.is_empty() {
            let debug: dx::Debug1 = dx::create_debug()
                .expect("failed to create debug")
                .try_into()
                .expect("failed to fetch debug1");

            debug.enable_debug_layer();
            debug.set_enable_gpu_based_validation(true);
            debug.set_callback(Box::new(|_, severity, _, msg| match severity {
                dx::MessageSeverity::Corruption => error!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Error => error!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Warning => warn!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Info => info!("[D3D12 Validation] {}", msg),
                dx::MessageSeverity::Message => debug!("[D3D12 Validation] {}", msg),
            }));

            Some(debug)
        } else {
            None
        };

        let mut gpus = vec![];

        if let Ok(factory) = TryInto::<dx::Factory7>::try_into(factory.clone()) {
            debug!("Factory7 is supported");

            let mut i = 0;

            while let Ok(adapter) =
                factory.enum_adapters_by_gpu_preference(i, dx::GpuPreference::HighPerformance)
            {
                let Ok(desc) = adapter.get_desc1() else {
                    i += 1;
                    continue;
                };

                if let Ok(device) = dx::create_device(Some(&adapter), dx::FeatureLevel::Level11) {
                    let mut feature = OptionsFeature::default();

                    device
                        .check_feature_support(&mut feature)
                        .expect("failed to check options");

                    let mut hardware = Architecture1Feature::new(0);

                    device
                        .check_feature_support(&mut hardware)
                        .expect("failed to check options");

                    let ty = if desc.flags().contains(dx::AdapterFlags::Sofware) {
                        DeviceType::Cpu
                    } else if hardware.uma() {
                        DeviceType::Integrated
                    } else {
                        DeviceType::Discrete
                    };

                    gpus.push((
                        adapter,
                        RenderDeviceInfo {
                            name: desc.description().trim_matches('\0').to_string(),
                            id: i as RenderDeviceId,
                            is_cross_adapter_texture_supported: feature
                                .cross_adapter_row_major_texture_supported(),
                            is_uma: hardware.uma(),
                            ty,
                        },
                    ));
                }

                i += 1;
            }
        } else {
            let mut i = 0;
            while let Ok(adapter) = factory.enum_adapters(i) {
                let Ok(desc) = adapter.get_desc1() else {
                    i += 1;
                    continue;
                };

                if let Ok(device) = dx::create_device(Some(&adapter), dx::FeatureLevel::Level11) {
                    let mut feature = OptionsFeature::default();

                    device
                        .check_feature_support(&mut feature)
                        .expect("failed to check options");

                    let mut hardware = Architecture1Feature::new(0);

                    device
                        .check_feature_support(&mut hardware)
                        .expect("failed to check options");

                    let ty = if desc.flags().contains(dx::AdapterFlags::Sofware) {
                        DeviceType::Cpu
                    } else if hardware.uma() {
                        DeviceType::Integrated
                    } else {
                        DeviceType::Discrete
                    };

                    gpus.push((
                        adapter,
                        RenderDeviceInfo {
                            name: desc.description().trim_matches('\0').to_string(),
                            id: i,
                            is_cross_adapter_texture_supported: feature
                                .cross_adapter_row_major_texture_supported(),
                            is_uma: hardware.uma(),
                            ty,
                        },
                    ));
                }

                i += 1;
            }

            gpus.sort_by(|(l, _), (r, _)| {
                let descs = (
                    l.get_desc1().map(|d| d.vendor_id()),
                    r.get_desc1().map(|d| d.vendor_id()),
                );

                match descs {
                    (Ok(0x8086), Ok(0x8086)) => std::cmp::Ordering::Equal,
                    (Ok(0x8086), Ok(_)) => std::cmp::Ordering::Less,
                    (Ok(_), Ok(0x8086)) => std::cmp::Ordering::Greater,
                    (_, _) => std::cmp::Ordering::Equal,
                }
            });
        }

        let (adapters, adapter_infos): (Vec<_>, Vec<_>) = gpus.into_iter().unzip();

        adapter_infos
            .iter()
            .for_each(|a| info!("Found adapter: {:?}", a));

        Self {
            factory,
            debug,
            adapters,
            adapter_infos,
        }
    }
}

impl Api for DxBackend {
    type Device = DxRenderContext;

    fn enumerate_devices<'a>(&'a self) -> impl Iterator<Item = &'a RenderDeviceInfo> + 'a {
        self.adapter_infos.iter()
    }

    fn create_device(&self, index: RenderDeviceId) -> Self::Device {
        DxRenderContext::new(
            self.adapters[index].clone(),
            self.factory.clone(),
            self.adapter_infos[index].clone(),
        )
    }
}
