//! rules.json → argmem.ps1 (PowerShell) / argmem.doskey (cmd) 생성.

use crate::store::{home, Rule, Store};
use std::{fs, io, path::PathBuf};

pub fn ps1_path() -> PathBuf {
    home().join("argmem.ps1")
}

pub fn doskey_path() -> PathBuf {
    home().join("argmem.doskey")
}

fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

pub fn render_ps1(rules: &[Rule]) -> String {
    let mut s = String::new();
    s.push_str("# argmem 이 생성한 파일입니다. 직접 고치지 말고 `argmem add/remove` 를 쓰세요.\r\n");
    s.push_str("$global:ArgmemRules = @{\r\n");
    for r in rules {
        s.push_str(&format!(
            "    {} = {}\r\n",
            ps_quote(&r.trigger),
            ps_quote(&r.expansion())
        ));
    }
    s.push_str("}\r\n\r\n");

    // 별칭은 Enter 로 바로 실행해도 동작하도록 함수 + 별칭을 만든다.
    // (gl 같은 PowerShell 기본 별칭은 함수보다 우선하므로 Set-Alias -Force 로 덮어쓴다)
    for r in rules.iter().filter(|r| r.is_alias()) {
        s.push_str(&format!(
            "function global:argmem_{t} {{ {} {} @args }}\r\nSet-Alias -Name {t} -Value argmem_{t} -Scope Global -Option AllScope -Force\r\n",
            r.command,
            r.args,
            t = r.trigger
        ));
    }

    s.push_str(
        r#"
if (-not (Get-Module PSReadLine) -and $Host.Name -eq 'ConsoleHost') {
    Import-Module PSReadLine -ErrorAction SilentlyContinue
}
if (Get-Module PSReadLine) {
    # 줄 전체가 등록된 이름일 때 Space → 파라미터를 입력줄에 채운다 (Enter 전 확인 가능).
    Set-PSReadLineKeyHandler -Chord Spacebar -BriefDescription ArgmemExpand -Description 'argmem: 등록된 파라미터 자동 입력' -ScriptBlock {
        $line = $null; $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        $key = $line.Trim()
        if ($cursor -eq $line.Length -and $key -and $line -notmatch '\s$' -and $global:ArgmemRules.ContainsKey($key)) {
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $global:ArgmemRules[$key] + ' ')
        } else {
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert(' ')
        }
    }
    # Shift+Space 는 확장 없이 그냥 공백
    Set-PSReadLineKeyHandler -Chord Shift+Spacebar -BriefDescription ArgmemPlainSpace -ScriptBlock {
        [Microsoft.PowerShell.PSConsoleReadLine]::Insert(' ')
    }
}
"#,
    );
    s
}

pub fn render_doskey(rules: &[Rule]) -> String {
    rules
        .iter()
        .map(|r| format!("{}={} $*\r\n", r.trigger, r.expansion().replace('$', "$$")))
        .collect()
}

/// 두 파일을 다시 생성한다.
pub fn apply(store: &Store) -> io::Result<()> {
    fs::create_dir_all(home())?;
    // PowerShell 5.1 이 한글을 올바로 읽도록 UTF-8 BOM 을 붙인다.
    let mut ps = vec![0xEF, 0xBB, 0xBF];
    ps.extend_from_slice(render_ps1(&store.rules).as_bytes());
    fs::write(ps1_path(), ps)?;
    fs::write(doskey_path(), render_doskey(&store.rules))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(t: &str, c: &str, a: &str) -> Rule {
        Rule { trigger: t.into(), command: c.into(), args: a.into() }
    }

    #[test]
    fn doskey_escapes_dollar() {
        let out = render_doskey(&[rule("x", "echo", "$HOME")]);
        assert_eq!(out, "x=echo $$HOME $*\r\n");
    }

    #[test]
    fn ps1_quotes_and_alias_function() {
        let out = render_ps1(&[
            rule("claude", "claude", "--dangerously-skip-permissions"),
            rule("gl", "git", "log --format='%h %s'"),
        ]);
        assert!(out.contains("'claude' = 'claude --dangerously-skip-permissions'"));
        assert!(out.contains("'gl' = 'git log --format=''%h %s'''"));
        assert!(out.contains("function global:argmem_gl { git log --format='%h %s' @args }"));
        assert!(out.contains("Set-Alias -Name gl -Value argmem_gl"));
        assert!(!out.contains("argmem_claude"));
    }
}
