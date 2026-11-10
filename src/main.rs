mod emit;
mod install;
mod learn;
mod store;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};
use std::ffi::OsString;
use store::{join_args, validate_name, Rule, Store};

const HELP_TEMPLATE: &str = "{about-with-newline}\n사용법: {usage}\n\n{all-args}{after-help}";
const ADD_USAGE: &str = "사용법: argmem add [--as <별칭>] <명령> <인자>...\n자세히: argmem add --help";

#[derive(Parser)]
#[command(
    name = "argmem",
    bin_name = "argmem",
    version,
    about = "자주 쓰는 명령 파라미터를 기억해 PowerShell / cmd 에서 자동 입력합니다",
    after_help = "예:\n  argmem add claude --dangerously-skip-permissions\n  argmem add --as gl git log --oneline --graph -20\n  argmem install\n\n각 명령의 자세한 사용법: argmem <명령> --help",
    subcommand_help_heading = "명령",
    subcommand_value_name = "명령",
    disable_help_subcommand = true,
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    /// 도움말 보기
    #[arg(short = 'h', long, action = ArgAction::Help, global = true)]
    help: Option<bool>,
    /// 버전 보기
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,
}

#[derive(Subcommand)]
enum Cmd {
    /// 규칙 추가/수정 (--as 를 주면 별칭, 안 주면 해당 명령 + Space 로 확장)
    #[command(
        override_usage = "argmem add [--as <별칭>] <명령> <인자>...",
        after_help = "예:\n  argmem add claude --dangerously-skip-permissions\n  argmem add --as gl git log --oneline --graph -20"
    )]
    Add {
        /// 별칭 이름
        #[arg(long = "as", value_name = "별칭")]
        alias: Option<String>,
        /// 실행할 명령 (예: claude)
        #[arg(value_name = "명령")]
        command: String,
        /// 기억할 파라미터
        #[arg(value_name = "인자", required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// 등록된 규칙 보기
    #[command(alias = "ls")]
    List,
    /// 규칙 삭제
    #[command(
        alias = "rm",
        override_usage = "argmem remove <이름>",
        after_help = "예:\n  argmem remove claude\n\n등록된 이름 확인: argmem list"
    )]
    Remove {
        /// 삭제할 규칙 이름 (확장 규칙은 명령 이름, 별칭 규칙은 별칭)
        #[arg(value_name = "이름")]
        trigger: String,
    },
    /// 셸 설정 파일(argmem.ps1 / argmem.doskey) 다시 생성
    Apply,
    /// PowerShell 프로필과 cmd AutoRun 에 연결
    Install,
    /// 연결 해제 (규칙 파일은 남겨 둠)
    Uninstall,
    /// PowerShell 히스토리에서 자주 쓰는 파라미터 추천
    #[command(
        override_usage = "argmem learn [--top <개수>] [--min <횟수>] [--pick <번호> [--as <별칭>]]",
        after_help = "예:\n  argmem learn\n  argmem learn --min 5 --top 20\n  argmem learn --pick 2 --as gs"
    )]
    Learn {
        /// 보여 줄 개수 (기본 10)
        #[arg(long, value_name = "개수", default_value_t = 10, hide_default_value = true)]
        top: usize,
        /// 최소 반복 횟수 (기본 3)
        #[arg(long, value_name = "횟수", default_value_t = 3, hide_default_value = true)]
        min: usize,
        /// 추천 목록의 해당 번호를 바로 등록
        #[arg(long, value_name = "번호")]
        pick: Option<usize>,
        /// --pick 과 함께: 별칭으로 등록
        #[arg(long = "as", value_name = "별칭", requires = "pick")]
        alias: Option<String>,
    },
    /// 설정 보기/변경
    #[command(
        override_usage = "argmem config [<설정> <on|off>]",
        after_help = "설정:\n  enter  PowerShell 에서 등록 이름만 입력하고 Enter 를 눌러도 확장해서 실행 (기본 off)\n\n예:\n  argmem config\n  argmem config enter on\n  argmem config enter off"
    )]
    Config {
        /// 설정 이름: enter
        #[arg(value_name = "설정", value_parser = ["enter"], hide_possible_values = true, requires = "value")]
        key: Option<String>,
        /// on 또는 off
        #[arg(value_name = "값", value_parser = ["on", "off"], hide_possible_values = true)]
        value: Option<String>,
    },
    /// 상태 확인 (파일 위치, 연결 여부)
    Status,
}

fn print_settings(store: &Store) {
    let on = |b: bool| if b { "on" } else { "off" };
    println!(
        "enter = {:<3}  PowerShell 에서 등록 이름만 입력하고 Enter 를 눌러도 확장해서 실행",
        on(store.settings.expand_on_enter)
    );
}

/// 도움말 제목·서식을 한국어로 맞춘 clap Command.
fn build_cli() -> clap::Command {
    fn localize(c: clap::Command) -> clap::Command {
        // 인자가 없는 명령은 "[OPTIONS]" 대신 깔끔한 사용법을 보여 준다.
        let c = if c.get_arguments().next().is_none() {
            let usage = format!("argmem {}", c.get_name());
            c.override_usage(usage)
        } else {
            c
        };
        let mut order = 0;
        c.help_template(HELP_TEMPLATE).mut_args(|a| {
            order += 1;
            let heading = if a.is_positional() { "인자" } else { "옵션" };
            // -h / -V 는 항상 맨 아래
            let o = match a.get_id().as_str() {
                "help" => 1000,
                "version" => 1001,
                _ => order,
            };
            a.help_heading(heading).display_order(o)
        })
    }
    localize(Cli::command()).mut_subcommands(localize)
}

fn parse(args: &[OsString]) -> Result<Cli, clap::Error> {
    let m = build_cli().try_get_matches_from(args)?;
    Cli::from_arg_matches(&m)
}

/// 파라미터를 잘못 입력했을 때: 한국어 오류 + 제안 + 해당 명령의 전체 사용법.
fn usage_error(e: &clap::Error, args: &[OsString]) -> String {
    let get = |k: ContextKind| match e.get(k) {
        Some(ContextValue::String(s)) => s.clone(),
        Some(ContextValue::Strings(v)) => v.join(", "),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    let mut msg = match e.kind() {
        ErrorKind::InvalidSubcommand => {
            format!("알 수 없는 명령입니다: '{}'", get(ContextKind::InvalidSubcommand))
        }
        ErrorKind::UnknownArgument => {
            format!("알 수 없는 옵션입니다: '{}'", get(ContextKind::InvalidArg))
        }
        ErrorKind::MissingRequiredArgument => {
            format!("필요한 값이 빠졌습니다: {}", get(ContextKind::InvalidArg))
        }
        ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            "명령을 입력하세요.".to_string()
        }
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => format!(
            "{} 에 잘못된 값입니다: '{}'",
            get(ContextKind::InvalidArg),
            get(ContextKind::InvalidValue)
        ),
        ErrorKind::ArgumentConflict => {
            let (a, b) = (get(ContextKind::InvalidArg), get(ContextKind::PriorArg));
            if b.is_empty() || a == b {
                format!("옵션을 두 번 이상 입력했습니다: {a}")
            } else {
                format!("함께 쓸 수 없는 옵션입니다: {a}, {b}")
            }
        }
        ErrorKind::TooManyValues => {
            format!("값이 너무 많습니다: {}", get(ContextKind::InvalidArg))
        }
        _ => e
            .render()
            .to_string()
            .lines()
            .next()
            .unwrap_or_default()
            .trim_start_matches("error: ")
            .to_string(),
    };
    for k in [ContextKind::SuggestedSubcommand, ContextKind::SuggestedArg] {
        let s = get(k);
        if !s.is_empty() {
            msg.push_str(&format!("\n  혹시 '{s}' 을(를) 입력하려던 건가요?"));
        }
    }

    // 명령 이름이 맞으면 그 명령의 사용법을, 아니면 전체 사용법을 보여 준다.
    let mut cmd = build_cli();
    cmd.build();
    let sub = args
        .iter()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .find(|a| !a.starts_with('-'))
        .filter(|s| cmd.find_subcommand(s).is_some());
    let help = match sub {
        Some(s) => cmd.find_subcommand_mut(&s).expect("checked above").render_help(),
        None => cmd.render_help(),
    };
    format!("오류: {msg}\n\n{help}")
}

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    let cli = match parse(&args) {
        Ok(c) => c,
        Err(e) if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) => e.exit(),
        Err(e) => {
            eprint!("{}", usage_error(&e, &args));
            std::process::exit(2);
        }
    };
    if let Err(e) = run(cli) {
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
            let trigger = alias.unwrap_or_else(|| command.clone());
            validate_name("명령", &command)
                .and_then(|_| validate_name("별칭", &trigger))
                .map_err(|e| format!("{e}\n\n{ADD_USAGE}"))?;
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
                let names: Vec<&str> = store.rules.iter().map(|r| r.trigger.as_str()).collect();
                let hint = if names.is_empty() {
                    "등록된 규칙이 없습니다.".to_string()
                } else {
                    format!("등록된 규칙: {}", names.join(", "))
                };
                return Err(format!("'{trigger}' 규칙이 없습니다.\n  {hint}"));
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
                let s = list.get(n.wrapping_sub(1)).ok_or_else(|| {
                    if list.is_empty() {
                        format!("{n} 번 추천이 없습니다. {min}회 이상 반복된 명령+파라미터가 없습니다.")
                    } else {
                        format!("{n} 번 추천이 없습니다. 1 ~ {} 번 중에서 고르세요 (목록: argmem learn).", list.len())
                    }
                })?;
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
                let note = if s.variants > 1 { "   (다른 파라미터로도 사용 → 별칭 권장)" } else { "" };
                println!("{:>2}. ({:>3}회) {} {}{note}", i + 1, s.count, s.command, s.args);
            }
            if !list.is_empty() {
                println!("\n등록: argmem learn --pick <번호> [--as <별칭>]");
            }
        }
        Cmd::Config { key, value } => match (key.as_deref(), value.as_deref()) {
            (Some("enter"), Some(v)) => {
                store.settings.expand_on_enter = v == "on";
                print_settings(&store);
                save_and_apply(&store)?;
            }
            _ => print_settings(&store),
        },
        Cmd::Status => {
            println!("규칙 파일 : {} ({}개)", store::rules_path().display(), store.rules.len());
            println!(
                "Enter 확장: {}",
                if store.settings.expand_on_enter { "on" } else { "off (argmem config enter on 으로 켜기)" }
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn err_text(args: &[&str]) -> String {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        let e = parse(&args).err().expect("should fail to parse");
        usage_error(&e, &args)
    }

    #[test]
    fn cli_is_consistent() {
        build_cli().debug_assert();
    }

    #[test]
    fn valid_add_keeps_hyphen_args() {
        let args: Vec<OsString> =
            ["argmem", "add", "claude", "--dangerously-skip-permissions"].iter().map(OsString::from).collect();
        match parse(&args).unwrap().cmd {
            Cmd::Add { command, args, .. } => {
                assert_eq!(command, "claude");
                assert_eq!(args, ["--dangerously-skip-permissions"]);
            }
            _ => panic!("expected add"),
        }
    }

    #[test]
    fn unknown_subcommand_suggests_and_shows_top_help() {
        let t = err_text(&["argmem", "ad"]);
        assert!(t.starts_with("오류: 알 수 없는 명령입니다: 'ad'"), "{t}");
        assert!(t.contains("혹시 'add'"), "{t}");
        assert!(t.contains("사용법: argmem <명령>"), "{t}");
        assert!(t.contains("명령:") && t.contains("learn"), "{t}");
    }

    #[test]
    fn missing_args_shows_subcommand_help() {
        let t = err_text(&["argmem", "add", "claude"]);
        assert!(t.contains("필요한 값이 빠졌습니다: <인자>..."), "{t}");
        assert!(t.contains("사용법: argmem add [--as <별칭>] <명령> <인자>..."), "{t}");
        assert!(t.find("<명령>     ").unwrap() < t.find("<인자>...  ").unwrap(), "{t}");
        assert!(t.contains("argmem add --as gl git log"), "{t}");
    }

    #[test]
    fn unknown_option_shows_subcommand_help() {
        let t = err_text(&["argmem", "learn", "--bogus"]);
        assert!(t.contains("알 수 없는 옵션입니다: '--bogus'"), "{t}");
        assert!(t.contains("사용법: argmem learn"), "{t}");
        assert!(t.contains("--pick <번호>"), "{t}");
    }

    #[test]
    fn similar_option_is_suggested() {
        let t = err_text(&["argmem", "learn", "--pik", "1"]);
        assert!(t.contains("혹시 '--pick'"), "{t}");
    }

    #[test]
    fn bad_value_is_reported() {
        let t = err_text(&["argmem", "learn", "--top", "abc"]);
        assert!(t.contains("잘못된 값입니다: 'abc'"), "{t}");
        assert!(t.contains("--top <개수>"), "{t}");
    }

    #[test]
    fn repeated_option() {
        let t = err_text(&["argmem", "learn", "--top", "1", "--top", "2"]);
        assert!(t.contains("두 번 이상 입력했습니다: --top"), "{t}");
    }

    #[test]
    fn config_parses_and_validates() {
        let args: Vec<OsString> = ["argmem", "config", "enter", "on"].iter().map(OsString::from).collect();
        assert!(matches!(parse(&args).unwrap().cmd, Cmd::Config { key: Some(_), value: Some(_) }));
        let t = err_text(&["argmem", "config", "enter", "yes"]);
        assert!(t.contains("잘못된 값입니다: 'yes'"), "{t}");
        assert!(t.contains("사용법: argmem config"), "{t}");
        let t = err_text(&["argmem", "config", "enter"]);
        assert!(t.contains("필요한 값이 빠졌습니다"), "{t}");
    }

    #[test]
    fn missing_subcommand() {
        let t = err_text(&["argmem"]);
        assert!(t.starts_with("오류: 명령을 입력하세요."), "{t}");
    }

    #[test]
    fn alias_subcommand_help() {
        let t = err_text(&["argmem", "rm"]);
        assert!(t.contains("사용법: argmem remove <이름>"), "{t}");
    }

    #[test]
    fn no_english_leftovers_in_help() {
        let mut cmd = build_cli();
        cmd.build();
        let mut all = cmd.render_help().to_string();
        for sc in cmd.get_subcommands_mut() {
            all.push_str(&sc.render_help().to_string());
        }
        for word in ["OPTIONS", "Usage", "Options", "Arguments", "Commands", "default:", "Print"] {
            assert!(!all.contains(word), "'{word}' in help:\n{all}");
        }
        assert!(all.contains("사용법: argmem list\n"), "{all}");
    }
}
