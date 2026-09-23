# Cortex-Debug для Zed

Расширение для Zed, которое подключает debug adapter из
[Cortex-Debug](https://github.com/Marus/cortex-debug) (VS Code) к встроенному
отладчику Zed. Можно отлаживать микроконтроллеры ARM Cortex-M через J-Link,
OpenOCD, pyOCD, ST-Link, Black Magic Probe или QEMU. Основная цель —
**J-Link на Windows**.

## Как устроено

Cortex-Debug в VS Code состоит из двух частей:

| Часть | Что делает | В Zed |
|---|---|---|
| Debug adapter (`src/gdb.ts`, Node.js) | DAP ⇄ GDB/MI, запуск gdb-server, flash, брейкпоинты, стек, переменные, регистры, дизассемблер | используется **без изменений** |
| Фронтенд (`src/frontend/*`, API VS Code) | подготовка `launch.json`, консоль gdb-server, окна SVD/RTOS/Memory/графики, Live Watch | заменён прослойкой `src/zed/adapter.ts` (из патча) |

Прослойка `src/zed/adapter.ts` делает без VS Code то же, что делал фронтенд:
проставляет значения по умолчанию, проверяет настройки для каждого `servertype`,
сама находит `JLinkGDBServerCL.exe` в `C:\Program Files\SEGGER\JLink*`,
приводит пути к абсолютным. Вывод gdb-server она перенаправляет в Debug
Console Zed.

Rust-часть (`src/lib.rs`) содержит адаптер внутри себя. При первом запуске она
распаковывает его в рабочую папку расширения и запускает через Node.js, который
поставляется вместе с Zed. Скачивать ничего не нужно.

## Что работает (MVP)

- `launch` (прошивка + старт) и `attach`
- брейкпоинты: обычные, условные, logpoints, по функциям; watchpoints
- шаги (step over / into / out, по инструкциям), pause, перезапуск сессии
- Debug Console = консоль GDB: `monitor reset`, `monitor halt`, `x/8wx 0x20000000`, `info registers` и любые другие команды
- стек вызовов, области Local, Global, Static и Registers, watch, hover
- вывод gdb-server и semihosting в Debug Console
- RTOS-потоки, если `"rtos"` поддерживает сам gdb-server (J-Link: FreeRTOS, embOS, Zephyr…)
- RTT: адрес TCP-порта для канала печатается в консоли, подключиться можно через `telnet`, PuTTY (Raw) или `nc`

## Чего пока нет

У Zed нет API для собственных панелей в расширениях, поэтому недоступны:
Peripheral/SVD viewer, RTOS viewer, Memory viewer, графики SWO/RTT, панель Live
Watch, multi-core (`chainedConfigurations`). Регистры периферии можно читать
вручную в Debug Console: `x/4wx 0x40020000` или
`p/x *(uint32_t*)0x40020000`. Сброс МК без перезапуска сессии: `monitor reset`.

## Установка (Windows + J-Link)

1. Установите:
   - [J-Link Software](https://www.segger.com/downloads/jlink/);
   - [Arm GNU Toolchain](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads)
     (`arm-none-eabi-*`, нужен `arm-none-eabi-gdb` версии 9 или новее). При
     установке отметьте «Add path to environment variable» или укажите
     `armToolchainPath` в конфиге;
   - [Rust через rustup](https://rustup.rs) — он нужен, чтобы Zed собрал
     расширение (dev extension).
2. В Zed: `Ctrl+Shift+P` → **zed: install dev extension** → выберите папку
   этого репозитория.
3. Скопируйте `examples/debug.json` в свой проект как `.zed/debug.json` и
   поменяйте `device` и `executable`.
4. Нажмите `F4` (или `debugger: start`) и выберите конфигурацию.

Если что-то не запускается, найдите в палитре команд пункт «debug adapter logs» —
там виден обмен DAP. Для
подробного лога добавьте в конфиг `"showDevDebugOutput": "raw"`: весь обмен с
GDB появится в Debug Console. Лог запуска и остановки процессов пишется в
`%TEMP%\cortex-debug-server.log`.

### Минимальный конфиг

```jsonc
[
  {
    "label": "Debug (J-Link)",
    "adapter": "cortex-debug",
    "request": "launch",
    "servertype": "jlink",
    "device": "STM32F407VG",
    "executable": "build/firmware.elf",
    "runToEntryPoint": "main"
  }
]
```

Поддерживаются все атрибуты `launch.json` из Cortex-Debug (см.
[debug_attributes.md](https://github.com/Marus/cortex-debug/blob/master/debug_attributes.md)):
`serverpath`, `armToolchainPath`, `gdbPath`, `svdFile`, `rtos`, `rttConfig`,
`preLaunchCommands`, `overrideLaunchCommands` и так далее. Поля, которые в VS
Code задавались в `settings.json` (`cortex-debug.armToolchainPath`,
`cortex-debug.JLinkGDBServerPath` и т. п.), теперь пишутся прямо в
конфигурацию. Секции `"windows": {…}`, `"linux": {…}` и `"osx": {…}` работают
так же, как в VS Code.

Дополнительные поля, специфичные для Zed:

| Поле | Значение |
|---|---|
| `gdbServerOutput` | `"console"` (по умолчанию), `"none"` или путь к лог-файлу |
| `nodePath` | свой `node.exe` вместо того, что поставляется с Zed |

Вместо `${workspaceFolder}` используйте `$ZED_WORKTREE_ROOT`. Если `cwd` не
указан, берётся корень проекта.

## Разработка

```
adapter/            собранный адаптер (встраивается в wasm через include_str!)
patches/            патч к Cortex-Debug + BASE_COMMIT, на котором он проверен
scripts/            build-adapter.sh / .ps1 — пересборка adapter/ из исходников
src/lib.rs          расширение Zed (Rust → wasm32-wasip2)
debug_adapter_schemas/cortex-debug.json   JSON-схема, собранная из package.json Cortex-Debug
```

Пересборка адаптера (нужны git и Node.js 18+):

```powershell
.\scripts\build-adapter.ps1
```

Затем в Zed откройте Extensions → Cortex-Debug → **Rebuild**. Если вы
правите сам адаптер, пересобирать расширение не обязательно: укажите путь к
своей копии в настройках Zed.

```jsonc
// settings.json
"dap": { "cortex-debug": { "binary": "C:/src/cortex-debug" } }  // папка с dist/zedadapter.js
```

Что меняет патч в Cortex-Debug:

1. Добавляет `src/zed/adapter.ts` — headless-замену фронтенда (подготовка
   конфигурации и консоль gdb-server).
2. Переносит запуск сессии из `gdb.ts` в `src/debugadapter-main.ts`, чтобы
   `GDBDebugSession` можно было импортировать без побочных эффектов.
3. Исправляет ошибку в `mi2.ts`: gdb 15 иногда отдаёт токен MI-ответа (`5` из
   `5^done`) отдельным куском, адаптер принимал его за вывод консоли и зависал
   на старте. Это стоит отправить и в upstream.

Лицензия: MIT, как у Cortex-Debug (`LICENSE`).
