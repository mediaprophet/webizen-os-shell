use dioxus::prelude::*;
use crate::state::ShellState;

#[component]
pub fn HaloBar(state: Signal<ShellState>) -> Element {
    let s = state.read();
    rsx! {
        header {
            class: "halo-bar",
            style: "display: flex; justify-content: space-between; align-items: center; padding: 10px 24px; background: rgba(14, 18, 28, 0.85); backdrop-filter: blur(16px); border-bottom: 1px solid rgba(255, 255, 255, 0.08); font-family: system-ui, sans-serif; color: #e2e8f0; z-index: 100;",
            div {
                style: "display: flex; align-items: center; gap: 12px;",
                span { style: "font-weight: 700; letter-spacing: 0.05em; font-size: 15px; color: #38bdf8;", "WEBIZEN OS" }
                span { style: "font-size: 11px; padding: 2px 8px; border-radius: 9999px; background: rgba(56, 189, 248, 0.15); color: #38bdf8;", "SPATIAL HALO" }
            }
            div {
                style: "display: flex; align-items: center; gap: 20px; font-size: 13px;",
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    span { style: "width: 8px; height: 8px; border-radius: 50%; background: #4ade80;" }
                    span { "{s.p2p_peers_count} Peers" }
                }
                div {
                    style: "display: flex; align-items: center; gap: 6px;",
                    span { style: if s.nym_mixnet_active { "color: #38bdf8;" } else { "color: #94a3b8;" }, "Nym Mixnet" }
                }
                div {
                    style: "font-family: monospace; font-size: 11px; color: #94a3b8;",
                    "{s.did_fingerprint}"
                }
            }
        }
    }
}
