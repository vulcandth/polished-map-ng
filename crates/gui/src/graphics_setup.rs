//! Native renderer policy; November 8 tester report on Intel macOS/Metal.
//! LGPL-3.0-only. Keep every compiled backend and the normal device limits.
use eframe::{WgpuConfiguration, egui_wgpu::WgpuSetup, wgpu};

pub(crate) fn configuration() -> WgpuConfiguration {
    let mut config = WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        setup.instance_descriptor.flags =
            platform_flags(setup.instance_descriptor.flags, cfg!(target_os = "macos")).with_env();
        let original = setup.device_descriptor.clone();
        setup.device_descriptor = std::sync::Arc::new(move |adapter| {
            let mut descriptor = original(adapter);
            descriptor.memory_hints = wgpu::MemoryHints::MemoryUsage;
            descriptor
        });
    }
    config
}

fn platform_flags(mut flags: wgpu::InstanceFlags, macos: bool) -> wgpu::InstanceFlags {
    if macos {
        // Older Intel Metal drivers fail to compile wgpu's indirect-validation
        // compute pipeline during device creation. egui-wgpu uses direct
        // draw_indexed calls; this application supplies no GPU callbacks or
        // indirect commands. Retain all other checks and allow an explicit
        // WGPU_VALIDATION_INDIRECT_CALL=1 override for diagnosis.
        flags.remove(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL);
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_workaround_preserves_every_unrelated_validation_flag() {
        let flags = wgpu::InstanceFlags::all();
        assert_eq!(platform_flags(flags, false), flags);
        assert_eq!(
            platform_flags(flags, true),
            flags.difference(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL)
        );
        let disabled = flags.difference(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL);
        assert_eq!(platform_flags(disabled, true), disabled);
    }
}
