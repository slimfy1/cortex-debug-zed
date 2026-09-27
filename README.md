# Cortex-M Debugger for Zed

Debug ARM Cortex-M microcontrollers in the [Zed](https://zed.dev) editor with
the debug adapter from [Cortex-Debug](https://github.com/Marus/cortex-debug).
It supports J-Link, OpenOCD, pyOCD, ST-Link, Black Magic Probe and QEMU.

> **Status: experimental (v0.4.0).** End-to-end debug sessions have been
> tested on Linux against QEMU (Cortex-M3), including the SVD peripheral
> view, the FreeRTOS task view and the memory view. It has not yet been tested on
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
- **Peripheral registers from an SVD file**: peripherals, registers and bit fields with enum names in the Variables panel; values can be edited ([details](#peripheral-registers-svd))
- **FreeRTOS task view**: all tasks with state, priority, stack high-water mark and CPU share ([details](#rtos-task-view-freertos))
- **Memory view**: Zed's built-in hex viewer and editor, with "Go To Memory" on variables, registers and tasks ([details](#memory-view))
- **Debug Console works as a GDB console**: `monitor reset`, `monitor halt`,
  `x/8wx 0x20000000`, `info registers`, `p/x *(uint32_t*)0x40020000`…
- gdb-server and semihosting output in the Debug Console (or in a log file)
- RTOS threads in the call stack, where the gdb-server supports it (`"rtos"`; J-Link: FreeRTOS, embOS, Zephyr, …)
- RTT: the TCP port for each channel is printed at session start, so you can
  read it with PuTTY (Raw), `telnet` or `nc`
- Auto-detects `JLinkGDBServerCL.exe` in `C:\Program Files\SEGGER\JLink*`
- Accepts almost all of your existing `launch.json` attributes from Cortex-Debug (the exceptions are listed under **Not supported yet**)
- Nothing extra to install for the adapter: it is downloaded from this repository's GitHub releases on first use and runs on the Node.js bundled with Zed

### Not supported yet

Zed does not yet let extensions add custom panels, so the following
Cortex-Debug UI features are missing:

- SWO/RTT graphs and decoders UI, Live Watch panel
- Multi-core / chained sessions (`chainedConfigurations`)
- The RTOS task view for kernels other than FreeRTOS (Zephyr, embOS,
  ThreadX, µC/OS). Per-thread call stacks still work through the
  gdb-server's `"rtos"` option.

## Requirements

| Tool | Notes |
|---|---|
| [Zed](https://zed.dev) | a version with the built-in debugger |
| [Rust via rustup](https://rustup.rs) | Zed needs it to compile a dev extension |
| [Arm GNU Toolchain](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads) | provides `arm-none-eabi-gdb` (GDB ≥ 9), `objdump`, `nm` |
| A gdb-server | e.g. [SEGGER J-Link Software](https://www.segger.com/downloads/jlink/), OpenOCD, pyOCD |

## Installation

1. In Zed, open **Extensions** (`Ctrl+Shift+X` / `Cmd+Shift+X`), search for
   **Cortex-M Debugger** and click **Install**.
2. Create `.zed/debug.json` in your firmware project (see below).
3. Start debugging with `F4` (**`debugger: start`**) and pick a configuration.

On the first debug session the extension downloads the adapter
(`cortex-debug-adapter.zip`, about 100 KB) from this repository's latest
GitHub release.

To try an unreleased version, clone this repository and run
**`zed: install dev extension`** on the folder (see [Development](#development)).

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
    "svdFile": "STM32F407.svd",
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
| `rtosView` | `true` | Show the FreeRTOS task view when FreeRTOS symbols are found |
| `svdAddrGapThreshold` | `16` | Registers closer than this many bytes are read in one transfer; `0` reads each register separately |

### Peripheral registers (SVD)

Set `svdFile` to your device's CMSIS-SVD file. You can get SVD files from the
vendor's device pack, from the STM32CubeIDE installation, or from
[cmsis-svd-data](https://github.com/cmsis-svd/cmsis-svd-data).
A **Peripherals** scope then appears in the Variables panel:

```
Peripherals (STM32F407)
├─ GPIOA          0x40020000  General-purpose I/Os
│  ├─ MODER       0xA8000000        rw u32 @ 0x40020000
│  │  ├─ MODER15  Alternate = 0x2 (2)   rw [31:30]
│  │  └─ …
│  └─ ODR         0x00000020
└─ USART1 …
```

- **Values are read only when you expand a node.** Expanding a peripheral
  reads its registers in a few grouped memory transfers. Values are cached
  until the target runs, steps or you type a command in the Debug Console.
- **You can edit registers and fields.** Edit a value in the Variables panel.
  Registers accept `0x…`, decimal, `0b…` or `#…`. Fields also accept an enum
  name such as `Alternate`. A field write is a read-modify-write of its
  register, and the value is read back from the hardware afterwards.
- **Some registers are never read automatically.** Registers with a
  `readAction` in the SVD (read has side effects, e.g. clear-on-read) and
  write-only registers are skipped. Keep in mind that expanding a peripheral
  such as a UART still reads all of its registers. This includes data
  registers whose read side effects the SVD does not declare, just like any
  memory viewer.
- **Registers can go to Watch.** Each register has an evaluate name such as
  `*(volatile unsigned int *)0x40020000`, so you can add it to Watch.
- **Supported SVD features:** `derivedFrom`, `dim` arrays and clusters,
  property inheritance, all three bit-range notations and enumerated values.

### RTOS task view (FreeRTOS)

If the program contains FreeRTOS, an **RTOS (FreeRTOS)** scope appears in the
Variables panel automatically. No configuration is needed:

```
RTOS (FreeRTOS)
├─ Blinker   Blocked · prio 1 · stack free 412 B
├─ Sensor    Ready · prio 2 · stack free 580 B
├─ Waiter    Running · prio 3 · stack free 352 B
│  ├─ Stack start (pxStack)        0x20000858
│  ├─ Stack size                   508 B
│  ├─ Stack free (high-water mark) 352 B
│  └─ TCB                          0x20000a60
├─ Sleeper   Suspended · prio 1 · stack free 292 B
└─ IDLE      Ready · prio 0 · stack free 436 B
```

- **Task data comes from the kernel's own lists.** The view reads the ready,
  delayed, pending, suspended and deleted lists through ordinary GDB
  expressions, so it works with any gdb-server and with GDB builds that
  have no Python.
- **Stack free** is the high-water mark: the bytes still holding the `0xA5`
  fill pattern. **Stack size** and **Stack used now** need
  `configRECORD_STACK_HIGH_ADDRESS 1`. **CPU %** needs
  `configGENERATE_RUN_TIME_STATS 1`.
- **Addresses open in the Memory View.** Right-click a task, its TCB or its
  stack address and choose **Go To Memory**.
- **Per-task call stacks** are a separate feature. For those, also set
  `"rtos": "FreeRTOS"` (J-Link, OpenOCD or pyOCD). The gdb-server then
  reports each task as a thread in Zed's Frames list.
- The kernel must be built with debug info (it usually is). Set
  `"rtosView": false` to turn the view off.

### Memory view

Zed has a built-in **Memory View** pane in the debug panel. If it is hidden,
add it from the pane's `+` menu. This adapter makes it work well on
microcontrollers:

- **Jump to an address or an expression.** Type an address (`0x20000000`) or
  an expression (`&rxBuffer`, `huart1.pRxBuffPtr`, `pxCurrentTCB`) in the
  address bar. Pointers jump to the memory they point to, other values to
  their own address.
- **Go To Memory** is available on local and global variables, watch
  expressions, SVD registers and RTOS tasks.
- **Reserved address space shows as unreadable** instead of blanking the
  whole 4 KiB page. For example, the end of flash and the gap before the
  next region are displayed correctly.
- **Memory can be edited in place.** Edits are written to the target.

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
`src/zed/svd.ts` and `src/zed/peripherals.ts` parse the SVD file and serve
it through the standard DAP `scopes`, `variables` and `setVariable`
requests. The peripheral viewer therefore needs no custom UI in Zed.
`src/zed/rtos.ts` does the same for FreeRTOS tasks. `src/zed/memory.ts`
implements partial `readMemory` responses (`unreadableBytes`), resolves the
Memory View's address-bar expressions and attaches a `memoryReference` to
variables and watch results.

The Rust part ([`src/lib.rs`](src/lib.rs)) downloads the adapter from the
latest GitHub release of this repository into the extension's work directory.
It then starts the adapter with Zed's Node.js. If GitHub is unreachable, it
uses the last downloaded version. It also implements `dap_request_kind` and
`dap_config_to_scenario` for Zed's debugger.

### Repository layout

```
.github/workflows/        release.yml — builds the adapter and attaches it to each GitHub release
debug_adapter_schemas/    JSON schema for debug.json, generated from Cortex-Debug's package.json
examples/                 sample .zed/debug.json
patches/                  patch against Cortex-Debug + BASE_COMMIT it applies to
scripts/                  build-adapter.sh / .ps1 — build the adapter into adapter/ (not committed)
                          set-owner.sh / .ps1 — fill in your GitHub user name, name and email (re-runnable)
src/lib.rs                the Zed extension (Rust → wasm32-wasip2)
```

### Changes to Cortex-Debug (`patches/`)

1. Adds `src/zed/adapter.ts`, the headless replacement for the VS Code frontend,
   with its Zed-only features: `svd.ts` + `peripherals.ts` (SVD peripheral
   scope), `rtos.ts` (FreeRTOS task view) and `memory.ts` (Memory View
   support). Adds the `fast-xml-parser` dependency.
2. Moves the session start-up from `gdb.ts` to `src/debugadapter-main.ts` so
   that `GDBDebugSession` can be imported without side effects. The VS Code
   build is unaffected.
3. Fixes a start-up hang in `mi2.ts`. Newer GDB (seen with GDB 15) can flush
   the token of an MI record (`5` of `5^done`) as a separate chunk, which was
   treated as console output. This fix also applies to upstream.

## Development

Build the adapter from Cortex-Debug sources (requires git and Node.js 18+).
The result goes to `adapter/`:

```sh
./scripts/build-adapter.sh          # Linux / macOS
.\scripts\build-adapter.ps1         # Windows PowerShell
```

Install the folder as a dev extension (**`zed: install dev extension`**; Zed
needs [Rust via rustup](https://rustup.rs) for that). Then point it at your
local adapter instead of the released one:

```jsonc
// Zed settings.json
"dap": { "cortex-debug": { "binary": "C:/src/zed-cortex-debug/adapter" } }  // folder containing dist/zedadapter.js
```

After changing `src/lib.rs`, click **Rebuild** on the extension in Zed's
Extensions page. After changing the adapter, just start a new debug session.

### Releasing

1. Bump `version` in `extension.toml` and `Cargo.toml`, then commit.
2. Tag and push:
   ```sh
   git tag v0.4.0
   git push origin main v0.4.0
   ```
   The [release workflow](.github/workflows/release.yml) builds the adapter
   and attaches `cortex-debug-adapter.zip` to the GitHub release.
3. Update the version in the
   [zed-industries/extensions](https://github.com/zed-industries/extensions)
   registry (new submodule commit + `version` in `extensions.toml`) with a PR.

## Roadmap

- [ ] Verify on Windows with J-Link hardware
- [ ] Publish to the Zed extension registry
- [ ] Debug locator: start debugging directly from CMake/Make tasks
- [x] SVD peripheral registers exposed as a variables scope
- [x] FreeRTOS task view; Memory View support
- [ ] RTOS task view for Zephyr, embOS, ThreadX, µC/OS
- [ ] Upstream the Zed entry point and the MI fix to Cortex-Debug

## Credits and license

All the hard work is in [Cortex-Debug](https://github.com/Marus/cortex-debug)
by Marcel Ball and contributors. Cortex-Debug is in turn based on
[code-debug](https://github.com/WebFreak001/code-debug) by Jan Jurzitza.

Released under the [MIT License](LICENSE), the same license as Cortex-Debug.
The bundled adapter's original license is in `adapter/LICENSE-cortex-debug`.
