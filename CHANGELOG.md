# 변경 이력

이 프로젝트의 버전별 변경 내용입니다. 릴리스 노트는 이 파일의 해당 버전 항목으로 자동 작성됩니다.

## [0.1.3] - 2026-11-10

### 문서
- README 맨 위에 주요 기능 요약을 추가했습니다.
- 변경 이력 파일(`CHANGELOG.md`)을 추가했습니다.

### 릴리스
- GitHub 릴리스 노트에 해당 버전의 변경 내용이 자동으로 들어갑니다.
- 릴리스 zip 에 `CHANGELOG.md` 를 포함했습니다.

## [0.1.2] - 2026-11-10

### 추가
- **Enter 확장** 설정을 추가했습니다: `argmem config enter on|off` (기본 off).
  - 켜면 PowerShell 에서 등록 이름만 입력하고 Enter 를 눌러도 확장한 뒤 실행합니다. 예: `claude` + Enter → `claude --dangerously-skip-permissions`
  - 확장된 줄이 화면과 히스토리에 남습니다.
- `argmem config` 로 현재 설정을 볼 수 있습니다. `argmem status` 에도 표시됩니다.

### 변경
- PowerShell 에서 명령 앞에 공백을 넣으면 (` claude`) Space·Enter 모두 확장하지 않습니다. cmd 의 doskey 와 같은 규칙입니다.

## [0.1.1] - 2026-11-10

### 추가
- 명령이나 옵션을 잘못 입력하면 다음을 함께 출력합니다.
  - 한국어 오류 메시지
  - 비슷한 이름 제안 (예: `ad` → `add`)
  - 해당 명령의 전체 사용법
- 실행 중 오류 안내를 보완했습니다.
  - `remove` 할 이름이 없으면 등록된 규칙 목록을 보여 줍니다.
  - `add` 에 잘못된 이름을 주면 사용법을 함께 보여 줍니다.
  - `learn --pick` 번호가 범위를 벗어나면 고를 수 있는 범위를 알려 줍니다.

### 변경
- 도움말을 한국어로 정리했습니다: 사용법·명령·옵션·인자 제목, 기본값 표시, 명령별 예시.
- 도움말과 오류에 `argmem.exe` 대신 `argmem` 으로 표시합니다.

## [0.1.0] - 2026-11-10

첫 릴리스입니다.

- **확장 규칙**: `argmem add claude --dangerously-skip-permissions`
  - PowerShell: `claude` + Space 를 누르면 파라미터가 입력줄에 채워집니다. Shift+Space 는 그냥 공백입니다.
  - cmd: doskey 매크로로 파라미터가 붙습니다.
- **별칭 규칙**: `argmem add --as gl git log --oneline`
- **learn**: PowerShell 히스토리에서 반복되는 명령+파라미터를 추천합니다. 같은 명령을 다른 파라미터로도 쓰면 별칭 등록을 요구합니다.
- **install / uninstall**: PowerShell 프로필(Windows PowerShell, pwsh)과 cmd AutoRun 에 연결하거나 해제합니다.
- **설치 스크립트**: 한 줄 설치를 지원합니다. SHA256 검증, 사용자 PATH 등록, `-Uninstall` 을 포함합니다.
- **실행 파일**: VC++ 런타임을 정적 링크해 재배포 패키지 없이 실행됩니다.
