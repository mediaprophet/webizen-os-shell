// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

#![cfg_attr(not(test), windows_subsystem = "windows")]

mod app;
mod components;
mod state;
mod tray;
pub mod uat;

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

fn attach_parent_console_for_cli() {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn AttachConsole(dw_process_id: u32) -> i32;
        }
        const ATTACH_PARENT_PROCESS: u32 = 0xFFFF_FFFF;
        unsafe {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() || std::env::var("WEBIZEN_C1_SMOKE").ok().as_deref() == Some("1") {
        attach_parent_console_for_cli();
    }
    let smoke = std::env::var("WEBIZEN_C1_SMOKE").ok().as_deref() == Some("1");
    let session = DeviceSession::resolve();

    // Handle CLI subcommands:
    // 1. `webizen-os-shell update check|verify|upgrade|apply [candidate_path]` (C7 / KD-16)
    // 2. `webizen-os-shell package-dual-sku` (C6 Guest bundle + D5/F4 Debian Trixie overlay + A7 LICENSE-BOUNDARY)
    // 3. `webizen-os-shell uat [stage_filter]` (Automated live-window UAT button sweep)
    // 4. `webizen-os-shell uat-mcp` (MCP JSON-RPC 2.0 server over stdio)
    // 5. `webizen-os-shell uat-cmd <snapshot|press|input|stage|suite> ...` (CLI client to live window)
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
        } else if first == "uat-mcp" {
            uat::run_mcp_stdio_server();
            std::process::exit(0);
        } else if first == "uat-cmd" {
            let port: u16 = std::env::var("WEBIZEN_UAT_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4249);
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("snapshot");
            let cmd = match sub {
                "press" | "click" => uat::UatCommand::PressButton {
                    target: args.get(2).cloned().unwrap_or_default(),
                },
                "input" => uat::UatCommand::SetInput {
                    target: args.get(2).cloned().unwrap_or_default(),
                    value: args.get(3).cloned().unwrap_or_default(),
                },
                "stage" => uat::UatCommand::SwitchStage {
                    stage: args.get(2).cloned().unwrap_or_else(|| "none".into()),
                },
                "suite" => uat::UatCommand::RunSuite {
                    stage_filter: args.get(2).cloned(),
                },
                _ => uat::UatCommand::Snapshot,
            };
            match uat::send_remote_uat_command(port, &cmd) {
                Ok(resp) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                    std::process::exit(if resp.ok { 0 } else { 1 });
                }
                Err(e) => {
                    eprintln!("uat-cmd failed: {e}");
                    std::process::exit(1);
                }
            }
        } else if first == "uat" {
            std::env::set_var("WEBIZEN_UAT_AUTO", "1");
            for arg in args.iter().skip(1) {
                if arg == "--all-themes" {
                    std::env::set_var("WEBIZEN_UAT_ALL_THEMES", "1");
                } else if !arg.starts_with('-') {
                    std::env::set_var("WEBIZEN_UAT_STAGE", arg);
                }
            }
        }
    }

    let uat_auto = std::env::var("WEBIZEN_UAT_AUTO").ok().as_deref() == Some("1");
    if !smoke && !uat_auto {
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

    if !smoke && !uat_auto {
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
