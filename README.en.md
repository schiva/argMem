# argmem

[한국어](README.md) | **English**

A Windows tool that remembers the parameters you always type after a command and fills them in for you in PowerShell and cmd.

## 🚀 One-line install

> [!IMPORTANT]
> Paste this single line into PowerShell. No administrator rights needed.
>
> ```powershell
> irm https://github.com/schiva/argMem/releases/latest/download/install.ps1 | iex
> ```
>
> Then open a **new window** and register a rule, e.g. `argmem add claude --dangerously-skip-permissions`. Running the same line again updates argmem to the latest version. For other install options and uninstalling, see [Install](#install).

```text
PS> claude␣          ← press Space
PS> claude --dangerously-skip-permissions ␣   ← the line is filled in (review/edit before Enter)

PS> claude⏎          ← or just press Enter
PS> claude --dangerously-skip-permissions     ← it is expanded and run (Enter expansion, on by default)
```

> [!NOTE]
> argmem's own messages and help text are in Korean. Every command and option is in English and listed below.

## Features

- **Expansion rules**: type `claude` + Space and the remembered parameters are filled into the input line. You can keep typing more parameters such as `-c`.
- **Enter expansion** (on by default): type just `claude` and press Enter to run it with the parameters. Turn it off with `argmem config enter off`.
- **Alias rules**: give a short name to a longer command, e.g. `gl` → `git log --oneline --graph -20`.
- **cmd support**: the same rules work in cmd through doskey macros.
- **learn**: analyzes your PowerShell history and suggests command + parameter combinations you repeat.
- **Helpful errors**: on a typo argmem suggests the closest command and prints that command's usage.
- **Easy install and update**: one line to install, the same line to update. No administrator rights.

See [CHANGELOG.md](CHANGELOG.md) (Korean) for changes in each version.

## Requirements

- 64-bit Windows 10/11
- Windows PowerShell 5.1 or PowerShell 7 (`pwsh`) with PSReadLine (included by default)
- No VC++ redistributable needed (the runtime is statically linked).

## Install

### Option 1. One line in PowerShell (recommended)

```powershell
irm https://github.com/schiva/argMem/releases/latest/download/install.ps1 | iex
```

This downloads the latest `argmem.exe` from [GitHub Releases](https://github.com/schiva/argMem/releases), then:

1. Verifies the file against `SHA256SUMS.txt`.
2. Copies it to `%LOCALAPPDATA%\Programs\argmem` and adds that folder to your user PATH.
3. Runs `argmem install`.

No administrator rights are needed.

To install a specific version, or to uninstall:

```powershell
& ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Version v0.2.0
& ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Uninstall
```

If argmem is already installed, the same one-liner updates it to the latest version. Your rules and settings are kept. Check the installed version with `argmem --version`.

### Option 2. zip

Download `argmem-<version>-windows-x64.zip` from [Releases](https://github.com/schiva/argMem/releases/latest), extract it, and double-click `install.cmd`. It installs the `argmem.exe` from the zip exactly like option 1. The zip also contains the README files and `CHANGELOG.md`.

### Release assets

| File | Contents |
|---|---|
| `install.ps1` | Install / update / uninstall script (used by option 1) |
| `argmem.exe` | The executable (64-bit, standalone) |
| `argmem-<version>-windows-x64.zip` | exe + install scripts + `install.cmd` + docs |
| `SHA256SUMS.txt` | SHA256 checksums of the files above |

### Option 3. Build from source

```powershell
winget install Rustlang.Rustup   # needs Rust (MSVC) + Visual C++ Build Tools
cargo install --path .           # argmem.exe → %USERPROFILE%\.cargo\bin
argmem install
```

### What `argmem install` does

1. Generates `%USERPROFILE%\.argmem\argmem.ps1` and `argmem.doskey`.
2. Adds this line to Windows PowerShell's `$PROFILE.CurrentUserAllHosts`, and to the `pwsh` profile too if PowerShell 7 is installed:
   ```powershell
   if (Test-Path '...\.argmem\argmem.ps1') { . '...\.argmem\argmem.ps1' } # argmem
   ```
3. Adds `doskey /macrofile=...\argmem.doskey` to the `AutoRun` value under `HKCU\Software\Microsoft\Command Processor`. An existing value is kept and joined with ` & `.

Running it again never adds duplicates. Changes apply to **newly opened windows**.

## Usage

| Command | Description |
|---|---|
| `argmem add <command> <params...>` | Add/update an expansion rule |
| `argmem add --as <alias> <command> <params...>` | Add/update an alias rule (`--as` goes **before** the command) |
| `argmem list` (`ls`) | Show registered rules |
| `argmem remove <name>` (`rm`) | Delete a rule |
| `argmem learn [--top 10] [--min 3]` | Suggest repeated command + parameter pairs from PowerShell history |
| `argmem learn --pick <n> [--as <alias>]` | Register suggestion number `n` right away |
| `argmem config [enter on\|off]` | Show / change settings ([Enter expansion](#enter-expansion)) |
| `argmem apply` | Regenerate the shell files from `rules.json` |
| `argmem status` | Show file locations, settings and whether the profile / AutoRun are hooked up |
| `argmem install` / `uninstall` | Hook into / unhook from the shells (rules are kept) |

```powershell
argmem add claude --dangerously-skip-permissions
argmem add --as gl git log --oneline --graph -20
argmem learn
argmem learn --pick 2 --as gs
```

- If you mistype a command or option, argmem prints the error, a suggestion when a similar name exists, and **the usage of that command**. Use `argmem <command> --help` for any command's help.
- Running `add` again with the same name replaces the rule. Names are case-insensitive.
- Command/alias names may contain letters (including Korean), digits, `.` `_` `-` `+`.
- Parameters containing spaces are stored wrapped in double quotes.
- `add` / `remove` regenerate the shell files automatically. To apply them to the PowerShell window that is already open, run `. "$env:USERPROFILE\.argmem\argmem.ps1"`.

### How each rule type behaves

| Rule | PowerShell | cmd |
|---|---|---|
| **Expansion** (`claude`) | `claude` + **Space** → parameters are filled into the line (review before Enter). `claude` + **Enter** → expanded and run ([Enter expansion](#enter-expansion), on by default) | Typing `claude` runs it with the parameters via doskey (not shown on screen) |
| **Alias** (`gl`) | `gl` + Space → expands, `gl` + Enter → runs. Overrides built-in aliases such as `gl` (= Get-Location) | Expanded and run via a doskey macro |

Extra parameters you type go after the remembered ones. For example `claude` + Space + `-c` runs `claude --dangerously-skip-permissions -c`. Typing `claude -c` in cmd gives the same result.

### Enter expansion

**On by default.** In PowerShell, typing just `claude` and pressing Enter runs `claude --dangerously-skip-permissions`, the same as in cmd.

```powershell
argmem config             # show settings
argmem config enter off   # turn off: only expand with Space, so you can review before running
argmem config enter on    # turn back on (default)
```

- Expansion happens on Enter only when the whole line is exactly a registered name. The expanded line stays on screen and in history.
- You cannot review the added parameters before pressing Enter. If you registered a dangerous flag and want to check before running, turn it off.
- **If you bound something else to Enter** (PSReadLine `Set-PSReadLineKeyHandler -Chord Enter ...`), argmem does not override it. Instead it prints a one-line notice when a window opens; run `argmem config enter off` to silence it. Expansion is applied only when Enter is the default (`AcceptLine` or `ValidateAndAcceptLine`), and that original function is called after expanding.
- Setting changes apply to new windows. cmd always applies macros on Enter, so this setting does not affect cmd.
- If you saved this setting with v0.1.2 or later, your value is kept. If you saved `off`, it stays off after updating.

### Running the original command

- PowerShell: type the space with **Shift+Space**, or put **one leading space** before the command (` claude`). Neither Space nor Enter will expand it.
- cmd: put **one leading space** before the command (` claude`) to bypass the doskey macro.

### learn

- Reads `%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\*_history.txt`. cmd does not save history to a file, so it is not covered.
- By default it shows the top 10 entries repeated 3 or more times, excluding rules you already registered.
- If you have used the same command with other parameters too (e.g. `git status -s` and `git diff`), `--pick` requires an alias via `--as`. As an expansion rule, those parameters would be added every time you type that command.

## Files

| Path | Contents |
|---|---|
| `%USERPROFILE%\.argmem\rules.json` | Source of truth for rules and settings (`settings.expand_on_enter`) |
| `%USERPROFILE%\.argmem\argmem.ps1` | Generated. Rule table, alias functions, PSReadLine key handlers (UTF-8 with BOM) |
| `%USERPROFILE%\.argmem\argmem.doskey` | Generated. doskey macros for cmd (`$` is written as `$$`) |

Do not edit the generated files; use `argmem add` / `remove` / `config` instead.

## Uninstall

```powershell
# Installed with the install script: unhook + clean PATH + delete the program folder
& ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Uninstall

# Or by hand
argmem uninstall                                  # remove only the profile line and the AutoRun entry
Remove-Item -Recurse "$env:USERPROFILE\.argmem"   # also delete your rules
cargo uninstall argmem                            # if you built from source
```

`uninstall` removes only the profile lines marked `# argmem` and only argmem's part of AutoRun. If AutoRun becomes empty, the value is deleted.

## Caveats

- In cmd, expansion rules add parameters **invisibly**. If you also use a dangerous flag like `--dangerously-skip-permissions` in cmd, registering it as an alias (`--as ccy`) is safer.
- In PowerShell, a **pasted** line such as `claude --resume` also expands at the space after `claude`.
- doskey macros and Space/Enter expansion only act on what you type interactively, so scripts and batch files are not affected. PowerShell alias rules, however, are defined as functions and aliases, so scripts run in the same session can see them.
- Alias-rule parameters are inserted into PowerShell code as-is. Write special characters such as `$` and quotes in valid PowerShell syntax.

## Development

```text
src/main.rs     CLI (clap)
src/store.rs    read/write rules.json, name validation
src/emit.rs     generate argmem.ps1 / argmem.doskey
src/install.rs  hook/unhook profile and AutoRun (via reg.exe)
src/learn.rs    PSReadLine history analysis
installer/      install.ps1 (ASCII only, for irm | iex) / install.cmd (double-click for the zip)
                changelog-section.ps1 / release-notes.ps1 (release notes from CHANGELOG)
.github/workflows/release.yml   build, test and release on v* tags
CHANGELOG.md    version history (source of release notes)
```

```powershell
cargo test
cargo build --release
```

### Releasing

1. Bump `version` in `Cargo.toml`.
2. Add a `## [x.y.z] - date` section at the top of `CHANGELOG.md`.
3. Commit, then push a tag with the same version.

```powershell
git tag v0.2.1
git push origin v0.2.1
```

GitHub Actions then:

1. Checks that the tag matches `Cargo.toml` and that `CHANGELOG.md` has a section for it; fails otherwise.
2. Runs the tests and builds.
3. Creates the release with notes made from that CHANGELOG section plus install instructions.
4. Uploads `argmem.exe`, `install.ps1`, `argmem-<tag>-windows-x64.zip` and `SHA256SUMS.txt`.

These environment variables redirect argmem's paths so you can test without touching your real profile or registry:

| Variable | Default |
|---|---|
| `ARGMEM_HOME` | `%USERPROFILE%\.argmem` |
| `ARGMEM_PROFILES` | profile paths reported by `powershell` / `pwsh` (`;`-separated) |
| `ARGMEM_REG_KEY` | `HKCU\Software\Microsoft\Command Processor` |
| `ARGMEM_HISTORY` | `*_history.txt` in the PSReadLine history folder |
