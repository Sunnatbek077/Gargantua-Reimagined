// =============================================================================
// crates/gargantua-core/src/platform/windows/gpu/vendor.rs
// =============================================================================
//
// PURPOSE:
//   Identifies the GPU vendor (NVIDIA, AMD, Intel) from the wgpu Adapter
//   and provides vendor-specific capability flags used by workgroup.rs,
//   quality presets, and video encoder selection.
//
//   Also detects the GPU architecture generation (e.g., NVIDIA Ampere vs
//   Ada Lovelace, AMD RDNA2 vs RDNA3) to select the correct quality tier.
//
// SIZE: ~200 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CoreError
//   External:
//     - wgpu::{Adapter, AdapterInfo, Backend}
//
// CALLED BY:
//   - crate::gpu::context::GpuContext::new()
//       — calls detect_vendor() after adapter creation
//   - crate::platform::windows::compute::workgroup::WorkgroupConfig::for_vendor()
//   - crate::platform::windows::quality::{nvidia_presets, amd_presets, intel_presets}
//   - crate::platform::windows::video::{nvenc, amf, qsv}
//       — each checks GpuVendor before attempting hardware encoder init
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum GpuVendor {
//     Nvidia,
//     Amd,
//     Intel,
//     Unknown,
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum NvidiaArch {
//     Turing,     // RTX 20 series (2018)  — SM 7.5
//     Ampere,     // RTX 30 series (2020)  — SM 8.6
//     AdaLovelace,// RTX 40 series (2022)  — SM 8.9
//     Blackwell,  // RTX 50 series (2025)  — SM 10.0
//     Unknown,
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum AmdArch {
//     Rdna2,    // RX 6000 series (2020)
//     Rdna3,    // RX 7000 series (2022)
//     Rdna4,    // RX 9000 series (2024)
//     Unknown,
//   }
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum IntelArch {
//     AlchemistArc,   // Arc A series (2022)
//     BattlemageArc,  // Arc B series (2024)
//     Unknown,
//   }
//
//   pub struct VendorDetails {
//     pub vendor:       GpuVendor,
//     pub nvidia_arch:  Option<NvidiaArch>,
//     pub amd_arch:     Option<AmdArch>,
//     pub intel_arch:   Option<IntelArch>,
//     pub device_name:  String,     // e.g. "NVIDIA GeForce RTX 4090"
//     pub vram_mb:      u64,        // dedicated VRAM in MB
//     pub vendor_id:    u32,        // PCI vendor ID
//     pub device_id:    u32,        // PCI device ID
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn detect_vendor(adapter: &wgpu::Adapter) -> VendorDetails
//     — reads adapter.get_info():
//         .vendor  — PCI vendor ID:
//           0x10DE = NVIDIA
//           0x1002 = AMD
//           0x8086 = Intel
//           other  = Unknown
//         .device  — PCI device ID (used for architecture detection)
//         .name    — device name string
//     — maps vendor_id to GpuVendor enum.
//     — maps device_id to NvidiaArch/AmdArch/IntelArch via known PCI ID ranges:
//
//         NVIDIA device_id ranges (from PCI IDs database):
//           0x2200..=0x2400  → Ampere (RTX 30)
//           0x2600..=0x2900  → Ada Lovelace (RTX 40)
//           0x2B00..=0x2FFF  → Blackwell (RTX 50)
//           0x1E00..=0x1F00  → Turing (RTX 20)
//
//         AMD device_id ranges:
//           0x73A0..=0x73FF  → RDNA2 (RX 6000)
//           0x7440..=0x74FF  → RDNA3 (RX 7000)
//           0x7580..=0x75FF  → RDNA4 (RX 9000)
//
//         Intel device_id ranges:
//           0x56A0..=0x56C0  → Arc Alchemist (A series)
//           0x5690..=0x56A0  → Arc Alchemist mobile
//           0xE200..=0xE2FF  → Arc Battlemage (B series)
//
//     — reads vram_mb from adapter limits (approximation; exact VRAM requires
//       DXGI QueryVideoMemoryInfo which is done in vram_budget.rs).
//     — returns VendorDetails with all fields populated.
//
//   pub fn supports_nvenc(details: &VendorDetails) -> bool
//     — returns true if vendor == Nvidia AND arch >= Turing.
//     — NVENC is available on GTX 10 series and newer.
//
//   pub fn supports_amf(details: &VendorDetails) -> bool
//     — returns true if vendor == Amd.
//     — AMF (Advanced Media Framework) is available on all AMD GCN+ GPUs.
//
//   pub fn supports_qsv(details: &VendorDetails) -> bool
//     — returns true if vendor == Intel.
//     — QuickSync Video is available on Intel GPUs with media engine.
//
// NOTES FOR AI:
//   - PCI device_id ranges above are approximate. Maintain a lookup table
//     for exact IDs of major GPUs (RTX 4090=0x2684, RX 7900 XTX=0x744C, etc.)
//     and fall back to range matching for unknown models.
//   - vendor_id and device_id are available in wgpu::AdapterInfo.
//     No DXGI/Vulkan HAL access needed for basic vendor detection.
//   - vram_mb from adapter.limits() is unreliable — it reflects wgpu's
//     internal estimate, not the actual VRAM. Use vram_budget.rs for accuracy.
// =============================================================================

#![cfg(target_os = "windows")]

use crate::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor { Nvidia, Amd, Intel, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvidiaArch { Turing, Ampere, AdaLovelace, Blackwell, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmdArch { Rdna2, Rdna3, Rdna4, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntelArch { AlchemistArc, BattlemageArc, Unknown }

pub struct VendorDetails {
    pub vendor:      GpuVendor,
    pub nvidia_arch: Option<NvidiaArch>,
    pub amd_arch:    Option<AmdArch>,
    pub intel_arch:  Option<IntelArch>,
    pub device_name: String,
    pub vram_mb:     u64,
    pub vendor_id:   u32,
    pub device_id:   u32,
}

pub fn detect_vendor(adapter: &wgpu::Adapter) -> VendorDetails {
    let info = adapter.get_info();
    let vendor = match info.vendor {
        0x10DE => GpuVendor::Nvidia,
        0x1002 => GpuVendor::Amd,
        0x8086 => GpuVendor::Intel,
        _      => GpuVendor::Unknown,
    };
    VendorDetails {
        vendor,
        nvidia_arch: if vendor == GpuVendor::Nvidia { Some(detect_nvidia_arch(info.device)) } else { None },
        amd_arch:    if vendor == GpuVendor::Amd    { Some(detect_amd_arch(info.device))    } else { None },
        intel_arch:  if vendor == GpuVendor::Intel  { Some(detect_intel_arch(info.device))  } else { None },
        device_name: info.name,
        vram_mb:     0, // populated by vram_budget.rs
        vendor_id:   info.vendor,
        device_id:   info.device,
    }
}

pub fn supports_nvenc(details: &VendorDetails) -> bool {
    details.vendor == GpuVendor::Nvidia
}

pub fn supports_amf(details: &VendorDetails) -> bool {
    details.vendor == GpuVendor::Amd
}

pub fn supports_qsv(details: &VendorDetails) -> bool {
    details.vendor == GpuVendor::Intel
}

fn detect_nvidia_arch(device_id: u32) -> NvidiaArch {
    match device_id {
        0x1E00..=0x1FFF => NvidiaArch::Turing,
        0x2200..=0x24FF => NvidiaArch::Ampere,
        0x2600..=0x29FF => NvidiaArch::AdaLovelace,
        0x2B00..=0x2FFF => NvidiaArch::Blackwell,
        _               => NvidiaArch::Unknown,
    }
}

fn detect_amd_arch(device_id: u32) -> AmdArch {
    match device_id {
        0x73A0..=0x73FF => AmdArch::Rdna2,
        0x7440..=0x74FF => AmdArch::Rdna3,
        0x7580..=0x75FF => AmdArch::Rdna4,
        _               => AmdArch::Unknown,
    }
}

fn detect_intel_arch(device_id: u32) -> IntelArch {
    match device_id {
        0x5690..=0x56C0 => IntelArch::AlchemistArc,
        0xE200..=0xE2FF => IntelArch::BattlemageArc,
        _               => IntelArch::Unknown,
    }
}