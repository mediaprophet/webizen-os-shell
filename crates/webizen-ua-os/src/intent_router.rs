// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Classifies local, user-entered shell commands. Dispatch and authority belong to the shell.

use crate::{
    browser_app::{resolve_mode_for_uri, BrowserExecutionMode},
    host_stewardship::validate_human_centric_host_nomenclature,
    identifiers::{q_hash_60, IdentifierFabricPlane},
};
use serde::{Deserialize, Deserializer, Serialize};
use vibescript_abi::NQuin48;

// The explicit deserializers resolve owned input against fixed labels. The alias
// keeps Serde from inferring a borrowed lifetime from the output type.
type IntentLabel = &'static str;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticIntentAction {
    OpenVolume {
        #[serde(deserialize_with = "deserialize_stage_slug")]
        stage_slug: IntentLabel,
        summary: String,
    },
    NavigateBrowser {
        uri: String,
        mode: BrowserExecutionMode,
    },
    OpenHolo10dSpecimen {
        specimen_query: String,
    },
    StageGgufForP64Conversion {
        gguf_path: String,
    },
    InvokeGovernedAgent {
        agent_slug: String,
        prompt: String,
    },
    SearchSemanticLibrary {
        query: String,
    },
    RunThreeNodeMeshSimulation,
    BuildTriTargetDebianSuite,
    TriggerPrincipalOkCheckIn,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifiedSemanticIntent {
    pub raw_input: String,
    #[serde(deserialize_with = "deserialize_badge")]
    pub badge: IntentLabel,
    pub target_plane: IdentifierFabricPlane,
    pub action: SemanticIntentAction,
    pub explanation: String,
    /// Parity for the classification record.
    pub sealed_nquin_parity: u64,
}

const MAX_INTENT_BYTES: usize = 4_096;

type VolumeName = (&'static str, &'static str, IdentifierFabricPlane);
const VOLUMES: &[VolumeName] = &[
    ("talk", "Talk", IdentifierFabricPlane::ClaimOpinion),
    ("mail", "Mail", IdentifierFabricPlane::SpatiotemporalHandle),
    (
        "directory",
        "Directory",
        IdentifierFabricPlane::NaturalAgent,
    ),
    (
        "browser",
        "Browser",
        IdentifierFabricPlane::SpatiotemporalHandle,
    ),
    ("keep", "Sanctuary", IdentifierFabricPlane::Instrument),
    ("library", "Library", IdentifierFabricPlane::ClaimOpinion),
    (
        "instruments",
        "Instruments",
        IdentifierFabricPlane::Instrument,
    ),
    ("poet", "Poet & Synth", IdentifierFabricPlane::Instrument),
    ("wellfair", "WellFair", IdentifierFabricPlane::ClaimOpinion),
    ("anatomy", "Anatomy", IdentifierFabricPlane::ClaimOpinion),
    ("wallet", "Wallet", IdentifierFabricPlane::Instrument),
    ("projects", "Projects", IdentifierFabricPlane::ClaimOpinion),
    ("settings", "Settings", IdentifierFabricPlane::Instrument),
    ("admin", "Admin", IdentifierFabricPlane::Instrument),
];

impl SemanticIntentAction {
    fn badge(&self) -> &'static str {
        match self {
            Self::OpenVolume { .. } => "Open volume",
            Self::NavigateBrowser { .. } => "Navigate",
            Self::OpenHolo10dSpecimen { .. } => "HOLO specimen",
            Self::StageGgufForP64Conversion { .. } => "Stage model",
            Self::InvokeGovernedAgent { .. } => "Review agent prompt",
            Self::SearchSemanticLibrary { .. } => "Search Library",
            Self::RunThreeNodeMeshSimulation => "Simulate mesh",
            Self::BuildTriTargetDebianSuite => "Build Debian suite",
            Self::TriggerPrincipalOkCheckIn => "Open care check-in",
        }
    }
}

fn deserialize_stage_slug<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<&'static str, D::Error> {
    let value = String::deserialize(deserializer)?;
    VOLUMES
        .iter()
        .find_map(|(slug, _, _)| (*slug == value).then_some(*slug))
        .ok_or_else(|| serde::de::Error::custom("Unknown semantic intent volume"))
}

fn deserialize_badge<'de, D: Deserializer<'de>>(deserializer: D) -> Result<&'static str, D::Error> {
    const BADGES: &[&str] = &[
        "Open volume",
        "Navigate",
        "HOLO specimen",
        "Stage model",
        "Review agent prompt",
        "Search Library",
        "Simulate mesh",
        "Build Debian suite",
        "Open care check-in",
    ];
    let value = String::deserialize(deserializer)?;
    BADGES
        .iter()
        .copied()
        .find(|badge| *badge == value)
        .ok_or_else(|| serde::de::Error::custom("Unknown semantic intent badge"))
}

pub fn classify_semantic_intent(raw_input: &str) -> Result<ClassifiedSemanticIntent, String> {
    validate_human_centric_host_nomenclature(raw_input)?;
    let raw_input = raw_input.trim();
    if raw_input.is_empty()
        || raw_input.len() > MAX_INTENT_BYTES
        || raw_input.chars().any(|ch| ch.is_control() && ch != '\t')
    {
        return Err("Enter a one-line command of at most 4096 bytes.".into());
    }

    let unquoted = raw_input
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw_input);
    let normalized = unquoted
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    let (action, target_plane, explanation) = if let Some(addressed) = raw_input.strip_prefix('@') {
        let (slug, prompt) = addressed
            .split_once(char::is_whitespace)
            .ok_or("Enter an @agent slug followed by a prompt.")?;
        let prompt = prompt.trim();
        if slug.is_empty()
            || slug.len() > 64
            || !slug
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
            || prompt.is_empty()
        {
            return Err("Enter a valid @agent slug and a prompt.".into());
        }
        (
            SemanticIntentAction::InvokeGovernedAgent {
                agent_slug: slug.to_ascii_lowercase(),
                prompt: prompt.to_owned(),
            },
            IdentifierFabricPlane::Instrument,
            "Prepare a prompt for review under the named agent's scope.".to_owned(),
        )
    } else if normalized.starts_with("holo:") {
        let query = unquoted
            .split_once(':')
            .map_or("", |(_, value)| value.trim());
        if query.is_empty() {
            return Err("Enter a HOLO specimen name or path after holo:.".into());
        }
        holo_intent(query)
    } else if ["https://", "http://", "qualia://", "webizen://"]
        .iter()
        .any(|scheme| normalized.starts_with(*scheme))
    {
        let authority = unquoted.split_once("://").map_or("", |(_, rest)| {
            rest.split(['/', '?', '#']).next().unwrap_or("")
        });
        if authority.is_empty() || unquoted.chars().any(char::is_whitespace) {
            return Err("Enter a browser address with a host and no whitespace.".into());
        }
        let mode = resolve_mode_for_uri(unquoted, BrowserExecutionMode::QualiaNative);
        (
            SemanticIntentAction::NavigateBrowser {
                uri: unquoted.to_owned(),
                mode,
            },
            IdentifierFabricPlane::SpatiotemporalHandle,
            format!("Open the address in {}.", mode.title()),
        )
    } else if normalized.ends_with(".gguf") {
        (
            SemanticIntentAction::StageGgufForP64Conversion {
                gguf_path: unquoted.to_owned(),
            },
            IdentifierFabricPlane::Instrument,
            "Prepare a local GGUF source for conversion review.".to_owned(),
        )
    } else if normalized.ends_with(".10d") {
        holo_intent(unquoted)
    } else {
        match normalized.as_str() {
            "mesh:sim" | "simulate mesh" => (
                SemanticIntentAction::RunThreeNodeMeshSimulation,
                IdentifierFabricPlane::Instrument,
                "Run the isolated three-node lifecycle simulation.".to_owned(),
            ),
            "deb:build" | "package deb" => (
                SemanticIntentAction::BuildTriTargetDebianSuite,
                IdentifierFabricPlane::Instrument,
                "Build Debian archives from the active host's package trees.".to_owned(),
            ),
            "care:ok" | "i am ok" => (
                SemanticIntentAction::TriggerPrincipalOkCheckIn,
                IdentifierFabricPlane::ClaimOpinion,
                "Open urgent care alerts for a scoped principal check-in.".to_owned(),
            ),
            candidate => match find_volume(candidate) {
                Some((slug, label, plane)) => (
                    SemanticIntentAction::OpenVolume {
                        stage_slug: slug,
                        summary: label.to_owned(),
                    },
                    plane,
                    format!("Open the {label} volume."),
                ),
                None => (
                    SemanticIntentAction::SearchSemanticLibrary {
                        query: raw_input.to_owned(),
                    },
                    IdentifierFabricPlane::ClaimOpinion,
                    "Search the Semantic Library with the entered text.".to_owned(),
                ),
            },
        }
    };

    let badge = action.badge();
    let quin = NQuin48::invariant(
        &format!("urn:webizen:semantic-intent:{:015x}", q_hash_60(raw_input)),
        "urn:webizen:semantic-intent:classifiedAs",
        q_hash_60(badge),
    );
    Ok(ClassifiedSemanticIntent {
        raw_input: raw_input.to_owned(),
        badge,
        target_plane,
        action,
        explanation,
        sealed_nquin_parity: quin.parity(),
    })
}

fn holo_intent(query: &str) -> (SemanticIntentAction, IdentifierFabricPlane, String) {
    (
        SemanticIntentAction::OpenHolo10dSpecimen {
            specimen_query: query.to_owned(),
        },
        IdentifierFabricPlane::ClaimOpinion,
        "Open the local HOLO specimen in the native viewport.".to_owned(),
    )
}

fn find_volume(query: &str) -> Option<VolumeName> {
    let candidate = ["open ", "go to ", "show "]
        .iter()
        .find_map(|prefix| query.strip_prefix(*prefix))
        .unwrap_or(query);
    let candidate = match candidate {
        "sanctuary" | "sanctuary vault" => "keep",
        "well fair" | "wellfair care" => "wellfair",
        "synth" | "poet & synth" | "poet synth" => "poet",
        "instrument" | "models" => "instruments",
        "project" => "projects",
        _ => candidate,
    };

    VOLUMES
        .iter()
        .copied()
        .find(|(slug, _, _)| *slug == candidate)
        .or_else(|| {
            if !candidate.is_ascii() || candidate.len() < 4 || candidate.contains(' ') {
                return None;
            }
            let mut matches = VOLUMES
                .iter()
                .copied()
                .filter(|(slug, _, _)| one_edit_apart(candidate.as_bytes(), slug.as_bytes()));
            let first = matches.next()?;
            matches.next().is_none().then_some(first)
        })
}

fn one_edit_apart(left: &[u8], right: &[u8]) -> bool {
    let first_mismatch = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    let left = &left[first_mismatch..];
    let right = &right[first_mismatch..];
    match (left, right) {
        ([], []) => false,
        ([_, rest @ ..], _) if left.len() == right.len() + 1 => rest == right,
        (_, [_, rest @ ..]) if right.len() == left.len() + 1 => left == rest,
        ([a, b, rest @ ..], [c, d, tail @ ..]) if a == d && b == c => rest == tail,
        ([_, rest @ ..], [_, tail @ ..]) => rest == tail,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_uris_models_specimens_agents_and_rejects_forbidden_nomenclature() {
        assert!(matches!(
            classify_semantic_intent("qualia://chora/universe")
                .unwrap()
                .action,
            SemanticIntentAction::NavigateBrowser {
                mode: BrowserExecutionMode::QualiaNative,
                ..
            }
        ));
        assert!(matches!(
            classify_semantic_intent("https://web.civics.au/work")
                .unwrap()
                .action,
            SemanticIntentAction::NavigateBrowser {
                mode: BrowserExecutionMode::HybridAugmented,
                ..
            }
        ));
        assert_eq!(
            classify_semantic_intent(r#""C:\Local Models\model.GGUF""#)
                .unwrap()
                .action,
            SemanticIntentAction::StageGgufForP64Conversion {
                gguf_path: r"C:\Local Models\model.GGUF".to_owned()
            }
        );
        assert_eq!(
            classify_semantic_intent("holo:poet-hyperbolic-mobius-v2")
                .unwrap()
                .action,
            SemanticIntentAction::OpenHolo10dSpecimen {
                specimen_query: "poet-hyperbolic-mobius-v2".to_owned()
            }
        );
        let agent = classify_semantic_intent("@Research-Tool Keep  Case").unwrap();
        assert_eq!(
            agent.action,
            SemanticIntentAction::InvokeGovernedAgent {
                agent_slug: "research-tool".to_owned(),
                prompt: "Keep  Case".to_owned()
            }
        );
        assert_eq!(
            serde_json::from_str::<ClassifiedSemanticIntent>(
                &serde_json::to_string(&agent).unwrap()
            )
            .unwrap(),
            agent
        );
        assert!(classify_semantic_intent("@agent").is_err());
        assert!(classify_semantic_intent("https:///").is_err());
        assert!(classify_semantic_intent("owl:thing").is_err());
    }

    #[test]
    fn resolves_explicit_commands_and_unambiguous_volume_typos() {
        for (input, expected) in [
            ("mesh:sim", SemanticIntentAction::RunThreeNodeMeshSimulation),
            (
                "package deb",
                SemanticIntentAction::BuildTriTargetDebianSuite,
            ),
            ("I am OK", SemanticIntentAction::TriggerPrincipalOkCheckIn),
        ] {
            assert_eq!(classify_semantic_intent(input).unwrap().action, expected);
        }
        let volume = classify_semantic_intent("open libarry").unwrap();
        assert!(matches!(
            volume.action,
            SemanticIntentAction::OpenVolume {
                stage_slug: "library",
                ..
            }
        ));
        let serialized = serde_json::to_string(&volume).unwrap();
        assert_eq!(
            serde_json::from_str::<ClassifiedSemanticIntent>(&serialized).unwrap(),
            volume,
        );
        assert_eq!(
            classify_semantic_intent("riparian catchment report")
                .unwrap()
                .action,
            SemanticIntentAction::SearchSemanticLibrary {
                query: "riparian catchment report".to_owned()
            }
        );
    }
}
