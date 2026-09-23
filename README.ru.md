# Cortex-Debug для Zed

Отладка микроконтроллеров ARM Cortex-M в редакторе [Zed](https://zed.dev)
с адаптером из [Cortex-Debug](https://github.com/Marus/cortex-debug).
Поддерживаются J-Link, OpenOCD, pyOCD, ST-Link, Black Magic Probe и QEMU.

[English version](README.md)

> **Статус: экспериментальный (v0.3.0).** Полные сессии отладки проверены на
> Linux с эмулятором QEMU (Cortex-M3), включая просмотр периферии по SVD,
> задач FreeRTOS и памяти. На реальном железе и внутри Zed на всех
> платформах расширение ещё не проверялось. Основная цель — **J-Link на
> Windows**. Будем рады баг-репортам.

Это неофициальный порт от сообщества. Он не связан ни с авторами
Cortex-Debug, ни с Zed Industries.

---

## Возможности

- **Launch** (прошивка + запуск) и **attach** (подключение к работающему МК без прошивки)
- Брейкпоинты: по строке, условные, по числу срабатываний, logpoints, по функциям; watchpoints
- Шаги: over, into, out, по инструкциям; pause
- Стек вызовов, области **Local / Global / Static / Registers**, watch, значения при наведении
- **Регистры периферии из SVD-файла**: периферия, регистры и битовые поля с именами значений прямо в панели Variables; значения можно менять ([подробнее](#регистры-периферии-svd))
- **Задачи FreeRTOS**: все задачи с состоянием, приоритетом, свободным стеком (high-water mark) и долей CPU ([подробнее](#задачи-rtos-freertos))
- **Просмотр памяти**: встроенный hex-просмотрщик Zed с редактированием и пунктом «Go To Memory» у переменных, регистров и задач ([подробнее](#просмотр-памяти))
- **Debug Console работает как консоль GDB**: `monitor reset`, `monitor halt`,
  `x/8wx 0x20000000`, `info registers`, `p/x *(uint32_t*)0x40020000`…
- Вывод gdb-server и semihosting в Debug Console (или в лог-файл)
- Потоки RTOS в стеке вызовов, если их поддерживает gdb-server (`"rtos"`; J-Link: FreeRTOS, embOS, Zephyr, …)
- RTT: TCP-порт для каждого канала печатается при старте, подключиться можно
  через PuTTY (Raw), `telnet` или `nc`
- `JLinkGDBServerCL.exe` находится автоматически в `C:\Program Files\SEGGER\JLink*`
- Подходят почти все атрибуты `launch.json` из Cortex-Debug (исключения — в разделе «Чего пока нет»)
- Ничего не нужно скачивать: адаптер встроен в расширение и запускается на Node.js, который идёт вместе с Zed

### Чего пока нет

Zed пока не позволяет расширениям добавлять свои панели, поэтому этих
функций интерфейса Cortex-Debug нет:

- графики и декодеры SWO/RTT, панель Live Watch
- multi-core и связанные сессии (`chainedConfigurations`)
- просмотр задач для ядер, кроме FreeRTOS (Zephyr, embOS, ThreadX, µC/OS).
  Стеки вызовов по потокам при этом работают через опцию gdb-server `"rtos"`.

## Что нужно установить

| Инструмент | Зачем |
|---|---|
| [Zed](https://zed.dev) | версия со встроенным отладчиком |
| [Rust через rustup](https://rustup.rs) | Zed собирает им dev-расширение |
| [Arm GNU Toolchain](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads) | `arm-none-eabi-gdb` (GDB ≥ 9), `objdump`, `nm` |
| gdb-server | например, [SEGGER J-Link Software](https://www.segger.com/downloads/jlink/), OpenOCD, pyOCD |

## Установка

Расширения пока нет в каталоге Zed, поэтому оно ставится как dev extension:

1. Клонируйте репозиторий:
   ```sh
   git clone https://github.com/<your-username>/zed-cortex-debug.git
   ```
2. В Zed откройте палитру команд (`Ctrl+Shift+P`), выполните
   **`zed: install dev extension`** и выберите папку с репозиторием.
3. Создайте в проекте прошивки файл `.zed/debug.json` (см. ниже).
4. Запустите отладку: `F4` (**`debugger: start`**) и выберите конфигурацию.

## Настройка

Минимальный `.zed/debug.json` для J-Link:

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

Полный пример с attach, RTT, сборкой перед запуском и явными путями к
инструментам — в [`examples/debug.json`](examples/debug.json).

- **Пути** считаются от `cwd`. Если `cwd` не указан, это корень проекта.
  Вместо `${workspaceFolder}` из VS Code пишите `$ZED_WORKTREE_ROOT`.
- **Атрибуты запуска Cortex-Debug** работают так, как описано в
  [debug_attributes.md](https://github.com/Marus/cortex-debug/blob/master/debug_attributes.md):
  `serverpath`, `armToolchainPath`, `gdbPath`, `svdFile`, `rtos`, `rttConfig`,
  `preLaunchCommands`, `overrideLaunchCommands` и т. д.
- **Настройки VS Code переходят в конфигурацию.** Например,
  `cortex-debug.armToolchainPath` становится полем `armToolchainPath`, а
  `cortex-debug.JLinkGDBServerPath` — полем `serverpath`.
- **Секции для ОС** `"windows": {…}`, `"linux": {…}` и `"osx": {…}` работают так же, как в VS Code.

Поля, специфичные для Zed:

| Поле | По умолчанию | Описание |
|---|---|---|
| `gdbServerOutput` | `"console"` | Куда идёт вывод gdb-server: `"console"`, `"none"` или путь к лог-файлу |
| `nodePath` | Node.js из Zed | Путь к другому `node` для запуска адаптера |
| `rtosView` | `true` | Показывать задачи FreeRTOS, если в программе найдены символы FreeRTOS |
| `svdAddrGapThreshold` | `16` | Регистры ближе этого числа байт читаются одним запросом; `0` — каждый регистр отдельно |

### Регистры периферии (SVD)

Укажите в `svdFile` CMSIS-SVD-файл вашего МК. Его можно взять из device
pack производителя, из установки STM32CubeIDE или из
[cmsis-svd-data](https://github.com/cmsis-svd/cmsis-svd-data).
В панели Variables появится область **Peripherals**:

```
Peripherals (STM32F407)
├─ GPIOA          0x40020000  General-purpose I/Os
│  ├─ MODER       0xA8000000        rw u32 @ 0x40020000
│  │  ├─ MODER15  Alternate = 0x2 (2)   rw [31:30]
│  │  └─ …
│  └─ ODR         0x00000020
└─ USART1 …
```

- **Значения читаются только при раскрытии узла.** Раскрытая периферия
  читается несколькими групповыми запросами к памяти. Значения хранятся в
  кэше до следующего запуска, шага или команды в Debug Console.
- **Регистры и поля можно менять.** Отредактируйте значение в панели
  Variables. Регистр принимает `0x…`, десятичное число, `0b…` или `#…`, поле —
  ещё и имя значения (например, `Alternate`). Запись поля делается как
  «прочитать — изменить — записать» для всего регистра, после чего значение
  перечитывается из железа.
- **Некоторые регистры не читаются автоматически.** Это регистры с
  `readAction` в SVD (чтение с побочным эффектом, например clear-on-read) и
  write-only регистры. Учтите, что раскрытие периферии вроде UART всё равно
  читает все её регистры. Сюда входят и регистры данных, у которых SVD не
  описывает побочные эффекты чтения, — как в любом просмотрщике памяти.
- **Регистр можно добавить в Watch.** У каждого регистра есть выражение
  вида `*(volatile unsigned int *)0x40020000`.
- **Поддерживаемые возможности SVD:** `derivedFrom`, массивы `dim` и
  кластеры, наследование свойств, все три формы записи битов и
  enumeratedValues.

### Задачи RTOS (FreeRTOS)

Если в программе есть FreeRTOS, в панели Variables автоматически появляется
область **RTOS (FreeRTOS)**. Настраивать ничего не нужно:

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

- **Данные берутся из списков самого ядра.** Списки ready, delayed, pending,
  suspended и deleted читаются обычными выражениями GDB, поэтому просмотр
  работает с любым gdb-server и с GDB без Python.
- **Stack free** — это high-water mark: сколько байт стека ещё хранят
  заполнитель `0xA5`. Для **Stack size** и **Stack used now** нужен
  `configRECORD_STACK_HIGH_ADDRESS 1`, для **CPU %** —
  `configGENERATE_RUN_TIME_STATS 1`.
- **Адреса открываются в Memory View.** Нажмите правой кнопкой на задачу, её
  TCB или адрес стека и выберите **Go To Memory**.
- **Стеки вызовов по задачам** — отдельная возможность. Для них добавьте
  `"rtos": "FreeRTOS"` (J-Link, OpenOCD или pyOCD), и gdb-server покажет
  каждую задачу потоком в списке Frames.
- Ядро должно быть собрано с отладочной информацией (обычно так и есть).
  Отключить просмотр можно через `"rtosView": false`.

### Просмотр памяти

В отладочной панели Zed есть встроенная панель **Memory View**. Если её не
видно, добавьте её через меню `+` панели. Адаптер настроен так, чтобы она
хорошо работала с микроконтроллерами:

- **Переход по адресу или выражению.** В строке адреса можно ввести адрес
  (`0x20000000`) или выражение (`&rxBuffer`, `huart1.pRxBuffPtr`,
  `pxCurrentTCB`). Для указателя откроется память, на которую он указывает,
  для остального — адрес самой переменной.
- **Go To Memory** доступен у локальных и глобальных переменных, watch, у
  регистров SVD и у задач RTOS.
- **Зарезервированные адреса показываются как нечитаемые**, а не гасят всю
  страницу 4 КБ. Например, конец flash и промежуток перед следующей областью
  отображаются правильно.
- **Память можно менять прямо в панели.** Изменения записываются в МК.

### Другие gdb-server

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

## Если что-то не работает

- **Адаптер не стартует или ничего не выводит.** Откройте логи адаптера через
  палитру команд (поиск «debug adapter logs»).
- **Нужен весь обмен с GDB.** Добавьте в конфиг `"showDevDebugOutput": "raw"`,
  и весь трафик GDB/MI появится в Debug Console.
- **Нужен лог запуска и остановки процессов.** Он лежит в
  `%TEMP%\cortex-debug-server.log` (на Windows).
- **`GDB executable "arm-none-eabi-gdb" was not found`.** Добавьте папку `bin`
  тулчейна в `PATH` или укажите `armToolchainPath` либо `gdbPath`.
- **`spawn JLinkGDBServerCL.exe ENOENT`.** Укажите в `serverpath` полный путь
  к `JLinkGDBServerCL.exe`.

В issue приложите `debug.json`, вывод Debug Console с
`"showDevDebugOutput": "raw"`, ОС, модель отладчика и версию gdb.

## Как это устроено

Cortex-Debug для VS Code состоит из двух частей:

| Часть | Что делает | В этом порте |
|---|---|---|
| **Debug adapter** (`src/gdb.ts`, Node.js) | DAP ⇄ GDB/MI, запуск gdb-server, прошивка, брейкпоинты, стек, переменные, регистры, дизассемблер | используется **без изменений** |
| **Фронтенд** (`src/frontend/*`, API VS Code) | подготовка `launch.json`, консоль gdb-server, окна SVD/RTOS/Memory, графики, Live Watch | заменён прослойкой `src/zed/adapter.ts` |

Прослойка проставляет значения по умолчанию и проверяет настройки для каждого
`servertype`. Она находит J-Link, приводит пути к абсолютным и передаёт вывод
gdb-server в Zed как DAP-события `output`. Специфичные для VS Code события
она отбрасывает, а полезные (ошибки, порты RTT) печатает в консоль.
`src/zed/svd.ts` и `src/zed/peripherals.ts` разбирают SVD и отдают его через
стандартные DAP-запросы `scopes`, `variables` и `setVariable`, поэтому для
просмотра периферии в Zed не нужен отдельный интерфейс.
`src/zed/rtos.ts` делает то же самое для задач FreeRTOS. `src/zed/memory.ts`
отдаёт частичные ответы `readMemory` (`unreadableBytes`), разбирает выражения
из строки адреса Memory View и добавляет `memoryReference` к переменным и
watch.

Rust-часть ([`src/lib.rs`](src/lib.rs)) содержит собранный адаптер. При первом
запуске она распаковывает его в рабочую папку расширения и запускает через
Node.js из Zed. Кроме того, в ней реализованы `dap_request_kind` и
`dap_config_to_scenario` для отладчика Zed.

### Структура репозитория

```
adapter/                  собранный адаптер (встраивается в wasm через include_str!)
debug_adapter_schemas/    JSON-схема debug.json, сгенерирована из package.json Cortex-Debug
examples/                 пример .zed/debug.json
patches/                  патч к Cortex-Debug + BASE_COMMIT, к которому он применяется
scripts/                  build-adapter.sh / build-adapter.ps1 — пересборка adapter/ из исходников
src/lib.rs                расширение Zed (Rust → wasm32-wasip2)
```

### Изменения в Cortex-Debug (`patches/`)

1. Добавлен `src/zed/adapter.ts` — замена фронтенда VS Code без интерфейса —
   и возможности, которые есть только в Zed: `svd.ts` + `peripherals.ts`
   (периферия из SVD), `rtos.ts` (задачи FreeRTOS), `memory.ts` (поддержка
   Memory View). Добавлена зависимость `fast-xml-parser`.
2. Запуск сессии перенесён из `gdb.ts` в `src/debugadapter-main.ts`, чтобы
   `GDBDebugSession` можно было импортировать без побочных эффектов. Сборка
   для VS Code при этом не меняется.
3. Исправлено зависание на старте в `mi2.ts`. Новые версии GDB (замечено на
   GDB 15) могут отдавать токен MI-ответа (`5` из `5^done`) отдельным куском,
   и адаптер принимал его за вывод консоли. Это исправление пригодится и в
   основном репозитории.

## Разработка

Пересборка адаптера из исходников Cortex-Debug (нужны git и Node.js 18+):

```sh
./scripts/build-adapter.sh          # Linux / macOS
.\scripts\build-adapter.ps1         # Windows PowerShell
```

После этого нажмите **Rebuild** у расширения на странице Extensions в Zed.

Пока вы правите сам адаптер, Zed можно направить на вашу копию вместо
встроенной:

```jsonc
// settings.json в Zed
"dap": { "cortex-debug": { "binary": "C:/src/cortex-debug" } }  // папка с dist/zedadapter.js
```

## Планы

- [ ] Проверить на Windows с J-Link
- [ ] Опубликовать в каталоге расширений Zed
- [ ] Debug locator: запуск отладки прямо из задач CMake/Make
- [x] Регистры периферии из SVD как отдельная область переменных
- [x] Задачи FreeRTOS; поддержка Memory View
- [ ] Просмотр задач для Zephyr, embOS, ThreadX, µC/OS
- [ ] Отправить Zed-entry point и исправление MI в Cortex-Debug

## Благодарности и лицензия

Основная работа сделана в [Cortex-Debug](https://github.com/Marus/cortex-debug)
(Marcel Ball и контрибьюторы), который, в свою очередь, основан на
[code-debug](https://github.com/WebFreak001/code-debug) Яна Юржицы.

Лицензия [MIT](LICENSE), как у Cortex-Debug. Исходная лицензия встроенного
адаптера — в `adapter/LICENSE-cortex-debug`.
