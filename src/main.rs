mod app;
mod components;
mod state;
mod tray;

use aether::prelude::*;
use app::App;
use state::init_session;
use std::path::PathBuf;
use webizen_ua_os::{
    generate_dual_sku_delivery_artifacts, generate_tri_target_debian_suite, run_updater,
    upgrade_browser_product_independent, DeviceSession, UpdateAction,
};

fn root() -> Element {
    rsx! { App {} }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let smoke = std::env::var("WEBIZEN_C1_SMOKE").ok().as_deref() == Some("1");
    let session = DeviceSession::resolve();

    // Handle CLI subcommands:
    // 1. `webizen-os-shell update check|verify|upgrade|apply [candidate_path]` (C7 / KD-16)
    // 2. `webizen-os-shell package-dual-sku` (C6 Guest bundle + D5/F4 Debian Trixie overlay + A7 LICENSE-BOUNDARY)
    if let Some(first) = args.first().map(|s| s.as_str()) {
        if first == "update" {
            let action = args
                .get(1)
                .and_then(|a| UpdateAction::parse(a))
                .unwrap_or(UpdateAction::Check);
            let candidate = args.get(2).map(PathBuf::from);
            let receipt = run_updater(&session.host, session.sku, action, candidate.as_deref());
            println!(
                "{}",
                serde_json::to_string_pretty(&receipt).unwrap_or_else(|_| receipt.detail)
            );
            std::process::exit(if receipt.ok { 0 } else { 1 });
        } else if first == "update-browser" {
            let target_ver = args.get(1).map(|s| s.as_str()).unwrap_or("0.1.1");
            match upgrade_browser_product_independent(&session.host, target_ver) {
                Ok(status) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&status).unwrap_or_default()
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("update-browser failed: {e}");
                    std::process::exit(1);
                }
            }
        } else if first == "package-dual-sku" || first == "--generate-delivery" {
            match generate_dual_sku_delivery_artifacts(&session.host, &session.gpu) {
                Ok(receipt) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&receipt).unwrap_or_default()
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("package-dual-sku failed: {e}");
                    std::process::exit(1);
                }
            }
        } else if first == "package-tri-target-deb" {
            match generate_tri_target_debian_suite(
                &session.host,
                &session.gpu,
                &session.host_stewardship,
            ) {
                Ok(receipt) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&receipt).unwrap_or_default()
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("package-tri-target-deb failed: {e}");
                    std::process::exit(1);
                }
            }
        }
    }

    if !smoke {
        match session.host.ensure_daemon(session.sku) {
            Ok(h) => eprintln!(
                "C0 daemon: reachable={} port={} {}",
                h.status.reachable, h.status.port, h.status.detail
            ),
            Err(e) => {
                eprintln!("C0 daemon: {e} (Settings still lists identifiers and vault paths)")
            }
        }
    }
    init_session(session);

    if !smoke {
        if let Err(e) = tray::install() {
            eprintln!("C0 tray: {e}");
        }
    }

    let cfg = aether_native::Config::new()
        .with_window_attributes(
            aether_native::WindowAttributes::default()
                .with_title("Webizen OS-Shell")
                .with_surface_size(aether_native::LogicalSize::new(1280.0, 800.0)),
        )
        .with_renderer_fallback(aether_native::RendererFallbackPolicy::CpuOnFailure);
    aether_native::try_launch_cfg(root, vec![], vec![Box::new(cfg)])
        .expect("Webizen OS-Shell native launch");

    if smoke {
        println!("C1 smoke: native window opened, Command Orb shell painted, process exited.");
    }
}
