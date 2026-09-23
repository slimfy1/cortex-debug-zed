# Cortex-Debug for Zed

Debug ARM Cortex-M microcontrollers in the [Zed](https://zed.dev) editor with
the debug adapter from [Cortex-Debug](https://github.com/Marus/cortex-debug).
It supports J-Link, OpenOCD, pyOCD, ST-Link, Black Magic Probe and QEMU.

[Русская версия](README.ru.md)

> **Status: experimental (v0.1.0).** An end-to-end debug session has been
> tested on Linux against QEMU (Cortex-M3). It has not yet been tested on
> real hardware or inside a running Zed instance on every platform. The
> primary target is **J-Link on Windows**. Bug reports are very welcome.

This is an unofficial community port. It is not affiliated with the
Cortex-Debug authors or with Zed Industries.

---

## Features

- **Launch** (flash + run) and **attach** (connect to a running target without flashing)
- Breakpoints: line, conditional, hit-count, logpoints and function breakpoints; data watchpoints
- Stepping: over, into and out, instruction-level stepping, pause
- Call stack; **Local / Global / Static / Registers** scopes; watch expressions; hover evaluation
- **Debug Console works as a GDB console**: `monitor reset`, `monitor halt`,
  `x/8wx 0x20000000`, `info registers`, `p/x *(uint32_t*)0x40020000`…
- gdb-server and semihosting output in the Debug Console (or in a log file)
- RTOS thread awareness, where the gdb-server supports it (J-Link: FreeRTOS, embOS, Zephyr, …)
- RTT: the TCP port for each channel is printed at session start, so you can
  read it with PuTTY (Raw), `telnet` or `nc`
- Auto-detects `JLinkGDBServerCL.exe` in `C:\Program Files\SEGGER\JLink*`
- Accepts almost all of your existing `launch.json` attributes from Cortex-Debug (the exceptions are listed under **Not supported yet**)
- No downloads: the adapter is embedded in the extension and runs on the Node.js bundled with Zed

### Not supported yet

Zed does not yet let extensions add custom panels, so the following
Cortex-Debug UI features are missing:

- Peripheral (SVD) viewer, RTOS viewer, Memory viewer
- SWO/RTT graphs and decoders UI, Live Watch panel
- Multi-core / chained sessions (`chainedConfigurations`)

Peripheral registers can still be read from the Debug Console (see the examples above).

## Requirements

| Tool | Notes |
|---|---|
| [Zed](https://zed.dev) | a version with the built-in debugger |
| [Rust via rustup](https://rustup.rs) | Zed needs it to compile a dev extension |
| [Arm GNU Toolchain](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads) | provides `arm-none-eabi-gdb` (GDB ≥ 9), `objdump`, `nm` |
| A gdb-server | e.g. [SEGGER J-Link Software](https://www.segger.com/downloads/jlink/), OpenOCD, pyOCD |

## Installation

The extension is not yet in the Zed extension registry, so install it as a
dev extension:

1. Clone this repository:
   ```sh
   git clone https://github.com/<your-username>/zed-cortex-debug.git
   ```
2. In Zed, open the command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`), run
   **`zed: install dev extension`** and select the cloned folder.
3. Create `.zed/debug.json` in your firmware project (see below).
4. Start debugging with `F4` (**`debugger: start`**) and pick a configuration.

## Configuration

Minimal `.zed/debug.json` for J-Link:

```jsonc
[
  {
    "label": "Debug (J-Link)",
    "adapter": "cortex-debug",
    "request": "launch",
    "servertype": "jlink",
    "device": "STM32F407VG",
    "interface": "swd",
    "executable": "build/firmware.elf",
    "runToEntryPoint": "main"
  }
]
```

A fuller example with attach, RTT, a build step and explicit tool paths is in
[`examples/debug.json`](examples/debug.json).

- **Paths** are relative to `cwd`, which defaults to the project root. Use
  `$ZED_WORKTREE_ROOT` wherever you used `${workspaceFolder}` in VS Code.
- **All Cortex-Debug launch attributes** work as documented in
  [debug_attributes.md](https://github.com/Marus/cortex-debug/blob/master/debug_attributes.md):
  `serverpath`, `armToolchainPath`, `gdbPath`, `svdFile`, `rtos`, `rttConfig`,
  `preLaunchCommands`, `overrideLaunchCommands`, etc.
- **VS Code settings become config fields.** Settings such as
  `cortex-debug.armToolchainPath` or `cortex-debug.JLinkGDBServerPath` go
  directly into the configuration (`armToolchainPath`, `serverpath`).
- **OS-specific sections** `"windows": {…}`, `"linux": {…}` and `"osx": {…}` work as in VS Code.

Zed-specific options:

| Field | Default | Description |
|---|---|---|
| `gdbServerOutput` | `"console"` | Where gdb-server output goes: `"console"`, `"none"` or a log file path |
| `nodePath` | Zed's Node.js | Path to a different `node` executable to run the adapter |

### Other gdb-servers

```jsonc
{ "label": "OpenOCD", "adapter": "cortex-debug", "request": "launch",
  "servertype": "openocd", "executable": "build/firmware.elf",
  "configFiles": ["interface/stlink.cfg", "target/stm32f4x.cfg"] }
```

```jsonc
{ "label": "QEMU", "adapter": "cortex-debug", "request": "launch",
  "servertype": "qemu", "executable": "build/firmware.elf",
  "cpu": "cortex-m3", "machine": "lm3s6965evb" }
```

## Troubleshooting

- **The adapter fails to start or you see no output.** Open the debug adapter
  logs from the command palette (search for "debug adapter logs").
- **You need to see the full GDB exchange.** Add `"showDevDebugOutput": "raw"`
  to the configuration and all GDB/MI traffic appears in the Debug Console.
- **You need the process start/stop log.** It is written to
  `%TEMP%\cortex-debug-server.log` on Windows and `/tmp/cortex-debug-server.log`
  on Linux and macOS.
- **`GDB executable "arm-none-eabi-gdb" was not found`.** Add the toolchain's
  `bin` folder to `PATH`, or set `armToolchainPath` or `gdbPath`.
- **`spawn JLinkGDBServerCL.exe ENOENT`.** Set `serverpath` to the full path of
  `JLinkGDBServerCL.exe`.

When you open an issue, please include your `debug.json`, the Debug Console
output with `"showDevDebugOutput": "raw"`, your OS, and your probe and gdb
versions.

## How it works

Cortex-Debug for VS Code has two parts:

| Part | Role | In this port |
|---|---|---|
| **Debug adapter** (`src/gdb.ts`, Node.js) | DAP ⇄ GDB/MI, starts the gdb-server, flashing, breakpoints, stack, variables, registers, disassembly | reused **unchanged** |
| **Frontend** (`src/frontend/*`, VS Code API) | resolves `launch.json`, hosts the gdb-server console, SVD/RTOS/Memory views, graphs, Live Watch | replaced by a headless shim, `src/zed/adapter.ts` |

The shim fills in defaults and validates each `servertype`. It also locates
J-Link, makes paths absolute, and forwards gdb-server output to Zed as DAP
`output` events. It drops VS Code-specific custom events and turns useful ones,
such as error pop-ups and RTT ports, into console messages.

The Rust part ([`src/lib.rs`](src/lib.rs)) embeds the bundled adapter and
unpacks it into the extension's work directory on first use. It then starts
the adapter with Zed's Node.js. It also implements `dap_request_kind` and
`dap_config_to_scenario` for Zed's debugger.

### Repository layout

```
adapter/                  bundled adapter (embedded into the wasm via include_str!)
debug_adapter_schemas/    JSON schema for debug.json, generated from Cortex-Debug's package.json
examples/                 sample .zed/debug.json
patches/                  patch against Cortex-Debug + BASE_COMMIT it applies to
scripts/                  build-adapter.sh / build-adapter.ps1 — rebuild adapter/ from source
src/lib.rs                the Zed extension (Rust → wasm32-wasip2)
```

### Changes to Cortex-Debug (`patches/`)

1. Adds `src/zed/adapter.ts`, the headless replacement for the VS Code frontend.
2. Moves the session start-up from `gdb.ts` to `src/debugadapter-main.ts` so
   that `GDBDebugSession` can be imported without side effects. The VS Code
   build is unaffected.
3. Fixes a start-up hang in `mi2.ts`. Newer GDB (seen with GDB 15) can flush
   the token of an MI record (`5` of `5^done`) as a separate chunk, which was
   treated as console output. This fix also applies to upstream.

## Development

Rebuild the adapter from Cortex-Debug sources (requires git and Node.js 18+):

```sh
./scripts/build-adapter.sh          # Linux / macOS
.\scripts\build-adapter.ps1         # Windows PowerShell
```

Then click **Rebuild** on the extension in Zed's Extensions page.

While you are working on the adapter itself, you can point Zed at your
checkout instead of the embedded copy:

```jsonc
// Zed settings.json
"dap": { "cortex-debug": { "binary": "C:/src/cortex-debug" } }  // folder containing dist/zedadapter.js
```

## Roadmap

- [ ] Verify on Windows with J-Link hardware
- [ ] Publish to the Zed extension registry
- [ ] Debug locator: start debugging directly from CMake/Make tasks
- [ ] SVD peripheral registers exposed as a variables scope
- [ ] Upstream the Zed entry point and the MI fix to Cortex-Debug

## Credits and license

All the hard work is in [Cortex-Debug](https://github.com/Marus/cortex-debug)
by Marcel Ball and contributors. Cortex-Debug is in turn based on
[code-debug](https://github.com/WebFreak001/code-debug) by Jan Jurzitza.

Released under the [MIT License](LICENSE), the same license as Cortex-Debug.
The bundled adapter's original license is in `adapter/LICENSE-cortex-debug`.
