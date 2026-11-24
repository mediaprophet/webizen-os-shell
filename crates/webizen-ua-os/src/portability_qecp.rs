// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Local, bounded portability operations. Pod destinations are metadata only.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

use qualia_core_db::{
    bundle::semantic_model::SemanticModelBuilder,
    crypto::network::{aead, fill_os},
    q42_volume::{
        Q42RangeQueryCursor, Q42RangeQueryPattern, Q42RangeQueryPlan, WasmSliceQ42Reader,
        MAX_COMPRESSED_SUPERBLOCK_SIZE, SUPERBLOCK_SIZE,
    },
    q_hash, NQuin,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha3::{Digest, Sha3_256};
use vibescript_abi::NQuin48;

use crate::{
    browser_app::{BrowserSocialGraph, BROWSER_SOCIAL_STORE_REL_PATH},
    host_stewardship::validate_human_centric_host_nomenclature,
    identifiers::{q_hash_60, VC_ORIGIN_INTEGRITY_DISCLAIMER},
    sanctuary_onboarding::{gf_multiply, interpolate_at_zero},
    semantic_fs::{
        CommonsShareScope, SemanticAssetEntry, SemanticFileSystem, HYPERMEDIA_LIBRARY_REL_PATH,
    },
};

const BACKUP_ROOT: &str = "backups/q42j";
const ARCHIVE_MAGIC: &[u8; 8] = b"WZQ42J\x01\0";
const SHARE_MAGIC: &[u8; 8] = b"WZSHAR\x01\0";
const ARCHIVE_HEADER_BYTES: usize = 8 + 12 + 16;
const SHARE_BYTES: usize = 8 + 16 + 3 + 32;
const MAX_ARCHIVE_PAYLOAD: usize = 4 * 1024 * 1024;
const MAX_SOURCE_STORE: usize = 8 * 1024 * 1024;
const MAX_TURTLE_BYTES: usize = 128 * 1024;
const MAX_TURTLE_TRIPLES: usize = 1024;
const MAX_QECP_BYTES: usize = 32 * 1024;
const MAX_SHARE_COUNT: u8 = 16;
const SOLID_CONTEXT: &str = "https://webizen.org/q42/portable-local.ttl";
const SOLID_NAMESPACE: &str = "https://webizen.org/q42#";
const DCTERMS_TITLE: &str = "http://purl.org/dc/terms/title";
const PROV_DERIVATION: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShamirQ42jBackupArchive {
    pub archive_id: String,
    pub threshold_m: u8,
    pub total_n: u8,
    /// SHA3-256 of the authenticated encrypted journal bytes.
    pub payload_sha3_256_hex: String,
    pub payload_bytes: usize,
    pub share_files: Vec<PathBuf>,
    pub included_sections: Vec<String>,
    pub sealed_nquin_parity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolidTurtleSyncReceipt {
    pub direction: String,
    pub pod_endpoint: String,
    pub turtle_document: String,
    pub triple_count: usize,
    pub sealed_nquin_parity: u64,
    pub file_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QecpCredentialEnvelope {
    pub credential_id: String,
    pub document_title: String,
    pub issuer_pairwise_did: String,
    pub claim_summary: String,
    pub c2pa_jumbf_label: String,
    pub cbor_ld_tag_hex: String,
    pub origin_integrity_disclaimer: String,
    pub signature_scheme: String,
    pub sealed_nquin_parity: u64,
}

/// Export only selected public metadata, without credentials or restricted records.
/// The files are locally staged until the operator places shares in independent custody.
pub fn export_shamir_q42j_backup(
    storage: &Path,
    threshold_m: u8,
    total_n: u8,
) -> Result<ShamirQ42jBackupArchive, String> {
    check_threshold(threshold_m, total_n)?;
    let sections = collect_portable_sections(storage)?;
    let payload = serde_json::to_vec(&sections).map_err(|e| e.to_string())?;
    if payload.len() > MAX_ARCHIVE_PAYLOAD {
        return Err("The public backup exceeds the journal size limit.".into());
    }

    let archive_id_bytes = random_array::<16>()?;
    let archive_id = hex(&archive_id_bytes);
    let nonce = random_array::<12>()?;
    let key = random_array::<32>()?;
    let mut ciphertext = payload;
    let mut tag = [0; 16];
    aead::encrypt_in_place(
        &key,
        &nonce,
        &archive_aad(&archive_id_bytes, threshold_m, total_n),
        &mut ciphertext,
        &mut tag,
    )
    .map_err(|_| "Unable to authenticate and encrypt the public journal.")?;
    let mut journal = Vec::with_capacity(ARCHIVE_HEADER_BYTES + ciphertext.len());
    journal.extend_from_slice(ARCHIVE_MAGIC);
    journal.extend_from_slice(&nonce);
    journal.extend_from_slice(&tag);
    journal.extend_from_slice(&ciphertext);

    let base = storage_directory(storage, BACKUP_ROOT, true)?;
    let transaction = base.join(format!(".{archive_id}.pending"));
    let directory = base.join(&archive_id);
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut permissions = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        permissions.mode(0o700);
    }
    permissions
        .create(&transaction)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        write_new(&transaction.join("journal.q42j"), &journal)?;
        let shares = split_key(&key, threshold_m, total_n)?;
        for (index, share) in (1..=total_n).zip(shares) {
            let mut bytes = Vec::with_capacity(SHARE_BYTES);
            bytes.extend_from_slice(SHARE_MAGIC);
            bytes.extend_from_slice(&archive_id_bytes);
            bytes.extend_from_slice(&[threshold_m, total_n, index]);
            bytes.extend_from_slice(&share);
            write_new(&transaction.join(share_name(index)), &bytes)?;
        }
        let digest = sha3_hex(&journal);
        let mut receipt = ShamirQ42jBackupArchive {
            archive_id: archive_id.clone(),
            threshold_m,
            total_n,
            payload_sha3_256_hex: digest,
            payload_bytes: ciphertext.len(),
            share_files: Vec::new(),
            included_sections: sections.keys().cloned().collect(),
            sealed_nquin_parity: 0,
        };
        receipt.sealed_nquin_parity = journal_parity(&receipt);
        write_new(
            &transaction.join("archive.json"),
            &serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
        )?;
        fs::rename(&transaction, &directory).map_err(|e| e.to_string())?;
        receipt.share_files = share_paths(&directory, total_n);
        Ok(receipt)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(transaction);
    }
    result
}

/// Locate a saved archive by its exact ID without trusting user-supplied paths.
pub fn load_shamir_q42j_backup(
    storage: &Path,
    archive_id: &str,
) -> Result<ShamirQ42jBackupArchive, String> {
    let directory = archive_directory(storage, archive_id)?;
    let mut manifest =
        read_json::<ShamirQ42jBackupArchive>(&directory.join("archive.json"), MAX_QECP_BYTES)?
            .ok_or("The backup manifest is missing.")?;
    check_threshold(manifest.threshold_m, manifest.total_n)?;
    if manifest.archive_id != archive_id
        || manifest.payload_bytes > MAX_ARCHIVE_PAYLOAD
        || decode_hex::<32>(&manifest.payload_sha3_256_hex).is_none()
        || manifest.sealed_nquin_parity != journal_parity(&manifest)
    {
        return Err("The backup manifest is invalid.".into());
    }
    manifest.share_files = share_paths(&directory, manifest.total_n);
    Ok(manifest)
}

/// Authenticate the encrypted metadata and write a recovered copy without overwriting live stores.
pub fn verify_and_restore_shamir_q42j_backup(
    storage: &Path,
    archive: &ShamirQ42jBackupArchive,
    selected_share_indices: &[u8],
) -> Result<String, String> {
    let saved = load_shamir_q42j_backup(storage, &archive.archive_id)?;
    if saved != *archive {
        return Err("The supplied archive receipt differs from the saved manifest.".into());
    }
    let archive_bytes = decode_hex::<16>(&saved.archive_id).ok_or("Invalid backup archive ID.")?;
    let directory = archive_directory(storage, &saved.archive_id)?;
    let unique: BTreeSet<u8> = selected_share_indices.iter().copied().collect();
    if unique.len() != selected_share_indices.len()
        || unique.len() < usize::from(saved.threshold_m)
        || unique
            .iter()
            .any(|&index| index == 0 || index > saved.total_n)
    {
        return Err("Select at least the required number of distinct, in-range shares.".into());
    }
    let mut shares = Vec::with_capacity(unique.len());
    for index in unique {
        let bytes = read_required(&directory.join(share_name(index)), SHARE_BYTES)?;
        if bytes.len() != SHARE_BYTES
            || &bytes[..8] != SHARE_MAGIC
            || bytes[8..24] != archive_bytes
            || bytes[24..27] != [saved.threshold_m, saved.total_n, index]
        {
            return Err("A selected share belongs to another journal or is malformed.".into());
        }
        let share = bytes[27..]
            .try_into()
            .map_err(|_| "Invalid share length.")?;
        shares.push((index, share));
    }
    let key = interpolate_at_zero(&shares);
    let journal = read_required(
        &directory.join("journal.q42j"),
        ARCHIVE_HEADER_BYTES + MAX_ARCHIVE_PAYLOAD,
    )?;
    if journal.len() != ARCHIVE_HEADER_BYTES + saved.payload_bytes
        || &journal[..8] != ARCHIVE_MAGIC
        || sha3_hex(&journal) != saved.payload_sha3_256_hex
    {
        return Err("The encrypted journal's size or digest does not match its manifest.".into());
    }
    let nonce = journal[8..20]
        .try_into()
        .map_err(|_| "Invalid journal nonce.")?;
    let tag = journal[20..ARCHIVE_HEADER_BYTES]
        .try_into()
        .map_err(|_| "Invalid journal tag.")?;
    let mut plaintext = journal[ARCHIVE_HEADER_BYTES..].to_vec();
    aead::decrypt_in_place(
        &key,
        &nonce,
        &archive_aad(&archive_bytes, saved.threshold_m, saved.total_n),
        &mut plaintext,
        &tag,
    )
    .map_err(|_| "The selected shares cannot authenticate this journal.")?;
    let sections: BTreeMap<String, Value> = serde_json::from_slice(&plaintext)
        .map_err(|_| "The recovered journal is not valid JSON.")?;
    if sections.keys().ne(saved.included_sections.iter()) {
        return Err("The restored sections differ from the journal manifest.".into());
    }
    let restored = directory.join("restored-public-metadata.json");
    write_local_copy(&restored, &plaintext)?;
    Ok(format!(
        "Authenticated encrypted journal with {} distinct shares. Restored public metadata to {}. Live stores were left intact.",
        shares.len(),
        restored.display()
    ))
}

fn collect_portable_sections(storage: &Path) -> Result<BTreeMap<String, Value>, String> {
    let mut sections = BTreeMap::new();
    if let Some(social) = read_json::<BrowserSocialGraph>(
        &storage.join(BROWSER_SOCIAL_STORE_REL_PATH),
        MAX_SOURCE_STORE,
    )? {
        let bookmarks: Vec<_> = social
            .bookmarks
            .iter()
            .filter(|bookmark| {
                bookmark.disclosure.lane == "commons"
                    && bookmark.disclosure.target_id.is_empty()
                    && public_uri(&bookmark.uri)
            })
            .take(MAX_TURTLE_TRIPLES)
            .map(|bookmark| json!({"uri": bookmark.uri, "title": bookmark.title}))
            .collect();
        let annotations: Vec<_> = social
            .annotations
            .iter()
            .filter(|annotation| {
                annotation.disclosure.lane == "commons"
                    && annotation.disclosure.target_id.is_empty()
                    && public_uri(&annotation.target_uri)
            })
            .take(MAX_TURTLE_TRIPLES)
            .map(|annotation| json!({"target_uri": annotation.target_uri}))
            .collect();
        sections.insert("commons_bookmarks".into(), json!(bookmarks));
        sections.insert("commons_annotation_targets".into(), json!(annotations));
    }
    if let Some(entries) = read_json::<Vec<SemanticAssetEntry>>(
        &storage.join(HYPERMEDIA_LIBRARY_REL_PATH),
        MAX_SOURCE_STORE,
    )? {
        let index: Vec<_> = entries
            .iter()
            .filter(|entry| may_export_entry(entry) && public_uri(&entry.asset_uri))
            .take(MAX_TURTLE_TRIPLES)
            .map(|entry| {
                json!({"uri": entry.asset_uri, "title": entry.title, "media_type": entry.media_type})
            })
            .collect();
        sections.insert("permissive_commons_library_index".into(), json!(index));
    }
    // Include only counts for local governance and lease records, with no actors,
    // domains, identifiers, granted scopes, alerts, or private ledger details.
    if let Some(fabric) = read_json::<Value>(
        &storage.join(crate::host_stewardship::HOST_STEWARDSHIP_STORE_REL_PATH),
        MAX_SOURCE_STORE,
    )? {
        let project_count = fabric["hosted_projects"].as_array().map_or(0, Vec::len);
        sections.insert(
            "stewardship_summary".into(),
            json!({"hosted_project_count": project_count}),
        );
    }
    if let Some(center) = read_json::<Value>(
        &storage.join("wellfair/os_control_center.json"),
        MAX_SOURCE_STORE,
    )? {
        let lease_count = center["leases"].as_array().map_or(0, Vec::len);
        sections.insert(
            "capability_lease_summary".into(),
            json!({"lease_record_count": lease_count}),
        );
    }
    Ok(sections)
}

fn check_threshold(threshold: u8, total: u8) -> Result<(), String> {
    if 2 <= threshold && threshold <= total && total <= MAX_SHARE_COUNT {
        Ok(())
    } else {
        Err(format!(
            "Use between 2 and {MAX_SHARE_COUNT} shares with threshold at least 2."
        ))
    }
}

fn archive_aad(archive_id: &[u8; 16], threshold: u8, total: u8) -> Vec<u8> {
    [ARCHIVE_MAGIC.as_slice(), archive_id, &[threshold, total]].concat()
}

fn journal_parity(archive: &ShamirQ42jBackupArchive) -> u64 {
    let digest = format!(
        "{}:{}:{}:{}:{}",
        archive.payload_sha3_256_hex,
        archive.payload_bytes,
        archive.threshold_m,
        archive.total_n,
        archive.included_sections.join(",")
    );
    NQuin48::invariant(
        &archive.archive_id,
        "urn:webizen:portability:encrypted-journal",
        q_hash_60(&digest),
    )
    .parity()
}

fn share_name(index: u8) -> String {
    format!("share-{index:02}.q42share")
}

fn share_paths(directory: &Path, total: u8) -> Vec<PathBuf> {
    (1..=total)
        .map(|index| directory.join(share_name(index)))
        .collect()
}

fn archive_directory(storage: &Path, archive_id: &str) -> Result<PathBuf, String> {
    if decode_hex::<16>(archive_id).is_none() {
        return Err("The archive ID must be 32 lowercase hexadecimal characters.".into());
    }
    storage_directory(storage, &format!("{BACKUP_ROOT}/{archive_id}"), false)
}

fn split_key(key: &[u8; 32], threshold: u8, total: u8) -> Result<Vec<[u8; 32]>, String> {
    let coefficients: Vec<[u8; 32]> = (1..threshold)
        .map(|_| random_array::<32>())
        .collect::<Result<_, _>>()?;
    Ok((1..=total)
        .map(|index| {
            let mut share = *key;
            let mut power = index;
            for coefficient in &coefficients {
                for (value, factor) in share.iter_mut().zip(coefficient) {
                    *value ^= gf_multiply(power, *factor);
                }
                power = gf_multiply(power, index);
            }
            share
        })
        .collect())
}

/// Return whether the record has explicit public Commons disclosure.
pub fn may_export_entry(entry: &SemanticAssetEntry) -> bool {
    entry.sensitivity == "public"
        && entry.commons_visibility == CommonsShareScope::Commons
        && !entry.is_secret()
}

pub fn export_semantic_entry_to_solid_turtle(
    storage: &Path,
    entry: &SemanticAssetEntry,
    pod_endpoint: &str,
) -> Result<SolidTurtleSyncReceipt, String> {
    if !may_export_entry(entry) || !public_uri(&entry.asset_uri) {
        return Err(
            "Export requires an explicitly public Permissive Commons entry and address.".into(),
        );
    }
    check_pod_label(pod_endpoint)?;
    let mut document = String::from(
        "@prefix qualia: <https://webizen.org/q42#> .\n\
         @prefix prov: <http://www.w3.org/ns/prov#> .\n\
         @prefix dcterms: <http://purl.org/dc/terms/> .\n",
    );
    let subject = format!("<{}>", entry.asset_uri);
    let mut literal = |predicate: &str, value: &str| -> Result<(), String> {
        if value.len() > 4096 || document.len() > MAX_TURTLE_BYTES {
            return Err("The selected metadata exceeds the Turtle size limit.".into());
        }
        let value = serde_json::to_string(value).map_err(|error| error.to_string())?;
        document.push_str(&format!("{subject} {predicate} {value} .\n"));
        Ok(())
    };
    literal("dcterms:title", &entry.title)?;
    literal("qualia:section", &entry.section)?;
    literal("qualia:primarySubject", &entry.primary_subject.to_string())?;
    literal(
        "qualia:graphParityDigest",
        &entry.graph_parity_digest.to_string(),
    )?;
    for (predicate, values) in [
        ("qualia:depicts", &entry.depicts),
        ("qualia:topic", &entry.topics),
        ("qualia:project", &entry.projects),
    ] {
        for value in values {
            literal(predicate, value)?;
        }
    }
    for origin in entry.derived_from.iter().filter(|uri| public_uri(uri)) {
        if document.len() > MAX_TURTLE_BYTES {
            return Err("The selected metadata exceeds the Turtle size limit.".into());
        }
        document.push_str(&format!("{subject} prov:wasDerivedFrom <{origin}> .\n"));
    }
    let (triple_count, sealed_nquin_parity) = inspect_solid_turtle(&document)?;
    let directory = storage_directory(storage, "backups/solid-export", true)?;
    let digest = sha3_hex(document.as_bytes());
    let path = directory.join(format!("{digest}.ttl"));
    write_local_copy(&path, document.as_bytes())?;
    Ok(SolidTurtleSyncReceipt {
        direction: "Local Turtle export".into(),
        pod_endpoint: pod_endpoint.into(),
        turtle_document: document,
        triple_count,
        sealed_nquin_parity,
        file_path: Some(path),
    })
}

pub fn import_solid_turtle_to_semantic_fs(
    semantic_fs: &mut SemanticFileSystem,
    turtle_source: &str,
    pod_endpoint: &str,
) -> Result<SolidTurtleSyncReceipt, String> {
    check_pod_label(pod_endpoint)?;
    let model = parse_solid_graph(turtle_source)?;
    let reader = WasmSliceQ42Reader::open(&model.volume).map_err(|error| error.to_string())?;
    let first = model.rows.first().ok_or("Turtle contains no triples.")?;
    let mut scratch = vec![0; MAX_COMPRESSED_SUPERBLOCK_SIZE.max(SUPERBLOCK_SIZE)];
    let mut text = [0; 8192];
    let mut lexical = |hash| -> Result<String, String> {
        let len = reader
            .lookup_lexicon_hash_into(hash, &mut scratch, &mut text)
            .map_err(|error| error.to_string())?
            .ok_or("The Turtle term is absent from the Q42 lexicon.")?;
        std::str::from_utf8(&text[..len])
            .map(str::to_owned)
            .map_err(|_| "The Turtle term is not UTF-8.".into())
    };
    let uri = lexical(first.subject)?;
    if !public_uri(&uri) {
        return Err(
            "Library import requires a single HTTPS or URN resource without query or fragment."
                .into(),
        );
    }
    let mut title = None;
    let mut facets = [Vec::new(), Vec::new(), Vec::new()];
    let mut derived_from = Vec::new();
    for row in &model.rows {
        if row.subject != first.subject {
            return Err(
                "This Library importer accepts one identified resource per document.".into(),
            );
        }
        let predicate = lexical(row.predicate)?;
        if predicate == DCTERMS_TITLE {
            let value = plain_literal(&lexical(row.object)?)?;
            if title.replace(value).is_some() {
                return Err("The Turtle resource contains more than one title.".into());
            }
        } else if predicate == PROV_DERIVATION {
            let source = lexical(row.object)?;
            if public_uri(&source) {
                derived_from.push(source);
            }
        } else if let Some(slot) = predicate.strip_prefix(SOLID_NAMESPACE).and_then(|name| {
            ["topic", "depicts", "project"]
                .iter()
                .position(|field| *field == name)
        }) {
            let value = plain_literal(&lexical(row.object)?)?;
            if !facets[slot].contains(&value) {
                facets[slot].push(value);
            }
        }
    }
    if semantic_fs
        .entries
        .iter()
        .any(|entry| entry.asset_uri == uri)
    {
        return Err("A Library record already owns this URI. Import will not overwrite it.".into());
    }
    let [topics, depicts, projects] = facets;
    let mut entry = SemanticAssetEntry {
        primary_subject: q_hash_60(&uri),
        title: title.unwrap_or_else(|| uri.clone()),
        asset_uri: uri,
        media_type: "text/turtle".into(),
        topics,
        depicts,
        projects,
        purposes: vec!["local-solid-import".into()],
        derived_from,
        bundled_assets: Vec::new(),
        analytics: Vec::new(),
        place: None,
        lat: None,
        lon: None,
        occurred_at: None,
        ingested_unix: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_secs(),
        excerpt: String::new(),
        sensitivity: "private".into(),
        section: "personal".into(),
        commons_visibility: CommonsShareScope::None,
        cml_signals: Vec::new(),
        cml_concept_count: 0,
        cof_segment_count: 0,
        sealed_nquin_count: 0,
        graph_parity_digest: 0,
    };
    entry.recompute_section_and_quin_edges();
    let mut candidate = semantic_fs.entries.clone();
    candidate.insert(0, entry);
    if !semantic_fs.store_path.as_os_str().is_empty() {
        let bytes = serde_json::to_vec_pretty(&candidate).map_err(|error| error.to_string())?;
        if bytes.len() > MAX_SOURCE_STORE {
            return Err("The updated Library exceeds its portable size limit.".into());
        }
        write_atomic(&semantic_fs.store_path, &bytes)?;
    }
    semantic_fs.entries = candidate;
    Ok(SolidTurtleSyncReceipt {
        direction: "Local Turtle import".into(),
        pod_endpoint: pod_endpoint.into(),
        turtle_document: turtle_source.into(),
        triple_count: model.rows.len(),
        sealed_nquin_parity: model.parity,
        file_path: (!semantic_fs.store_path.as_os_str().is_empty())
            .then(|| semantic_fs.store_path.clone()),
    })
}

pub(crate) fn inspect_solid_turtle(source: &str) -> Result<(usize, u64), String> {
    let model = parse_solid_graph(source)?;
    Ok((model.rows.len(), model.parity))
}

struct SolidGraph {
    volume: Vec<u8>,
    rows: Vec<NQuin>,
    parity: u64,
}

fn parse_solid_graph(source: &str) -> Result<SolidGraph, String> {
    if source.len() > MAX_TURTLE_BYTES {
        return Err("The Turtle document exceeds the local byte limit.".into());
    }
    let (volume, count) = SemanticModelBuilder::new(MAX_TURTLE_BYTES * 8)
        .seal_solid_turtle(SOLID_CONTEXT, source, &AtomicBool::new(false))
        .map_err(|error| error.to_string())?;
    if count == 0 || count > MAX_TURTLE_TRIPLES {
        return Err(
            "The Turtle document exceeds the local triple budget or contains no triples.".into(),
        );
    }
    let reader = WasmSliceQ42Reader::open(&volume).map_err(|error| error.to_string())?;
    let mut compressed = vec![0; MAX_COMPRESSED_SUPERBLOCK_SIZE];
    let mut decoded = vec![0; SUPERBLOCK_SIZE];
    let mut rows = vec![NQuin::default(); count + 1];
    let page = reader
        .execute_query_page_into(
            Q42RangeQueryPlan::for_pattern(Q42RangeQueryPattern {
                context: Some(q_hash(SOLID_CONTEXT)),
                ..Q42RangeQueryPattern::default()
            }),
            Q42RangeQueryCursor::default(),
            &mut compressed,
            &mut decoded,
            &mut rows,
        )
        .map_err(|error| error.to_string())?;
    rows.truncate(page.returned);
    if rows.is_empty() || rows.len() > count || page.next_cursor.is_some() {
        return Err("Could not verify the complete local Q42 graph.".into());
    }
    let parity = rows.iter().try_fold(0, |parity, row| {
        row.verify_ecc_parity()
            .then_some(parity ^ row.parity)
            .ok_or_else(|| "The Solid graph contains a damaged NQuin48 row.".to_string())
    })?;
    Ok(SolidGraph {
        volume,
        rows,
        parity,
    })
}

fn plain_literal(value: &str) -> Result<String, String> {
    serde_json::from_str::<String>(value).map_err(|_| {
        "Library title, topic, entity and project metadata require plain RDF string literals."
            .into()
    })
}

fn check_pod_label(endpoint: &str) -> Result<(), String> {
    if endpoint.is_empty() || (endpoint.starts_with("https://") && public_uri(endpoint)) {
        Ok(())
    } else {
        Err("Enter an HTTPS Pod address without credentials, query or fragment, or leave it empty for offline transfer.".into())
    }
}

fn public_uri(uri: &str) -> bool {
    if uri.len() > 2048
        || uri
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "<>\"{}|\\^\x60?#".contains(c))
    {
        return false;
    }
    if let Some(address) = uri.strip_prefix("https://") {
        let authority = address.split('/').next().unwrap_or_default();
        return !authority.is_empty() && !authority.contains('@');
    }
    uri.strip_prefix("urn:")
        .and_then(|urn| urn.split_once(':'))
        .is_some_and(|(namespace, name)| !namespace.is_empty() && !name.is_empty())
}

pub fn seal_and_inspect_qecp_envelope(
    storage: &Path,
    document_title: &str,
    issuer_pairwise_did: &str,
    claim_summary: &str,
) -> Result<QecpCredentialEnvelope, String> {
    let envelope = prepare_qecp_envelope(document_title, issuer_pairwise_did, claim_summary)?;
    let directory = storage_directory(storage, "library/qecp", true)?;
    let filename = envelope
        .credential_id
        .trim_start_matches("urn:webizen:qecp:");
    let path = directory.join(format!("{filename}.qecp.json"));
    let bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| error.to_string())?;
    write_local_copy(&path, &bytes)?;
    Ok(envelope)
}

pub(crate) fn prepare_qecp_envelope(
    document_title: &str,
    issuer_pairwise_did: &str,
    claim_summary: &str,
) -> Result<QecpCredentialEnvelope, String> {
    for (value, limit) in [
        (document_title, 256),
        (issuer_pairwise_did, 256),
        (claim_summary, 4096),
    ] {
        if value.trim().is_empty() || value.len() > limit || value.chars().any(char::is_control) {
            return Err(
                "The QECP title, claimed issuer and claim must be nonempty bounded text.".into(),
            );
        }
        validate_human_centric_host_nomenclature(value)?;
    }
    if issuer_pairwise_did.chars().any(char::is_whitespace)
        || !issuer_pairwise_did
            .strip_prefix("did:")
            .and_then(|did| did.split_once(':'))
            .is_some_and(|(method, name)| !method.is_empty() && !name.is_empty())
    {
        return Err("Provide a syntactically valid claimed pairwise issuer DID.".into());
    }
    let encoded = serde_json::to_vec(&(document_title, issuer_pairwise_did, claim_summary))
        .map_err(|error| error.to_string())?;
    let digest = sha3_hex(&encoded);
    let credential_id = format!("urn:webizen:qecp:{digest}");
    let parity = NQuin48::invariant(
        &credential_id,
        "urn:webizen:qecp:unsigned-local-envelope",
        q_hash_60(&digest),
    )
    .parity();
    Ok(QecpCredentialEnvelope {
        credential_id,
        document_title: document_title.into(),
        issuer_pairwise_did: issuer_pairwise_did.into(),
        claim_summary: claim_summary.into(),
        c2pa_jumbf_label: "Awaiting External Signer / Verifier".into(),
        cbor_ld_tag_hex: String::new(),
        origin_integrity_disclaimer: VC_ORIGIN_INTEGRITY_DISCLAIMER.into(),
        signature_scheme: "Unsigned JSON companion with SHA3-256 and NQuin48 parity".into(),
        sealed_nquin_parity: parity,
    })
}

pub(crate) fn inspect_qecp_envelope_json(source: &str) -> Result<QecpCredentialEnvelope, String> {
    if source.len() > MAX_QECP_BYTES {
        return Err("The QECP companion exceeds the local byte limit.".into());
    }
    let envelope: QecpCredentialEnvelope =
        serde_json::from_str(source).map_err(|error| error.to_string())?;
    let expected = prepare_qecp_envelope(
        &envelope.document_title,
        &envelope.issuer_pairwise_did,
        &envelope.claim_summary,
    )?;
    if expected != envelope {
        return Err(
            "The unsigned QECP digest, parity or status fields disagree with the claim.".into(),
        );
    }
    Ok(envelope)
}

fn storage_directory(storage: &Path, relative: &str, create: bool) -> Result<PathBuf, String> {
    let mut directory = storage.canonicalize().map_err(|error| error.to_string())?;
    for part in relative.split('/') {
        directory.push(part);
        if create {
            match fs::create_dir(&directory) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        if !fs::symlink_metadata(&directory)
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            return Err("Portability paths must be ordinary local directories.".into());
        }
    }
    Ok(directory)
}

fn read_json<T: DeserializeOwned>(path: &Path, max: usize) -> Result<Option<T>, String> {
    read_optional(path, max)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(|error| error.to_string()))
        .transpose()
}

fn read_required(path: &Path, max: usize) -> Result<Vec<u8>, String> {
    read_optional(path, max)?.ok_or_else(|| "A required portability file is missing.".into())
}

fn read_optional(path: &Path, max: usize) -> Result<Option<Vec<u8>>, String> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if !meta.is_file() || meta.len() > max as u64 {
        return Err(
            "The portability input is not a regular file or exceeds the size limit.".into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(max as u64 + 1).read_to_end(&mut bytes))
        .map_err(|error| error.to_string())?;
    if bytes.len() > max {
        return Err("The portability input exceeds the size limit.".into());
    }
    Ok(Some(bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string())
}

fn write_local_copy(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(previous) = read_optional(path, bytes.len())? {
        if previous == bytes {
            return Ok(());
        }
        return Err("A different local file already exists at the portability destination.".into());
    }
    write_atomic(path, bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let label = hex(&random_array::<16>()?);
    let temporary = path.with_extension(format!("solid-{label}.tmp"));
    let result = write_new(&temporary, bytes)
        .and_then(|_| fs::rename(&temporary, path).map_err(|error| error.to_string()));
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn random_array<const N: usize>() -> Result<[u8; N], String> {
    let mut output = [0; N];
    fill_os(&mut output).map_err(|_| "Operating-system entropy is unavailable.")?;
    Ok(output)
}

fn sha3_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha3_256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex<const N: usize>(text: &str) -> Option<[u8; N]> {
    if text.len() != N * 2
        || !text
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return None;
    }
    let mut result = [0; N];
    for (index, value) in result.iter_mut().enumerate() {
        *value = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStorage(PathBuf);

    impl TestStorage {
        fn new() -> Self {
            let label = hex(&random_array::<16>().unwrap());
            let path = std::env::temp_dir().join(format!("webizen-portability-{label}"));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestStorage {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const PUBLIC_TURTLE: &str = concat!(
        "@prefix dcterms: <http://purl.org/dc/terms/> .\n",
        "@prefix qualia: <https://webizen.org/q42#> .\n",
        "<https://example.test/record> dcterms:title \"River survey\" ; ",
        "qualia:topic \"Fresh water\" ; qualia:depicts \"River\" .\n",
    );

    #[test]
    fn round_trips_shamir_q42j_backup_and_fails_closed_below_threshold() {
        let storage = TestStorage::new();
        let mut library = SemanticFileSystem::default();
        import_solid_turtle_to_semantic_fs(&mut library, PUBLIC_TURTLE, "").unwrap();
        library.entries[0].sensitivity = "public".into();
        library.entries[0].commons_visibility = CommonsShareScope::Commons;
        let mut private = library.entries[0].clone();
        private.asset_uri = "https://example.test/private".into();
        private.title = "private off-limit marker".into();
        private.sensitivity = "restricted".into();
        library.entries.push(private);
        fs::create_dir(storage.0.join("wellfair")).unwrap();
        write_new(
            &storage.0.join(HYPERMEDIA_LIBRARY_REL_PATH),
            &serde_json::to_vec(&library.entries).unwrap(),
        )
        .unwrap();
        let archive = export_shamir_q42j_backup(&storage.0, 2, 3).unwrap();
        assert_eq!(
            load_shamir_q42j_backup(&storage.0, &archive.archive_id).unwrap(),
            archive
        );
        assert!(verify_and_restore_shamir_q42j_backup(&storage.0, &archive, &[1]).is_err());
        assert!(verify_and_restore_shamir_q42j_backup(&storage.0, &archive, &[2, 2]).is_err());
        assert!(verify_and_restore_shamir_q42j_backup(&storage.0, &archive, &[1, 4]).is_err());
        verify_and_restore_shamir_q42j_backup(&storage.0, &archive, &[3, 1]).unwrap();
        let directory = archive_directory(&storage.0, &archive.archive_id).unwrap();
        let recovery = fs::read_to_string(directory.join("restored-public-metadata.json")).unwrap();
        assert!(recovery.contains("River survey"));
        assert!(!recovery.contains("private off-limit marker"));
        let mut bytes = fs::read(&archive.share_files[0]).unwrap();
        bytes[SHARE_BYTES - 1] ^= 1;
        fs::write(&archive.share_files[0], bytes).unwrap();
        assert!(verify_and_restore_shamir_q42j_backup(&storage.0, &archive, &[1, 3]).is_err());
        assert!(load_shamir_q42j_backup(&storage.0, "../elsewhere").is_err());
    }

    #[test]
    fn exports_and_imports_w3c_solid_turtle_with_nquin48_parity() {
        let storage = TestStorage::new();
        let mut library = SemanticFileSystem::default();
        let imported = import_solid_turtle_to_semantic_fs(&mut library, PUBLIC_TURTLE, "").unwrap();
        assert_eq!(imported.triple_count, 3);
        assert_eq!(library.entries[0].title, "River survey");
        assert_eq!(library.entries[0].topics, vec!["Fresh water"]);
        assert_eq!(library.entries[0].depicts, vec!["River"]);
        assert!(import_solid_turtle_to_semantic_fs(&mut library, PUBLIC_TURTLE, "").is_err());
        assert!(
            export_semantic_entry_to_solid_turtle(&storage.0, &library.entries[0], "").is_err()
        );
        library.entries[0].sensitivity = "public".into();
        library.entries[0].commons_visibility = CommonsShareScope::Commons;
        let exported = export_semantic_entry_to_solid_turtle(
            &storage.0,
            &library.entries[0],
            "https://pod.example.test/resource",
        )
        .unwrap();
        let mut destination = SemanticFileSystem::default();
        let receipt =
            import_solid_turtle_to_semantic_fs(&mut destination, &exported.turtle_document, "")
                .unwrap();
        assert_eq!(receipt.sealed_nquin_parity, exported.sealed_nquin_parity);
        assert_eq!(receipt.triple_count, exported.triple_count);
        assert_eq!(destination.entries[0].title, "River survey");
        assert!(exported.file_path.unwrap().is_file());
        assert!(inspect_solid_turtle("<https://example.test/record> dcterms:title .").is_err());
    }

    #[test]
    fn seals_qecp_envelope_with_origin_integrity_disclaimer() {
        let storage = TestStorage::new();
        let envelope = seal_and_inspect_qecp_envelope(
            &storage.0,
            "Survey note",
            "did:qi:pairwise-survey",
            "The contributor attributes a field survey to the attached note.",
        )
        .unwrap();
        assert_eq!(
            envelope.origin_integrity_disclaimer,
            VC_ORIGIN_INTEGRITY_DISCLAIMER
        );
        assert_eq!(
            envelope.c2pa_jumbf_label,
            "Awaiting External Signer / Verifier"
        );
        assert!(envelope.cbor_ld_tag_hex.is_empty());
        let text = serde_json::to_string(&envelope).unwrap();
        assert_eq!(inspect_qecp_envelope_json(&text).unwrap(), envelope);
        assert!(inspect_qecp_envelope_json(&text.replace("Survey note", "Another note")).is_err());
        assert!(prepare_qecp_envelope("owl:Thing", "did:qi:pairwise-survey", "A claim").is_err());
    }
}
