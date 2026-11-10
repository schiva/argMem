use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

/// 규칙 하나.
/// - trigger == command  → "확장" 규칙: `claude` + Space 를 누르면 파라미터가 붙는다.
/// - trigger != command  → "별칭" 규칙: `ccy` 가 `claude --...` 로 바뀐다.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Rule {
    pub trigger: String,
    pub command: String,
    pub args: String,
}

impl Rule {
    pub fn is_alias(&self) -> bool {
        !self.trigger.eq_ignore_ascii_case(&self.command)
    }

    pub fn expansion(&self) -> String {
        format!("{} {}", self.command, self.args)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Store {
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// 데이터 폴더: %ARGMEM_HOME% 또는 %USERPROFILE%\.argmem
pub fn home() -> PathBuf {
    if let Some(h) = std::env::var_os("ARGMEM_HOME") {
        return PathBuf::from(h);
    }
    let base = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".argmem")
}

pub fn rules_path() -> PathBuf {
    home().join("rules.json")
}

impl Store {
    pub fn load() -> io::Result<Store> {
        let p = rules_path();
        if !p.exists() {
            return Ok(Store::default());
        }
        let text = fs::read_to_string(&p)?;
        let text = text.trim_start_matches('\u{feff}');
        serde_json::from_str(text).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("{}: {e}", p.display()))
        })
    }

    pub fn save(&self) -> io::Result<()> {
        fs::create_dir_all(home())?;
        let json = serde_json::to_string_pretty(self).expect("serialize");
        fs::write(rules_path(), json + "\n")
    }

    /// 같은 trigger 가 있으면 교체하고 true 를 돌려준다.
    pub fn upsert(&mut self, rule: Rule) -> bool {
        if let Some(r) = self
            .rules
            .iter_mut()
            .find(|r| r.trigger.eq_ignore_ascii_case(&rule.trigger))
        {
            *r = rule;
            true
        } else {
            self.rules.push(rule);
            self.rules
                .sort_by(|a, b| a.trigger.to_lowercase().cmp(&b.trigger.to_lowercase()));
            false
        }
    }

    pub fn remove(&mut self, trigger: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| !r.trigger.eq_ignore_ascii_case(trigger));
        before != self.rules.len()
    }
}

/// trigger / command 이름 검사: 공백·doskey 특수문자 금지.
pub fn validate_name(kind: &str, name: &str) -> Result<(), String> {
    let ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'));
    if ok {
        Ok(())
    } else {
        Err(format!(
            "{kind} '{name}' 은(는) 사용할 수 없습니다 (영문/숫자/. _ - + 만 허용)"
        ))
    }
}

/// OS 가 나눠 준 인자들을 다시 한 줄로 합친다. 공백이 있으면 큰따옴표로 감싼다.
pub fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() || a.chars().any(char::is_whitespace) {
                format!("\"{a}\"")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
