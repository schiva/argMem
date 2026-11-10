//! PSReadLine 히스토리에서 "같은 명령 + 같은 파라미터" 반복을 찾아 추천.

use crate::store::Store;
use std::{collections::HashMap, fs, path::PathBuf};

pub struct Suggestion {
    pub command: String,
    pub args: String,
    pub count: usize,
    /// 같은 명령을 다른 파라미터로 쓴 경우의 수 (1 이면 항상 이 파라미터로만 씀)
    pub variants: usize,
}

/// %ARGMEM_HISTORY% 또는 %APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\*_history.txt
fn history_files() -> Vec<PathBuf> {
    if let Some(p) = std::env::var_os("ARGMEM_HISTORY") {
        return vec![PathBuf::from(p)];
    }
    let Some(appdata) = std::env::var_os("APPDATA") else { return vec![] };
    let dir = PathBuf::from(appdata).join(r"Microsoft\Windows\PowerShell\PSReadLine");
    let Ok(rd) = fs::read_dir(dir) else { return vec![] };
    rd.filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.to_string_lossy().ends_with("_history.txt"))
        .collect()
}

pub fn parse_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() || line.ends_with('`') {
        return None;
    }
    let (cmd, rest) = line.split_once(char::is_whitespace)?;
    let rest = rest.trim();
    let bad_cmd = cmd.starts_with(['$', '.', '&', '(', '[', '#', '\'', '"']) || cmd.contains(['=', '|', ';']);
    if rest.is_empty() || bad_cmd {
        return None;
    }
    Some((cmd, rest))
}

pub fn suggest(store: &Store, min: usize) -> Vec<Suggestion> {
    let mut counts: HashMap<(String, String), (String, usize)> = HashMap::new();
    for f in history_files() {
        let Ok(bytes) = fs::read(&f) else { continue };
        let text = String::from_utf8_lossy(&bytes);
        for line in text.lines() {
            if let Some((cmd, args)) = parse_line(line) {
                let e = counts
                    .entry((cmd.to_lowercase(), args.to_string()))
                    .or_insert_with(|| (cmd.to_string(), 0));
                e.1 += 1;
            }
        }
    }
    let mut variants: HashMap<String, usize> = HashMap::new();
    for (cmd, _) in counts.keys() {
        *variants.entry(cmd.clone()).or_default() += 1;
    }
    let mut out: Vec<Suggestion> = counts
        .into_iter()
        .filter(|(_, (_, n))| *n >= min)
        .filter(|((cmd, args), _)| {
            !store
                .rules
                .iter()
                .any(|r| r.command.eq_ignore_ascii_case(cmd) && &r.args == args)
        })
        .map(|((key, args), (command, count))| Suggestion {
            command,
            args,
            count,
            variants: variants[&key],
        })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then(a.command.cmp(&b.command)));
    out
}

#[cfg(test)]
mod tests {
    use super::parse_line;

    #[test]
    fn parses() {
        assert_eq!(
            parse_line("  claude --dangerously-skip-permissions "),
            Some(("claude", "--dangerously-skip-permissions"))
        );
        assert_eq!(parse_line("ls"), None);
        assert_eq!(parse_line("$x = 1"), None);
        assert_eq!(parse_line("foo `"), None);
    }
}
