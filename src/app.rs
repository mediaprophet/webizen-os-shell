use aether::prelude::*;
use aether_native::winit::event::WindowEvent;
use crate::components::{HaloBar, OrbitWheel, StageHost};
use crate::state::ShellState;
use std::sync::atomic::{AtomicUsize, Ordering};

static C1_SMOKE_REDRAWS: AtomicUsize = AtomicUsize::new(0);

/// Bounded WEBIZEN_C1_SMOKE=1 path: three native redraws then event-loop exit.
/// Hook order is fixed; the env flag only changes whether redraws request exit.
fn arm_c1_smoke() {
    let smoke = std::env::var("WEBIZEN_C1_SMOKE").ok().as_deref() == Some("1");
    let window = aether_native::use_window();
    if smoke {
        window.request_redraw();
    }
    aether_native::use_window_event(move |event, event_loop| {
        if !smoke {
            return;
        }
        if matches!(event, WindowEvent::RedrawRequested) {
            let frames = C1_SMOKE_REDRAWS.fetch_add(1, Ordering::SeqCst) + 1;
            if frames >= 3 {
                event_loop.exit();
            } else {
                window.request_redraw();
            }
        }
    });
}

#[component]
pub fn App() -> Element {
    let state = use_signal(ShellState::default);
    arm_c1_smoke();

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
