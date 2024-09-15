use crate::util::IterAverage;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::{fs::read_to_string, sync::Mutex};
use sysinfo::System;
use serde_json::{json, Value, Map};

#[cfg(feature = "nvidia")]
use nvml_wrapper::{Nvml, enum_wrappers::device::Clock};

struct RefreshTime(std::time::Instant);
impl RefreshTime {
    pub fn new() -> Self {
        Self(std::time::Instant::now())
    }

    pub fn next_refresh(&mut self) -> std::time::Duration {
        let now = std::time::Instant::now();
        let duration = now.duration_since(self.0);
        self.0 = now;
        duration
    }
}

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
static DISKS: Lazy<Mutex<sysinfo::Disks>> = Lazy::new(|| Mutex::new(sysinfo::Disks::new_with_refreshed_list()));
static COMPONENTS: Lazy<Mutex<sysinfo::Components>> = Lazy::new(|| Mutex::new(sysinfo::Components::new_with_refreshed_list()));
static NETWORKS: Lazy<Mutex<(RefreshTime, sysinfo::Networks)>> =
    Lazy::new(|| Mutex::new((RefreshTime::new(), sysinfo::Networks::new_with_refreshed_list())));

pub fn get_disks() -> String {
    let mut disks = DISKS.lock().unwrap();
    disks.refresh_list();
    disks.refresh();

    disks
        .iter()
        .map(|c| {
            let total_space = c.total_space();
            let available_space = c.available_space();
            let used_space = total_space - available_space;

            (
                c.mount_point().display().to_string(),
                serde_json::json!({
                    "name": c.name(),
                    "total": total_space,
                    "free": available_space,
                    "used": used_space,
                    "used_perc": (used_space as f32 / total_space as f32) * 100f32
                }),
            )
        })
        .collect::<serde_json::Value>()
        .to_string()
}

pub fn get_ram() -> String {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_memory();

    let total_memory = system.total_memory();
    let available_memory = system.available_memory();
    let used_memory = total_memory as f32 - available_memory as f32;
    serde_json::json!({
        "total_mem": total_memory,
        "free_mem": system.free_memory(),
        "total_swap": system.total_swap(),
        "free_swap": system.free_swap(),
        "available_mem": available_memory,
        "used_mem": used_memory,
        "used_mem_perc": (used_memory / total_memory as f32) * 100f32,
    })
    .to_string()
}

pub fn get_temperatures() -> String {
    let mut components = COMPONENTS.lock().unwrap();
    components.refresh_list();
    components.refresh();

    // Allow unused mut because we only need it if the nvidia feature is enabled
    #[allow(unused_mut)]
    let mut temps: Map<String, Value> = components
        .iter()
        .map(|c| {(
            c.label().to_uppercase().replace(' ', "_"),
            if c.temperature().is_nan() {
                Value::Null
            } else {
                Value::from(format!("{:.1}", c.temperature()))
            },
        )})
        .collect();

    #[cfg(feature = "nvidia")]
    if let Some(gpu_temps) = get_all_nvidia_gpu_temperatures() {
        for (index, gpu_temp) in gpu_temps.into_iter().enumerate() {
            temps.insert(
                format!("NVIDIA_GPU_{}", index),
                if gpu_temp.is_nan() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::from(gpu_temp)
                },
            );
        }
    }

    serde_json::to_string(&json!(temps)).unwrap()
}

#[cfg(feature = "nvidia")]
fn get_all_nvidia_gpu_temperatures() -> Option<Vec<f64>> {
    let nvml = match Nvml::init() {
        Ok(nvml) => nvml,
        Err(e) => {
            log::error!("Are you shure you have nvidia gpu and proprietary drivers installed? \
              Failed to initialize NVML: {:?}", e);
            return None;
        }
    };

    let device_count = match nvml.device_count() {
        Ok(count) => {
            if count == 0 {
                log::warn!("NVML was initialized, but no devices were found.");
                return None;
            }
            count
        },
        Err(e) => {
            log::error!("Failed to get NVML device count: {:?}", e);
            return None;
        }
    };

    let mut gpu_temps = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            if let Ok(temp) = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                gpu_temps.push(temp as f64);
            } else {
                gpu_temps.push(f64::NAN);
            }
        }
    }

    Some(gpu_temps)
}


pub fn get_cpus() -> String {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_cpu_specifics(sysinfo::CpuRefreshKind::everything());
    let cpus = system.cpus();
    serde_json::json!({
        "cores": cpus.iter()
            .map(|a| {
                serde_json::json!({
                    "core": a.name(),
                    "freq": a.frequency(),
                    "usage": a.cpu_usage() as i64
                })
            }).collect::<Vec<_>>(),
        "avg": cpus.iter().map(|a| a.cpu_usage()).avg()
    })
    .to_string()
}

pub fn get_gpus() -> String {
    #[allow(unused_mut)]
    let mut gpus_data: Map<String, Value> = Map::new();
    #[cfg(feature = "nvidia")]
    {
        let nvml = match Nvml::init() {
            Ok(nvml) => nvml,
            Err(e) => {
                log::error!("Are you shure you have nvidia gpu and proprietary drivers installed? \
                  Failed to initialize NVML: {:?}", e);
                return "".to_string();
            }
        };
        let nvidia_device_count = match nvml.device_count() {
            Ok(count) => {
                if count == 0 {
                    log::warn!("NVML was initialized, but no devices were found.");
                    return "".to_string();
                }
                count
            },
            Err(e) => {
                log::error!("Failed to get NVML device count: {:?}", e);
                return "".to_string();
            }
        };

        let mut nvidia_gpus_load: Map<String, Value> = Map::new();
        if let Some(gpu_loads) = get_nvidia_load(&nvml, nvidia_device_count) {
            for (index, gpu_load) in gpu_loads.into_iter().enumerate() {
                nvidia_gpus_load.insert(
                    format!("NVIDIA_GPU_LOAD_{}", index),
                    serde_json::Value::from(gpu_load),
                );
            }
        }

        let mut nvidia_gpus_vram_current: Map<String, Value> = Map::new();
        if let Some(gpu_vram_current) = get_nvidia_vram_current(&nvml, nvidia_device_count) {
            for (index, vram_current) in gpu_vram_current.into_iter().enumerate() {
                nvidia_gpus_vram_current.insert(
                    format!("NVIDIA_GPU_VRAM_CURRENT_{}", index),
                    serde_json::Value::from(vram_current),
                );
            }
        }

        let mut nvidia_gpus_vram_max: Map<String, Value> = Map::new();
        if let Some(gpu_vram_max) = get_nvidia_vram_max(&nvml, nvidia_device_count) {
            for (index, vram_max) in gpu_vram_max.into_iter().enumerate() {
                nvidia_gpus_vram_max.insert(
                    format!("NVIDIA_GPU_VRAM_MAX_{}", index),
                    serde_json::Value::from(vram_max),
                );
            }
        }

        let mut nvidia_gpus_freq_graphics_current: Map<String, Value> = Map::new();
        if let Some(gpu_freq) = get_nvidia_freq_graphics_current(&nvml, nvidia_device_count) {
            for (index, freq) in gpu_freq.into_iter().enumerate() {
                nvidia_gpus_freq_graphics_current.insert(
                    format!("NVIDIA_GPU_FREQ_GRAPHICS_CURRENT_{}", index),
                    serde_json::Value::from(freq),
                );
            }
        }

        let mut nvidia_gpus_freq_graphics_max: Map<String, Value> = Map::new();
        if let Some(gpu_freq) = get_nvidia_freq_graphics_max(&nvml, nvidia_device_count) {
            for (index, freq) in gpu_freq.into_iter().enumerate() {
                nvidia_gpus_freq_graphics_max.insert(
                    format!("NVIDIA_GPU_FREQ_GRAPHICS_MAX_{}", index),
                    serde_json::Value::from(freq),
                );
            }
        }

        let mut nvidia_gpus_freq_vram_current: Map<String, Value> = Map::new();
        if let Some(gpu_freq) = get_nvidia_freq_vram_current(&nvml, nvidia_device_count) {
            for (index, freq) in gpu_freq.into_iter().enumerate() {
                nvidia_gpus_freq_vram_current.insert(
                    format!("NVIDIA_GPU_FREQ_MEMORY_CURRENT_{}", index),
                    serde_json::Value::from(freq),
                );
            }
        }

        let mut nvidia_gpus_freq_vram_max: Map<String, Value> = Map::new();
        if let Some(gpu_freq) = get_nvidia_freq_vram_max(&nvml, nvidia_device_count) {
            for (index, freq) in gpu_freq.into_iter().enumerate() {
                nvidia_gpus_freq_vram_max.insert(
                    format!("NVIDIA_GPU_FREQ_MEMORY_MAX_{}", index),
                    serde_json::Value::from(freq),
                );
            }
        }
        gpus_data.extend(nvidia_gpus_load);
        gpus_data.extend(nvidia_gpus_vram_current);
        gpus_data.extend(nvidia_gpus_vram_max);
        gpus_data.extend(nvidia_gpus_freq_graphics_current);
        gpus_data.extend(nvidia_gpus_freq_graphics_max);
        gpus_data.extend(nvidia_gpus_freq_vram_current);
        gpus_data.extend(nvidia_gpus_freq_vram_max);
    }

    serde_json::to_string(&json!(gpus_data)).unwrap()
}

pub fn get_battery_capacity() -> Result<String> {
    use std::{collections::HashMap, sync::atomic::AtomicBool};

    #[derive(serde::Serialize)]
    struct BatteryData {
        capacity: i64,
        status: String,
    }

    #[derive(serde::Serialize)]
    struct Data {
        #[serde(flatten)]
        batteries: HashMap<String, BatteryData>,
        total_avg: f64,
    }

    let mut current = 0_f64;
    let mut total = 0_f64;
    let mut batteries = HashMap::new();
    let power_supply_dir = std::path::Path::new("/sys/class/power_supply");
    let power_supply_entries = power_supply_dir.read_dir().context("Couldn't read /sys/class/power_supply directory")?;
    for entry in power_supply_entries {
        let entry = entry?.path();
        if !entry.is_dir() {
            continue;
        }
        if let (Ok(capacity), Ok(status)) = (read_to_string(entry.join("capacity")), read_to_string(entry.join("status"))) {
            batteries.insert(
                entry.file_name().context("Couldn't get filename")?.to_string_lossy().to_string(),
                BatteryData {
                    status: status.trim_end_matches('\n').to_string(),
                    capacity: capacity.trim_end_matches('\n').parse::<f64>()?.round() as i64,
                },
            );
            if let (Ok(charge_full), Ok(charge_now), Ok(voltage_now)) = (
                read_to_string(entry.join("charge_full")),
                read_to_string(entry.join("charge_now")),
                read_to_string(entry.join("voltage_now")),
            ) {
                // (uAh / 1000000) * U = p and that / one million so that we have microwatt
                current += ((charge_now.trim_end_matches('\n').parse::<f64>()? / 1000000_f64)
                    * voltage_now.trim_end_matches('\n').parse::<f64>()?)
                    / 1000000_f64;
                total += ((charge_full.trim_end_matches('\n').parse::<f64>()? / 1000000_f64)
                    * voltage_now.trim_end_matches('\n').parse::<f64>()?)
                    / 1000000_f64;
            } else if let (Ok(energy_full), Ok(energy_now)) =
                (read_to_string(entry.join("energy_full")), read_to_string(entry.join("energy_now")))
            {
                current += energy_now.trim_end_matches('\n').parse::<f64>()?;
                total += energy_full.trim_end_matches('\n').parse::<f64>()?;
            } else {
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.load(std::sync::atomic::Ordering::Relaxed) {
                    WARNED.store(true, std::sync::atomic::Ordering::Relaxed);
                    log::warn!(
                        "Failed to get/calculate uWh: the total_avg value of the battery magic var will probably be a garbage \
                         value that can not be trusted."
                    );
                }
            }
        }
    }
    if total == 0_f64 {
        return Ok(String::from(""));
    }

    Ok(serde_json::to_string(&(Data { batteries, total_avg: (current / total) * 100_f64 })).unwrap())
}

pub fn net() -> String {
    let (ref mut last_refresh, ref mut networks) = &mut *NETWORKS.lock().unwrap();

    networks.refresh_list();
    let elapsed = last_refresh.next_refresh();

    networks
        .iter()
        .map(|(name, data)| {
            let transmitted = data.transmitted() as f64 / elapsed.as_secs_f64();
            let received = data.received() as f64 / elapsed.as_secs_f64();
            (name, serde_json::json!({ "NET_UP": transmitted, "NET_DOWN": received }))
        })
        .collect::<serde_json::Value>()
        .to_string()
}

pub fn get_time() -> String {
    chrono::offset::Utc::now().timestamp().to_string()
}

#[cfg(feature = "nvidia")]
pub fn get_nvidia_load(nvml: &Nvml, device_count: u32) -> Option<Vec<u32>> {
    let mut gpu_loads = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            match device.utilization_rates() {
                Ok(util) => {
                    gpu_loads.push(util.gpu);
                }
                Err(e) => {
                    log::warn!("Failed to get Nvidia GPU utilization: {:?}", e);
                    return None;
                }
            }
        }
    }
    Some(gpu_loads)
}

#[cfg(feature = "nvidia")]
fn get_nvidia_vram_current(nvml: &Nvml, device_count: u32) -> Option<Vec<u64>> {
    let mut gpu_vram_current = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            match device.memory_info() {
                Ok(mem) => {
                    gpu_vram_current.push(mem.used);
                }
                Err(e) => {
                    log::warn!("Failed to get Nvidia GPU current memory info: {:?}", e);
                    return None;
                }
            }
        }
    }
    Some(gpu_vram_current)
}

#[cfg(feature = "nvidia")]
fn get_nvidia_vram_max(nvml: &Nvml, device_count: u32) -> Option<Vec<u64>> {
    let mut gpu_vram_max = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            match device.memory_info() {
                Ok(mem) => {
                    gpu_vram_max.push(mem.total);
                }
                Err(e) => {
                    log::warn!("Failed to get Nvidia GPU max memory info: {:?}", e);
                    return None;
                }
            }
        }
    }
    Some(gpu_vram_max)
}

#[cfg(feature = "nvidia")]
fn get_nvidia_freq_graphics_current(nvml: &Nvml, device_count: u32) -> Option<Vec<u32>> {
    let mut gpu_freq = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            match device.clock_info(Clock::Graphics) {
                Ok(clock) => {
                    gpu_freq.push(clock);
                }
                Err(e) => {
                    log::warn!("Failed to get Nvidia GPU current clock info: {:?}", e);
                    return None;
                }
            }
        }
    }
    Some(gpu_freq)
}

#[cfg(feature = "nvidia")]
fn get_nvidia_freq_graphics_max(nvml: &Nvml, device_count: u32) -> Option<Vec<u32>> {
    let mut gpu_freq = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            match device.max_clock_info(Clock::Graphics) {
                Ok(clock) => {
                    gpu_freq.push(clock);
                }
                Err(e) => {
                    log::warn!("Failed to get Nvidia GPU max clock info: {:?}", e);
                    return None;
                }
            }
        }
    }
    Some(gpu_freq)
}

#[cfg(feature = "nvidia")]
fn get_nvidia_freq_vram_current(nvml: &Nvml, device_count: u32) -> Option<Vec<u32>> {
    let mut gpu_freq = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            match device.clock_info(Clock::Memory) {
                Ok(clock) => {
                    gpu_freq.push(clock);
                }
                Err(e) => {
                    log::warn!("Failed to get Nvidia VRAM current clock info: {:?}", e);
                    return None;
                }
            }
        }
    }
    Some(gpu_freq)
}

#[cfg(feature = "nvidia")]
fn get_nvidia_freq_vram_max(nvml: &Nvml, device_count: u32) -> Option<Vec<u32>> {
    let mut gpu_freq = Vec::new();
    for i in 0..device_count {
        if let Ok(device) = nvml.device_by_index(i) {
            match device.max_clock_info(Clock::Memory) {
                Ok(clock) => {
                    gpu_freq.push(clock);
                }
                Err(e) => {
                    log::warn!("Failed to get Nvidia VRAM max clock info: {:?}", e);
                    return None;
                }
            }
        }
    }
    Some(gpu_freq)
}
