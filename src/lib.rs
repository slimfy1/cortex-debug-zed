//! Zed extension that exposes the Cortex-Debug debug adapter.
//!
//! The adapter itself is the Node.js program from the Cortex-Debug project
//! (built from `src/zed/adapter.ts`, see `patches/` and `scripts/`). Following
//! the Zed extension guidelines it is not bundled: CI attaches it to each GitHub
//! release of this repository, and the extension downloads it on first use.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use zed_extension_api::{
    self as zed,
    serde_json::{self, json, Map, Value},
    DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario, DebugTaskDefinition,
    DownloadedFileType, GithubReleaseOptions, StartDebuggingRequestArguments,
    StartDebuggingRequestArgumentsRequest, Worktree,
};

/// GitHub repository whose releases carry the adapter (`owner/name`).
const GITHUB_REPO: &str = "Andry/zed-cortex-debug";
/// Release asset built by `.github/workflows/release.yml`.
const ADAPTER_ASSET: &str = "cortex-debug-adapter.zip";

const ADAPTER_NAME: &str = "cortex-debug";
const INSTALL_DIR_PREFIX: &str = "cortex-debug-adapter-";
const ADAPTER_ENTRY: &str = "dist/zedadapter.js";

struct CortexDebugExtension {
    /// Absolute path of the installed adapter package (checked once per Zed session).
    installed: Option<PathBuf>,
}

fn abs_path(rel: &Path) -> Result<PathBuf, String> {
    Ok(env::current_dir()
        .map_err(|e| format!("cortex-debug: cannot determine work directory: {e}"))?
        .join(rel))
}

/// Newest adapter already unpacked in the work directory (used when offline).
fn newest_installed() -> Option<String> {
    fs::read_dir(".")
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (name.starts_with(INSTALL_DIR_PREFIX) && Path::new(&name).join(ADAPTER_ENTRY).exists())
                .then_some(name)
        })
        .max_by(|a, b| version_key(a).cmp(&version_key(b)))
}

fn version_key(dir: &str) -> Vec<u64> {
    dir.trim_start_matches(INSTALL_DIR_PREFIX)
        .trim_start_matches('v')
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|p| p.parse().ok())
        .collect()
}

impl CortexDebugExtension {
    /// Makes sure the adapter from the latest GitHub release is unpacked in the
    /// extension work directory and returns its absolute path.
    fn install_adapter(&mut self) -> Result<PathBuf, String> {
        if let Some(path) = &self.installed {
            if path.join(ADAPTER_ENTRY).exists() {
                return Ok(path.clone());
            }
        }

        let release = zed::latest_github_release(
            GITHUB_REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        );

        let dir_name = match release {
            Ok(release) => {
                let dir_name = format!("{INSTALL_DIR_PREFIX}{}", release.version);
                if !Path::new(&dir_name).join(ADAPTER_ENTRY).exists() {
                    let asset = release
                        .assets
                        .iter()
                        .find(|a| a.name == ADAPTER_ASSET)
                        .ok_or_else(|| {
                            format!(
                                "cortex-debug: release {} of {GITHUB_REPO} has no {ADAPTER_ASSET}",
                                release.version
                            )
                        })?;
                    fs::remove_dir_all(&dir_name).ok();
                    zed::download_file(&asset.download_url, &dir_name, DownloadedFileType::Zip)
                        .map_err(|e| format!("cortex-debug: failed to download the adapter: {e}"))?;
                    if !Path::new(&dir_name).join(ADAPTER_ENTRY).exists() {
                        return Err(format!(
                            "cortex-debug: downloaded {ADAPTER_ASSET} does not contain {ADAPTER_ENTRY}"
                        ));
                    }
                    // Remove adapters from older releases.
                    if let Ok(entries) = fs::read_dir(".") {
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().into_owned();
                            if name.starts_with(INSTALL_DIR_PREFIX) && name != dir_name {
                                fs::remove_dir_all(entry.path()).ok();
                            }
                        }
                    }
                }
                dir_name
            }
            // Offline or GitHub rate limit: fall back to whatever we installed before.
            Err(e) => newest_installed().ok_or_else(|| {
                format!("cortex-debug: cannot fetch the adapter from github.com/{GITHUB_REPO}: {e}")
            })?,
        };

        let abs = abs_path(Path::new(&dir_name))?;
        self.installed = Some(abs.clone());
        Ok(abs)
    }

    /// Path of the adapter's JS entry point: a user override from Zed settings
    /// (`"dap": { "cortex-debug": { "binary": "..." } }`), or the downloaded copy.
    fn adapter_entry(&mut self, user_path: Option<String>) -> Result<String, String> {
        if let Some(user_path) = user_path {
            let p = PathBuf::from(&user_path);
            let is_js = p
                .extension()
                .map(|e| e.eq_ignore_ascii_case("js"))
                .unwrap_or(false);
            let entry = if is_js { p } else { p.join(ADAPTER_ENTRY) };
            return Ok(entry.to_string_lossy().into_owned());
        }
        let root = self.install_adapter()?;
        Ok(root.join(ADAPTER_ENTRY).to_string_lossy().into_owned())
    }

    fn request_kind(config: &Value) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        match config.get("request").and_then(Value::as_str) {
            Some("launch") | None => Ok(StartDebuggingRequestArgumentsRequest::Launch),
            Some("attach") => Ok(StartDebuggingRequestArgumentsRequest::Attach),
            Some(other) => Err(format!(
                "cortex-debug: invalid \"request\": \"{other}\" (expected \"launch\" or \"attach\")"
            )),
        }
    }
}

fn is_absolute_any_os(p: &str) -> bool {
    // The extension runs as WASM, so std::path follows Unix rules; accept Windows forms too.
    let b = p.as_bytes();
    p.starts_with('/')
        || p.starts_with('\\')
        || (b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic())
}

impl zed::Extension for CortexDebugExtension {
    fn new() -> Self {
        Self { installed: None }
    }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        if adapter_name != ADAPTER_NAME {
            return Err(format!("cortex-debug: unknown adapter {adapter_name}"));
        }

        let mut configuration: Value = serde_json::from_str(&config.config)
            .map_err(|e| format!("cortex-debug: invalid JSON configuration: {e}"))?;
        let obj = configuration
            .as_object_mut()
            .ok_or("cortex-debug: configuration must be a JSON object")?;

        // Relative paths in the config (executable, svdFile, ...) are resolved against cwd,
        // which defaults to the project root like ${workspaceFolder} in VS Code.
        let root = worktree.root_path();
        let cwd = match obj.get("cwd").and_then(Value::as_str) {
            Some(c) if is_absolute_any_os(c) => c.to_string(),
            Some(c) if !c.is_empty() && c != "." => {
                let sep = if root.contains('\\') { "\\" } else { "/" };
                format!("{}{sep}{}", root.trim_end_matches(['/', '\\']), c)
            }
            _ => root.clone(),
        };
        obj.insert("cwd".into(), Value::String(cwd));
        if !obj.contains_key("request") {
            obj.insert("request".into(), Value::String("launch".into()));
        }
        if !obj.contains_key("name") {
            obj.insert("name".into(), Value::String(config.label.clone()));
        }

        let request = Self::request_kind(&configuration)?;
        let entry = self.adapter_entry(user_provided_debug_adapter_path)?;

        // Prefer a user-chosen Node.js, fall back to the one managed by Zed.
        let node = configuration
            .get("nodePath")
            .and_then(Value::as_str)
            .map(str::to_string)
            .map(Ok)
            .unwrap_or_else(zed::node_binary_path)?;

        Ok(DebugAdapterBinary {
            command: Some(node),
            arguments: vec![entry],
            // The login shell environment makes toolchains on PATH (arm-none-eabi-gdb, J-Link) visible.
            envs: worktree.shell_env(),
            cwd: Some(root),
            connection: None,
            request_args: StartDebuggingRequestArguments {
                configuration: configuration.to_string(),
                request,
            },
        })
    }

    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        config: Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        Self::request_kind(&config)
    }

    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        match &config.request {
            DebugRequest::Launch(launch) => {
                let mut obj = Map::new();
                obj.insert("request".into(), json!("launch"));
                obj.insert("servertype".into(), json!("jlink"));
                obj.insert("executable".into(), json!(launch.program));
                if let Some(cwd) = &launch.cwd {
                    obj.insert("cwd".into(), json!(cwd));
                }
                if config.stop_on_entry.unwrap_or(false) {
                    obj.insert("breakAfterReset".into(), json!(true));
                } else {
                    obj.insert("runToEntryPoint".into(), json!("main"));
                }
                Ok(DebugScenario {
                    label: config.label,
                    adapter: config.adapter,
                    build: None,
                    config: Value::Object(obj).to_string(),
                    tcp_connection: None,
                })
            }
            DebugRequest::Attach(_) => Err(
                "cortex-debug: attaching needs a probe configuration; add an \"attach\" entry to .zed/debug.json"
                    .into(),
            ),
        }
    }
}

zed::register_extension!(CortexDebugExtension);
