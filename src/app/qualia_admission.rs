// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Standalone shell admission. The launcher supplies an independent project trust anchor.
//! Only a fresh canonical controller proof enables a native session.

use aether::prelude::*;
use aether_qualia_host::{
    qualia_cooperative_core::authoring::DocumentRevision,
    qualia_core_db::vibe_host::native_authority::{
        graph::SignedGraphGrant, NativeAuthority, SessionChallenge, SessionHandle, SignedAdmission,
        TrustedProject,
    },
    vibe::crypto::sha256_bytes,
};
use serde_json::{Map, Value};
use std::{
    cell::RefCell,
    fmt::Write as _,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
use webizen_ua_os::{validate_human_centric_host_nomenclature, AdoptedQualiaFoundation};

const MAX_CONFIG_BYTES: usize = 256 * 1024;
const MAX_SOURCE_BYTES: usize = 64 * 1024;
const JOURNAL_ENTRY_BYTES: usize = 1024 * 1024;
const JOURNAL_ENTRIES: u64 = 256;
const SESSION_LIFETIME_SECONDS: u64 = 300;

const CONFIG_KEYS: &[&str] = &[
    "project_id",
    "issuer_identifier",
    "issuer_public_key_hex",
    "subject",
    "journal_root",
    "document",
    "source_path",
    "signed_admission",
    "signed_graph_grant",
];

struct AdmissionConfig {
    anchor: TrustedProject,
    subject: String,
    journal_root: PathBuf,
    document: DocumentRevision,
    source_path: PathBuf,
    admission: Option<SignedAdmission>,
    graph_grant: Option<SignedGraphGrant>,
}

impl AdmissionConfig {
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        let Value::Object(mut fields) = serde_json::from_slice(bytes)
            .map_err(|error| format!("Invalid Qualia host configuration: {error}"))?
        else {
            return Err("The Qualia host configuration must be an object.".into());
        };
        if fields
            .keys()
            .any(|key| !CONFIG_KEYS.contains(&key.as_str()))
        {
            return Err("The Qualia host configuration has an unknown field.".into());
        }
        let project_id = required_identifier(&fields, "project_id")?;
        let document: DocumentRevision = serde_json::from_value(
            fields
                .remove("document")
                .ok_or("Missing document revision.")?,
        )
        .map_err(|error| format!("Invalid Qualia document revision: {error}"))?;
        if document.project_id != project_id
            || !valid_identifier(&document.document_id)
            || !valid_identifier(&document.source_id)
            || document.source_digest == [0; 32]
        {
            return Err("The document revision is outside the provisioned project.".into());
        }
        let admission = fields
            .remove("signed_admission")
            .filter(|value| !value.is_null())
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| format!("Invalid signed project admission: {error}"))?;
        let graph_grant = fields
            .remove("signed_graph_grant")
            .filter(|value| !value.is_null())
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| format!("Invalid signed graph grant: {error}"))?;

        Ok(Self {
            anchor: TrustedProject {
                project_id,
                issuer_identifier: required_identifier(&fields, "issuer_identifier")?,
                public_key: decode_hex::<32>(&required_identifier(
                    &fields,
                    "issuer_public_key_hex",
                )?)
                .ok_or("The project anchor must be a 32-byte hexadecimal verification key.")?,
            },
            subject: required_identifier(&fields, "subject")?,
            journal_root: absolute_path(&fields, "journal_root")?,
            document,
            source_path: absolute_path(&fields, "source_path")?,
            admission,
            graph_grant,
        })
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.chars().any(char::is_control)
        && value.trim() == value
        && validate_human_centric_host_nomenclature(value).is_ok()
}

fn required_identifier(fields: &Map<String, Value>, name: &str) -> Result<String, String> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| valid_identifier(value))
        .map(str::to_owned)
        .ok_or_else(|| format!("Missing or invalid Qualia host configuration field {name}."))
}

fn absolute_path(fields: &Map<String, Value>, name: &str) -> Result<PathBuf, String> {
    let value = fields
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| value.len() <= 4096 && !value.chars().any(char::is_control))
        .ok_or_else(|| format!("Missing or invalid Qualia host path {name}."))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(format!(
            "The Qualia host field {name} requires an absolute path."
        ));
    }
    Ok(path)
}

fn decode_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    let bytes = value.as_bytes();
    if bytes.len() != N * 2 || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut decoded = [0; N];
    for (slot, pair) in decoded.iter_mut().zip(bytes.chunks_exact(2)) {
        *slot = (pair[0] as char).to_digit(16)? as u8 * 16 + (pair[1] as char).to_digit(16)? as u8;
    }
    Some(decoded)
}

fn bounded_file(path: &Path, maximum: usize) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Cannot inspect the provisioned file: {error}"))?;
    if !metadata.is_file() || metadata.len() > maximum as u64 {
        return Err("The provisioned file violates the regular-file or size limit.".into());
    }

    let file =
        File::open(path).map_err(|error| format!("Cannot open the provisioned file: {error}"))?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("The provisioned file is no longer a regular file.".into());
    }
    let mut bytes = Vec::new();
    file.take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read the provisioned file: {error}"))?;
    if bytes.len() > maximum {
        return Err("The provisioned file exceeds its size limit.".into());
    }
    Ok(bytes)
}

fn verify_source(path: &Path, revision: &DocumentRevision) -> Result<(), String> {
    let bytes = bounded_file(path, MAX_SOURCE_BYTES)?;
    if std::str::from_utf8(&bytes).is_err() || sha256_bytes(&bytes) != revision.source_digest {
        return Err(
            "The exact UTF-8 source differs from the provisioned document revision.".into(),
        );
    }
    Ok(())
}

struct ProjectAdmission {
    authority: Arc<NativeAuthority>,
    subject: String,
    document: DocumentRevision,
    source_path: PathBuf,
    graph_grant: Option<SignedGraphGrant>,
}

impl ProjectAdmission {
    fn open(path: &Path) -> Result<Self, String> {
        if !path.is_absolute() {
            return Err("WEBIZEN_QUALIA_HOST_CONFIG requires an absolute path.".into());
        }
        // The launcher-selected trust anchor must remain separate from the mutable shell vault.
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("Cannot resolve the Qualia host configuration: {error}"))?;
        if crate::state::session()
            .and_then(|session| session.host.storage.canonicalize().ok())
            .is_some_and(|storage| canonical.starts_with(storage))
        {
            return Err("Provision the Qualia host trust file outside the shell vault.".into());
        }
        let config = AdmissionConfig::parse(&bounded_file(path, MAX_CONFIG_BYTES)?)?;
        verify_source(&config.source_path, &config.document)?;
        let authority = if let Some(admission) = config.admission {
            NativeAuthority::open(
                &config.journal_root,
                config.anchor,
                admission,
                config.document.clone(),
                JOURNAL_ENTRY_BYTES,
                JOURNAL_ENTRIES,
            )
        } else {
            NativeAuthority::open_retained(
                &config.journal_root,
                config.anchor,
                config.document.clone(),
                JOURNAL_ENTRY_BYTES,
                JOURNAL_ENTRIES,
            )
        }?;

        Ok(Self {
            authority: Arc::new(authority),
            subject: config.subject,
            document: config.document,
            source_path: config.source_path,
            graph_grant: config.graph_grant,
        })
    }
}

struct OwnedSession {
    authority: Arc<NativeAuthority>,
    handle: Option<SessionHandle>,
}

impl OwnedSession {
    fn disconnect(&mut self) -> Result<(), String> {
        if let Some(handle) = &self.handle {
            self.authority.disconnect(handle)?;
            self.handle = None;
        }
        Ok(())
    }
}

impl Drop for OwnedSession {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

#[derive(Default)]
struct AdmissionController {
    project: Option<ProjectAdmission>,
    pending: Option<SessionChallenge>,
    live: Option<OwnedSession>,
}

impl AdmissionController {
    fn begin(&mut self, config: &Path) -> Result<String, String> {
        if self.live.is_some() {
            return Err(
                "Disconnect the current Qualia session before requesting a challenge.".into(),
            );
        }
        self.cancel();
        if self.project.is_none() {
            self.project = Some(ProjectAdmission::open(config)?);
        }
        let project = self
            .project
            .as_ref()
            .ok_or("Native project admission is unavailable.")?;
        verify_source(&project.source_path, &project.document)?;
        let challenge = project
            .authority
            .begin_session(&project.subject, SESSION_LIFETIME_SECONDS)?;
        let mut digest = String::with_capacity(64);
        for byte in challenge.message_digest() {
            let _ = write!(&mut digest, "{byte:02x}");
        }
        self.pending = Some(challenge);
        Ok(digest)
    }

    fn authenticate(&mut self, signature: &[u8; 64]) -> Result<AdoptedQualiaFoundation, String> {
        let project = self
            .project
            .as_ref()
            .ok_or("Native project admission is unavailable.")?;
        verify_source(&project.source_path, &project.document)?;
        let challenge = self
            .pending
            .take()
            .ok_or("Request a native controller challenge first.")?;
        let handle = match project
            .authority
            .complete_session(challenge.clone(), signature)
        {
            Ok(handle) => handle,
            Err(error) => {
                let _ = project.authority.cancel_session_challenge(&challenge);
                return Err(error);
            }
        };
        let live = OwnedSession {
            authority: Arc::clone(&project.authority),
            handle: Some(handle.clone()),
        };
        let evidence = live.authority.workflow_evidence(&handle)?;
        if evidence.target != project.document {
            return Err(
                "The admitted source revision changed. Reconcile the project before connecting."
                    .into(),
            );
        }
        let foundation = AdoptedQualiaFoundation::connect(
            Arc::clone(&live.authority),
            handle,
            project.graph_grant.clone(),
        )?;
        self.live = Some(live);
        Ok(foundation)
    }

    fn disconnect(&mut self) -> Result<(), String> {
        self.live
            .take()
            .map_or(Ok(()), |mut live| live.disconnect())
    }

    fn cancel(&mut self) {
        if let (Some(project), Some(challenge)) = (&self.project, self.pending.take()) {
            let _ = project.authority.cancel_session_challenge(&challenge);
        }
    }
}

impl Drop for AdmissionController {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Mount once at the shell owner. Absence of an operator configuration leaves local rehearsal available.
#[component]
pub(crate) fn QualiaAdmissionPanel(
    mut adopted: Signal<Option<AdoptedQualiaFoundation>>,
) -> Element {
    let config = use_hook(|| std::env::var_os("WEBIZEN_QUALIA_HOST_CONFIG").map(PathBuf::from));
    let controller = use_hook(|| Rc::new(RefCell::new(AdmissionController::default())));
    let mut challenge = use_signal(String::new);
    let mut signature = use_signal(String::new);
    let mut status = use_signal(String::new);

    let Some(config) = config else {
        return rsx! {};
    };
    let connected = adopted.read().is_some();
    let digest = challenge();
    let waiting = !digest.is_empty();
    let controller_for_begin = Rc::clone(&controller);
    let controller_for_finish = Rc::clone(&controller);
    let controller_for_cancel = Rc::clone(&controller);

    rsx! {
        details {
            class: "qualia-admission",
            summary { "QualiaDB project session" }
            if connected {
                p { role: "status", "Controller proof accepted. QualiaDB revalidates authority on each operation." }
                button {
                    r#type: "button",
                    onclick: move |_| {
                        let result = controller.borrow_mut().disconnect();
                        adopted.set(None);
                        status.set(match result {
                            Ok(()) => "Native project session disconnected.".into(),
                            Err(error) => format!("Project access closed in the shell. Native disconnect could not be confirmed: {error}"),
                        });
                    },
                    "Disconnect project session"
                }
            } else if waiting {
                p { "Sign this SHA-256 challenge with the admitted controller using qualia.native-authoring-session.v1." }
                code { "{digest}" }
                form {
                    onsubmit: move |event| {
                        event.prevent_default();
                        let proof = signature();
                        let Some(decoded) = decode_hex::<64>(proof.trim()) else {
                            status.set("Enter the controller signature as 128 hexadecimal characters.".into());
                            return;
                        };
                        signature.set(String::new());
                        let result = controller_for_finish.borrow_mut().authenticate(&decoded);
                        match result {
                            Ok(foundation) => {
                                adopted.set(Some(foundation));
                                challenge.set(String::new());
                                status.set(String::new());
                            }
                            Err(error) => {
                                if controller_for_finish.borrow().pending.is_none() {
                                    challenge.set(String::new());
                                }
                                status.set(error);
                            }
                        }
                    },
                    label {
                        "Controller signature"
                        input {
                            r#type: "text",
                            autocomplete: "off",
                            maxlength: "128",
                            spellcheck: "false",
                            value: "{signature}",
                            oninput: move |event| signature.set(event.value()),
                        }
                    }
                    button { r#type: "submit", "Verify signature" }
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            controller_for_cancel.borrow_mut().cancel();
                            challenge.set(String::new());
                            signature.set(String::new());
                            status.set("Native project challenge cancelled.".into());
                        },
                        "Cancel challenge"
                    }
                }
            } else {
                p { "Awaiting External Signer / Verifier" }
                button {
                    r#type: "button",
                    onclick: move |_| {
                        match controller_for_begin.borrow_mut().begin(&config) {
                            Ok(digest) => {
                                signature.set(String::new());
                                challenge.set(digest);
                                status.set(String::new());
                            }
                            Err(error) => status.set(error),
                        }
                    },
                    "Request controller challenge"
                }
            }
            if !status.read().is_empty() {
                p { role: "status", "{status}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_controller_proof_decoding_requires_exact_hex_length() {
        let signature = format!("{}{}", "01".repeat(63), "fF");
        let result = decode_hex::<64>(&signature).unwrap();
        assert_eq!(result[..63], [1; 63]);
        assert_eq!(result[63], 255);
        assert!(decode_hex::<64>(&signature[..127]).is_none());
        assert!(decode_hex::<64>(&signature.replace('f', "g")).is_none());
    }

    #[test]
    fn provisioned_authority_rejects_relative_paths_and_wrong_project_revisions() {
        let project_path = if cfg!(windows) {
            "C:/qualia-project"
        } else {
            "/qualia-project"
        };
        let mut config = serde_json::json!({
            "project_id": "project:test",
            "issuer_identifier": "did:example:issuer",
            "issuer_public_key_hex": "01".repeat(32),
            "subject": "did:example:author",
            "journal_root": project_path,
            "document": {
                "project_id": "project:test",
                "document_id": "document:test",
                "source_id": "source:test",
                "revision": 1,
                "source_digest": sha256_bytes(b"42"),
                "author_subject": null
            },
            "source_path": format!("{project_path}/source.vibe")
        });
        assert!(AdmissionConfig::parse(config.to_string().as_bytes()).is_ok());
        config["source_path"] = Value::String("source.vibe".into());
        assert!(AdmissionConfig::parse(config.to_string().as_bytes()).is_err());
        config["source_path"] = Value::String(format!("{project_path}/source.vibe"));
        config["document"]["project_id"] = Value::String("project:another".into());
        assert!(AdmissionConfig::parse(config.to_string().as_bytes()).is_err());
    }
}
