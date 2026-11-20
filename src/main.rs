mod app;
mod components;
mod state;

use aether::prelude::*;
use app::App;

fn root() -> Element {
    rsx! { App {} }
}

fn main() {
    let cfg = aether_native::Config::new().with_window_attributes(
        aether_native::WindowAttributes::default()
            .with_title("Webizen OS-Shell")
            .with_surface_size(aether_native::LogicalSize::new(1280.0, 800.0)),
    );

    aether_native::try_launch_cfg(root, vec![], vec![Box::new(cfg)])
        .expect("Webizen OS-Shell native launch");

    if std::env::var("WEBIZEN_C1_SMOKE").ok().as_deref() == Some("1") {
        println!(
            "C1 smoke: native window opened, HaloBar/OrbitWheel painted, process exited."
        );
    }
}
