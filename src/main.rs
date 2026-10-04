mod app;
mod components;
mod state;

use app::App;

fn main() {
    dioxus::launch(App);
}
