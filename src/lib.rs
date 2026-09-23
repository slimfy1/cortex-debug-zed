//! Zed extension that exposes the Cortex-Debug debug adapter.
//!
//! The adapter itself is the Node.js program from the Cortex-Debug project
//! (built from `src/zed/adapter.ts`, see `patches/` and `scripts/`). It is
//! embedded into this extension and unpacked into the extension's work
//! directory on first use, so no download is required.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use zed_extension_api::{
    self as zed,
    serde_json::{self, json, Map, Value},
    DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario, DebugTaskDefinition,
    StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest, Worktree,
};

const ADAPTER_NAME: &str = "cortex-debug";
const INSTALL_DIR_PREFIX: &str = "cortex-debug-adapter-";

/// Files of the adapter package, relative to the package root.
const ADAPTER_FILES: &[(&str, &str)] = &[
    (
        "dist/zedadapter.js",
        include_str!("../adapter/dist/zedadapter.js"),
    ),
    (
        "support/gdbsupport.init",
        include_str!("../adapter/support/gdbsupport.init"),
    ),
    (
        "support/gdb-swo.init",
        include_str!("../adapter/support/gdb-swo.init"),
    ),
    (
        "LICENSE-cortex-debug",
        include_str!("../adapter/LICENSE-cortex-debug"),
    ),
];
const ADAPTER_ENTRY: &str = "dist/zedadapter.js";

struct CortexDebugExtension {
    /// Absolute path of the unpacked adapter package (once installed).
    installed: Option<PathBuf>,
}

/// 64-bit FNV-1a, used to give each embedded adapter build its own directory.
fn fnv1a(data: &[&str]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in data {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

impl CortexDebugExtension {
    /// Unpacks the embedded adapter into the extension work directory (once per build).
    fn install_embedded_adapter(&mut self) -> Result<PathBuf, String> {
        if let Some(path) = &self.installed {
            return Ok(path.clone());
        }

        let contents: Vec<&str> = ADAPTER_FILES.iter().map(|(_, c)| *c).collect();
        let dir_name = format!("{INSTALL_DIR_PREFIX}{:016x}", fnv1a(&contents));
        let dir = Path::new(&dir_name);

        if !dir.join(ADAPTER_ENTRY).exists() {
            // Remove adapters unpacked by older versions of this extension.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with(INSTALL_DIR_PREFIX) && name != dir_name {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
            for (rel, content) in ADAPTER_FILES {
                let target = dir.join(rel);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("cortex-debug: cannot create {parent:?}: {e}"))?;
                }
                fs::write(&target, content)
                    .map_err(|e| format!("cortex-debug: cannot write {target:?}: {e}"))?;
            }
        }

        let abs = env::current_dir()
            .map_err(|e| format!("cortex-debug: cannot determine work directory: {e}"))?
            .join(dir);
        self.installed = Some(abs.clone());
        Ok(abs)
    }

    /// Path of the adapter's JS entry point: a user override from Zed settings
    /// (`"dap": { "cortex-debug": { "binary": "..." } }`), or the embedded copy.
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
        let root = self.install_embedded_adapter()?;
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
