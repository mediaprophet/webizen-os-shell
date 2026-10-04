use dioxus::prelude::*;
use crate::components::{HaloBar, OrbitWheel, StageHost};
use crate::state::ShellState;

#[component]
pub fn App() -> Element {
    let state = use_signal(ShellState::default);

    rsx! {
        div {
            class: "webizen-shell-root",
            style: "width: 100vw; height: 100vh; background: radial-gradient(circle at 50% 30%, #1e1b4b 0%, #030712 100%); display: flex; flex-direction: column; overflow: hidden; user-select: none;",
            
            HaloBar { state }
            
            main {
                style: "flex: 1; display: flex; align-items: center; justify-content: center; position: relative;",
                OrbitWheel { state }
                StageHost { state }
            }
        }
    }
}
