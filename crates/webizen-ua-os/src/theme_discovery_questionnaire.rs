// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Local sensory and accessibility intake. Answers never leave the device
//! (`SENSITIVITY_PRIVATE`). The recommender returns three ranked environment options.

use crate::llm_management::SENSITIVITY_RESTRICTED;
use crate::theme_cartridge::{builtin_cartridges, ThemeCartridgeManifest};
use serde::{Deserialize, Serialize};

/// Answers stay on this device. Never copy them into a public or remote prompt.
pub const SENSITIVITY_PRIVATE: u8 = SENSITIVITY_RESTRICTED;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContrastGlareComfort {
    SoftAmbient,
    Balanced,
    HighClarity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorVisionProfile {
    Standard,
    DeuteranopiaProtanopia,
    Tritanopia,
    Monochrome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionComfort {
    Full3dKinetics,
    GentleDamped,
    StrictZeroMotionStatic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceDensityStyle {
    CompactTiles,
    BalancedDesk,
    SpaciousSpatial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcousticFeedbackPreference {
    FullBinaural,
    SubtleCues,
    Silent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserEnvironmentQuestionnaire {
    pub reading_scale_percent: u16,
    pub contrast_glare: ContrastGlareComfort,
    pub color_vision: ColorVisionProfile,
    pub motion: MotionComfort,
    pub density: WorkspaceDensityStyle,
    pub acoustic: AcousticFeedbackPreference,
}

impl Default for UserEnvironmentQuestionnaire {
    fn default() -> Self {
        Self {
            reading_scale_percent: 115,
            contrast_glare: ContrastGlareComfort::Balanced,
            color_vision: ColorVisionProfile::Standard,
            motion: MotionComfort::GentleDamped,
            density: WorkspaceDensityStyle::BalancedDesk,
            acoustic: AcousticFeedbackPreference::SubtleCues,
        }
    }
}

impl UserEnvironmentQuestionnaire {
    pub fn clamp_reading_scale(&mut self) {
        self.reading_scale_percent = self.reading_scale_percent.clamp(100, 165);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendedEnvironmentOption {
    pub rank_label: &'static str,
    pub cartridge_id: String,
    pub title: String,
    pub ui_scale_percent: u16,
    pub reduced_motion: bool,
    pub rationale: String,
}

fn cartridge<'a>(id: &str, all: &'a [ThemeCartridgeManifest; 6]) -> &'a ThemeCartridgeManifest {
    all.iter()
        .find(|c| c.cartridge_id == id)
        .unwrap_or(&all[0])
}

/// Rank three tailored environments: primary match, high-clarity comfort, spatial exploration.
pub fn recommend_environment_options(
    answers: &UserEnvironmentQuestionnaire,
) -> [RecommendedEnvironmentOption; 3] {
    let mut answers = answers.clone();
    answers.clamp_reading_scale();
    let all = builtin_cartridges();
    let scale = answers.reading_scale_percent;
    let reduced = matches!(answers.motion, MotionComfort::StrictZeroMotionStatic);

    let primary_id = match (answers.density, answers.contrast_glare, answers.motion) {
        (WorkspaceDensityStyle::SpaciousSpatial, _, MotionComfort::Full3dKinetics) => {
            "theme-spatial-commons"
        }
        (WorkspaceDensityStyle::CompactTiles, _, MotionComfort::Full3dKinetics) => {
            "theme-cyber-grid"
        }
        (_, ContrastGlareComfort::HighClarity, _) => "theme-obsidian",
        (WorkspaceDensityStyle::SpaciousSpatial, _, _) => "theme-holoslate",
        (_, ContrastGlareComfort::SoftAmbient, _) => "theme-commons",
        _ => "theme-holosonic",
    };
    let primary_id = match answers.color_vision {
        ColorVisionProfile::Monochrome | ColorVisionProfile::DeuteranopiaProtanopia => {
            "theme-obsidian"
        }
        ColorVisionProfile::Tritanopia if primary_id == "theme-commons" => "theme-holosonic",
        _ => primary_id,
    };

    let comfort_id = if primary_id == "theme-obsidian" {
        "theme-holosonic"
    } else {
        "theme-obsidian"
    };
    let explore_id = match answers.density {
        WorkspaceDensityStyle::CompactTiles => "theme-cyber-grid",
        WorkspaceDensityStyle::SpaciousSpatial => "theme-spatial-commons",
        WorkspaceDensityStyle::BalancedDesk => "theme-holoslate",
    };
    let explore_id = if explore_id == primary_id {
        "theme-spatial-commons"
    } else {
        explore_id
    };

    let primary = cartridge(primary_id, &all);
    let comfort = cartridge(comfort_id, &all);
    let explore = cartridge(explore_id, &all);

    [
        RecommendedEnvironmentOption {
            rank_label: "Primary Tailored Match",
            cartridge_id: primary.cartridge_id.clone(),
            title: primary.title.clone(),
            ui_scale_percent: scale,
            reduced_motion: reduced,
            rationale: format!(
                "Matches your reading size ({scale}%), contrast comfort, and motion preference while keeping every app button in the same place."
            ),
        },
        RecommendedEnvironmentOption {
            rank_label: "High-Clarity Comfort Variant",
            cartridge_id: comfort.cartridge_id.clone(),
            title: comfort.title.clone(),
            ui_scale_percent: scale.max(115),
            reduced_motion: true,
            rationale: "High ink-on-panel contrast and still motion for glare, fatigue, or vestibular rest.".into(),
        },
        RecommendedEnvironmentOption {
            rank_label: "Spatial Exploration Variant",
            cartridge_id: explore.cartridge_id.clone(),
            title: explore.title.clone(),
            ui_scale_percent: scale,
            reduced_motion: reduced,
            rationale: "Opens a deeper spatial stage with the same labels and destinations as the desk layout.".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_clarity_and_static_motion_prefer_obsidian_comfort() {
        let mut q = UserEnvironmentQuestionnaire::default();
        q.contrast_glare = ContrastGlareComfort::HighClarity;
        q.motion = MotionComfort::StrictZeroMotionStatic;
        q.reading_scale_percent = 145;
        let options = recommend_environment_options(&q);
        assert_eq!(options[0].rank_label, "Primary Tailored Match");
        assert_eq!(options[0].cartridge_id, "theme-obsidian");
        assert!(options[0].reduced_motion);
        assert_eq!(options[1].rank_label, "High-Clarity Comfort Variant");
        assert_eq!(options[2].rank_label, "Spatial Exploration Variant");
        assert_eq!(options[0].ui_scale_percent, 145);
    }

    #[test]
    fn spacious_full_kinetics_recommends_orbital_amphitheatre() {
        let mut q = UserEnvironmentQuestionnaire::default();
        q.density = WorkspaceDensityStyle::SpaciousSpatial;
        q.motion = MotionComfort::Full3dKinetics;
        let options = recommend_environment_options(&q);
        assert_eq!(options[0].cartridge_id, "theme-spatial-commons");
    }

    #[test]
    fn answers_are_tagged_private() {
        assert_eq!(SENSITIVITY_PRIVATE, SENSITIVITY_RESTRICTED);
        assert!(SENSITIVITY_PRIVATE > 0);
    }
}
