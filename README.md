# argmem

자주 쓰는 명령 파라미터를 기억해 두었다가 PowerShell / cmd 에서 자동으로 입력해 주는 도구.

## 설치

```powershell
cargo install --path .     # argmem.exe → %USERPROFILE%\.cargo\bin
argmem install             # PowerShell 프로필 + cmd AutoRun 에 연결
```

## 사용

```powershell
argmem add claude --dangerously-skip-permissions     # 확장 규칙
argmem add --as gl git log --oneline --graph -20     # 별칭 규칙
argmem list
argmem remove gl
argmem learn                    # PowerShell 히스토리에서 반복 패턴 추천
argmem learn --pick 2 --as gs   # 추천 2번을 별칭 gs 로 등록
argmem status
argmem uninstall                # 연결 해제 (규칙은 유지)
```

| 규칙 | PowerShell | cmd |
|---|---|---|
| 확장 (`claude`) | 줄 맨 앞에 `claude` 입력 후 **Space** → 파라미터가 입력줄에 채워짐 (Enter 전 확인/수정 가능). **Shift+Space** 는 그냥 공백 | `claude` 입력 시 doskey 가 파라미터를 붙여 실행 (화면엔 안 보임) |
| 별칭 (`gl`) | `gl` + Space → 펼쳐짐 / `gl` + Enter → 바로 실행. 기본 별칭(`gl`=Get-Location 등)도 덮어씀 | doskey 매크로 |

### 원래 명령 그대로 실행하기
- PowerShell: **Shift+Space** 로 띄어쓰기
- cmd: 명령 앞에 **공백 한 칸** (` claude`) → doskey 매크로를 거치지 않음

## 파일

| 경로 | 내용 |
|---|---|
| `%USERPROFILE%\.argmem\rules.json` | 규칙 (원본) |
| `%USERPROFILE%\.argmem\argmem.ps1` | 생성됨. `$PROFILE`(CurrentUserAllHosts) 에서 dot-source |
| `%USERPROFILE%\.argmem\argmem.doskey` | 생성됨. `HKCU\Software\Microsoft\Command Processor\AutoRun` 에서 로드 |

## 주의
- cmd 의 확장 규칙은 파라미터가 **보이지 않게** 붙습니다. `--dangerously-skip-permissions` 같은 위험한 플래그는 별칭(`--as`)으로 등록하는 것을 권장합니다.
- 줄 맨 앞이 정확히 등록 이름일 때만 확장됩니다. 그래서 `claude --resume` 처럼 **붙여넣은** 줄도 `claude` 다음 공백에서 확장됩니다.
- cmd 는 히스토리를 파일로 남기지 않아 `learn` 은 PowerShell 히스토리만 분석합니다.

## 테스트용 환경 변수
`ARGMEM_HOME`, `ARGMEM_PROFILES`(`;` 구분), `ARGMEM_REG_KEY`, `ARGMEM_HISTORY` 로 경로를 바꿔 실제 프로필이나 레지스트리를 건드리지 않고 시험할 수 있습니다.
