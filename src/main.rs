mod app;
mod components;
mod state;

use app::App;

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, WindowBuilder};
        let cfg = Config::new()
            .with_window(
                WindowBuilder::new()
                    .with_title("Webizen OS-Shell")
                    .with_resizable(true)
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 800.0)),
            );
        dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App);
    }

    #[cfg(feature = "web")]
    {
        dioxus::LaunchBuilder::web().launch(App);
    }
}
