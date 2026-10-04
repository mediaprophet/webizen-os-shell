use dioxus::prelude::*;
use crate::state::{ShellState, VolumeStage};

const ITEMS: &[(&str, VolumeStage, &str, &str)] = &[
    ("Talk", VolumeStage::Talk, "💬", "Encrypted Mesh Voice & Chat"),
    ("Mail", VolumeStage::Mail, "✉️", "Asynchronous Semantic Mail"),
    ("Directory", VolumeStage::Directory, "👥", "Identity & Peer Ledger"),
    ("Wallet", VolumeStage::Wallet, "💳", "Multi-Rail Sovereign Vault"),
    ("Wellfair", VolumeStage::Wellfair, "🩺", "Clinical Health & Welfare"),
    ("Library", VolumeStage::Library, "📚", "Sovereign Hypermedia Vault"),
    ("Projects", VolumeStage::Projects, "⚙️", "Cooperative Tasks & DAG"),
    ("Settings", VolumeStage::Settings, "🛠️", "Hardware, Privacy & Models"),
];

#[component]
pub fn OrbitWheel(state: Signal<ShellState>) -> Element {
    let current = state.read().current_stage;

    rsx! {
        div {
            class: "orbit-wheel-container",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 20px; padding: 40px 20px;",

            // Central Poet Hub
            button {
                style: "width: 80px; height: 80px; border-radius: 50%; background: radial-gradient(circle, #38bdf8 0%, #1e40af 100%); border: 3px solid rgba(255, 255, 255, 0.4); box-shadow: 0 0 30px rgba(56, 189, 248, 0.5); color: white; font-size: 28px; cursor: pointer; transition: transform 0.2s, box-shadow 0.2s;",
                onclick: move |_| {
                    let mut s = state.write();
                    if s.current_stage == VolumeStage::Poet {
                        s.current_stage = VolumeStage::None;
                    } else {
                        s.current_stage = VolumeStage::Poet;
                    }
                },
                "✦"
            }
            div { style: "font-size: 13px; font-weight: 600; color: #94a3b8; letter-spacing: 0.1em;", "POET HUB" }

            // 8 Orbital Volume Tiles Grid
            div {
                style: "display: grid; grid-template-columns: repeat(4, 160px); gap: 16px; margin-top: 20px;",
                for (name, stage_item, icon, desc) in ITEMS.iter() {
                    {
                        let stage = *stage_item;
                        rsx! {
                            button {
                                key: "{name}",
                                style: if current == stage {
                                    "padding: 18px 12px; border-radius: 16px; background: rgba(56, 189, 248, 0.2); border: 2px solid #38bdf8; color: white; display: flex; flex-direction: column; align-items: center; gap: 8px; cursor: pointer; transition: all 0.2s; box-shadow: 0 0 15px rgba(56, 189, 248, 0.3);"
                                } else {
                                    "padding: 18px 12px; border-radius: 16px; background: rgba(30, 41, 59, 0.7); border: 1px solid rgba(255, 255, 255, 0.08); color: #e2e8f0; display: flex; flex-direction: column; align-items: center; gap: 8px; cursor: pointer; transition: all 0.2s;"
                                },
                                onclick: move |_| {
                                    let mut s = state.write();
                                    if s.current_stage == stage {
                                        s.current_stage = VolumeStage::None;
                                    } else {
                                        s.current_stage = stage;
                                    }
                                },
                                span { style: "font-size: 24px;", "{icon}" }
                                span { style: "font-weight: 600; font-size: 14px;", "{name}" }
                                span { style: "font-size: 10px; color: #94a3b8; text-align: center; line-height: 1.2;", "{desc}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
