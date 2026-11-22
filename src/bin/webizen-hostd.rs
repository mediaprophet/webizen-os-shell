// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use std::process::ExitCode;
use webizen_ua_os::{execute_hostd_command, parse_hostd_args, HostDaemonCommand, HOSTD_USAGE};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (cfg, command) = match parse_hostd_args(&args) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("webizen-hostd: {error}\n{HOSTD_USAGE}");
            return ExitCode::from(2);
        }
    };

    if command == HostDaemonCommand::Help && !cfg.json {
        print!("{HOSTD_USAGE}");
        return ExitCode::SUCCESS;
    }
    match execute_hostd_command(&cfg, command) {
        Ok(receipt) => {
            let encoded = if cfg.json {
                serde_json::to_string_pretty(&receipt)
            } else {
                serde_json::to_string(&receipt)
            };
            match encoded {
                Ok(text) => {
                    println!("{text}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("webizen-hostd: serialize receipt: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            eprintln!("webizen-hostd: {error}");
            ExitCode::FAILURE
        }
    }
}
