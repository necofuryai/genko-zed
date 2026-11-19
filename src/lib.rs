use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use zed_extension_api::{self as zed, Architecture, DownloadedFileType, Os};

const LANGUAGE_SERVER_NAME: &str = "genko-ls";
const RELEASE_REPOSITORY: &str = "necofuryai/genko-zed";

struct GenkoExtension {
    cached_binary_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AssetSpec {
    name: &'static str,
    file_type: DownloadedFileType,
    binary_name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Vec<PreReleaseId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PreReleaseId {
    Numeric(u64),
    AlphaNumeric(String),
}

impl PartialOrd for PreReleaseId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreReleaseId {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Numeric(left), Self::Numeric(right)) => left.cmp(right),
            (Self::Numeric(_), Self::AlphaNumeric(_)) => Ordering::Less,
            (Self::AlphaNumeric(_), Self::Numeric(_)) => Ordering::Greater,
            (Self::AlphaNumeric(left), Self::AlphaNumeric(right)) => left.cmp(right),
        }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(
                || match (self.pre_release.is_empty(), other.pre_release.is_empty()) {
                    (true, true) => Ordering::Equal,
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    (false, false) => self.pre_release.cmp(&other.pre_release),
                },
            )
    }
}

fn parse_ascii_u64(segment: &str) -> Option<u64> {
    if segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    segment.parse().ok()
}

fn parse_semver(version: &str) -> Option<SemVer> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let version = match version.split_once('+') {
        Some((without_build, build)) => {
            if build.is_empty() {
                return None;
            }
            without_build
        }
        None => version,
    };

    let (core, pre_release_str) = match version.split_once('-') {
        Some((core, pre_release)) => (core, Some(pre_release)),
        None => (version, None),
    };

    let mut parts = core.split('.');
    let major = parse_ascii_u64(parts.next()?)?;
    let minor = parse_ascii_u64(parts.next()?)?;
    let patch = parse_ascii_u64(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }

    let pre_release = match pre_release_str {
        Some(pre_release) => {
            if pre_release.is_empty() {
                return None;
            }
            let mut ids = Vec::new();
            for part in pre_release.split('.') {
                if part.is_empty() {
                    return None;
                }
                if let Some(number) = parse_ascii_u64(part) {
                    ids.push(PreReleaseId::Numeric(number));
                } else {
                    ids.push(PreReleaseId::AlphaNumeric(part.to_owned()));
                }
            }
            ids
        }
        None => Vec::new(),
    };

    Some(SemVer {
        major,
        minor,
        patch,
        pre_release,
    })
}

fn version_dir_suffix(entry_name: &str) -> Option<&str> {
    let suffix = entry_name
        .strip_prefix(LANGUAGE_SERVER_NAME)?
        .strip_prefix('-')?;
    if suffix.is_empty() {
        return None;
    }
    Some(suffix)
}

fn find_cached_binary(base_dir: &Path, binary_name: &str) -> Option<String> {
    let entries = fs::read_dir(base_dir).ok()?;
    let mut best: Option<(Option<SemVer>, String)> = None;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(dir_name) = file_name.to_str() else {
            continue;
        };
        let Some(version_str) = version_dir_suffix(dir_name) else {
            continue;
        };
        if !entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
            continue;
        }

        let candidate_path = entry.path().join(binary_name);
        if !fs::metadata(&candidate_path).is_ok_and(|metadata| metadata.is_file()) {
            continue;
        }

        let candidate = (parse_semver(version_str), dir_name.to_owned());
        if best.as_ref().is_none_or(|current| candidate > *current) {
            best = Some(candidate);
        }
    }

    best.map(|(_, dir_name)| format!("{dir_name}/{binary_name}"))
}

fn prune_stale_version_dirs(base_dir: &Path, active_version_dir: &str) {
    let Ok(entries) = fs::read_dir(base_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };

        if name == active_version_dir || version_dir_suffix(name).is_none() {
            continue;
        }

        if entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

fn asset_spec(os: Os, architecture: Architecture) -> zed::Result<AssetSpec> {
    let spec = match (os, architecture) {
        (Os::Mac, Architecture::Aarch64) => AssetSpec {
            name: "genko-ls-aarch64-apple-darwin.tar.gz",
            file_type: DownloadedFileType::GzipTar,
            binary_name: "genko-ls",
        },
        (Os::Mac, Architecture::X8664) => AssetSpec {
            name: "genko-ls-x86_64-apple-darwin.tar.gz",
            file_type: DownloadedFileType::GzipTar,
            binary_name: "genko-ls",
        },
        (Os::Linux, Architecture::Aarch64) => AssetSpec {
            name: "genko-ls-aarch64-unknown-linux-musl.tar.gz",
            file_type: DownloadedFileType::GzipTar,
            binary_name: "genko-ls",
        },
        (Os::Linux, Architecture::X8664) => AssetSpec {
            name: "genko-ls-x86_64-unknown-linux-musl.tar.gz",
            file_type: DownloadedFileType::GzipTar,
            binary_name: "genko-ls",
        },
        (Os::Windows, Architecture::Aarch64) => AssetSpec {
            name: "genko-ls-aarch64-pc-windows-msvc.zip",
            file_type: DownloadedFileType::Zip,
            binary_name: "genko-ls.exe",
        },
        (Os::Windows, Architecture::X8664) => AssetSpec {
            name: "genko-ls-x86_64-pc-windows-msvc.zip",
            file_type: DownloadedFileType::Zip,
            binary_name: "genko-ls.exe",
        },
        _ => {
            return Err(format!(
                "genko-ls does not provide a release asset for {os:?}/{architecture:?}"
            ));
        }
    };

    Ok(spec)
}

fn resolve_language_server_binary(
    language_server_id: &zed::LanguageServerId,
) -> zed::Result<String> {
    let (os, architecture) = zed::current_platform();
    let spec = asset_spec(os, architecture)?;

    let release = match zed::latest_github_release(
        RELEASE_REPOSITORY,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    ) {
        Ok(release) => release,
        Err(error) => {
            if let Some(binary_path) = find_cached_binary(Path::new("."), spec.binary_name) {
                return Ok(binary_path);
            }
            return Err(error);
        }
    };

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == spec.name)
        .ok_or_else(|| {
            format!(
                "release {} does not contain the expected asset {}",
                release.version, spec.name
            )
        })?;

    let version_dir = format!("{LANGUAGE_SERVER_NAME}-{}", release.version);
    let binary_path = format!("{version_dir}/{}", spec.binary_name);

    if !fs::metadata(&binary_path).is_ok_and(|metadata| metadata.is_file()) {
        fs::create_dir_all(&version_dir)
            .map_err(|error| format!("failed to create {version_dir}: {error}"))?;
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );
        zed::download_file(&asset.download_url, &version_dir, spec.file_type)
            .map_err(|error| format!("failed to download {}: {error}", spec.name))?;
        zed::make_file_executable(&binary_path)
            .map_err(|error| format!("failed to make {binary_path} executable: {error}"))?;
    }

    prune_stale_version_dirs(Path::new("."), &version_dir);
    Ok(binary_path)
}

impl GenkoExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        if let Some(path) = worktree.which(LANGUAGE_SERVER_NAME) {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path
            && fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
        {
            return Ok(path.clone());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        match resolve_language_server_binary(language_server_id) {
            Ok(binary_path) => {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::None,
                );
                self.cached_binary_path = Some(binary_path.clone());
                Ok(binary_path)
            }
            Err(error) => {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(error.clone()),
                );
                Err(error)
            }
        }
    }
}

impl zed::Extension for GenkoExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec!["--stdio".to_owned()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(GenkoExtension);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let unique = COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("genko-zed-{label}-{}-{unique}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("failed to create test temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn create_file(&self, relative: &str) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("failed to create parent directory");
            }
            fs::write(path, b"").expect("failed to create test file");
        }

        fn create_dir(&self, relative: &str) {
            fs::create_dir_all(self.path.join(relative)).expect("failed to create test directory");
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn selects_release_assets_for_supported_platforms() {
        let cases = [
            (
                Os::Mac,
                Architecture::Aarch64,
                "genko-ls-aarch64-apple-darwin.tar.gz",
                DownloadedFileType::GzipTar,
                "genko-ls",
            ),
            (
                Os::Mac,
                Architecture::X8664,
                "genko-ls-x86_64-apple-darwin.tar.gz",
                DownloadedFileType::GzipTar,
                "genko-ls",
            ),
            (
                Os::Linux,
                Architecture::Aarch64,
                "genko-ls-aarch64-unknown-linux-musl.tar.gz",
                DownloadedFileType::GzipTar,
                "genko-ls",
            ),
            (
                Os::Linux,
                Architecture::X8664,
                "genko-ls-x86_64-unknown-linux-musl.tar.gz",
                DownloadedFileType::GzipTar,
                "genko-ls",
            ),
            (
                Os::Windows,
                Architecture::Aarch64,
                "genko-ls-aarch64-pc-windows-msvc.zip",
                DownloadedFileType::Zip,
                "genko-ls.exe",
            ),
            (
                Os::Windows,
                Architecture::X8664,
                "genko-ls-x86_64-pc-windows-msvc.zip",
                DownloadedFileType::Zip,
                "genko-ls.exe",
            ),
        ];

        for (os, architecture, name, file_type, binary_name) in cases {
            assert_eq!(
                asset_spec(os, architecture),
                Ok(AssetSpec {
                    name,
                    file_type,
                    binary_name,
                })
            );
        }
    }

    #[test]
    fn rejects_unsupported_x86_assets() {
        for os in [Os::Mac, Os::Linux, Os::Windows] {
            assert!(asset_spec(os, Architecture::X86).is_err());
        }
    }

    #[test]
    fn parses_and_orders_semver_versions() {
        let ordered = [
            "v0.1.2",
            "0.1.9",
            "v0.1.10-alpha",
            "v0.1.10-alpha.1",
            "v0.1.10-alpha.beta",
            "v0.1.10-beta",
            "v0.1.10-beta.2",
            "v0.1.10-beta.11",
            "v0.1.10-rc.1",
            "v0.1.10",
            "0.2.0",
            "v1.0.0",
        ];

        let parsed: Vec<SemVer> = ordered
            .iter()
            .map(|version| parse_semver(version).expect("expected valid semver"))
            .collect();

        for window in parsed.windows(2) {
            assert!(
                window[0] < window[1],
                "expected {:?} < {:?}",
                window[0],
                window[1]
            );
        }

        assert_eq!(parse_semver("v0.1.2"), parse_semver("0.1.2"));
        assert_eq!(
            parse_semver("v0.1.2+build.1"),
            parse_semver("v0.1.2+meta-foo")
        );

        for invalid in [
            "",
            "v",
            "0.1",
            "0.1.2.3",
            "+1.0.0",
            "v0.1.2-",
            "v0.1.2+",
            "v0.1.2-rc..1",
            "custom",
        ] {
            assert_eq!(parse_semver(invalid), None, "expected {invalid:?} to fail");
        }
    }

    #[test]
    fn finds_highest_semver_cached_binary() {
        let dir = TempDir::new("fallback-semver");
        dir.create_file("genko-ls-v0.1.2/genko-ls");
        dir.create_file("genko-ls-0.1.9/genko-ls");
        dir.create_file("genko-ls-v0.1.10-rc.1/genko-ls");
        dir.create_file("genko-ls-v0.1.10/genko-ls");

        assert_eq!(
            find_cached_binary(dir.path(), "genko-ls"),
            Some("genko-ls-v0.1.10/genko-ls".to_owned())
        );
    }

    #[test]
    fn ignores_invalid_entries_when_scanning_cached_binaries() {
        let dir = TempDir::new("fallback-invalid");
        dir.create_dir("genko-ls-v9.9.9");
        dir.create_dir("genko-ls-v8.8.8/genko-ls");
        dir.create_file("genko-ls-v7.7.7");
        dir.create_file("other-ls-v6.6.6/genko-ls");
        dir.create_file("genko-ls-/genko-ls");
        dir.create_file("genko-ls-v0.1.10/genko-ls.exe");
        dir.create_file("genko-ls-v0.1.2/genko-ls");

        assert_eq!(
            find_cached_binary(dir.path(), "genko-ls"),
            Some("genko-ls-v0.1.2/genko-ls".to_owned())
        );
        assert_eq!(
            find_cached_binary(dir.path(), "genko-ls.exe"),
            Some("genko-ls-v0.1.10/genko-ls.exe".to_owned())
        );
    }

    #[test]
    fn breaks_equal_semver_ties_deterministically() {
        let dir = TempDir::new("fallback-tie");
        dir.create_file("genko-ls-0.1.2/genko-ls");
        dir.create_file("genko-ls-v0.1.2/genko-ls");

        assert_eq!(
            find_cached_binary(dir.path(), "genko-ls"),
            Some("genko-ls-v0.1.2/genko-ls".to_owned())
        );
    }

    #[test]
    fn prefers_semver_over_non_semver_cached_dirs() {
        let dir = TempDir::new("fallback-non-semver");
        dir.create_file("genko-ls-custom/genko-ls");

        assert_eq!(
            find_cached_binary(dir.path(), "genko-ls"),
            Some("genko-ls-custom/genko-ls".to_owned())
        );

        dir.create_file("genko-ls-v0.1.0/genko-ls");

        assert_eq!(
            find_cached_binary(dir.path(), "genko-ls"),
            Some("genko-ls-v0.1.0/genko-ls".to_owned())
        );
    }

    #[test]
    fn returns_none_when_no_cached_binary_exists() {
        let dir = TempDir::new("fallback-empty");

        assert_eq!(find_cached_binary(dir.path(), "genko-ls"), None);
        assert_eq!(
            find_cached_binary(&dir.path().join("missing"), "genko-ls"),
            None
        );
    }

    #[test]
    fn prunes_older_version_dirs_while_keeping_active_and_unrelated_entries() {
        let dir = TempDir::new("prune");
        dir.create_file("genko-ls-v0.1.1/genko-ls");
        dir.create_file("genko-ls-v0.1.2/genko-ls");
        dir.create_file("genko-ls-v0.1.3/genko-ls");
        dir.create_file("unrelated-dir/file.txt");
        dir.create_file("genko-ls/file.txt");
        dir.create_file("genko-ls-/file.txt");
        dir.create_file("genko-ls-v0.1.0.tar.gz");
        dir.create_file("notes.txt");

        prune_stale_version_dirs(dir.path(), "genko-ls-v0.1.3");

        assert!(!dir.path().join("genko-ls-v0.1.1").exists());
        assert!(!dir.path().join("genko-ls-v0.1.2").exists());
        assert!(dir.path().join("genko-ls-v0.1.3/genko-ls").is_file());
        assert!(dir.path().join("unrelated-dir/file.txt").is_file());
        assert!(dir.path().join("genko-ls/file.txt").is_file());
        assert!(dir.path().join("genko-ls-/file.txt").is_file());
        assert!(dir.path().join("genko-ls-v0.1.0.tar.gz").is_file());
        assert!(dir.path().join("notes.txt").is_file());
    }

    #[test]
    fn prune_ignores_missing_base_dir() {
        let dir = TempDir::new("prune-missing");
        prune_stale_version_dirs(&dir.path().join("does-not-exist"), "genko-ls-v0.1.3");
    }
}
