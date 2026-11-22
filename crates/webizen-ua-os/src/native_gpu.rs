//! Native Etalon GPU runtime & Anvil `NodeProfile64` hardware capability probe.
//!
//! Ensures Etalon runs in genuine native mode (`etalon::Instance` with Vulkan / DX12 / Metal
//! backends and `TargetProfileMask::NATIVE_DESKTOP` VibeScript ABI providers), never via
//! browser WASM shims. Pairs live adapter PCI IDs with Anvil's `NodeProfile64` and
//! `SpecializerHints`, and enforces fail-closed hardware-attested vault advertising.

use anvil_core::{probe_live_profile, NodeProfile64, PROFILE_HAS_GPU_UUID, PROFILE_HAS_IOMMU};
use anvil_gpu::{hints_for_pci, SpecializerHints};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use vibe::extension_registry::{
    dispatch_prepared_call, prepare_extension_call, CallExecutionContext, ExtensionRegistry,
    BUILTIN_EXTENSION_ALIASES,
};
use vibescript_abi::{
    BorrowedSliceView, CallerOutputSlice, ExecutionMode, NQuin48, PrimitiveKind, SemVerReq,
    TargetProfileMask,
};

pub const NODE_PROFILE_FILE_NAME: &str = "anvil_node_profile.np64";

const REQUIRED_NATIVE_PROVIDERS: &[&str] = &[
    "mech", "gpu", "forge", "render", "signal", "sense", "p64", "nquin", "infer", "qgraph",
    "modality",
];

/// One physical GPU adapter enumerated directly from native `etalon::Instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeGpuAdapterInfo {
    pub name: String,
    pub vendor_id: u16,
    pub device_id: u16,
    pub backend: String,
    pub device_type: String,
    pub driver: String,
    pub driver_info: String,
}

/// Complete native Etalon + Anvil hardware capability snapshot for this host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeEtalonStatus {
    /// True when native `etalon::Instance` enumerated at least one hardware adapter.
    pub native_gpu_active: bool,
    /// Primary selected native adapter (`Vulkan` preferred, then `Dx12` / `Metal`).
    pub primary_adapter: Option<NativeGpuAdapterInfo>,
    /// All native adapters discovered by Etalon on this machine.
    pub adapters: Vec<NativeGpuAdapterInfo>,
    /// Anvil 64-byte hardware capability profile (`NP64`).
    pub anvil_profile: NodeProfile64,
    /// Anvil WGSL / Naga specialization hints for the primary GPU PCI ID.
    pub specializer_hints: SpecializerHints,
    /// Number of `vibescript-abi` providers admitted for `TargetProfileMask::NATIVE_DESKTOP`.
    pub native_vibe_provider_count: usize,
    /// True when 48-byte `NQuin48` wire-parity dispatch succeeded on native Etalon.
    pub nquin_parity_verified: bool,
    /// Fail-closed N2/N4 gate: true ONLY when IOMMU + GPU UUID + attestation tier <= 1.
    pub may_advertise_hardware_attested_vault: bool,
    /// Concise status line for the OS-Shell chrome header/footer.
    pub summary_label: String,
}

/// Probe native Etalon GPU adapters, Anvil `NodeProfile64`, and `TargetProfileMask::NATIVE_DESKTOP`
/// VibeScript ABI providers, persisting `{storage}/anvil_node_profile.np64`.
pub fn probe_native_etalon(storage: &Path) -> NativeEtalonStatus {
    let mut anvil_profile = probe_live_profile();
    let adapters = enumerate_native_etalon_adapters();
    let primary_adapter = select_primary_adapter(&adapters);

    let (vendor_id, device_id) = primary_adapter
        .as_ref()
        .map(|a| (a.vendor_id, a.device_id))
        .unwrap_or((anvil_profile.vendor_id, anvil_profile.device_id));

    if vendor_id != 0 {
        anvil_profile.vendor_id = vendor_id;
    }
    if device_id != 0 {
        anvil_profile.device_id = device_id;
    }

    let np64_path = storage.join(NODE_PROFILE_FILE_NAME);
    if let Ok(existing_bytes) = fs::read(&np64_path) {
        if let Ok(existing) = NodeProfile64::parse(&existing_bytes) {
            if existing.has_flag(PROFILE_HAS_GPU_UUID) {
                anvil_profile.bind_gpu_device_uuid(existing.gpu_device_uuid);
            }
        }
    }
    let _ = fs::create_dir_all(storage);
    let _ = fs::write(&np64_path, anvil_profile.to_bytes());

    let specializer_hints = hints_for_pci(vendor_id, device_id);
    let (native_vibe_provider_count, nquin_parity_verified) = probe_native_vibe_providers();
    let may_advertise_hardware_attested_vault =
        anvil_profile.supports_attested_vault_advertising();

    let native_gpu_active = primary_adapter.is_some();
    let summary_label = match &primary_adapter {
        Some(a) => format!(
            "Etalon Native {} · {} ({:04x}:{:04x} · wg={} · {} native providers)",
            a.backend,
            a.name,
            a.vendor_id,
            a.device_id,
            specializer_hints.workgroup_size,
            native_vibe_provider_count
        ),
        None => format!(
            "Etalon Native ({:04x}:{:04x} · wg={} · {} native providers)",
            vendor_id, device_id, specializer_hints.workgroup_size, native_vibe_provider_count
        ),
    };

    NativeEtalonStatus {
        native_gpu_active,
        primary_adapter,
        adapters,
        anvil_profile,
        specializer_hints,
        native_vibe_provider_count,
        nquin_parity_verified,
        may_advertise_hardware_attested_vault,
        summary_label,
    }
}

fn enumerate_native_etalon_adapters() -> Vec<NativeGpuAdapterInfo> {
    let instance = etalon::Instance::new(etalon::InstanceDescriptor {
        backends: etalon::Backends::VULKAN | etalon::Backends::DX12 | etalon::Backends::METAL,
        flags: etalon::InstanceFlags::empty(),
        memory_budget_thresholds: etalon::MemoryBudgetThresholds::default(),
        backend_options: etalon::BackendOptions::default(),
        display: None,
    });

    let raw_adapters = block_on_ready(instance.enumerate_adapters(
        etalon::Backends::VULKAN | etalon::Backends::DX12 | etalon::Backends::METAL,
    ));
    let mut out = Vec::with_capacity(raw_adapters.len());
    for adapter in raw_adapters {
        let info = adapter.get_info();
        out.push(NativeGpuAdapterInfo {
            name: info.name,
            vendor_id: (info.vendor & 0xFFFF) as u16,
            device_id: (info.device & 0xFFFF) as u16,
            backend: format!("{:?}", info.backend),
            device_type: format!("{:?}", info.device_type),
            driver: info.driver,
            driver_info: info.driver_info,
        });
    }
    out
}

fn block_on_ready<F: std::future::Future>(fut: F) -> F::Output {
    use std::pin::pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn noop_clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = pin!(fut);
    loop {
        if let Poll::Ready(val) = fut.as_mut().poll(&mut cx) {
            return val;
        }
        std::thread::yield_now();
    }
}

fn select_primary_adapter(adapters: &[NativeGpuAdapterInfo]) -> Option<NativeGpuAdapterInfo> {
    // Prefer DiscreteGpu on Vulkan, then DiscreteGpu on any native backend, then IntegratedGpu.
    adapters
        .iter()
        .find(|a| a.device_type == "DiscreteGpu" && a.backend == "Vulkan")
        .or_else(|| adapters.iter().find(|a| a.device_type == "DiscreteGpu"))
        .or_else(|| adapters.iter().find(|a| a.backend == "Vulkan"))
        .or_else(|| adapters.first())
        .cloned()
}

/// Verify that all 11 Etalon/Qualia `vibescript-abi` providers admit under
/// `TargetProfileMask::NATIVE_DESKTOP` and pass the 48-byte `NQuin48` wire-parity check.
pub fn probe_native_vibe_providers() -> (usize, bool) {
    let registry = ExtensionRegistry::with_builtin_providers();
    for alias in REQUIRED_NATIVE_PROVIDERS {
        let Some((_, provider, descriptor)) = registry.find_by_namespace_or_alias(alias) else {
            return (registry.active_count(), false);
        };
        if !descriptor.versions.is_engine_compatible()
            || (descriptor.supported_targets.0 & TargetProfileMask::NATIVE_DESKTOP.0) == 0
            || provider
                .static_ontology_nquins()
                .iter()
                .any(|q| !q.verify_parity())
        {
            return (registry.active_count(), false);
        }
    }

    let record = NQuin48::invariant(
        "https://qualiadb.org/app/webizen-os-shell/native-gpu",
        "https://qualiadb.org/schema/vibe#NativeEtalonParity",
        42,
    );
    let expected = record.0[..5].iter().fold(0u64, |acc, w| acc ^ w);
    if record.parity() != expected {
        return (registry.active_count(), false);
    }

    let mut bytes = [0u8; 40];
    for (word, slot) in record.0[..5].iter().zip(bytes.chunks_exact_mut(8)) {
        slot.copy_from_slice(&word.to_le_bytes());
    }

    let ok = (|| -> Result<bool, vibescript_abi::VibeTrap> {
        let handle = prepare_extension_call(
            &registry,
            "nquin",
            "wire",
            "compute_wire_parity",
            SemVerReq::AnyStable,
            TargetProfileMask::NATIVE_DESKTOP,
        )?;
        let input = BorrowedSliceView::from_raw_bytes(&bytes, PrimitiveKind::U64, 5)?;
        let mut result = [0u8; 8];
        let mut output = CallerOutputSlice::new(&mut result, PrimitiveKind::U64)?;
        let context = CallExecutionContext {
            actor_did_hash: 0,
            context_world_hash: 0,
            lamport_timestamp: 0,
            mode: ExecutionMode::StandaloneDeveloper,
        };
        let outcome =
            dispatch_prepared_call(&registry, &handle, &context, None, &[input], &mut output)?;
        Ok(u64::from_le_bytes(result) == expected
            && outcome.output_elems_written == 1
            && !outcome.receipt.simulated
            && outcome.audit_nquins.iter().all(|q| q.verify_parity()))
    })()
    .unwrap_or(false);

    let _ = BUILTIN_EXTENSION_ALIASES;
    (registry.active_count(), ok)
}

impl NativeEtalonStatus {
    pub fn iommu_present(&self) -> bool {
        self.anvil_profile.has_flag(PROFILE_HAS_IOMMU)
    }

    pub fn has_gpu_uuid(&self) -> bool {
        self.anvil_profile.has_flag(PROFILE_HAS_GPU_UUID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_vibe_providers_admit_on_native_desktop_profile() {
        let (count, parity_ok) = probe_native_vibe_providers();
        assert!(count >= REQUIRED_NATIVE_PROVIDERS.len());
        assert!(parity_ok, "48-byte NQuin48 wire parity must verify on native Etalon");
    }

    #[test]
    fn probe_native_etalon_writes_np64_and_enforces_attestation_gate() {
        let tmp = std::env::temp_dir().join(format!("wz-native-gpu-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let status = probe_native_etalon(&tmp);
        assert!(tmp.join(NODE_PROFILE_FILE_NAME).is_file());
        assert!(status.nquin_parity_verified);
        assert!(status.native_vibe_provider_count >= 11);
        // Without IOMMU + bound GPU UUID, attestation advertising must fail closed.
        if !status.iommu_present() || !status.has_gpu_uuid() {
            assert!(!status.may_advertise_hardware_attested_vault);
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}
