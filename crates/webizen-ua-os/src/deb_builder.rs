// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Deterministic Debian 2.0 archives built from finalized package staging trees.

use crate::host_stewardship::{
    validate_human_centric_host_nomenclature, TriTargetDebianSuiteReceipt,
};
use crate::identifiers::q_hash_60;
use flate2::{read::GzDecoder, Compression, GzBuilder};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use vibescript_abi::NQuin48;

const AR_MAGIC: &[u8; 8] = b"!<arch>\n";
const AR_MEMBERS: [&str; 3] = ["debian-binary", "control.tar.gz", "data.tar.gz"];
const TAR_BLOCK_BYTES: usize = 512;
const ZEROES: [u8; TAR_BLOCK_BYTES * 2] = [0; TAR_BLOCK_BYTES * 2];
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Metadata and digest of the published archive, with member sizes before ar padding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebBuildReceipt {
    pub package_name: String,
    pub version: String,
    pub architecture: String,
    pub output_deb_path: PathBuf,
    pub archive_bytes: u64,
    pub control_tar_gz_bytes: u64,
    pub data_tar_gz_bytes: u64,
    /// Number of regular files in the control and data archives.
    pub member_file_count: usize,
    pub sha3_256_hex: String,
}

struct PackageControl {
    name: String,
    version: String,
    architecture: String,
}

impl PackageControl {
    fn read(root: &Path) -> Result<Self, String> {
        let control_root = root.join("DEBIAN");
        let path = control_root.join("control");
        if !is_plain_directory(&metadata(&control_root)?) || !is_plain_file(&metadata(&path)?) {
            return Err(format!(
                "expected regular Debian control file at {}",
                path.display()
            ));
        }
        let text = fs::read_to_string(&path).map_err(|error| path_error(&path, error))?;
        Self::parse(&text)
    }

    fn parse(text: &str) -> Result<Self, String> {
        validate_human_centric_host_nomenclature(text)?;
        let mut name = None;
        let mut version = None;
        let mut architecture = None;
        for line in text.lines() {
            if line.starts_with([' ', '\t']) || line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once(':').ok_or("invalid Debian control field")?;
            let slot = match key.to_ascii_lowercase().as_str() {
                "package" => &mut name,
                "version" => &mut version,
                "architecture" => &mut architecture,
                _ => continue,
            };
            if slot.replace(value.trim().to_owned()).is_some() {
                return Err(format!("duplicate Debian control field: {key}"));
            }
        }

        let name = name.ok_or("missing Package field in Debian control")?;
        let version = version.ok_or("missing Version field in Debian control")?;
        let architecture = architecture.ok_or("missing Architecture field in Debian control")?;
        let valid_name_byte = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
        if name.len() < 2
            || !valid_name_byte(name.as_bytes()[0])
            || !name
                .bytes()
                .all(|byte| valid_name_byte(byte) || b"+-.".contains(&byte))
        {
            return Err("invalid Debian package name".into());
        }
        let version_without_epoch = match version.split_once(':') {
            Some((epoch, value))
                if !epoch.is_empty() && epoch.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                value
            }
            Some(_) => return Err("invalid Debian version epoch".into()),
            None => &version,
        };
        if !version_without_epoch
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_digit)
            || !version_without_epoch
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"+.~:-".contains(&byte))
        {
            return Err("invalid Debian package version".into());
        }
        if architecture.is_empty()
            || !architecture
                .bytes()
                .all(|byte| valid_name_byte(byte) || byte == b'-')
        {
            return Err("invalid Debian package architecture".into());
        }
        Ok(Self {
            name,
            version,
            architecture,
        })
    }
}

/// Build a Debian package from a finalized tree with a regular `DEBIAN/control`.
///
/// Files and directories are sorted by archive path. Their host permissions and
/// timestamps are replaced by fixed package metadata. Links and special files
/// are rejected so the package only includes content staged under the tree.
pub fn build_deb_from_tree(
    staging_root: &Path,
    output_deb_path: &Path,
) -> Result<DebBuildReceipt, String> {
    if !is_plain_directory(&metadata(staging_root)?) {
        return Err(format!(
            "expected package staging directory at {}",
            staging_root.display()
        ));
    }
    let control = PackageControl::read(staging_root)?;
    let parent = output_deb_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let input_root = staging_root
        .canonicalize()
        .map_err(|error| path_error(staging_root, error))?;
    let output_parent = parent
        .canonicalize()
        .map_err(|error| path_error(parent, error))?;
    if output_parent.starts_with(&input_root) {
        return Err("Debian package output must be outside the staging tree".into());
    }

    let mut checksums = PayloadChecksums::default();
    let (data_tar, data_count) = gzip_tar(staging_root, true, &[], Some(&mut checksums))?;
    let (control_tar, control_count) = gzip_tar(
        &staging_root.join("DEBIAN"),
        false,
        &[
            ("./md5sums", checksums.md5.as_bytes()),
            ("./sha3sums", checksums.sha3.as_bytes()),
        ],
        None,
    )?;
    let mut archive = AR_MAGIC.to_vec();
    for (name, content) in AR_MEMBERS
        .into_iter()
        .zip([&b"2.0\n"[..], &control_tar, &data_tar])
    {
        write_ar_member(&mut archive, name, content)?;
    }
    let sha3_256_hex = format!("{:x}", Sha3_256::digest(&archive));
    let output_name = output_deb_path
        .file_name()
        .ok_or("Debian package output has no filename")?;
    publish_file_if_changed(parent, Path::new(output_name), &archive)?;
    Ok(DebBuildReceipt {
        package_name: control.name,
        version: control.version,
        architecture: control.architecture,
        output_deb_path: output_deb_path.to_owned(),
        archive_bytes: archive.len() as u64,
        control_tar_gz_bytes: control_tar.len() as u64,
        data_tar_gz_bytes: data_tar.len() as u64,
        member_file_count: control_count + data_count,
        sha3_256_hex,
    })
}

/// Build Desktop, Mobile AVF and Host packages in the suite's output directory.
pub fn build_tri_target_debs_from_receipt(
    suite: &TriTargetDebianSuiteReceipt,
) -> Result<Vec<DebBuildReceipt>, String> {
    [
        &suite.desktop_deb_tree,
        &suite.mobile_avf_deb_tree,
        &suite.host_deb_tree,
    ]
    .into_iter()
    .map(|tree| {
        let name = tree
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "invalid package staging directory name at {}",
                    tree.display()
                )
            })?;
        build_deb_from_tree(tree, &suite.suite_root_dir.join(format!("{name}.deb")))
    })
    .collect()
}

const APT_ARCHITECTURES: [&str; 2] = ["amd64", "arm64"];
const MAX_CONTROL_TAR_BYTES: u64 = 16 * 1024 * 1024;
const MAX_APT_TRUST_BYTES: u64 = 1024 * 1024;

/// Repository indexes and the cryptographic evidence available for this publication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AptRepositoryReceipt {
    pub repo_root: PathBuf,
    pub packages_amd64_path: PathBuf,
    pub packages_arm64_path: PathBuf,
    pub release_path: PathBuf,
    /// Empty until provisioned signers authenticate this Release.
    pub inrelease_path: PathBuf,
    pub signed: bool,
    /// Hex-encoded OpenPGP signature armor from the verified InRelease.
    pub quorum_signature_hex: String,
    pub sealed_nquin_parity: u64,
}

/// Stage deterministic Packages, Packages.gz and Release indexes for offline approval.
pub fn generate_apt_repository_indexes(
    suite_root: &Path,
    built_debs: &[DebBuildReceipt],
    host_domain: &str,
    quorum_threshold_m: u8,
    steward_dids: &[String],
) -> Result<AptRepositoryReceipt, String> {
    generate_apt_repository(
        suite_root,
        built_debs,
        host_domain,
        quorum_threshold_m,
        steward_dids,
        false,
    )
}

/// Publish an APT InRelease after verifying a quorum of independent OpenPGP signatures.
///
/// The host points WEBIZEN_APT_TRUST_DIR at an independently provisioned directory with
/// stewards.json (host_domain and steward_fingerprints, keyed by eligible DID) and
/// stewards.gpg (current trusted public keys, with expired and revoked keys removed).
/// The separate apt-signing/InRelease beside the suite is artifact input. Provide the same
/// SOURCE_DATE_EPOCH for reproducible Release dates across different staging directories.
pub fn generate_signed_apt_repository(
    suite_root: &Path,
    built_debs: &[DebBuildReceipt],
    host_domain: &str,
    quorum_threshold_m: u8,
    steward_dids: &[String],
) -> Result<AptRepositoryReceipt, String> {
    generate_apt_repository(
        suite_root,
        built_debs,
        host_domain,
        quorum_threshold_m,
        steward_dids,
        true,
    )
}

pub(crate) fn apt_signing_evidence_exists(suite_root: &Path) -> Result<bool, String> {
    let path = apt_signing_directory(suite_root).join("InRelease");
    match fs::symlink_metadata(&path) {
        Ok(meta) if is_plain_file(&meta) => Ok(true),
        Ok(_) => Err("APT signing evidence must be a regular file".into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(path_error(&path, error)),
    }
}

fn apt_signing_directory(suite_root: &Path) -> PathBuf {
    suite_root
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("apt-signing")
}

fn generate_apt_repository(
    suite_root: &Path,
    built_debs: &[DebBuildReceipt],
    host_domain: &str,
    quorum_threshold_m: u8,
    steward_dids: &[String],
    signed: bool,
) -> Result<AptRepositoryReceipt, String> {
    validate_human_centric_host_nomenclature(host_domain)?;
    if host_domain.is_empty()
        || host_domain.len() > 253
        || !host_domain
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-.".contains(&byte))
        || built_debs.is_empty()
    {
        return Err("APT staging requires a host domain and at least one package".into());
    }
    let stewards = steward_dids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for &did in &stewards {
        validate_human_centric_host_nomenclature(did)?;
        if !did.starts_with("did:") || did.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err("Invalid steward DID in APT roster".into());
        }
    }
    if signed && (quorum_threshold_m == 0 || usize::from(quorum_threshold_m) > stewards.len()) {
        return Err("Insufficient eligible steward identities for APT quorum".into());
    }
    if !is_plain_directory(&metadata(suite_root)?) {
        return Err("APT suite root must be a regular directory".into());
    }
    let root = suite_root
        .canonicalize()
        .map_err(|error| path_error(suite_root, error))?;
    let packages = build_apt_package_indices(&root, built_debs)?;
    let date = apt_release_date(built_debs)?;
    let mut release = format!(
        "Origin: {host_domain}\nLabel: Webizen ({host_domain})\nSuite: trixie\nCodename: trixie\nDate: {date}\n\
         Architectures: amd64 arm64\nComponents: main\nAcquire-By-Hash: yes\n\
         X-Webizen-Quorum-Threshold: {quorum_threshold_m}\nX-Webizen-Steward-Roster-SHA3-256: {:x}\n",
        Sha3_256::digest(stewards.iter().copied().collect::<Vec<_>>().join("\n").as_bytes())
    );
    release.push_str("SHA256:\n");
    for (name, bytes) in &packages {
        let _ = writeln!(release, " {} {} {name}", sha256_hex(bytes), bytes.len());
    }
    release.push_str("SHA3-256:\n");
    for (name, bytes) in &packages {
        let _ = writeln!(
            release,
            " {:x} {} {name}",
            Sha3_256::digest(bytes),
            bytes.len()
        );
    }

    let (inrelease, quorum_signature_hex) = if signed {
        verify_apt_signatures(
            suite_root,
            release.as_bytes(),
            host_domain,
            quorum_threshold_m,
            &stewards,
        )?
    } else {
        (Vec::new(), String::new())
    };
    let release_dir = apt_directory(suite_root, Path::new("dists/trixie"))?;
    let retained_signature = release_dir.join("InRelease");
    let release_path = release_dir.join("Release");
    // A changed unsigned Release must not inherit an older signature.
    if !signed && retained_signature.exists() {
        if !is_plain_file(&metadata(&retained_signature)?) {
            return Err("Refusing to replace an unexpected InRelease filesystem entry".into());
        }
        if fs::read(&release_path).ok().as_deref() != Some(release.as_bytes()) {
            fs::remove_file(&retained_signature)
                .map_err(|error| path_error(&retained_signature, error))?;
        }
    }
    for (name, bytes) in packages {
        let path = Path::new(&name);
        let relative_parent = path.parent().ok_or("APT index has no parent directory")?;
        let parent = apt_directory(&release_dir, relative_parent)?;
        let by_hash = apt_directory(&parent, Path::new("by-hash/SHA256"))?;
        let name = path.file_name().ok_or("APT index has no filename")?;
        publish_file_if_changed(&by_hash, Path::new(&sha256_hex(&bytes)), &bytes)?;
        publish_file_if_changed(&parent, Path::new(name), &bytes)?;
    }
    publish_file_if_changed(&release_dir, Path::new("Release"), release.as_bytes())?;
    if signed {
        publish_file_if_changed(&release_dir, Path::new("InRelease"), &inrelease)?;
    }

    let digest = format!("{:x}", Sha3_256::digest(release.as_bytes()));
    let predicate = if signed {
        "https://ns.webcivics.net/host#verifiedAptReleaseQuorum"
    } else {
        "https://ns.webcivics.net/host#stagedAptReleaseIndexes"
    };
    let seal = NQuin48::invariant(host_domain, predicate, q_hash_60(&digest));
    debug_assert!(seal.verify_parity());
    Ok(AptRepositoryReceipt {
        repo_root: suite_root.to_owned(),
        packages_amd64_path: release_dir.join("main/binary-amd64/Packages"),
        packages_arm64_path: release_dir.join("main/binary-arm64/Packages"),
        release_path,
        inrelease_path: if signed {
            retained_signature
        } else {
            PathBuf::new()
        },
        signed,
        quorum_signature_hex,
        sealed_nquin_parity: seal.parity(),
    })
}

fn apt_release_date(packages: &[DebBuildReceipt]) -> Result<String, String> {
    let epoch = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|_| "Invalid SOURCE_DATE_EPOCH")?,
        Err(std::env::VarError::NotPresent) => {
            let mut newest = UNIX_EPOCH;
            for package in packages {
                let path = &package.output_deb_path;
                let modified = metadata(path)?
                    .modified()
                    .map_err(|error| path_error(path, error))?;
                newest = newest.max(modified);
            }
            newest
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "APT archive time precedes the Unix epoch")?
                .as_secs()
        }
        Err(_) => return Err("Invalid SOURCE_DATE_EPOCH encoding".into()),
    };
    format_apt_date(epoch)
}

fn format_apt_date(epoch: u64) -> Result<String, String> {
    const WEEKDAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if epoch > 253_402_300_799 {
        return Err("APT release date falls outside the supported calendar".into());
    }
    let leap = |year: u32| year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let mut days = epoch / 86_400;
    let weekday = WEEKDAYS[(days % 7) as usize];
    let mut year = 1970;
    loop {
        let count = 365 + u64::from(leap(year));
        if days < count {
            break;
        }
        days -= count;
        year += 1;
    }
    let mut month = 0;
    for count in [
        31,
        28 + u64::from(leap(year)),
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ] {
        if days < count {
            break;
        }
        days -= count;
        month += 1;
    }
    let month = MONTHS[month];
    let day = days + 1;
    let time = epoch % 86_400;
    let (hour, minute, second) = (time / 3600, (time / 60) % 60, time % 60);
    Ok(format!(
        "{weekday}, {day:02} {month} {year:04} {hour:02}:{minute:02}:{second:02} UTC"
    ))
}

fn build_apt_package_indices(
    root: &Path,
    built_debs: &[DebBuildReceipt],
) -> Result<[(String, Vec<u8>); 4], String> {
    let mut indices = BTreeMap::<&str, Vec<(PackageControl, String)>>::new();
    let mut unique = BTreeSet::new();
    for receipt in built_debs {
        let canonical = receipt
            .output_deb_path
            .canonicalize()
            .map_err(|error| path_error(&receipt.output_deb_path, error))?;
        if canonical.parent() != Some(root) || !is_plain_file(&metadata(&receipt.output_deb_path)?)
        {
            return Err("APT archive must be a regular file in the suite root".into());
        }
        let archive = fs::read(&canonical).map_err(|error| path_error(&canonical, error))?;
        let sha3 = format!("{:x}", Sha3_256::digest(&archive));
        let size = archive.len() as u64;
        if size != receipt.archive_bytes || sha3 != receipt.sha3_256_hex {
            return Err("APT archive no longer matches the package receipt".into());
        }
        let (package, control) = apt_package_control(&archive)?;
        if package.name != receipt.package_name
            || package.version != receipt.version
            || package.architecture != receipt.architecture
            || !(APT_ARCHITECTURES.contains(&package.architecture.as_str())
                || package.architecture == "all")
        {
            return Err(
                "APT package receipt does not match its control metadata or architecture".into(),
            );
        }
        if !unique.insert((
            package.name.clone(),
            package.version.clone(),
            package.architecture.clone(),
        )) {
            return Err("Duplicate package, version and architecture in APT archive".into());
        }
        let digest = sha256_hex(&archive);
        let pool_parent = Path::new("pool/main").join(&package.name).join(&digest);
        let filename_version = package
            .version
            .split_once(':')
            .map_or(package.version.as_str(), |(_, version)| version)
            .replace(':', "-");
        let package_file = format!(
            "{}_{filename_version}_{}.deb",
            package.name, package.architecture
        );
        let parent = apt_directory(root, &pool_parent)?;
        publish_file_if_changed(&parent, Path::new(&package_file), &archive)?;
        let filename = format!("pool/main/{}/{digest}/{package_file}", package.name);
        let stanza = format!(
            "{control}\nFilename: {filename}\nSize: {size}\nSHA256: {digest}\nSHA3-256: {sha3}\n\n"
        );
        let arch = match package.architecture.as_str() {
            "amd64" => "amd64",
            "arm64" => "arm64",
            _ => "all",
        };
        indices.entry(arch).or_default().push((package, stanza));
    }

    let mut result = Vec::with_capacity(4);
    for architecture in APT_ARCHITECTURES {
        let mut records: Vec<_> = [architecture, "all"]
            .into_iter()
            .flat_map(|arch| indices.get(arch).into_iter().flatten())
            .collect();
        records.sort_unstable_by(|(a, _), (b, _)| {
            (&a.name, &a.version, &a.architecture).cmp(&(&b.name, &b.version, &b.architecture))
        });
        let mut index = Vec::new();
        for (_, stanza) in records {
            index.extend_from_slice(stanza.as_bytes());
        }
        let mut compressed = gzip_encoder();
        compressed
            .write_all(&index)
            .map_err(|error| error.to_string())?;
        let directory = format!("main/binary-{architecture}");
        result.push((format!("{directory}/Packages"), index));
        result.push((
            format!("{directory}/Packages.gz"),
            compressed.finish().map_err(|error| error.to_string())?,
        ));
    }
    result
        .try_into()
        .map_err(|_| "APT architecture index count differs from the suite layout".into())
}

fn apt_package_control(archive: &[u8]) -> Result<(PackageControl, String), String> {
    let members = verify_deb_ar_container(archive)?;
    let control_start = AR_MAGIC.len() + 60 + members[0].1 + members[0].1 % 2 + 60;
    let payload = &archive[control_start..control_start + members[1].1];
    let mut gzip = GzDecoder::new(payload).take(MAX_CONTROL_TAR_BYTES + 1);
    let mut control_tar = Vec::new();
    gzip.read_to_end(&mut control_tar)
        .map_err(|error| format!("Invalid control tar gzip: {error}"))?;
    if control_tar.len() as u64 > MAX_CONTROL_TAR_BYTES {
        return Err("Debian control tar exceeds the APT metadata size limit".into());
    }
    let mut remaining = control_tar.as_slice();
    let mut control = None;
    while let Some((tar_header, contents)) = remaining.split_at_checked(TAR_BLOCK_BYTES) {
        if tar_header.iter().all(|&byte| byte == 0) {
            break;
        }
        let name = tar_header[..100]
            .split(|&byte| byte == 0)
            .next()
            .unwrap_or_default();
        let size_field = trim_tar_field(&tar_header[124..136])?;
        let size = usize::from_str_radix(size_field, 8).map_err(|_| "Invalid control tar size")?;
        let padded = size
            .checked_add(TAR_BLOCK_BYTES - 1)
            .ok_or("Control tar entry size overflow")?
            / TAR_BLOCK_BYTES
            * TAR_BLOCK_BYTES;
        remaining = contents
            .get(padded..)
            .ok_or("Truncated Debian control tar")?;
        if matches!(name, b"./control" | b"control")
            && tar_header[345..500].iter().all(|byte| *byte == 0)
        {
            if control.is_some() || !matches!(tar_header[156], b'0' | 0) {
                return Err("Debian control tar must contain one regular control file".into());
            }
            control = Some(
                std::str::from_utf8(&contents[..size])
                    .map_err(|_| "Debian control file is not UTF-8")?,
            );
        }
    }
    let text = control.ok_or("Debian control tar did not contain control metadata")?;
    let package = PackageControl::parse(text)?;
    Ok((package, apt_package_stanza(text)?))
}

fn trim_tar_field(field: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(field.split(|&byte| byte == 0).next().unwrap_or_default())
        .map(str::trim)
        .map_err(|_| "Invalid decimal field in control tar".into())
}

fn apt_package_stanza(control: &str) -> Result<String, String> {
    let mut fields = BTreeSet::new();
    let mut result = String::new();
    let mut ended = false;
    for line in control.lines() {
        if line.is_empty() {
            ended = true;
            continue;
        }
        if ended || line.chars().any(|c| c.is_control() && c != '\t') {
            return Err("APT control metadata must contain one paragraph".into());
        }
        if line.starts_with([' ', '\t']) {
            if fields.is_empty() {
                return Err("Debian control continuation has no field".into());
            }
        } else {
            let (field, _) = line
                .split_once(':')
                .ok_or("APT control field needs a name")?;
            let field = field.to_ascii_lowercase();
            if field.is_empty()
                || !field
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                || matches!(
                    field.as_str(),
                    "filename" | "size" | "md5sum" | "sha1" | "sha256" | "sha512" | "sha3-256"
                )
                || !fields.insert(field)
            {
                return Err("Unexpected or repeated APT control field".into());
            }
        }
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(line);
    }
    Ok(result)
}

fn apt_directory(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let mut path = root.to_owned();
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            return Err("APT directory must be relative to its publication root".into());
        };
        path.push(name);
        if let Err(error) = fs::create_dir(&path) {
            if error.kind() != io::ErrorKind::AlreadyExists {
                return Err(path_error(&path, error));
            }
        }
        if !is_plain_directory(&metadata(&path)?) {
            return Err("APT publication directory must not be a symlink or reparse point".into());
        }
    }
    Ok(path)
}

fn publish_file_if_changed(parent: &Path, name: &Path, bytes: &[u8]) -> Result<(), String> {
    let path = parent.join(name);
    match fs::symlink_metadata(&path) {
        Ok(meta) if is_plain_file(&meta) => {
            if meta.len() == bytes.len() as u64
                && fs::read(&path).map_err(|error| path_error(&path, error))? == bytes
            {
                return Ok(());
            }
        }
        Ok(_) => return Err("Package output must be a regular file".into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(path_error(&path, error)),
    }
    publish_archive(&path, parent, bytes)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AptStewardTrust {
    host_domain: String,
    steward_fingerprints: BTreeMap<String, String>,
}

fn verify_apt_signatures(
    suite_root: &Path,
    release: &[u8],
    host_domain: &str,
    threshold: u8,
    stewards: &BTreeSet<&str>,
) -> Result<(Vec<u8>, String), String> {
    let directory = apt_signing_directory(suite_root);
    if !is_plain_directory(&metadata(&directory)?) {
        return Err(
            "APT signing proofs must come from the host-provisioned signing directory".into(),
        );
    }
    let trust_directory = std::env::var_os("WEBIZEN_APT_TRUST_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or("Signed APT publication requires an absolute WEBIZEN_APT_TRUST_DIR")?;
    if !is_plain_directory(&metadata(&trust_directory)?) {
        return Err(
            "APT trust configuration must be an independently provisioned directory".into(),
        );
    }
    let trust_directory = trust_directory
        .canonicalize()
        .map_err(|error| path_error(&trust_directory, error))?;
    let delivery_directory = directory
        .parent()
        .ok_or("APT proof input is outside the delivery directory")?;
    let delivery_directory = delivery_directory
        .canonicalize()
        .map_err(|error| path_error(delivery_directory, error))?;
    if trust_directory.starts_with(&delivery_directory) {
        return Err("APT trust configuration must stay outside the generated delivery tree".into());
    }
    let trust_file = trust_directory.join("stewards.json");
    let trust: AptStewardTrust = serde_json::from_slice(&read_apt_trust_file(&trust_file)?)
        .map_err(|error| format!("Invalid APT steward-key policy: {error}"))?;
    if trust.host_domain != host_domain {
        return Err("APT steward-key policy was provisioned for a different host".into());
    }
    let mut allowed = BTreeSet::new();
    for (did, key) in &trust.steward_fingerprints {
        let fingerprint = key.to_ascii_uppercase();
        if !stewards.contains(did.as_str())
            || !valid_fingerprint(&fingerprint)
            || !allowed.insert(fingerprint)
        {
            return Err(
                "APT trust policy requires distinct fingerprints for eligible registered stewards"
                    .into(),
            );
        }
    }
    if allowed.len() < usize::from(threshold) {
        return Err("APT trust policy has insufficient distinct steward keys".into());
    }
    let keyring = trust_directory.join("stewards.gpg");
    if !is_plain_file(&metadata(&keyring)?) {
        return Err("APT verifier keyring must be a regular file".into());
    }
    let keyring = keyring
        .canonicalize()
        .map_err(|error| path_error(&keyring, error))?;
    let armor = read_apt_trust_file(&directory.join("InRelease"))?;
    let armor_text = std::str::from_utf8(&armor).map_err(|_| "APT InRelease armor is not UTF-8")?;
    let armor_normalized = armor_text.replace("\r\n", "\n");
    let signature = apt_signature_armor(&armor_normalized)?;
    let (cleartext, status) = run_gpgv(&keyring, &armor)?;
    let cleartext = std::str::from_utf8(&cleartext)
        .map_err(|_| "APT signature does not cover a text Release")?
        .replace("\r\n", "\n");
    let released = std::str::from_utf8(release).map_err(|_| "Invalid staged APT Release text")?;
    if cleartext.strip_suffix('\n').unwrap_or(&cleartext)
        != released.strip_suffix('\n').unwrap_or(released)
    {
        return Err("APT InRelease is signed for different Release contents".into());
    }
    verify_gpg_status(&status, &allowed, threshold)?;
    Ok((armor, hex_bytes(signature.as_bytes())))
}

fn read_apt_trust_file(path: &Path) -> Result<Vec<u8>, String> {
    let meta = metadata(path)?;
    if !is_plain_file(&meta) || meta.len() > MAX_APT_TRUST_BYTES {
        return Err("APT signing evidence must be a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(MAX_APT_TRUST_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|error| path_error(path, error))?;
    if bytes.len() as u64 > MAX_APT_TRUST_BYTES {
        return Err("APT signing evidence exceeds the size limit".into());
    }
    Ok(bytes)
}

fn apt_signature_armor(text: &str) -> Result<&str, String> {
    const START: &str = "-----BEGIN PGP SIGNED MESSAGE-----\n";
    const SIGNATURE: &str = "\n-----BEGIN PGP SIGNATURE-----\n";
    const END: &str = "\n-----END PGP SIGNATURE-----";
    if !text.starts_with(START)
        || text.matches(SIGNATURE).count() != 1
        || text.matches(END).count() != 1
    {
        return Err("APT InRelease must be an OpenPGP clear-signed document".into());
    }
    let (_, signature) = text
        .split_once(SIGNATURE)
        .ok_or("Missing APT signature block")?;
    let (signature_body, trailing) = signature
        .split_once(END)
        .ok_or("Missing APT signature end marker")?;
    if signature_body.trim().is_empty() || !matches!(trailing, "" | "\n") {
        return Err("Malformed OpenPGP signature armor".into());
    }
    let offset = text.len() - signature.len() - SIGNATURE.len() + 1;
    Ok(text[offset..].strip_suffix('\n').unwrap_or(&text[offset..]))
}

struct AptVerifierScratch(PathBuf);

impl AptVerifierScratch {
    fn create() -> Result<Self, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "APT verifier clock precedes the Unix epoch")?
            .as_nanos();
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "webizen-apt-verifier-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        #[cfg_attr(not(unix), allow(unused_mut))]
        let mut directory = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            directory.mode(0o700);
        }
        directory
            .create(&path)
            .map_err(|error| path_error(&path, error))?;
        Ok(Self(path))
    }

    fn new_file(&self, name: &str) -> Result<File, String> {
        let path = self.0.join(name);
        File::create_new(&path).map_err(|error| path_error(&path, error))
    }
}

impl Drop for AptVerifierScratch {
    fn drop(&mut self) {
        for name in ["signed", "cleartext", "status"] {
            let _ = fs::remove_file(self.0.join(name));
        }
        let _ = fs::remove_dir(&self.0);
    }
}

fn run_gpgv(keyring: &Path, armor: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let executable = std::env::var_os("WEBIZEN_GPGV_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/bin/gpgv"));
    if !executable.is_absolute() || !executable.is_file() {
        return Err(
            "Set WEBIZEN_GPGV_BIN to an absolute path to an installed gpgv verifier".into(),
        );
    }
    let scratch = AptVerifierScratch::create()?;
    let mut input = scratch.new_file("signed")?;
    input
        .write_all(armor)
        .map_err(|error| path_error(&scratch.0, error))?;
    drop(input);
    let input =
        File::open(scratch.0.join("signed")).map_err(|error| path_error(&scratch.0, error))?;
    let discarded_log = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let verifier = Command::new(executable)
        .args([
            "--log-file",
            discarded_log,
            "--status-fd",
            "2",
            "--output",
            "-",
            "--keyring",
        ])
        .arg(keyring)
        .arg("-")
        .stdin(Stdio::from(input))
        .stdout(Stdio::from(scratch.new_file("cleartext")?))
        .stderr(Stdio::from(scratch.new_file("status")?))
        .spawn()
        .map_err(|error| format!("Cannot start APT OpenPGP verifier: {error}"))?;
    let status = crate::updater::wait_for_verifier(verifier, std::time::Duration::from_secs(30))
        .map_err(|error| format!("APT OpenPGP verification did not complete: {error}"))?;
    if !status.success() {
        return Err("APT InRelease failed OpenPGP verification".into());
    }
    let cleartext = read_apt_trust_file(&scratch.0.join("cleartext"))?;
    let status = read_apt_trust_file(&scratch.0.join("status"))?;
    Ok((cleartext, status))
}

fn valid_fingerprint(key: &str) -> bool {
    matches!(key.len(), 40 | 64) && key.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn verify_gpg_status(
    status: &[u8],
    allowed: &BTreeSet<String>,
    threshold: u8,
) -> Result<(), String> {
    let status = String::from_utf8_lossy(status);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "APT verifier clock precedes the Unix epoch")?
        .as_secs();
    let mut signers = BTreeSet::new();
    for line in status
        .lines()
        .filter_map(|line| line.strip_prefix("[GNUPG:] "))
    {
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        match fields.first().copied() {
            Some("BADSIG" | "ERRSIG" | "EXPSIG" | "EXPKEYSIG" | "REVKEYSIG" | "NO_PUBKEY") => {
                return Err(
                    "APT verifier reported an invalid, expired or revoked signature".into(),
                );
            }
            Some("VALIDSIG") => {
                let fingerprint = fields
                    .get(10)
                    .or_else(|| fields.get(1))
                    .ok_or("APT signature status lacks a fingerprint")?
                    .to_ascii_uppercase();
                let expiration = fields
                    .get(4)
                    .and_then(|field| field.parse::<u64>().ok())
                    .ok_or("APT signature status lacks an expiration field")?;
                if expiration != 0 && expiration <= now {
                    return Err("APT verifier reported an expired signature".into());
                }
                if !allowed.contains(&fingerprint)
                    || !matches!(fields.get(8), Some(&("8" | "9" | "10")))
                    || fields.get(9) != Some(&"01")
                {
                    return Err("APT signature is outside the trusted roster or lacks strong text-signing parameters".into());
                }
                signers.insert(fingerprint);
            }
            _ => {}
        }
    }
    if threshold == 0 || signers.len() < usize::from(threshold) {
        return Err(
            "APT InRelease lacks the configured quorum of distinct steward signatures".into(),
        );
    }
    Ok(())
}

/// Verify the three required ar members in order and return their unpadded sizes.
pub fn verify_deb_ar_container(bytes: &[u8]) -> Result<Vec<(String, usize)>, String> {
    let mut rest = bytes
        .strip_prefix(AR_MAGIC)
        .ok_or("invalid ar archive signature")?;
    let mut members = Vec::with_capacity(AR_MEMBERS.len());
    for expected in AR_MEMBERS {
        let (header, payload) = rest
            .split_at_checked(60)
            .ok_or("truncated ar member header")?;
        if &header[58..60] != b"`\n" {
            return Err("invalid ar member header terminator".into());
        }
        let name = std::str::from_utf8(&header[..16])
            .map_err(|_| "ar member name is not UTF-8")?
            .trim_end_matches(' ');
        if name.strip_suffix('/').unwrap_or(name) != expected {
            return Err(format!("expected ar member {expected}, found {name}"));
        }
        let raw_size = std::str::from_utf8(&header[48..58])
            .map_err(|_| "invalid ar member size")?
            .trim_matches(' ');
        if raw_size.is_empty() || !raw_size.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("invalid ar member size".into());
        }
        let size = raw_size
            .parse::<usize>()
            .map_err(|_| "ar member size overflow")?;
        let (content, next) = payload
            .split_at_checked(size)
            .ok_or("truncated ar member payload")?;
        match expected {
            "debian-binary" if content != b"2.0\n" => {
                return Err("unsupported Debian package version".into())
            }
            "control.tar.gz" | "data.tar.gz" if !content.starts_with(&[0x1f, 0x8b, 8]) => {
                return Err(format!("ar member {expected} is missing the gzip header"));
            }
            _ => {}
        }
        rest = if size % 2 == 1 {
            next.strip_prefix(b"\n")
                .ok_or("invalid ar member padding")?
        } else {
            next
        };
        members.push((expected.to_owned(), size));
    }
    if !rest.is_empty() {
        return Err("unexpected trailing data after the Debian package members".into());
    }
    Ok(members)
}

struct TarEntry {
    name: String,
    path: PathBuf,
    directory: bool,
}

fn visit_tree(
    dir: &Path,
    prefix: &str,
    exclude_debian: bool,
    entries: &mut Vec<TarEntry>,
) -> Result<(), String> {
    for child in fs::read_dir(dir).map_err(|error| path_error(dir, error))? {
        let child = child.map_err(|error| path_error(dir, error))?;
        let path = child.path();
        let name = child.file_name();
        if exclude_debian && name == "DEBIAN" {
            continue;
        }
        let component = name
            .to_str()
            .filter(|name| {
                !name.contains(['/', '\\'])
                    && !name.chars().any(char::is_control)
                    && *name != ".."
                    && *name != "."
            })
            .ok_or_else(|| format!("invalid portable archive path at {}", path.display()))?;
        let info = metadata(&path)?;
        let directory = is_plain_directory(&info);
        if !directory && !is_plain_file(&info) {
            return Err(format!("unsupported package entry at {}", path.display()));
        }
        let mut name = format!("{prefix}{component}");
        if directory {
            name.push('/');
            visit_tree(&path, &name, false, entries)?;
        }
        entries.push(TarEntry {
            name,
            path,
            directory,
        });
    }
    Ok(())
}

#[derive(Default)]
struct PayloadChecksums {
    md5: String,
    sha3: String,
}

enum TarContent {
    Staged(TarEntry),
    Generated(&'static str, Vec<u8>),
}

impl TarContent {
    fn name(&self) -> &str {
        match self {
            Self::Staged(entry) => &entry.name,
            Self::Generated(name, _) => name,
        }
    }
}

fn gzip_tar(
    root: &Path,
    exclude_debian: bool,
    generated: &[(&'static str, &[u8])],
    mut checksums: Option<&mut PayloadChecksums>,
) -> Result<(Vec<u8>, usize), String> {
    let mut staged = Vec::new();
    visit_tree(root, "./", exclude_debian, &mut staged)?;
    let mut entries: Vec<_> = staged
        .into_iter()
        .filter(|entry| !generated.iter().any(|(name, _)| *name == entry.name))
        .map(TarContent::Staged)
        .collect();
    for &(name, bytes) in generated {
        if entries.iter().any(|entry| {
            entry
                .name()
                .strip_prefix(name)
                .is_some_and(|rest| rest.starts_with('/'))
        }) {
            return Err(format!(
                "generated control entry conflicts with a directory: {name}"
            ));
        }
        entries.push(TarContent::Generated(name, bytes.to_vec()));
    }
    entries.sort_unstable_by(|left, right| left.name().cmp(right.name()));
    let mut tar = gzip_encoder();
    let mut count = 0;
    for entry in entries {
        match entry {
            TarContent::Staged(entry) if entry.directory => {
                let header = ustar_header(&entry.name, 0, true, true)?;
                tar.write_all(&header)
                    .map_err(|error| path_error(&entry.path, error))?;
            }
            TarContent::Staged(entry) => {
                append_file(&mut tar, &entry, checksums.as_deref_mut())?;
                count += 1;
            }
            TarContent::Generated(name, bytes) => {
                let size = bytes.len() as u64;
                let header = ustar_header(name, size, false, false)?;
                tar.write_all(&header)
                    .and_then(|()| tar.write_all(&bytes))
                    .and_then(|()| pad_tar_entry(&mut tar, size))
                    .map_err(|error| path_error(root, error))?;
                count += 1;
            }
        }
    }
    tar.write_all(&ZEROES)
        .map_err(|error| path_error(root, error))?;
    Ok((
        tar.finish().map_err(|error| path_error(root, error))?,
        count,
    ))
}

fn gzip_encoder() -> flate2::write::GzEncoder<Vec<u8>> {
    GzBuilder::new()
        .mtime(0)
        .operating_system(3)
        .write(Vec::new(), Compression::default())
}

fn pad_tar_entry(tar: &mut impl Write, len: u64) -> io::Result<()> {
    let remainder = (len % TAR_BLOCK_BYTES as u64) as usize;
    if remainder != 0 {
        tar.write_all(&ZEROES[..TAR_BLOCK_BYTES - remainder])?;
    }
    Ok(())
}

fn append_file(
    mut tar: impl Write,
    entry: &TarEntry,
    checksums: Option<&mut PayloadChecksums>,
) -> Result<(), String> {
    let path = &entry.path;
    if !is_plain_file(&metadata(path)?) {
        return Err(format!("unsupported package entry at {}", path.display()));
    }
    let file = File::open(path).map_err(|error| path_error(path, error))?;
    let len = file
        .metadata()
        .map_err(|error| path_error(path, error))?
        .len();
    let mut input = BufReader::new(file);
    let shebang = input
        .fill_buf()
        .map_err(|error| path_error(path, error))?
        .starts_with(b"#!");
    let executable = shebang
        || matches!(entry.name.as_str(), "./postinst" | "./prerm")
        || entry.name.ends_with(".sh")
        || entry.name.starts_with("./opt/webizen/bin/")
        || entry.name.starts_with("./usr/local/bin/");
    let header = ustar_header(&entry.name, len, false, executable)?;
    tar.write_all(&header)
        .map_err(|error| path_error(path, error))?;
    let copied = if let Some(checksums) = checksums {
        let mut sha3 = Sha3_256::new();
        let mut md5 = Md5::default();
        let mut remaining = input.by_ref().take(len);
        let mut buffer = [0u8; 16 * 1024];
        loop {
            let count = remaining
                .read(&mut buffer)
                .map_err(|error| path_error(path, error))?;
            if count == 0 {
                break;
            }
            let bytes = &buffer[..count];
            tar.write_all(bytes)
                .map_err(|error| path_error(path, error))?;
            sha3.update(bytes);
            md5.update(bytes);
        }
        let name = entry.name.strip_prefix("./").unwrap_or(&entry.name);
        let _ = writeln!(checksums.md5, "{}  {name}", hex_bytes(&md5.finish()));
        let _ = writeln!(checksums.sha3, "{:x}  {name}", sha3.finalize());
        len - remaining.limit()
    } else {
        io::copy(&mut input.by_ref().take(len), &mut tar)
            .map_err(|error| path_error(path, error))?
    };
    if copied != len
        || !input
            .fill_buf()
            .map_err(|error| path_error(path, error))?
            .is_empty()
    {
        return Err(format!(
            "package entry changed size while reading {}",
            path.display()
        ));
    }
    pad_tar_entry(&mut tar, len).map_err(|error| path_error(path, error))
}

const MD5_ROTATIONS: [u32; 16] = [7, 12, 17, 22, 5, 9, 14, 20, 4, 11, 16, 23, 6, 10, 15, 21];
const MD5_CONSTANTS: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// RFC 1321 checksums for compatibility with dpkg's package file manifest.
struct Md5 {
    words: [u32; 4],
    tail: [u8; 64],
    length: u64,
}

impl Default for Md5 {
    fn default() -> Self {
        Self {
            words: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            tail: [0; 64],
            length: 0,
        }
    }
}

impl Md5 {
    fn update(&mut self, mut bytes: &[u8]) {
        let used = (self.length % 64) as usize;
        self.length = self.length.wrapping_add(bytes.len() as u64);
        if used != 0 {
            let take = (64 - used).min(bytes.len());
            self.tail[used..used + take].copy_from_slice(&bytes[..take]);
            if used + take != 64 {
                return;
            }
            md5_block(&mut self.words, &self.tail);
            bytes = &bytes[take..];
        }
        let mut blocks = bytes.chunks_exact(64);
        for block in &mut blocks {
            md5_block(&mut self.words, block);
        }
        let tail = blocks.remainder();
        self.tail[..tail.len()].copy_from_slice(tail);
    }

    fn finish(mut self) -> [u8; 16] {
        let remaining = (self.length % 64) as usize;
        let (padded, length) = hash_padding(
            &self.tail[..remaining],
            self.length.wrapping_mul(8).to_le_bytes(),
        );
        for block in padded[..length].chunks_exact(64) {
            md5_block(&mut self.words, block);
        }
        let mut result = [0; 16];
        for (out, word) in result.chunks_exact_mut(4).zip(self.words) {
            out.copy_from_slice(&word.to_le_bytes());
        }
        result
    }
}

fn md5_block(state: &mut [u32; 4], block: &[u8]) {
    let words: [u32; 16] = std::array::from_fn(|index| {
        let start = index * 4;
        u32::from_le_bytes(block[start..start + 4].try_into().expect("four bytes"))
    });
    let [mut a, mut b, mut c, mut d] = *state;
    for (round, constant) in MD5_CONSTANTS.into_iter().enumerate() {
        let (mixed, index) = match round {
            0..=15 => ((b & c) | (!b & d), round),
            16..=31 => ((d & b) | (!d & c), (5 * round + 1) % 16),
            32..=47 => (b ^ c ^ d, (3 * round + 5) % 16),
            _ => (c ^ (b | !d), (7 * round) % 16),
        };
        let rotated = a
            .wrapping_add(mixed)
            .wrapping_add(constant)
            .wrapping_add(words[index])
            .rotate_left(MD5_ROTATIONS[(round / 16) * 4 + round % 4]);
        (a, b, c, d) = (d, b.wrapping_add(rotated), b, c);
    }
    for (word, mixed) in state.iter_mut().zip([a, b, c, d]) {
        *word = word.wrapping_add(mixed);
    }
}

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hex(bytes: &[u8]) -> String {
    let mut state = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut blocks = bytes.chunks_exact(64);
    let (padding, len) = hash_padding(
        blocks.remainder(),
        (bytes.len() as u64).wrapping_mul(8).to_be_bytes(),
    );
    for block in blocks.by_ref().chain(padding[..len].chunks_exact(64)) {
        sha256_block(&mut state, block);
    }
    let mut hex = String::with_capacity(64);
    for word in state {
        let _ = write!(hex, "{word:08x}");
    }
    hex
}

fn sha256_block(state: &mut [u32; 8], block: &[u8]) {
    let mut schedule = [0u32; 64];
    for (word, chunk) in schedule.iter_mut().zip(block.chunks_exact(4)) {
        *word = u32::from_be_bytes(chunk.try_into().expect("four bytes"));
    }
    for index in 16..64 {
        let x = schedule[index - 15];
        let y = schedule[index - 2];
        let small_sigma0 = x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3);
        let small_sigma1 = y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(small_sigma0)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(small_sigma1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for (word, constant) in schedule.into_iter().zip(SHA256_ROUND_CONSTANTS) {
        let sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ (!e & g);
        let first = h
            .wrapping_add(sigma1)
            .wrapping_add(choose)
            .wrapping_add(constant)
            .wrapping_add(word);
        let sigma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let second = sigma0.wrapping_add(majority);
        (a, b, c, d, e, f, g, h) = (
            first.wrapping_add(second),
            a,
            b,
            c,
            d.wrapping_add(first),
            e,
            f,
            g,
        );
    }
    for (word, mixed) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *word = word.wrapping_add(mixed);
    }
}

fn hash_padding(tail: &[u8], bit_length: [u8; 8]) -> ([u8; 128], usize) {
    let mut padding = [0; 128];
    padding[..tail.len()].copy_from_slice(tail);
    padding[tail.len()] = 0x80;
    let len = if tail.len() < 56 { 64 } else { 128 };
    padding[len - 8..len].copy_from_slice(&bit_length);
    (padding, len)
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn ustar_header(
    name: &str,
    len: u64,
    directory: bool,
    executable: bool,
) -> Result<[u8; TAR_BLOCK_BYTES], String> {
    let (prefix, tail) = if name.len() <= 100 {
        ("", name)
    } else {
        name.rmatch_indices('/')
            .map(|(at, _)| (&name[..at], &name[at + 1..]))
            .find(|(prefix, tail)| prefix.len() <= 155 && !tail.is_empty() && tail.len() <= 100)
            .ok_or_else(|| format!("package path does not fit POSIX ustar: {name}"))?
    };
    let size = format!("{len:011o}\0");
    if size.len() != 12 {
        return Err(format!(
            "package entry size does not fit POSIX ustar: {name}"
        ));
    }
    let mut header = [0; TAR_BLOCK_BYTES];
    header[..tail.len()].copy_from_slice(tail.as_bytes());
    header[100..108].copy_from_slice(if executable {
        b"0000755\0"
    } else {
        b"0000644\0"
    });
    header[108..116].copy_from_slice(b"0000000\0");
    header[116..124].copy_from_slice(b"0000000\0");
    header[124..136].copy_from_slice(size.as_bytes());
    header[136..148].copy_from_slice(b"00000000000\0");
    header[148..156].fill(b' ');
    header[156] = if directory { b'5' } else { b'0' };
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    header[265..269].copy_from_slice(b"root");
    header[297..301].copy_from_slice(b"root");
    header[329..337].copy_from_slice(b"0000000\0");
    header[337..345].copy_from_slice(b"0000000\0");
    header[345..345 + prefix.len()].copy_from_slice(prefix.as_bytes());
    let checksum: u32 = header.iter().map(|&byte| u32::from(byte)).sum();
    let checksum = format!("{checksum:06o}\0 ");
    header[148..156].copy_from_slice(checksum.as_bytes());
    Ok(header)
}

fn write_ar_member(archive: &mut Vec<u8>, name: &str, bytes: &[u8]) -> Result<(), String> {
    let size = bytes.len();
    let header = format!(
        "{name:<16}{:<12}{:<6}{:<6}{:<8}{size:<10}`\n",
        0, 0, 0, "100644"
    );
    if header.len() != 60 {
        return Err(format!(
            "package member {name} exceeds the ar member size limit"
        ));
    }
    archive.extend_from_slice(header.as_bytes());
    archive.extend_from_slice(bytes);
    if size % 2 == 1 {
        archive.push(b'\n');
    }
    Ok(())
}

fn publish_archive(path: &Path, parent: &Path, bytes: &[u8]) -> Result<(), String> {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".webizen-deb-{}-{sequence}.tmp",
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| path_error(&temporary, error))?;
    let result = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let result = result.and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| path_error(path, error))
}

fn metadata(path: &Path) -> Result<Metadata, String> {
    fs::symlink_metadata(path).map_err(|error| path_error(path, error))
}

fn is_plain_directory(metadata: &Metadata) -> bool {
    metadata.is_dir() && !is_reparse_point(metadata)
}

fn is_plain_file(metadata: &Metadata) -> bool {
    metadata.is_file() && !is_reparse_point(metadata)
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_: &Metadata) -> bool {
    false
}

fn path_error(path: &Path, error: io::Error) -> String {
    format!("{}: {error}", path.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "webizen-deb-fixture-{}-{unique}-{}",
                std::process::id(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn stage_package(root: &Path, name: &str, arch: &str) -> PathBuf {
        let package = root.join(format!("{name}_0.1.0_{arch}"));
        fs::create_dir_all(package.join("DEBIAN")).unwrap();
        fs::create_dir_all(package.join("opt/webizen/bin")).unwrap();
        let long_component = "x".repeat(95);
        let settings = package.join("etc").join(&long_component);
        fs::create_dir_all(&settings).unwrap();
        fs::write(
            package.join("DEBIAN/control"),
            format!(
                "Package: {name}\nVersion: 0.1.0\nArchitecture: {arch}\nMaintainer: Steward <steward@example.test>\nDescription: Human-Centric fixture\n Package contents for testing.\n"
            ),
        )
        .unwrap();
        fs::write(package.join("DEBIAN/postinst"), b"#!/bin/sh\nexit 0\n").unwrap();
        fs::write(package.join("opt/webizen/bin/service"), [0u8, 1, 2, 3]).unwrap();
        fs::write(settings.join("settings.conf"), b"enabled=true\n").unwrap();
        package
    }

    fn member_contents<'a>(mut bytes: &'a [u8], sizes: &[(String, usize)]) -> Vec<&'a [u8]> {
        bytes = bytes.strip_prefix(AR_MAGIC).unwrap();
        sizes
            .iter()
            .map(|(_, len)| {
                let payload = &bytes[60..60 + *len];
                bytes = &bytes[60 + *len + (*len % 2)..];
                payload
            })
            .collect()
    }

    fn tar_entries(payload: &[u8]) -> Vec<(String, u32, u8)> {
        assert_eq!(&payload[..3], &[0x1f, 0x8b, 8]);
        assert_eq!(&payload[4..8], &[0; 4]);
        assert_eq!(payload[9], 3);
        let mut tar = Vec::new();
        GzDecoder::new(payload).read_to_end(&mut tar).unwrap();
        fn text(field: &[u8]) -> &str {
            std::str::from_utf8(field.split(|&byte| byte == 0).next().unwrap()).unwrap()
        }
        let mut entries = Vec::new();
        let mut remaining = &tar[..];
        while remaining != &ZEROES[..] {
            let (header, next) = remaining.split_at(TAR_BLOCK_BYTES);
            assert_eq!(&header[257..265], b"ustar\x0000");
            assert_eq!(&header[108..124], b"0000000\x000000000\x00");
            assert_eq!(&header[136..148], b"00000000000\0");
            let recorded = u32::from_str_radix(text(&header[148..156]), 8).unwrap();
            let actual: u32 = header
                .iter()
                .enumerate()
                .map(|(index, &byte)| {
                    u32::from(if (148..156).contains(&index) {
                        b' '
                    } else {
                        byte
                    })
                })
                .sum();
            assert_eq!(recorded, actual);
            let size = usize::from_str_radix(text(&header[124..136]), 8).unwrap();
            let mode = u32::from_str_radix(text(&header[100..108]), 8).unwrap();
            let tail = text(&header[..100]);
            let prefix = text(&header[345..500]);
            let name = if prefix.is_empty() {
                tail.to_owned()
            } else {
                format!("{prefix}/{tail}")
            };
            entries.push((name, mode, header[156]));
            let padded = size.div_ceil(TAR_BLOCK_BYTES) * TAR_BLOCK_BYTES;
            assert!(next[size..padded].iter().all(|&byte| byte == 0));
            remaining = &next[padded..];
        }
        entries
    }

    #[test]
    fn builds_deterministic_ar_and_ustar_tri_target_debian_packages() {
        let tmp = Fixture::new();
        let targets = [
            ("webizen-desktop-appliance", "amd64"),
            ("webizen-mobile-avf", "arm64"),
            ("webizen-hostd", "amd64"),
        ];
        let [desktop_deb_tree, mobile_avf_deb_tree, host_deb_tree] =
            targets.map(|(name, arch)| stage_package(&tmp.0, name, arch));
        let suite = TriTargetDebianSuiteReceipt {
            suite_root_dir: tmp.0.clone(),
            desktop_deb_tree,
            mobile_avf_deb_tree,
            host_deb_tree,
            built_deb_packages: Vec::new(),
            apt_repository: None,
            host_state_cbor_ld_path: PathBuf::new(),
            host_quorum_gate_script_path: PathBuf::new(),
            avf_crosvm_launcher_path: PathBuf::new(),
            build_all_debs_script_path: PathBuf::new(),
            steward_count: 0,
            quorum_threshold_m: 0,
        };
        let receipts = build_tri_target_debs_from_receipt(&suite).unwrap();
        assert_eq!(receipts.len(), 3);
        let expected_settings_path = format!("./etc/{}/settings.conf", "x".repeat(95));
        for ((name, arch), receipt) in targets.into_iter().zip(&receipts) {
            assert_eq!(receipt.package_name, name);
            assert_eq!(receipt.architecture, arch);
            assert_eq!(receipt.version, "0.1.0");
            assert_eq!(receipt.member_file_count, 6);
            assert_eq!(receipt.sha3_256_hex.len(), 64);
            assert_eq!(
                receipt.output_deb_path,
                tmp.0.join(format!("{name}_0.1.0_{arch}.deb"))
            );
            let bytes = fs::read(&receipt.output_deb_path).unwrap();
            assert_eq!(receipt.archive_bytes, bytes.len() as u64);
            assert_eq!(
                receipt.sha3_256_hex,
                format!("{:x}", Sha3_256::digest(&bytes))
            );
            let members = verify_deb_ar_container(&bytes).unwrap();
            assert_eq!(
                members,
                vec![
                    ("debian-binary".into(), 4),
                    (
                        "control.tar.gz".into(),
                        receipt.control_tar_gz_bytes as usize
                    ),
                    ("data.tar.gz".into(), receipt.data_tar_gz_bytes as usize),
                ]
            );
            let payloads = member_contents(&bytes, &members);
            assert_eq!(payloads[0], b"2.0\n");
            assert_eq!(
                tar_entries(payloads[1]),
                vec![
                    ("./control".into(), 0o644, b'0'),
                    ("./md5sums".into(), 0o644, b'0'),
                    ("./postinst".into(), 0o755, b'0'),
                    ("./sha3sums".into(), 0o644, b'0')
                ]
            );
            let mut manifest = Vec::new();
            GzDecoder::new(payloads[1])
                .read_to_end(&mut manifest)
                .unwrap();
            let expected_md5 = b"37b59afd592725f9305e484a5d7f5168  opt/webizen/bin/service\n";
            assert!(manifest
                .windows(expected_md5.len())
                .any(|entry| entry == expected_md5));
            let expected_sha3 = format!(
                "{:x}  opt/webizen/bin/service\n",
                Sha3_256::digest([0, 1, 2, 3])
            );
            assert!(manifest
                .windows(expected_sha3.len())
                .any(|entry| entry == expected_sha3.as_bytes()));
            let data = tar_entries(payloads[2]);
            assert!(data.windows(2).all(|pair| pair[0].0 < pair[1].0));
            assert!(data.contains(&(expected_settings_path.clone(), 0o644, b'0')));
            assert!(data.contains(&("./opt/webizen/bin/service".into(), 0o755, b'0')));
            assert!(data.contains(&("./opt/webizen/bin/".into(), 0o755, b'5')));
            assert!(data
                .iter()
                .all(|(name, _, _)| !name.starts_with("./DEBIAN")));
            assert!(verify_deb_ar_container(&bytes[..bytes.len() - 1]).is_err());
        }
        assert_eq!(
            build_tri_target_debs_from_receipt(&suite).unwrap(),
            receipts
        );
    }

    #[test]
    fn packaging_digests_match_standard_vectors_and_date_format() {
        let mut md5 = Md5::default();
        for digits in b"1234567890".repeat(8).chunks(7) {
            md5.update(digits);
        }
        assert_eq!(hex_bytes(&md5.finish()), "57edf4a22be3c955ac49da2e2107b67a");
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(format_apt_date(0).unwrap(), "Thu, 01 Jan 1970 00:00:00 UTC");
        assert_eq!(
            format_apt_date(951_782_400).unwrap(),
            "Tue, 29 Feb 2000 00:00:00 UTC"
        );
    }

    #[test]
    fn generates_signed_apt_repository_with_packages_and_inrelease() {
        let fixture = Fixture::new();
        let repo_root = fixture.0.join("suite");
        fs::create_dir(&repo_root).unwrap();
        let packages: Vec<_> = [
            ("webizen-desktop-appliance", "amd64"),
            ("webizen-mobile-avf", "arm64"),
            ("webizen-hostd", "amd64"),
        ]
        .into_iter()
        .map(|(package, arch)| {
            let staged = stage_package(&repo_root, package, arch);
            build_deb_from_tree(
                &staged,
                &repo_root.join(format!("{package}_0.1.0_{arch}.deb")),
            )
            .unwrap()
        })
        .collect();
        let dids = [
            "did:qi:steward:desktop".to_owned(),
            "did:qi:steward:pocket".to_owned(),
        ];
        let staged =
            generate_apt_repository_indexes(&repo_root, &packages, "host.example.test", 2, &dids)
                .unwrap();
        assert!(!staged.signed);
        assert!(staged.inrelease_path.as_os_str().is_empty());
        assert!(!repo_root.join("dists/trixie/InRelease").exists());
        let release = fs::read_to_string(&staged.release_path).unwrap();
        assert!(release.contains("\nDate: "));
        assert!(release.contains("\nX-Webizen-Quorum-Threshold: 2\n"));
        for (path, expected, other) in [
            (
                &staged.packages_amd64_path,
                "webizen-desktop-appliance",
                "webizen-mobile-avf",
            ),
            (
                &staged.packages_arm64_path,
                "webizen-mobile-avf",
                "webizen-desktop-appliance",
            ),
        ] {
            let index = fs::read_to_string(path).unwrap();
            assert!(index.contains(&format!("Package: {expected}\n")));
            assert!(!index.contains(&format!("Package: {other}\n")));
            assert!(index.contains("\nSHA256: ") && index.contains("\nSHA3-256: "));
            let mut unzipped = String::new();
            GzDecoder::new(File::open(path.with_extension("gz")).unwrap())
                .read_to_string(&mut unzipped)
                .unwrap();
            assert_eq!(unzipped, index);
            let index_hash = sha256_hex(index.as_bytes());
            assert!(release.contains(&index_hash));
            let by_hash = path
                .parent()
                .unwrap()
                .join("by-hash/SHA256")
                .join(index_hash);
            assert_eq!(fs::read_to_string(by_hash).unwrap(), index);
            for filename in index
                .lines()
                .filter_map(|line| line.strip_prefix("Filename: "))
            {
                assert!(repo_root.join(filename).is_file());
            }
        }
        assert_eq!(
            generate_apt_repository_indexes(&repo_root, &packages, "host.example.test", 2, &dids)
                .unwrap(),
            staged
        );
        assert!(generate_signed_apt_repository(
            &repo_root,
            &packages,
            "host.example.test",
            2,
            &dids
        )
        .is_err());
        assert!(!repo_root.join("dists/trixie/InRelease").exists());
        assert!(apt_signature_armor("-----BEGIN WEBIZEN QUORUM ATTESTATION-----\ndigest").is_err());
    }

    #[test]
    fn apt_threshold_counts_distinct_provisioned_primary_keys() {
        let desktop = "A".repeat(40);
        let pocket = "B".repeat(40);
        let subkey = "C".repeat(40);
        let allowed = BTreeSet::from([desktop.clone(), pocket.clone()]);
        let status_line = |signer: &str, primary: &str, expiry: u64| {
            format!(
                "[GNUPG:] VALIDSIG {signer} 2026-11-22 1795316400 {expiry} 4 0 22 8 01 {primary}\n"
            )
        };
        let desktop_sig = status_line(&desktop, &desktop, 0);
        let desktop_subkey_sig = status_line(&subkey, &desktop, 0);
        let pocket_sig = status_line(&pocket, &pocket, 0);
        assert!(
            verify_gpg_status(format!("{desktop_sig}{pocket_sig}").as_bytes(), &allowed, 2).is_ok()
        );
        assert!(verify_gpg_status(
            format!("{desktop_sig}{desktop_subkey_sig}").as_bytes(),
            &allowed,
            2
        )
        .is_err());
        assert!(
            verify_gpg_status(status_line(&desktop, &desktop, 1).as_bytes(), &allowed, 1).is_err()
        );
        assert!(verify_gpg_status(
            format!("{desktop_sig}[GNUPG:] REVKEYSIG\n").as_bytes(),
            &allowed,
            1
        )
        .is_err());
    }

    #[cfg(unix)]
    #[test]
    fn package_tree_rejects_symbolic_links() {
        let tmp = Fixture::new();
        let tree = stage_package(&tmp.0, "webizen-hostd", "amd64");
        let target = tmp.0.join("outside.txt");
        fs::write(&target, b"not staged").unwrap();
        std::os::unix::fs::symlink(target, tree.join("etc/escape")).unwrap();
        let output = tmp.0.join("package.deb");
        assert!(build_deb_from_tree(&tree, &output).is_err());
        assert!(!output.exists());
    }
}
