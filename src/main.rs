mod emit;
mod install;
mod learn;
mod store;

use clap::{Parser, Subcommand};
use store::{join_args, validate_name, Rule, Store};

#[derive(Parser)]
#[command(
    name = "argmem",
    version,
    about = "자주 쓰는 명령 파라미터를 기억해 PowerShell / cmd 에서 자동 입력합니다",
    after_help = "예:\n  argmem add claude --dangerously-skip-permissions\n  argmem add --as gl git log --oneline --graph -20\n  argmem install"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 규칙 추가/수정 (--as 를 주면 별칭, 안 주면 해당 명령 + Space 로 확장)
    Add {
        /// 별칭 이름 (명령 앞에 써야 함)
        #[arg(long = "as", value_name = "ALIAS")]
        alias: Option<String>,
        /// 실행할 명령 (예: claude)
        command: String,
        /// 기억할 파라미터
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// 등록된 규칙 보기
    #[command(alias = "ls")]
    List,
    /// 규칙 삭제
    #[command(alias = "rm")]
    Remove { trigger: String },
    /// 셸 설정 파일(argmem.ps1 / argmem.doskey) 다시 생성
    Apply,
    /// PowerShell 프로필과 cmd AutoRun 에 연결
    Install,
    /// 연결 해제 (규칙 파일은 남겨 둠)
    Uninstall,
    /// PowerShell 히스토리에서 자주 쓰는 파라미터 추천
    Learn {
        /// 보여 줄 개수
        #[arg(long, default_value_t = 10)]
        top: usize,
        /// 최소 반복 횟수
        #[arg(long, default_value_t = 3)]
        min: usize,
        /// 추천 목록의 N 번을 바로 등록
        #[arg(long, value_name = "N")]
        pick: Option<usize>,
        /// --pick 과 함께: 별칭으로 등록
        #[arg(long = "as", value_name = "ALIAS", requires = "pick")]
        alias: Option<String>,
    },
    /// 상태 확인 (파일 위치, 연결 여부)
    Status,
}

fn main() {
    if let Err(e) = run(Cli::parse()) {
        eprintln!("오류: {e}");
        std::process::exit(1);
    }
}

fn save_and_apply(store: &Store) -> Result<(), String> {
    store.save().map_err(|e| e.to_string())?;
    emit::apply(store).map_err(|e| e.to_string())?;
    println!(
        "  → 새 창부터 적용됩니다. 현재 PowerShell 창에 바로 적용: . '{}'",
        emit::ps1_path().display()
    );
    Ok(())
}

fn add_rule(store: &mut Store, rule: Rule) -> Result<(), String> {
    let kind = if rule.is_alias() { "별칭" } else { "확장" };
    let line = format!("[{kind}] {}  →  {}", rule.trigger, rule.expansion());
    let replaced = store.upsert(rule);
    println!("{} {line}", if replaced { "수정:" } else { "추가:" });
    save_and_apply(store)
}

fn run(cli: Cli) -> Result<(), String> {
    let mut store = Store::load().map_err(|e| e.to_string())?;
    match cli.cmd {
        Cmd::Add { alias, command, args } => {
            validate_name("명령", &command)?;
            let trigger = alias.unwrap_or_else(|| command.clone());
            validate_name("별칭", &trigger)?;
            add_rule(&mut store, Rule { trigger, command, args: join_args(&args) })?;
        }
        Cmd::List => {
            if store.rules.is_empty() {
                println!("등록된 규칙이 없습니다. 예: argmem add claude --dangerously-skip-permissions");
            }
            let w = store.rules.iter().map(|r| r.trigger.chars().count()).max().unwrap_or(0);
            for r in &store.rules {
                let kind = if r.is_alias() { "별칭" } else { "확장" };
                println!("[{kind}] {:<w$}  →  {}", r.trigger, r.expansion());
            }
        }
        Cmd::Remove { trigger } => {
            if !store.remove(&trigger) {
                return Err(format!("'{trigger}' 규칙이 없습니다"));
            }
            println!("삭제: {trigger}");
            save_and_apply(&store)?;
        }
        Cmd::Apply => {
            emit::apply(&store).map_err(|e| e.to_string())?;
            println!("생성: {}", emit::ps1_path().display());
            println!("생성: {}", emit::doskey_path().display());
        }
        Cmd::Install => {
            emit::apply(&store).map_err(|e| e.to_string())?;
            let profiles = install::profile_paths();
            if profiles.is_empty() {
                println!("PowerShell 프로필 경로를 찾지 못했습니다.");
            }
            for p in profiles {
                let added = install::add_to_profile(&p).map_err(|e| format!("{}: {e}", p.display()))?;
                println!("PowerShell 프로필 {}: {}", if added { "연결" } else { "이미 연결됨" }, p.display());
            }
            let added = install::add_autorun()?;
            println!("cmd AutoRun {}", if added { "등록" } else { "이미 등록됨" });
            println!("완료. 새 PowerShell / cmd 창을 열어 사용하세요.");
        }
        Cmd::Uninstall => {
            for p in install::profile_paths() {
                if install::remove_from_profile(&p).map_err(|e| format!("{}: {e}", p.display()))? {
                    println!("PowerShell 프로필 연결 해제: {}", p.display());
                }
            }
            if install::remove_autorun()? {
                println!("cmd AutoRun 해제");
            }
            println!("완료. 규칙 파일은 남아 있습니다: {}", store::rules_path().display());
        }
        Cmd::Learn { top, min, pick, alias } => {
            let list = learn::suggest(&store, min);
            if let Some(n) = pick {
                let s = list
                    .get(n.wrapping_sub(1))
                    .ok_or_else(|| format!("{n} 번 추천이 없습니다"))?;
                validate_name("명령", &s.command)?;
                if alias.is_none() && s.variants > 1 {
                    return Err(format!(
                        "'{}' 는 다른 파라미터로도 {}가지 방식으로 쓰고 있어, 확장 규칙으로 등록하면 \
                         매번 '{}' 가 붙습니다.\n  별칭으로 등록하세요: argmem learn --pick {n} --as <별칭>",
                        s.command,
                        s.variants - 1,
                        s.args
                    ));
                }
                let trigger = alias.unwrap_or_else(|| s.command.clone());
                validate_name("별칭", &trigger)?;
                let rule = Rule { trigger, command: s.command.clone(), args: s.args.clone() };
                return add_rule(&mut store, rule);
            }
            if list.is_empty() {
                println!("{min}회 이상 반복된 명령+파라미터가 없습니다.");
            }
            for (i, s) in list.iter().take(top).enumerate() {
                let note = if s.variants > 1 {
                    format!("   (다른 파라미터로도 사용 → 별칭 권장)")
                } else {
                    String::new()
                };
                println!("{:>2}. ({:>3}회) {} {}{note}", i + 1, s.count, s.command, s.args);
            }
            if !list.is_empty() {
                println!("\n등록: argmem learn --pick <번호> [--as <별칭>]");
            }
        }
        Cmd::Status => {
            println!("규칙 파일 : {} ({}개)", store::rules_path().display(), store.rules.len());
            println!("PowerShell: {}", emit::ps1_path().display());
            println!("cmd       : {}", emit::doskey_path().display());
            for p in install::profile_paths() {
                let on = std::fs::read(&p)
                    .map(|b| String::from_utf8_lossy(&b).contains("# argmem"))
                    .unwrap_or(false);
                println!("프로필    : {} [{}]", p.display(), if on { "연결됨" } else { "미연결" });
            }
            let on = install::get_autorun().is_some_and(|v| v.contains("argmem.doskey"));
            println!("cmd AutoRun: {}", if on { "등록됨" } else { "미등록" });
        }
    }
    Ok(())
}
