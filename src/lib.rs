use std::fs;

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

        let release = zed::latest_github_release(
            RELEASE_REPOSITORY,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let (os, architecture) = zed::current_platform();
        let spec = asset_spec(os, architecture)?;
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

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
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
    use super::*;

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
}
