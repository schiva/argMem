# argmem

자주 쓰는 명령 파라미터를 기억해 두었다가 PowerShell / cmd 에서 자동으로 입력해 주는 Windows용 도구입니다.

## 🚀 한 줄 설치

> [!IMPORTANT]
> PowerShell 에 아래 한 줄을 붙여넣으면 끝납니다. 관리자 권한은 필요 없습니다.
>
> ```powershell
> irm https://github.com/schiva/argMem/releases/latest/download/install.ps1 | iex
> ```
>
> 설치 후 **새 창**을 열고 `argmem add claude --dangerously-skip-permissions` 처럼 규칙을 등록하세요. 같은 명령을 다시 실행하면 최신 버전으로 업데이트됩니다. 다른 설치 방법과 제거 방법은 [설치](#설치) 를 보세요.

```text
PS> claude␣          ← Space 를 누르면
PS> claude --dangerously-skip-permissions ␣   ← 입력줄이 이렇게 채워짐 (Enter 전 확인·수정 가능)
```

## 요구 사항

- 64비트 Windows 10/11
- Windows PowerShell 5.1 또는 PowerShell 7 (`pwsh`), PSReadLine (기본 포함)
- VC++ 재배포 패키지는 필요 없습니다 (런타임 정적 링크).

## 설치

### 방법 1. PowerShell 한 줄 (권장)

```powershell
irm https://github.com/schiva/argMem/releases/latest/download/install.ps1 | iex
```

[GitHub Releases](https://github.com/schiva/argMem/releases) 에서 최신 `argmem.exe` 를 내려받습니다. 이어서 다음을 처리합니다.

1. `SHA256SUMS.txt` 로 파일을 검증합니다.
2. `%LOCALAPPDATA%\Programs\argmem` 에 복사하고 사용자 PATH 에 추가합니다.
3. `argmem install` 을 실행합니다.

관리자 권한은 필요 없습니다.

특정 버전을 설치하거나 제거할 때:

```powershell
& ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Version v0.1.0
& ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Uninstall
```

### 방법 2. zip

Releases 에서 `argmem-<버전>-windows-x64.zip` 을 받아 압축을 풀고 `install.cmd` 를 더블클릭합니다. zip 안의 `argmem.exe` 로 방법 1 과 똑같이 설치합니다.

### 방법 3. 소스에서 빌드

```powershell
winget install Rustlang.Rustup   # Rust (MSVC) + Visual C++ Build Tools 필요
cargo install --path .           # argmem.exe → %USERPROFILE%\.cargo\bin
argmem install
```

### `argmem install` 이 하는 일

1. `%USERPROFILE%\.argmem\argmem.ps1` 과 `argmem.doskey` 를 생성합니다.
2. Windows PowerShell 의 `$PROFILE.CurrentUserAllHosts` 에 아래 한 줄을 추가합니다. `pwsh` 가 설치돼 있으면 그쪽 프로필에도 추가합니다.
   ```powershell
   if (Test-Path '...\.argmem\argmem.ps1') { . '...\.argmem\argmem.ps1' } # argmem
   ```
3. `HKCU\Software\Microsoft\Command Processor` 의 `AutoRun` 값에 `doskey /macrofile=...\argmem.doskey` 를 추가합니다. 기존 값이 있으면 ` & ` 로 이어 붙입니다.

여러 번 실행해도 중복으로 추가되지 않습니다. **새로 연 창부터 적용**됩니다.

## 사용법

| 명령 | 설명 |
|---|---|
| `argmem add <명령> <파라미터...>` | 확장 규칙 추가/수정 |
| `argmem add --as <별칭> <명령> <파라미터...>` | 별칭 규칙 추가/수정 (`--as` 는 **명령 앞**에 씀) |
| `argmem list` (`ls`) | 등록된 규칙 보기 |
| `argmem remove <이름>` (`rm`) | 규칙 삭제 |
| `argmem learn [--top 10] [--min 3]` | PowerShell 히스토리에서 반복되는 명령+파라미터 추천 |
| `argmem learn --pick <번호> [--as <별칭>]` | 추천 항목을 바로 등록 |
| `argmem apply` | `rules.json` 으로 셸 설정 파일 다시 생성 |
| `argmem status` | 파일 위치와 프로필·AutoRun 연결 여부 확인 |
| `argmem install` / `uninstall` | 셸에 연결 / 연결 해제 (규칙 파일은 유지) |

```powershell
argmem add claude --dangerously-skip-permissions
argmem add --as gl git log --oneline --graph -20
argmem learn
argmem learn --pick 2 --as gs
```

- 명령이나 옵션을 잘못 입력하면 오류 내용(비슷한 이름이 있으면 제안 포함)과 함께 **해당 명령의 사용법**이 출력됩니다. 각 명령의 도움말은 `argmem <명령> --help` 로 볼 수 있습니다.
- 같은 이름으로 다시 `add` 하면 기존 규칙을 덮어씁니다. 이름은 대소문자를 구분하지 않습니다.
- 명령/별칭 이름에는 영문, 숫자, 한글, `.` `_` `-` `+` 만 쓸 수 있습니다.
- 공백이 들어간 파라미터는 큰따옴표로 감싸서 저장합니다.
- `add` / `remove` 를 하면 셸 설정 파일이 자동으로 다시 생성됩니다. 지금 열린 PowerShell 창에 바로 반영하려면 `. "$env:USERPROFILE\.argmem\argmem.ps1"` 을 실행하세요.

### 규칙 종류별 동작

| 규칙 | PowerShell | cmd |
|---|---|---|
| **확장** (`claude`) | 줄 전체가 `claude` 이고 커서가 줄 끝일 때 **Space** → 파라미터가 입력줄에 채워짐 | `claude` 입력 시 doskey 가 파라미터를 붙여 실행 (화면에는 안 보임) |
| **별칭** (`gl`) | `gl` + Space → 펼쳐짐, `gl` + Enter → 바로 실행. `gl`(=Get-Location) 같은 기본 별칭도 덮어씀 | doskey 매크로로 펼쳐서 실행 |

### 원래 명령 그대로 실행하기

- PowerShell: **Shift+Space** 로 띄어쓰면 확장되지 않습니다.
- cmd: 명령 앞에 **공백 한 칸**을 넣으면 (` claude`) doskey 매크로를 거치지 않습니다.

### learn

- `%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\*_history.txt` 를 분석합니다. cmd 는 히스토리를 파일로 남기지 않아 대상이 아닙니다.
- 기본값은 3회 이상 반복된 항목을 위에서 10개 보여 줍니다. 이미 등록된 규칙은 제외합니다.
- 같은 명령을 다른 파라미터로도 쓴 적이 있으면 (`git status -s`, `git diff` 등) `--pick` 에 `--as` 별칭을 요구합니다. 확장 규칙으로 등록하면 그 명령을 칠 때마다 파라미터가 붙기 때문입니다.

## 파일

| 경로 | 내용 |
|---|---|
| `%USERPROFILE%\.argmem\rules.json` | 규칙 원본 |
| `%USERPROFILE%\.argmem\argmem.ps1` | 생성 파일. 규칙 표, 별칭 함수, PSReadLine 키 처리 (UTF-8 BOM) |
| `%USERPROFILE%\.argmem\argmem.doskey` | 생성 파일. cmd 용 doskey 매크로 (`$` 는 `$$` 로 변환) |

생성 파일은 직접 고치지 말고 `argmem add/remove` 를 쓰세요.

## 제거

```powershell
# 설치 스크립트로 설치한 경우: 연결 해제 + PATH 정리 + 프로그램 폴더 삭제
& ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Uninstall

# 직접 하려면
argmem uninstall                                  # 프로필 줄과 AutoRun 항목만 제거
Remove-Item -Recurse "$env:USERPROFILE\.argmem"   # 규칙까지 지우려면
cargo uninstall argmem                            # 소스에서 설치한 경우
```

`uninstall` 은 프로필에서 `# argmem` 표시가 붙은 줄만 지우고, AutoRun 에서 argmem 부분만 뺍니다. 그 결과 AutoRun 이 비면 값 자체를 삭제합니다.

## 주의

- cmd 의 확장 규칙은 파라미터가 **보이지 않게** 붙습니다. `--dangerously-skip-permissions` 같은 위험한 플래그를 cmd 에서도 쓴다면 별칭(`--as ccy`)으로 등록하는 편이 안전합니다.
- PowerShell 에서 `claude --resume` 처럼 **붙여넣은** 줄도 `claude` 다음 공백에서 확장됩니다.
- doskey 매크로와 Space 확장은 사람이 직접 입력할 때만 동작하므로 스크립트와 배치 파일에는 영향이 없습니다. 반면 PowerShell 별칭 규칙은 함수와 별칭으로 정의되므로 같은 세션에서 실행하는 스크립트에도 보입니다.
- 별칭 규칙의 파라미터는 PowerShell 코드로 그대로 들어갑니다. `$`, 따옴표 같은 특수문자는 PowerShell 문법에 맞게 적어야 합니다.

## 개발

```text
src/main.rs     CLI (clap)
src/store.rs    rules.json 읽기/쓰기, 이름 검사
src/emit.rs     argmem.ps1 / argmem.doskey 생성
src/install.rs  프로필·AutoRun 연결/해제 (reg.exe 사용)
src/learn.rs    PSReadLine 히스토리 분석
installer/      install.ps1 (ASCII 전용, irm | iex 용) / install.cmd (zip 더블클릭용)
.github/workflows/release.yml   v* 태그 push 시 빌드·테스트·릴리스
```

```powershell
cargo test
cargo build --release
```

### 릴리스

`Cargo.toml` 의 `version` 을 올리고 커밋한 뒤 같은 버전의 태그를 push 합니다.

```powershell
git tag v0.1.1
git push origin v0.1.1
```

GitHub Actions 가 다음을 자동으로 처리합니다. 태그와 `Cargo.toml` 버전이 다르면 실패합니다.

1. 테스트하고 빌드합니다.
2. Release 를 만듭니다.
3. 다음 파일을 올립니다: `argmem.exe`, `install.ps1`, `argmem-<태그>-windows-x64.zip`, `SHA256SUMS.txt`

아래 환경 변수로 경로를 바꾸면 실제 프로필이나 레지스트리를 건드리지 않고 시험할 수 있습니다.

| 변수 | 기본값 |
|---|---|
| `ARGMEM_HOME` | `%USERPROFILE%\.argmem` |
| `ARGMEM_PROFILES` | `powershell` / `pwsh` 에 물어본 프로필 경로 (`;` 로 여러 개) |
| `ARGMEM_REG_KEY` | `HKCU\Software\Microsoft\Command Processor` |
| `ARGMEM_HISTORY` | PSReadLine 히스토리 폴더의 `*_history.txt` |
