// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Appearance studio: cartridge switcher, local sensory questionnaire,
//! conversational look co-designer, and Permissive Commons sharing.

use crate::state::{session, ShellState};
use aether::prelude::*;
use webizen_ua_os::{
    activate_collected_bundle, co_design_theme_with_llm, export_shared_theme_bundle,
    export_theme_cartridge_hmc, import_shared_theme_bundle, recommend_environment_options,
    AcousticFeedbackPreference, ColorVisionProfile, ContrastGlareComfort, MotionComfort,
    ThemeCollectionState, WorkspaceDensityStyle, SENSITIVITY_PUBLIC_ONLY,
};

#[component]
pub fn ThemeStudio(state: Signal<ShellState>) -> Element {
    rsx! {
        ThemeCartridgeSwitcher { state }
        EnvironmentQuestionnairePanel { state }
        ThemeCodesignerPanel { state }
        SharedThemeCollectionPanel { state }
    }
}

#[component]
pub fn ThemeCartridgeSwitcher(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let active = s.theme_registry.active_id.clone();
    let cartridges: Vec<(String, String, String)> = s
        .theme_registry
        .cartridges
        .iter()
        .map(|c| {
            (
                c.cartridge_id.clone(),
                c.title.clone(),
                c.geometry_blurb.clone(),
            )
        })
        .collect();
    let vibe = s.custom_theme_vibe.clone();
    drop(s);
    rsx! {
        section {
            class: "id-list theme-studio",
            h4 { "Looks" }
            p { class: "muted", "Pick a spatial look. Each one is a different geometry, light, and motion style." }
            div {
                class: "plane-grid",
                for (id, title, blurb) in cartridges {
                    {
                        let selected = id == active;
                        let adopt_id = id.clone();
                        rsx! {
                            div {
                                key: "{id}",
                                class: if selected { "plane-card theme-card active" } else { "plane-card theme-card" },
                                h5 { "{title}" }
                                p { "{blurb}" }
                                button {
                                    r#type: "button",
                                    class: "sanctum-issue",
                                    id: "adopt-{id}",
                                    onclick: move |_| {
                                        let mut shell = state.write();
                                        match shell.adopt_theme(&adopt_id) {
                                            Ok(()) => shell.orb_toast = format!("Using {title}"),
                                            Err(error) => shell.orb_toast = error,
                                        }
                                    },
                                    if selected { "Using this look" } else { "Use this look" }
                                }
                            }
                        }
                    }
                }
            }
            div {
                class: "input-row",
                label {
                    "Projection script for the active look"
                    textarea {
                        class: "input-field",
                        rows: "4",
                        value: "{vibe}",
                        aria_label: "Look projection script",
                        oninput: move |evt| state.write().custom_theme_vibe = evt.value(),
                    }
                }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    onclick: move |_| {
                        let mut shell = state.write();
                        let source = shell.custom_theme_vibe.clone();
                        let active_id = shell.theme_registry.active_id.clone();
                        if let Some(cartridge) = shell
                            .theme_registry
                            .cartridges
                            .iter_mut()
                            .find(|cartridge| cartridge.cartridge_id == active_id)
                        {
                            cartridge.vibe_source = source;
                            shell.orb_toast = "Updated the active look’s projection script".into();
                        }
                    },
                    "Apply script"
                }
            }
        }
    }
}

#[component]
pub fn EnvironmentQuestionnairePanel(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let answers = s.environment_questionnaire.clone();
    let options = s.recommended_environments.clone();
    drop(s);
    let scale = answers.reading_scale_percent;
    rsx! {
        section {
            class: "id-list theme-studio",
            h4 { "How should this place feel?" }
            p { class: "muted", "Answers stay on this device. They are used only to suggest three looks." }
            div {
                class: "input-row",
                span { "Reading size" }
                for pct in [100u16, 115, 130, 145, 165] {
                    {
                        rsx! {
                            button {
                                key: "{pct}",
                                r#type: "button",
                                class: if scale == pct { "ribbon-btn active" } else { "ribbon-btn" },
                                onclick: move |_| {
                                    state.write().environment_questionnaire.reading_scale_percent = pct;
                                },
                                "{pct}%"
                            }
                        }
                    }
                }
            }
            div {
                class: "input-row",
                span { "Glare and contrast" }
                ComfortButton {
                    label: "Soft light",
                    selected: answers.contrast_glare == ContrastGlareComfort::SoftAmbient,
                    onclick: move |_| {
                        state.write().environment_questionnaire.contrast_glare =
                            ContrastGlareComfort::SoftAmbient;
                    },
                }
                ComfortButton {
                    label: "Balanced",
                    selected: answers.contrast_glare == ContrastGlareComfort::Balanced,
                    onclick: move |_| {
                        state.write().environment_questionnaire.contrast_glare =
                            ContrastGlareComfort::Balanced;
                    },
                }
                ComfortButton {
                    label: "High clarity",
                    selected: answers.contrast_glare == ContrastGlareComfort::HighClarity,
                    onclick: move |_| {
                        state.write().environment_questionnaire.contrast_glare =
                            ContrastGlareComfort::HighClarity;
                    },
                }
            }
            div {
                class: "input-row",
                span { "Colour vision" }
                ComfortButton {
                    label: "Standard",
                    selected: answers.color_vision == ColorVisionProfile::Standard,
                    onclick: move |_| {
                        state.write().environment_questionnaire.color_vision =
                            ColorVisionProfile::Standard;
                    },
                }
                ComfortButton {
                    label: "Red–green ease",
                    selected: answers.color_vision == ColorVisionProfile::DeuteranopiaProtanopia,
                    onclick: move |_| {
                        state.write().environment_questionnaire.color_vision =
                            ColorVisionProfile::DeuteranopiaProtanopia;
                    },
                }
                ComfortButton {
                    label: "Blue–yellow ease",
                    selected: answers.color_vision == ColorVisionProfile::Tritanopia,
                    onclick: move |_| {
                        state.write().environment_questionnaire.color_vision =
                            ColorVisionProfile::Tritanopia;
                    },
                }
                ComfortButton {
                    label: "Grey only",
                    selected: answers.color_vision == ColorVisionProfile::Monochrome,
                    onclick: move |_| {
                        state.write().environment_questionnaire.color_vision =
                            ColorVisionProfile::Monochrome;
                    },
                }
            }
            div {
                class: "input-row",
                span { "Motion" }
                ComfortButton {
                    label: "Full 3D",
                    selected: answers.motion == MotionComfort::Full3dKinetics,
                    onclick: move |_| {
                        state.write().environment_questionnaire.motion =
                            MotionComfort::Full3dKinetics;
                    },
                }
                ComfortButton {
                    label: "Gentle",
                    selected: answers.motion == MotionComfort::GentleDamped,
                    onclick: move |_| {
                        state.write().environment_questionnaire.motion =
                            MotionComfort::GentleDamped;
                    },
                }
                ComfortButton {
                    label: "Still",
                    selected: answers.motion == MotionComfort::StrictZeroMotionStatic,
                    onclick: move |_| {
                        state.write().environment_questionnaire.motion =
                            MotionComfort::StrictZeroMotionStatic;
                    },
                }
            }
            div {
                class: "input-row",
                span { "Workspace" }
                ComfortButton {
                    label: "Compact tiles",
                    selected: answers.density == WorkspaceDensityStyle::CompactTiles,
                    onclick: move |_| {
                        state.write().environment_questionnaire.density =
                            WorkspaceDensityStyle::CompactTiles;
                    },
                }
                ComfortButton {
                    label: "Desk",
                    selected: answers.density == WorkspaceDensityStyle::BalancedDesk,
                    onclick: move |_| {
                        state.write().environment_questionnaire.density =
                            WorkspaceDensityStyle::BalancedDesk;
                    },
                }
                ComfortButton {
                    label: "Open spatial",
                    selected: answers.density == WorkspaceDensityStyle::SpaciousSpatial,
                    onclick: move |_| {
                        state.write().environment_questionnaire.density =
                            WorkspaceDensityStyle::SpaciousSpatial;
                    },
                }
            }
            div {
                class: "input-row",
                span { "Sound cues" }
                ComfortButton {
                    label: "Full",
                    selected: answers.acoustic == AcousticFeedbackPreference::FullBinaural,
                    onclick: move |_| {
                        state.write().environment_questionnaire.acoustic =
                            AcousticFeedbackPreference::FullBinaural;
                    },
                }
                ComfortButton {
                    label: "Subtle",
                    selected: answers.acoustic == AcousticFeedbackPreference::SubtleCues,
                    onclick: move |_| {
                        state.write().environment_questionnaire.acoustic =
                            AcousticFeedbackPreference::SubtleCues;
                    },
                }
                ComfortButton {
                    label: "Silent",
                    selected: answers.acoustic == AcousticFeedbackPreference::Silent,
                    onclick: move |_| {
                        state.write().environment_questionnaire.acoustic =
                            AcousticFeedbackPreference::Silent;
                    },
                }
            }
            button {
                r#type: "button",
                class: "sanctum-issue",
                id: "recommend-environments",
                onclick: move |_| {
                    let mut shell = state.write();
                    let ranked = recommend_environment_options(&shell.environment_questionnaire);
                    shell.recommended_environments = Some(ranked);
                },
                "Suggest three looks"
            }
            if let Some(ranked) = options {
                div {
                    class: "plane-grid",
                    for option in ranked {
                        {
                            let cartridge_id = option.cartridge_id.clone();
                            let preview_id = cartridge_id.clone();
                            let scale = option.ui_scale_percent;
                            let still = option.reduced_motion;
                            let title = option.title.clone();
                            rsx! {
                                div {
                                    key: "{option.rank_label}",
                                    class: "plane-card",
                                    h5 { "{option.rank_label}" }
                                    p { "{title}" }
                                    p { class: "muted", "{option.rationale}" }
                                    div {
                                        class: "input-row",
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                let mut shell = state.write();
                                                let _ = shell.apply_semantic_intent(
                                                    &webizen_ua_os::SemanticIntent::PreviewTheme {
                                                        cartridge_id: preview_id.clone(),
                                                    },
                                                );
                                                shell.ui_scale_percent = scale;
                                                shell.reduced_motion = still;
                                            },
                                            "Preview"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "sanctum-issue",
                                            onclick: move |_| {
                                                let mut shell = state.write();
                                                match shell.adopt_theme(&cartridge_id) {
                                                    Ok(()) => {
                                                        shell.ui_scale_percent = scale;
                                                        shell.reduced_motion = still;
                                                        shell.orb_toast = format!("Using {title}");
                                                    }
                                                    Err(error) => shell.orb_toast = error,
                                                }
                                            },
                                            "Use this look"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ComfortButton(label: &'static str, selected: bool, onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: if selected { "ribbon-btn active" } else { "ribbon-btn" },
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}

#[component]
pub fn ThemeCodesignerPanel(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let prompt = s.theme_codesign_prompt.clone();
    let status = s.theme_codesign_status.clone();
    drop(s);
    rsx! {
        section {
            class: "id-list theme-studio",
            h4 { "Describe a look" }
            p { class: "muted", "Talk in everyday words. Drafts stay public-only, then contrast and motion are checked before you can turn them on." }
            textarea {
                class: "input-field",
                rows: "3",
                value: "{prompt}",
                aria_label: "Describe the look you want",
                placeholder: "Example: curved glass panels with a soft glowing edge and gentle motion…",
                oninput: move |evt| state.write().theme_codesign_prompt = evt.value(),
            }
            div {
                class: "input-row",
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    id: "codesign-theme",
                    onclick: move |_| {
                        let mut shell = state.write();
                        let prompt = shell.theme_codesign_prompt.clone();
                        match co_design_theme_with_llm(&prompt, SENSITIVITY_PUBLIC_ONLY, None) {
                            Ok(cartridge) => {
                                let title = cartridge.title.clone();
                                shell.custom_theme_vibe = cartridge.vibe_source.clone();
                                match shell.theme_registry.insert_collected(cartridge) {
                                    Ok(()) => {
                                        shell.theme_codesign_status = format!(
                                            "Draft “{title}” is collected. Inspect it, then turn it on."
                                        );
                                    }
                                    Err(error) => shell.theme_codesign_status = error,
                                }
                            }
                            Err(error) => shell.theme_codesign_status = error,
                        }
                    },
                    "Draft a look"
                }
            }
            if !status.is_empty() {
                p { class: "muted", role: "status", "{status}" }
            }
        }
    }
}

#[component]
pub fn SharedThemeCollectionPanel(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let collected: Vec<(String, String, bool)> = s
        .collected_themes
        .iter()
        .map(|bundle| {
            (
                bundle.cartridge.cartridge_id.clone(),
                bundle.cartridge.title.clone(),
                bundle.collection_state == ThemeCollectionState::Active,
            )
        })
        .collect();
    drop(s);
    rsx! {
        section {
            class: "id-list theme-studio",
            h4 { "Share and collect looks" }
            p { class: "muted", "Collecting a shared look does not turn it on. You inspect it first." }
            div {
                class: "input-row",
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    id: "export-theme-hmc",
                    onclick: move |_| {
                        let mut shell = state.write();
                        let cartridge = shell.active_cartridge().clone();
                        match export_shared_theme_bundle(&cartridge, "did:qi:author:local") {
                            Ok(bundle) => {
                                if let Some(sess) = session() {
                                    let receipt = export_theme_cartridge_hmc(&sess.host, &cartridge);
                                    shell.theme_codesign_status = receipt.output_summary;
                                }
                                shell.collected_themes.push(bundle);
                            }
                            Err(error) => shell.theme_codesign_status = error,
                        }
                    },
                    "Package current look (.hmc)"
                }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    onclick: move |_| {
                        let mut shell = state.write();
                        let raw = shell.theme_codesign_prompt.clone();
                        match serde_json::from_str(&raw) {
                            Ok(bundle) => match import_shared_theme_bundle(bundle) {
                                Ok(imported) => {
                                    let title = imported.cartridge.title.clone();
                                    let _ = shell.theme_registry.insert_collected(imported.cartridge.clone());
                                    shell.collected_themes.push(imported);
                                    shell.theme_codesign_status = format!(
                                        "Collected “{title}”. It is not on yet."
                                    );
                                }
                                Err(error) => shell.theme_codesign_status = error,
                            },
                            Err(_) => {
                                shell.theme_codesign_status =
                                    "Paste a shared look bundle as JSON to collect it".into();
                            }
                        }
                    },
                    "Collect pasted bundle"
                }
            }
            p { class: "muted", "Paste a shared bundle into the look description box, then collect it. Collecting never activates." }
            ul {
                for (id, title, active) in collected {
                    {
                        let activate_id = id.clone();
                        rsx! {
                            li {
                                key: "{id}",
                                strong { "{title}" }
                                span { class: "muted", if active { " · on" } else { " · collected, off" } }
                                button {
                                    r#type: "button",
                                    class: "ribbon-btn",
                                    onclick: move |_| {
                                        let mut shell = state.write();
                                        if let Some(bundle) = shell
                                            .collected_themes
                                            .iter_mut()
                                            .find(|bundle| bundle.cartridge.cartridge_id == activate_id)
                                        {
                                            match activate_collected_bundle(bundle) {
                                                Ok(()) => {
                                                    let cid = bundle.cartridge.cartridge_id.clone();
                                                    let _ = shell.adopt_theme(&cid);
                                                }
                                                Err(error) => shell.theme_codesign_status = error,
                                            }
                                        }
                                    },
                                    "Turn on"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
