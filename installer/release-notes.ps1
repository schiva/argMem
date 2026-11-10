# Builds GitHub release notes: install instructions + the CHANGELOG section for the tag.
param(
    [Parameter(Mandatory)][string]$Tag,
    [string]$Repo = 'schiva/argMem'
)
$ErrorActionPreference = 'Stop'
$changes = & (Join-Path $PSScriptRoot 'changelog-section.ps1') -Version $Tag
$fence = '```'
@"
## 설치 / 업데이트

PowerShell 에 붙여넣으세요 (관리자 권한 불필요). 이미 설치돼 있으면 최신 버전으로 업데이트됩니다.

${fence}powershell
irm https://github.com/$Repo/releases/latest/download/install.ps1 | iex
${fence}

이 버전을 지정해서 설치하려면:

${fence}powershell
& ([scriptblock]::Create((irm https://github.com/$Repo/releases/download/$Tag/install.ps1))) -Version $Tag
${fence}

또는 ``argmem-$Tag-windows-x64.zip`` 을 받아 압축을 풀고 ``install.cmd`` 를 더블클릭하세요.

## 변경 사항

$changes

---
제거: ``& ([scriptblock]::Create((irm https://github.com/$Repo/releases/latest/download/install.ps1))) -Uninstall``
· 전체 변경 이력: [CHANGELOG.md](https://github.com/$Repo/blob/main/CHANGELOG.md)
"@
