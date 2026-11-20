use aether::prelude::*;
use crate::state::{ShellState, VolumeStage};

#[component]
pub fn StageHost(mut state: Signal<ShellState>) -> Element {
    let current = state.read().current_stage;
    if current == VolumeStage::None {
        return rsx! { div {} };
    }

    let title = match current {
        VolumeStage::Talk => "Webizen Talk · Encrypted Comms",
        VolumeStage::Mail => "Webizen Mail · Semantic Inbound",
        VolumeStage::Directory => "Webizen Directory · Graph",
        VolumeStage::Wallet => "Webizen Wallet · Multi-Rail Vault",
        VolumeStage::Wellfair => "Webizen Wellfair · Clinical & Health",
        VolumeStage::Library => "Webizen Library · Hypermedia Keep",
        VolumeStage::Projects => "Webizen Projects · Cooperative Work",
        VolumeStage::Poet => "Webizen Poet · Synthesis & Creation",
        VolumeStage::Settings => "System Settings · Hardware & Privacy",
        VolumeStage::None => "",
    };

    rsx! {
        div {
            class: "stage-modal",
            style: "position: fixed; inset: 60px 40px 40px 40px; background: rgba(15, 23, 42, 0.95); backdrop-filter: blur(24px); border: 1px solid rgba(56, 189, 248, 0.3); border-radius: 20px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7); display: flex; flex-direction: column; overflow: hidden; z-index: 50;",
            
            // Stage Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; padding: 14px 24px; border-bottom: 1px solid rgba(255, 255, 255, 0.08); background: rgba(30, 41, 59, 0.5);",
                span { style: "font-weight: 700; font-size: 15px; color: #f8fafc;", "{title}" }
                button {
                    style: "background: rgba(239, 68, 68, 0.2); border: 1px solid rgba(239, 68, 68, 0.4); color: #f87171; border-radius: 8px; padding: 4px 12px; cursor: pointer; font-size: 13px;",
                    onclick: move |_| state.write().current_stage = VolumeStage::None,
                    "✕ Close Stage"
                }
            }
            
            // Stage Content Viewport
            div {
                style: "flex: 1; padding: 24px; overflow-y: auto; color: #cbd5e1; font-family: system-ui, sans-serif;",
                div {
                    style: "max-width: 800px; margin: 0 auto; display: flex; flex-direction: column; gap: 16px;",
                    p { style: "font-size: 14px; line-height: 1.6;",
                        "Placeholder stage (C1). QualiaDB Qap install and the native-RSX card arrive at C2."
                    }
                    div {
                        style: "background: rgba(30, 41, 59, 0.6); border: 1px solid rgba(255, 255, 255, 0.06); border-radius: 12px; padding: 18px;",
                        h4 { style: "margin: 0 0 10px 0; color: #38bdf8;", "Active Integration" }
                        ul { style: "margin: 0; padding-left: 20px; font-size: 13px; line-height: 1.8;",
                            li { "Zero-Heap Hot Paths (Tier-1 NQuin evaluation active)" }
                            li { "42MB Prolog Sentinel memory bounds enforced" }
                            li { "Local Peer-to-Peer gossip and KeyVault security active" }
                        }
                    }
                }
            }
        }
    }
}
