//! $PROFILE 와 cmd AutoRun 레지스트리에 argmem 연결/해제.

use crate::emit::{doskey_path, ps1_path};
use std::{fs, path::PathBuf, process::Command};

const MARKER: &str = "# argmem";
const DEFAULT_REG_KEY: &str = r"HKCU\Software\Microsoft\Command Processor";

// ---------- PowerShell 프로필 ----------

/// Windows PowerShell 과 (있으면) pwsh 의 CurrentUserAllHosts 프로필 경로.
/// 테스트용으로 %ARGMEM_PROFILES% (세미콜론 구분) 로 덮어쓸 수 있다.
pub fn profile_paths() -> Vec<PathBuf> {
    if let Ok(v) = std::env::var("ARGMEM_PROFILES") {
        return v.split(';').filter(|s| !s.is_empty()).map(PathBuf::from).collect();
    }
    let mut out = Vec::new();
    for exe in ["powershell", "pwsh"] {
        let res = Command::new(exe)
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", "$PROFILE.CurrentUserAllHosts"])
            .output();
        if let Ok(o) = res {
            let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if o.status.success() && !p.is_empty() {
                let p = PathBuf::from(p);
                if !out.contains(&p) {
                    out.push(p);
                }
            }
        }
    }
    out
}

fn profile_line() -> String {
    let p = ps1_path().display().to_string().replace('\'', "''");
    format!("if (Test-Path '{p}') {{ . '{p}' }} {MARKER}")
}

/// 반환: true = 새로 추가함, false = 이미 있음
pub fn add_to_profile(path: &PathBuf) -> std::io::Result<bool> {
    let mut bytes = fs::read(path).unwrap_or_default();
    if contains(&bytes, MARKER.as_bytes()) {
        return Ok(false);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    if bytes.is_empty() {
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    } else if !bytes.ends_with(b"\n") {
        bytes.extend_from_slice(b"\r\n");
    }
    bytes.extend_from_slice(profile_line().as_bytes());
    bytes.extend_from_slice(b"\r\n");
    fs::write(path, bytes)?;
    Ok(true)
}

/// 반환: true = 제거함
pub fn remove_from_profile(path: &PathBuf) -> std::io::Result<bool> {
    let Ok(bytes) = fs::read(path) else { return Ok(false) };
    if !contains(&bytes, MARKER.as_bytes()) {
        return Ok(false);
    }
    let kept: Vec<&[u8]> = bytes
        .split_inclusive(|&b| b == b'\n')
        .filter(|line| !contains(line, MARKER.as_bytes()))
        .collect();
    fs::write(path, kept.concat())?;
    Ok(true)
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

// ---------- cmd AutoRun ----------

fn reg_key() -> String {
    std::env::var("ARGMEM_REG_KEY").unwrap_or_else(|_| DEFAULT_REG_KEY.to_string())
}

fn autorun_segment() -> String {
    let p = doskey_path().display().to_string();
    if p.contains(' ') {
        format!("doskey /macrofile=\"{p}\"")
    } else {
        format!("doskey /macrofile={p}")
    }
}

pub fn get_autorun() -> Option<String> {
    let o = Command::new("reg")
        .args(["query", &reg_key(), "/v", "AutoRun"])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&o.stdout);
    for line in text.lines() {
        let t = line.trim_start();
        if !t.starts_with("AutoRun") {
            continue;
        }
        for ty in ["REG_EXPAND_SZ", "REG_SZ"] {
            if let Some(i) = t.find(ty) {
                return Some(t[i + ty.len()..].trim().to_string());
            }
        }
    }
    None
}

fn set_autorun(value: &str) -> Result<(), String> {
    let st = Command::new("reg")
        .args(["add", &reg_key(), "/v", "AutoRun", "/t", "REG_SZ", "/d", value, "/f"])
        .output()
        .map_err(|e| e.to_string())?;
    if st.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&st.stderr).into()) }
}

fn delete_autorun() -> Result<(), String> {
    let st = Command::new("reg")
        .args(["delete", &reg_key(), "/v", "AutoRun", "/f"])
        .output()
        .map_err(|e| e.to_string())?;
    if st.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&st.stderr).into()) }
}

/// 반환: true = 새로 등록함
pub fn add_autorun() -> Result<bool, String> {
    let seg = autorun_segment();
    let cur = get_autorun().unwrap_or_default();
    if cur.contains("argmem.doskey") {
        return Ok(false);
    }
    let new = if cur.trim().is_empty() { seg } else { format!("{cur} & {seg}") };
    set_autorun(&new)?;
    Ok(true)
}

/// 반환: true = 제거함
pub fn remove_autorun() -> Result<bool, String> {
    let Some(cur) = get_autorun() else { return Ok(false) };
    let seg = autorun_segment();
    if !cur.contains(&seg) {
        return Ok(false);
    }
    let new = cur
        .replace(&format!(" & {seg}"), "")
        .replace(&format!("{seg} & "), "")
        .replace(&seg, "");
    if new.trim().is_empty() { delete_autorun()? } else { set_autorun(new.trim())? }
    Ok(true)
}
