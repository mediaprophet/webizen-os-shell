//! Guest host-OS tray. Desktop product chrome, not a qualia-cli feature.
//!
//! Tauri webizen-desktop already had a tray. Successor chrome is aether-native,
//! so this uses `tray-icon` directly (no Wry, no aether-desktop).

use aether_native::winit::window::Window;
use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::{Arc, Mutex, Weak},
};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::components::auxiliary_windows::AuxiliaryKind;
use crate::state::VolumeStage;

thread_local! {
    static TRAY: RefCell<Option<TrayIcon>> = RefCell::new(None);
}
struct TrayActions {
    stage: Option<VolumeStage>,
    overlays: VecDeque<AuxiliaryKind>,
    window: Option<Weak<dyn Window>>,
}

static PENDING: Mutex<TrayActions> = Mutex::new(TrayActions {
    stage: None,
    overlays: VecDeque::new(),
    window: None,
});

pub(crate) fn bind_window(window: &Arc<dyn Window>) {
    let mut actions = PENDING.lock().unwrap_or_else(|error| error.into_inner());
    actions.window = Some(Arc::downgrade(window));
}

pub fn take_stage() -> Option<VolumeStage> {
    PENDING
        .lock()
        .ok()
        .and_then(|mut actions| actions.stage.take())
}

pub(crate) fn take_overlays() -> VecDeque<AuxiliaryKind> {
    PENDING
        .lock()
        .map(|mut actions| std::mem::take(&mut actions.overlays))
        .unwrap_or_default()
}

pub fn install() -> Result<(), String> {
    let menu = Menu::new();
    let settings = MenuItem::with_id("settings", "Settings", true, None);
    let talk = MenuItem::with_id("talk", "Talk", true, None);
    let mail = MenuItem::with_id("mail", "Mail", true, None);
    let library = MenuItem::with_id("library", "Library", true, None);
    let orb = MenuItem::with_id("command-orb", "8-Sector Command Orb", true, None);
    let palette = MenuItem::with_id("toolchest", "Semantic ToolChest", true, None);
    let alert = MenuItem::with_id("care-alerts", "Urgent Care Alerts", true, None);
    menu.append(&settings)
        .and_then(|_| menu.append(&talk))
        .and_then(|_| menu.append(&mail))
        .and_then(|_| menu.append(&library))
        .and_then(|_| menu.append(&PredefinedMenuItem::separator()))
        .and_then(|_| menu.append(&orb))
        .and_then(|_| menu.append(&palette))
        .and_then(|_| menu.append(&alert))
        .and_then(|_| menu.append(&PredefinedMenuItem::separator()))
        .and_then(|_| menu.append(&PredefinedMenuItem::quit(Some("Quit Webizen"))))
        .map_err(|e| e.to_string())?;

    let icon = Icon::from_rgba(teal_icon_32(), 32, 32).map_err(|e| e.to_string())?;
    let tray = TrayIconBuilder::new()
        .with_tooltip("Webizen")
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .with_menu_on_left_click(true)
        .build()
        .map_err(|e| e.to_string())?;
    TRAY.with(|slot| {
        *slot.borrow_mut() = Some(tray);
    });

    MenuEvent::set_event_handler(Some(|event: MenuEvent| {
        let stage = match event.id.as_ref() {
            "settings" => Some(VolumeStage::Settings),
            "talk" => Some(VolumeStage::Talk),
            "mail" => Some(VolumeStage::Mail),
            "library" => Some(VolumeStage::Library),
            _ => None,
        };
        let overlay = match event.id.as_ref() {
            "command-orb" => Some(AuxiliaryKind::Orb),
            "toolchest" => Some(AuxiliaryKind::Palette),
            "care-alerts" => Some(AuxiliaryKind::Alert),
            _ => None,
        };
        if stage.is_none() && overlay.is_none() {
            return;
        }
        let window = {
            let Ok(mut actions) = PENDING.lock() else {
                return;
            };
            if let Some(stage) = stage {
                actions.stage = Some(stage);
            }
            if let Some(overlay) = overlay {
                if actions.overlays.len() < 16 {
                    actions.overlays.push_back(overlay);
                }
            }
            actions.window.as_ref().and_then(Weak::upgrade)
        };
        if let Some(window) = window {
            window.request_redraw();
        }
    }));
    Ok(())
}

fn teal_icon_32() -> Vec<u8> {
    let mut px = vec![0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let dx = x as i32 - 16;
            let dy = y as i32 - 16;
            if dx * dx + dy * dy <= 14 * 14 {
                let i = (y * 32 + x) * 4;
                px[i] = 47;
                px[i + 1] = 224;
                px[i + 2] = 200;
                px[i + 3] = 255;
            }
        }
    }
    px
}
