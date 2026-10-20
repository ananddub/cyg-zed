use std::fs;

use zed_extension_api::{self as zed, Command, LanguageServerId, Result, Worktree};

const LANGUAGE_SERVER_ID: &str = "cyg-lsp";
const RELEASE_REPOSITORY: &str = "ananddub/cyg-zed";
const RELEASE_DLL: &str = "sdk/unity/Tools~/CYG.Lsp/bin/Release/net10.0/CYG.Lsp.dll";
const DEBUG_DLL: &str = "sdk/unity/Tools~/CYG.Lsp/bin/Debug/net10.0/CYG.Lsp.dll";

struct CygExtension {
    cached_server_path: Option<String>,
}

impl CygExtension {
    fn download_server(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
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
        )
        .map_err(|error| format!("CYG: failed to query GitHub releases: {error}"))?;

        let (os, arch) = zed::current_platform();
        let platform = match (os, arch) {
            (zed::Os::Linux, zed::Architecture::X8664) => "linux-x64",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "linux-arm64",
            (zed::Os::Mac, zed::Architecture::X8664) => "macos-x64",
            (zed::Os::Mac, zed::Architecture::Aarch64) => "macos-arm64",
            (zed::Os::Windows, zed::Architecture::X8664) => "windows-x64",
            _ => {
                return Err(
                    "CYG: no language server build is available for this platform.".to_string(),
                );
            }
        };
        let is_windows = matches!(os, zed::Os::Windows);

        let asset_name = format!("cyg-lsp-{platform}.zip");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "CYG: release {} does not contain {asset_name}.",
                    release.version
                )
            })?;

        let version_dir = format!("cyg-lsp-{}", release.version);
        let binary_name = if is_windows { "cyg-lsp.exe" } else { "cyg-lsp" };
        let binary_path = format!("{version_dir}/{binary_name}");

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|error| format!("CYG: failed to download {asset_name}: {error}"))?;
            zed::make_file_executable(&binary_path)?;

            if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
                return Err(format!(
                    "CYG: {asset_name} did not contain {binary_name}."
                ));
            }

            remove_outdated_versions(&version_dir);
        }

        self.cached_server_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for CygExtension {
    fn new() -> Self {
        Self {
            cached_server_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        if language_server_id.as_ref() != LANGUAGE_SERVER_ID {
            return Err(format!("Unknown CYG language server: {language_server_id}"));
        }

        let env = worktree.shell_env();

        if let Some(command) = worktree.which("cyg-lsp") {
            return Ok(Command {
                command,
                args: Vec::new(),
                env,
            });
        }

        if let Some(path) = &self.cached_server_path
            && fs::metadata(path).is_ok_and(|stat| stat.is_file())
        {
            return Ok(Command {
                command: path.clone(),
                args: Vec::new(),
                env,
            });
        }

        match self.download_server(language_server_id) {
            Ok(command) => Ok(Command {
                command,
                args: Vec::new(),
                env,
            }),
            Err(error) => {
                if let Some((command, args)) = dotnet_fallback(worktree) {
                    return Ok(Command {
                        command,
                        args,
                        env,
                    });
                }

                Err(error)
            }
        }
    }
}

fn dotnet_fallback(worktree: &Worktree) -> Option<(String, Vec<String>)> {
    let dotnet = worktree.which("dotnet")?;
    let root = worktree.root_path();

    for relative in [RELEASE_DLL, DEBUG_DLL] {
        let dll = format!("{root}/{relative}");
        let runtime_config = dll.replace("CYG.Lsp.dll", "CYG.Lsp.runtimeconfig.json");
        if worktree.read_text_file(&runtime_config).is_ok() {
            return Some((dotnet, vec![dll]));
        }
    }

    None
}

fn remove_outdated_versions(current_dir: &str) {
    let Ok(entries) = fs::read_dir(".") else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.starts_with("cyg-lsp-") && name != current_dir {
            fs::remove_dir_all(path).ok();
        }
    }
}

zed::register_extension!(CygExtension);
