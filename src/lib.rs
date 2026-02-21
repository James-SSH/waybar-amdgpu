use amdsmi::AmdsmiTemperatureMetricT::AmdsmiTempCurrent;
use amdsmi::AmdsmiTemperatureTypeT::AmdsmiTemperatureTypeHotspot;
use amdsmi::{
    amdsmi_get_gpu_activity, amdsmi_get_gpu_board_info, amdsmi_get_gpu_memory_total, amdsmi_get_gpu_memory_usage,
    amdsmi_get_processor_handles, amdsmi_get_socket_handles, amdsmi_get_temp_metric,
    amdsmi_init, amdsmi_shut_down, AmdsmiBoardInfoT,
    AmdsmiInitFlagsT, AmdsmiMemoryTypeT, AmdsmiProcessorHandle,
};
use std::time::Duration;
use waybar_cffi::gtk::prelude::{ContainerExt, LabelExt};
use waybar_cffi::{gtk, Module};
use waybar_cffi::{waybar_module, InitInfo};

#[allow(non_upper_case_globals)]
const KiB: usize = 0x400;
#[allow(non_upper_case_globals)]
const MiB: usize = KiB * 0x400;
#[allow(non_upper_case_globals)]
const GiB: usize = MiB * 0x400;
#[allow(non_upper_case_globals)]
const TiB: usize = GiB * 0x400;

pub struct WaybarGpuModule;

pub struct AmdGPUStats {
    pub gpu_usage: u32,
    pub mem_used: usize,
    pub mem_used_percent: f32,
    pub mem_total: usize,
    pub mem_free: usize,
    pub gpu_temp: i64,
}

impl AmdGPUStats {
    pub fn from_gpu_handle(gpu_handle: AmdsmiProcessorHandle) -> Self {
        let mut stats = Self {
            gpu_usage: 0,
            mem_used: 0,
            mem_used_percent: 0.0,
            mem_total: 0,
            mem_free: 0,
            gpu_temp: 0,
        };

        stats.update_all_sensors(gpu_handle);
        stats
    }

    pub fn update_gpu_usage(&mut self, gpu_handle: AmdsmiProcessorHandle) {
        self.gpu_usage = amdsmi_get_gpu_activity(gpu_handle)
            .expect("Cannot get current usage from GPU Handle")
            .gfx_activity;
    }

    pub fn get_gpu_info(&self, gpu_handle: AmdsmiProcessorHandle) -> AmdsmiBoardInfoT {
        amdsmi_get_gpu_board_info(gpu_handle).expect("Cannot get GPU INFO from GPU Handle")
    }

    pub fn update_gpu_mem_info(&mut self, gpu_handle: AmdsmiProcessorHandle) {
        let (mem_total, mem_used) = (
            amdsmi_get_gpu_memory_total(gpu_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram)
                .expect("Could not get AMDGPU mem total"),
            amdsmi_get_gpu_memory_usage(gpu_handle, AmdsmiMemoryTypeT::AmdsmiMemTypeVram)
                .expect("Could not get AMDGPU mem usage"),
        );

        self.mem_used = mem_used as usize;
        self.mem_total = mem_total as usize;

        self.mem_used_percent = ((mem_used as f32 / mem_total as f32) * 100f32).round() / 100f32;
        self.mem_free = (mem_total - mem_used) as usize;
    }

    pub fn update_gpu_temp_info(&mut self, gpu_handle: AmdsmiProcessorHandle) {
        self.gpu_temp = amdsmi_get_temp_metric(
            gpu_handle,
            AmdsmiTemperatureTypeHotspot,
            AmdsmiTempCurrent,
        )
            .expect("Could notget AMDGPU Thermal")
    }

    pub fn update_all_sensors(&mut self, gpu_handle: AmdsmiProcessorHandle) {
        self.update_gpu_mem_info(gpu_handle);
        self.update_gpu_usage(gpu_handle);
        self.update_gpu_temp_info(gpu_handle);
    }

    pub fn build_label_string(&self, format_string: &str) -> String {
        format_string
            .replace("{gpu_usage_percent}", self.gpu_usage.to_string().as_ref())
            .replace("{gpu_mem_total}", format_iec(self.mem_total as f64).as_str())
            .replace("{gpu_mem_used}", format_iec(self.mem_used as f64).as_str())
            .replace(
                "{gpu_mem_used_percent}",
                self.mem_used_percent.to_string().as_str(),
            )
            .replace("{gpu_mem_free}", format_iec(self.mem_free as f64).as_str())
            .replace("{gpu_usage}", self.gpu_usage.to_string().as_str())
            .replace("{gpu_temp}", self.gpu_temp.to_string().as_str())
    }
}

fn format_iec(value: f64) -> String {
    if value < KiB as f64 {
        format!("{value}B")
    } else if value < MiB as f64 {
        format!("{:.2}KiB", value / KiB as f64)
    } else if value < GiB as f64 {
        format!("{:.2}MiB", value / MiB as f64)
    } else if value < TiB as f64 {
        format!("{:.2}GiB", value / GiB as f64)
    } else {
        format!("{:.2}TiB", value / TiB as f64)
    }
}

#[must_use]
unsafe fn init_gpu(gpu_idx: usize) -> AmdsmiProcessorHandle {
    amdsmi_init(AmdsmiInitFlagsT::AmdsmiInitAmdGpus).expect("Unable to initialise AMDGPU"); // AMDSMI_INIT_AMD_GPUS
    let amdgpu_handles = match amdsmi_get_socket_handles() {
        Ok(handles) => {
            let mut processor_handles = Vec::new();
            handles
                .into_iter()
                .filter_map(|h| {
                    Some(
                        amdsmi_get_processor_handles(h)
                            .expect("Cannot get Processor Handle from AMDGPU"),
                    )
                })
                .for_each(|phs| processor_handles.extend(phs));

            processor_handles
        }
        Err(e) => {
            eprintln!("Failed to get socket handles: {}", e);
            amdsmi_shut_down().expect("Failed to shutdown AMD SMI");
            panic!()
        }
    };

    if amdgpu_handles.is_empty() || gpu_idx > amdgpu_handles.len() {
        panic!("No GPU found at index {}. Available: {}", gpu_idx, amdgpu_handles.len());
    }

    amdgpu_handles[gpu_idx]
}

impl Module for WaybarGpuModule {
    type Config = Config;

    fn init(info: &InitInfo, config: Self::Config) -> Self {
        #[cfg(debug_assertions)]
        eprintln!("\n\n\n{:?}\n\n\n\n", config);

        let gpu_handle = unsafe { init_gpu(config.gpu_idx.unwrap_or(0)) };

        let mut gpu_stats = AmdGPUStats::from_gpu_handle(gpu_handle);

        let cont = info.get_root_widget();
        let label = gtk::Label::builder()
            .name("gpu")
            .label(gpu_stats.build_label_string(&config.format.clone().unwrap_or("{gpu_usage_percent}%".to_string())))
            .build();
        cont.add(&label);

        let label_clone = label.clone();
        let format_string = config.format.unwrap_or("{gpu_usage_percent}%".to_string());

        gtk::glib::timeout_add_local(
            Duration::from_millis((config.interval.unwrap_or(1f32) * 1000f32) as u64),
            move || {
                gpu_stats.update_all_sensors(gpu_handle);
                let output = gpu_stats.build_label_string(&format_string);
                label_clone.set_label(&output);
                gtk::glib::ControlFlow::Continue
            }
        );

        Self
    }
}

impl Drop for WaybarGpuModule {
    fn drop(&mut self) {
        let _ = amdsmi_shut_down();
    }
}

waybar_module!(WaybarGpuModule);

/// Format Args:
///     <br>&emsp;{gpu_usage_percent}
///     <br>&emsp;{gpu_mem_total}
///     <br>&emsp;{gpu_mem_used}
///     <br>&emsp;{gpu_mem_used_percent}
///     <br>&emsp;{gpu_mem_free}
///     <br>&emsp;{gpu_usage}
///     <br>&emsp;{gpu_temp}
#[derive(serde::Deserialize, Debug)]
pub struct Config {
    format: Option<String>,
    gpu_idx: Option<usize>,
    interval: Option<f32>
}
