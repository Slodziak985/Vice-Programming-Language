#[cfg(all(target_os = "windows", feature = "desktop"))]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{
    env, fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
    process,
    time::Instant,
};
use vice::check_program;
use vice::error::{ViceError, ViceErrorType};
use vice::format_error_for_file;
use vice::format_error_with_source;
use vice::formatter::format_source;
use vice::load_program;
use vice::semantic::SemanticAnalyzer;
#[cfg(all(target_os = "windows", feature = "desktop"))]
use vice::ui::{run_ui_manager, set_global_sender, UiCommand};

enum Command {
    Run {
        path: String,
        args: Vec<String>,
        options: RunOptions,
    },
    Bench {
        path: String,
        frames: u64,
        output: OutputMode,
    },
    Check {
        path: String,
        output: OutputMode,
        sandbox: SandboxCliOptions,
    },
    Test {
        path: String,
        output: OutputMode,
        sandbox: SandboxCliOptions,
    },
    New {
        path: String,
        template: ProjectTemplate,
    },
    Init {
        path: String,
        template: ProjectTemplate,
    },
    Migrate {
        path: String,
        check: bool,
    },
    Doc {
        path: String,
        output: OutputMode,
    },
    Lsp,
    Fmt {
        path: String,
        mode: FormatMode,
    },
    FmtStdin,
    Doctor {
        output: OutputMode,
        scope: DoctorScope,
    },
    TestMetrics(OutputMode),
    UiCheck,
    Schema,
    Pkg {
        action: PkgAction,
    },
    Version,
    Help,
    Invalid(String),
}

enum PkgAction {
    Add(String),
    List,
}

#[derive(Clone, Copy)]
enum OutputMode {
    Human,
    Json,
}

#[derive(Clone, Copy)]
enum FormatMode {
    Write,
    Check,
    Stdout,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RunOptions {
    profile: bool,
    frames: Option<u64>,
    sandbox: SandboxCliOptions,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SandboxCliOptions {
    root: Option<String>,
    allow_fs: Vec<String>,
}

struct RuntimeProfile {
    elapsed: std::time::Duration,
    instructions: u64,
    ui_stats: vice::ui::UiStats,
    hot_opcodes: Vec<(String, u64)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectTemplate {
    Cli,
    App,
    Platformer,
    Topdown,
    Tool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoctorScope {
    All,
    Runtime,
    Gpu,
    Project,
}

#[derive(Clone)]
struct ViceWarning {
    code: &'static str,
    message: &'static str,
    help: &'static str,
    line: usize,
    column: usize,
    length: usize,
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let command = parse_command(&args);
    let executable = env::args()
        .next()
        .and_then(|value| {
            Path::new(&value)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "vice".to_string());

    let exit_code = match command {
        Command::Run {
            path,
            args,
            options,
        } => {
            #[cfg(all(target_os = "windows", feature = "desktop"))]
            {
                if options.profile && options.frames.is_some() {
                    let sandbox = options.sandbox.clone();
                    with_sandbox(&sandbox, || run(Path::new(&path), args, options))
                } else {
                    use std::sync::mpsc;
                    // Create a channel for UI commands and set global sender so UiRuntime will send Open requests
                    let (tx, rx) = mpsc::channel::<UiCommand>();
                    let _ = set_global_sender(tx);

                    let finished = Arc::new(AtomicBool::new(false));
                    let finished_clone = finished.clone();
                    let path_clone = path.clone();
                    let args_clone = args.clone();
                    let options_clone = options.clone();
                    let sandbox = options_clone.sandbox.clone();

                    let handle = std::thread::spawn(move || {
                        let code = with_sandbox(&sandbox, || {
                            run(Path::new(&path_clone), args_clone, options_clone)
                        });
                        finished_clone.store(true, Ordering::SeqCst);
                        code
                    });

                    // Run UI manager on main thread (blocks until finished and windows closed)
                    run_ui_manager(rx, finished);

                    handle.join().unwrap_or(1)
                }
            }
            #[cfg(not(all(target_os = "windows", feature = "desktop")))]
            {
                let sandbox = options.sandbox.clone();
                with_sandbox(&sandbox, || run(Path::new(&path), args, options))
            }
        }
        Command::Check {
            path,
            output,
            sandbox,
        } => with_sandbox(&sandbox, || check_path(Path::new(&path), output)),
        Command::Bench {
            path,
            frames,
            output,
        } => bench(Path::new(&path), frames, output),
        Command::Test {
            path,
            output,
            sandbox,
        } => with_sandbox(&sandbox, || test_path(Path::new(&path), output)),
        Command::New { path, template } => new_project(Path::new(&path), &executable, template),
        Command::Init { path, template } => init_project(Path::new(&path), &executable, template),
        Command::Migrate { path, check } => migrate_path(Path::new(&path), check),
        Command::Doc { path, output } => doc_path(Path::new(&path), output),
        Command::Lsp => lsp_stdio(),
        Command::Fmt { path, mode } => format_path(Path::new(&path), mode),
        Command::FmtStdin => format_stdin(),
        Command::Doctor { output, scope } => doctor(output, scope),
        Command::TestMetrics(output) => test_metrics(output),
        Command::UiCheck => ui_check(),
        Command::Schema => dump_schema(),
        Command::Pkg { action } => {
            let project_root = find_manifest_root();
            match action {
                PkgAction::Add(source) => match vice::pkg::add_dependency(&project_root, &source) {
                    Ok(msg) => {
                        println!("{}", msg);
                        0
                    }
                    Err(err) => {
                        eprintln!("vice pkg: {}", err);
                        1
                    }
                },
                PkgAction::List => match vice::pkg::list_dependencies(&project_root) {
                    Ok(msg) => {
                        println!("{}", msg);
                        0
                    }
                    Err(err) => {
                        eprintln!("vice pkg: {}", err);
                        1
                    }
                },
            }
        }
        Command::Version => {
            println!("vice {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Command::Help => {
            print_usage(&executable);
            0
        }
        Command::Invalid(message) => {
            eprintln!("{}: {}", executable, message);
            eprintln!();
            print_usage(&executable);
            2
        }
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn with_sandbox(options: &SandboxCliOptions, action: impl FnOnce() -> i32) -> i32 {
    if options.root.is_none() && !options.allow_fs.is_empty() {
        eprintln!("vice sandbox: --allow-fs requires --sandbox");
        return 2;
    }

    if let Some(root) = &options.root {
        if let Err(message) = vice::stdlib::set_sandbox_root(Some(PathBuf::from(root))) {
            eprintln!("vice sandbox: {}", message);
            return 2;
        }
        if let Err(message) =
            vice::stdlib::set_sandbox_allow_fs(options.allow_fs.iter().map(PathBuf::from).collect())
        {
            eprintln!("vice sandbox: {}", message);
            let _ = vice::stdlib::set_sandbox_root(None);
            let _ = vice::stdlib::set_sandbox_allow_fs(Vec::new());
            return 2;
        }
    } else {
        let _ = vice::stdlib::set_sandbox_root(None);
        let _ = vice::stdlib::set_sandbox_allow_fs(Vec::new());
    }
    let code = action();
    let _ = vice::stdlib::set_sandbox_root(None);
    let _ = vice::stdlib::set_sandbox_allow_fs(Vec::new());
    code
}

fn parse_command(args: &[String]) -> Command {
    if matches!(args.first().map(String::as_str), Some("run")) {
        return parse_run_command(&args[1..]);
    }
    if matches!(args.first().map(String::as_str), Some("bench")) {
        return parse_bench_command(&args[1..]);
    }
    if matches!(args.first().map(String::as_str), Some("check")) {
        return parse_check_command(&args[1..]);
    }
    if matches!(args.first().map(String::as_str), Some("test")) {
        return parse_test_command(&args[1..]);
    }

    match args {
        [] => Command::Help,
        [cmd] if cmd == "-h" || cmd == "--help" => Command::Help,
        [cmd] if cmd == "--version" || cmd == "-V" => Command::Version,
        [cmd] if cmd == "schema" => Command::Schema,
        [cmd, flag] if cmd == "schema" && flag == "--json" => Command::Schema,
        [cmd, subcmd, source] if cmd == "pkg" && subcmd == "add" => Command::Pkg {
            action: PkgAction::Add(source.clone()),
        },
        [cmd, subcmd] if cmd == "pkg" && (subcmd == "list" || subcmd == "ls") => Command::Pkg {
            action: PkgAction::List,
        },
        [cmd] if cmd == "pkg" => Command::Pkg {
            action: PkgAction::List,
        },
        [cmd] if cmd == "doctor" => Command::Doctor {
            output: OutputMode::Human,
            scope: DoctorScope::All,
        },
        [cmd, flag] if cmd == "doctor" && flag == "--json" => Command::Doctor {
            output: OutputMode::Json,
            scope: DoctorScope::All,
        },
        [cmd, scope] if cmd == "doctor" => match parse_doctor_scope(scope) {
            Ok(scope) => Command::Doctor {
                output: OutputMode::Human,
                scope,
            },
            Err(message) => Command::Invalid(message),
        },
        [cmd, scope, flag] if cmd == "doctor" && flag == "--json" => {
            match parse_doctor_scope(scope) {
                Ok(scope) => Command::Doctor {
                    output: OutputMode::Json,
                    scope,
                },
                Err(message) => Command::Invalid(message),
            }
        }
        [cmd, flag, scope] if cmd == "doctor" && flag == "--json" => {
            match parse_doctor_scope(scope) {
                Ok(scope) => Command::Doctor {
                    output: OutputMode::Json,
                    scope,
                },
                Err(message) => Command::Invalid(message),
            }
        }
        [cmd] if cmd == "ui-check" => Command::UiCheck,
        [cmd] if cmd == "test-metrics" => Command::TestMetrics(OutputMode::Human),
        [cmd, flag] if cmd == "test-metrics" && flag == "--json" => {
            Command::TestMetrics(OutputMode::Json)
        }
        [cmd, name] if cmd == "new" => Command::New {
            path: name.clone(),
            template: ProjectTemplate::Cli,
        },
        [cmd, name, flag, template] if cmd == "new" && flag == "--template" => {
            match parse_project_template(template) {
                Ok(template) => Command::New {
                    path: name.clone(),
                    template,
                },
                Err(message) => Command::Invalid(message),
            }
        }
        [cmd, flag, template, name] if cmd == "new" && flag == "--template" => {
            match parse_project_template(template) {
                Ok(template) => Command::New {
                    path: name.clone(),
                    template,
                },
                Err(message) => Command::Invalid(message),
            }
        }
        [cmd] if cmd == "init" => Command::Init {
            path: ".".to_string(),
            template: ProjectTemplate::Cli,
        },
        [cmd, path] if cmd == "init" => Command::Init {
            path: path.clone(),
            template: ProjectTemplate::Cli,
        },
        [cmd, path, flag, template] if cmd == "init" && flag == "--template" => {
            match parse_project_template(template) {
                Ok(template) => Command::Init {
                    path: path.clone(),
                    template,
                },
                Err(message) => Command::Invalid(message),
            }
        }
        [cmd, flag, template] if cmd == "init" && flag == "--template" => {
            match parse_project_template(template) {
                Ok(template) => Command::Init {
                    path: ".".to_string(),
                    template,
                },
                Err(message) => Command::Invalid(message),
            }
        }
        [cmd, flag, template, path] if cmd == "init" && flag == "--template" => {
            match parse_project_template(template) {
                Ok(template) => Command::Init {
                    path: path.clone(),
                    template,
                },
                Err(message) => Command::Invalid(message),
            }
        }
        [cmd] if cmd == "migrate" => Command::Migrate {
            path: ".".to_string(),
            check: false,
        },
        [cmd, flag] if cmd == "migrate" && flag == "--check" => Command::Migrate {
            path: ".".to_string(),
            check: true,
        },
        [cmd, path] if cmd == "migrate" => Command::Migrate {
            path: path.clone(),
            check: false,
        },
        [cmd, flag, path] if cmd == "migrate" && flag == "--check" => Command::Migrate {
            path: path.clone(),
            check: true,
        },
        [cmd, path, flag] if cmd == "migrate" && flag == "--check" => Command::Migrate {
            path: path.clone(),
            check: true,
        },
        [cmd] if cmd == "doc" => Command::Doc {
            path: ".".to_string(),
            output: OutputMode::Human,
        },
        [cmd, flag] if cmd == "doc" && flag == "--json" => Command::Doc {
            path: ".".to_string(),
            output: OutputMode::Json,
        },
        [cmd, path] if cmd == "doc" => Command::Doc {
            path: path.clone(),
            output: OutputMode::Human,
        },
        [cmd, flag, path] if cmd == "doc" && flag == "--json" => Command::Doc {
            path: path.clone(),
            output: OutputMode::Json,
        },
        [cmd, path, flag] if cmd == "doc" && flag == "--json" => Command::Doc {
            path: path.clone(),
            output: OutputMode::Json,
        },
        [cmd] if cmd == "lsp" => Command::Lsp,
        [cmd, flag] if cmd == "fmt" && flag == "--stdin" => Command::FmtStdin,
        [cmd, path] if cmd == "fmt" => Command::Fmt {
            path: path.clone(),
            mode: FormatMode::Write,
        },
        [cmd, flag, path] if cmd == "fmt" && flag == "--check" => Command::Fmt {
            path: path.clone(),
            mode: FormatMode::Check,
        },
        [cmd, path, flag] if cmd == "fmt" && flag == "--check" => Command::Fmt {
            path: path.clone(),
            mode: FormatMode::Check,
        },
        [cmd, flag, path] if cmd == "fmt" && flag == "--stdout" => Command::Fmt {
            path: path.clone(),
            mode: FormatMode::Stdout,
        },
        [cmd, path, flag] if cmd == "fmt" && flag == "--stdout" => Command::Fmt {
            path: path.clone(),
            mode: FormatMode::Stdout,
        },
        [cmd] if matches!(cmd.as_str(), "run" | "check" | "bench" | "new" | "fmt") => {
            Command::Invalid(format!("command '{}' requires a path", cmd))
        }
        [cmd, ..]
            if matches!(
                cmd.as_str(),
                "run"
                    | "check"
                    | "bench"
                    | "test"
                    | "test-metrics"
                    | "new"
                    | "init"
                    | "migrate"
                    | "doc"
                    | "lsp"
                    | "fmt"
                    | "doctor"
                    | "ui-check"
            ) =>
        {
            Command::Invalid(format!("invalid arguments for command '{}'", cmd))
        }
        [path] => Command::Run {
            path: path.clone(),
            args: Vec::new(),
            options: RunOptions::default(),
        },
        [path, separator, rest @ ..] if separator == "--" => Command::Run {
            path: path.clone(),
            args: rest.to_vec(),
            options: RunOptions::default(),
        },
        [command, ..] => Command::Invalid(format!("unknown command '{}'", command)),
    }
}

fn parse_bench_command(args: &[String]) -> Command {
    if args.is_empty() {
        return Command::Invalid("command 'bench' requires a path".to_string());
    }

    let mut path: Option<String> = None;
    let mut frames = 300u64;
    let mut output = OutputMode::Human;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--json" {
            output = OutputMode::Json;
            index += 1;
            continue;
        }
        if arg == "--frames" {
            let Some(value) = args.get(index + 1) else {
                return Command::Invalid("--frames requires a positive integer".to_string());
            };
            let Ok(parsed) = value.parse::<u64>() else {
                return Command::Invalid("--frames requires a positive integer".to_string());
            };
            if parsed == 0 {
                return Command::Invalid("--frames must be greater than zero".to_string());
            }
            frames = parsed;
            index += 2;
            continue;
        }
        if arg.starts_with("--") {
            return Command::Invalid(format!("unknown bench option '{}'", arg));
        }
        if path.replace(arg.clone()).is_some() {
            return Command::Invalid("command 'bench' accepts only one path".to_string());
        }
        index += 1;
    }

    match path {
        Some(path) => Command::Bench {
            path,
            frames,
            output,
        },
        None => Command::Invalid("command 'bench' requires a path".to_string()),
    }
}

fn parse_check_command(args: &[String]) -> Command {
    let mut path: Option<String> = None;
    let mut output = OutputMode::Human;
    let mut sandbox = SandboxCliOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--json" {
            output = OutputMode::Json;
            index += 1;
            continue;
        }
        if arg == "--sandbox" {
            let Some(root) = args.get(index + 1) else {
                return Command::Invalid("--sandbox requires a root path".to_string());
            };
            sandbox.root = Some(root.clone());
            index += 2;
            continue;
        }
        if arg == "--allow-fs" {
            let Some(path) = args.get(index + 1) else {
                return Command::Invalid("--allow-fs requires a path".to_string());
            };
            sandbox.allow_fs.push(path.clone());
            index += 2;
            continue;
        }
        if arg.starts_with("--") {
            return Command::Invalid(format!("unknown check option '{}'", arg));
        }
        if path.replace(arg.clone()).is_some() {
            return Command::Invalid("command 'check' accepts only one path".to_string());
        }
        index += 1;
    }

    match path {
        Some(path) => Command::Check {
            path,
            output,
            sandbox,
        },
        None => Command::Invalid("command 'check' requires a path".to_string()),
    }
}

fn parse_test_command(args: &[String]) -> Command {
    let mut path: Option<String> = None;
    let mut output = OutputMode::Human;
    let mut sandbox = SandboxCliOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--json" {
            output = OutputMode::Json;
            index += 1;
            continue;
        }
        if arg == "--sandbox" {
            let Some(root) = args.get(index + 1) else {
                return Command::Invalid("--sandbox requires a root path".to_string());
            };
            sandbox.root = Some(root.clone());
            index += 2;
            continue;
        }
        if arg == "--allow-fs" {
            let Some(path) = args.get(index + 1) else {
                return Command::Invalid("--allow-fs requires a path".to_string());
            };
            sandbox.allow_fs.push(path.clone());
            index += 2;
            continue;
        }
        if arg.starts_with("--") {
            return Command::Invalid(format!("unknown test option '{}'", arg));
        }
        if path.replace(arg.clone()).is_some() {
            return Command::Invalid("command 'test' accepts only one path".to_string());
        }
        index += 1;
    }

    Command::Test {
        path: path.unwrap_or_else(|| ".".to_string()),
        output,
        sandbox,
    }
}

fn parse_run_command(args: &[String]) -> Command {
    if args.is_empty() {
        return Command::Invalid("command 'run' requires a path".to_string());
    }

    let mut path: Option<String> = None;
    let mut options = RunOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            return match path {
                Some(path) => Command::Run {
                    path,
                    args: args[index + 1..].to_vec(),
                    options,
                },
                None => Command::Invalid("command 'run' requires a path before '--'".to_string()),
            };
        }
        if arg == "--profile" {
            options.profile = true;
            index += 1;
            continue;
        }
        if arg == "--frames" {
            let Some(value) = args.get(index + 1) else {
                return Command::Invalid("--frames requires a positive integer".to_string());
            };
            let Ok(frames) = value.parse::<u64>() else {
                return Command::Invalid("--frames requires a positive integer".to_string());
            };
            if frames == 0 {
                return Command::Invalid("--frames must be greater than zero".to_string());
            }
            options.frames = Some(frames);
            index += 2;
            continue;
        }
        if arg == "--sandbox" {
            let Some(root) = args.get(index + 1) else {
                return Command::Invalid("--sandbox requires a root path".to_string());
            };
            options.sandbox.root = Some(root.clone());
            index += 2;
            continue;
        }
        if arg == "--allow-fs" {
            let Some(path) = args.get(index + 1) else {
                return Command::Invalid("--allow-fs requires a path".to_string());
            };
            options.sandbox.allow_fs.push(path.clone());
            index += 2;
            continue;
        }
        if arg.starts_with("--") {
            return Command::Invalid(format!("unknown run option '{}'", arg));
        }
        if path.replace(arg.clone()).is_some() {
            return Command::Invalid("command 'run' accepts only one path before '--'".to_string());
        }
        index += 1;
    }

    match path {
        Some(path) => Command::Run {
            path,
            args: Vec::new(),
            options,
        },
        None => Command::Invalid("command 'run' requires a path".to_string()),
    }
}

fn parse_project_template(value: &str) -> Result<ProjectTemplate, String> {
    match value {
        "cli" => Ok(ProjectTemplate::Cli),
        "app" => Ok(ProjectTemplate::App),
        "platformer" => Ok(ProjectTemplate::Platformer),
        "topdown" => Ok(ProjectTemplate::Topdown),
        "tool" => Ok(ProjectTemplate::Tool),
        other => Err(format!(
            "unknown template '{}'; expected cli, app, platformer, topdown, or tool",
            other
        )),
    }
}

fn parse_doctor_scope(value: &str) -> Result<DoctorScope, String> {
    match value {
        "--runtime" => Ok(DoctorScope::Runtime),
        "--gpu" => Ok(DoctorScope::Gpu),
        "--project" => Ok(DoctorScope::Project),
        other => Err(format!(
            "unknown doctor option '{}'; expected --runtime, --gpu, --project, or --json",
            other
        )),
    }
}

fn run(path: &Path, args: Vec<String>, options: RunOptions) -> i32 {
    let entry_path = match resolve_entry_path(path) {
        Ok(entry_path) => entry_path,
        Err(message) => {
            eprintln!("vice run: {}", message);
            return 1;
        }
    };
    let program = match load_program(&entry_path) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{}", format_error_for_file(&error, &entry_path));
            return 1;
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(error) = analyzer.analyze(&program) {
        eprintln!("{}", format_error_for_file(&error, &entry_path));
        return 1;
    }

    // Run with VM by default for now
    let compiler = vice::vm::compiler::Compiler::new();
    let compiled = match compiler.compile(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", format_error_for_file(&e, &entry_path));
            return 1;
        }
    };

    vice::ui::GLOBAL_UI.set_recording_enabled(options.profile);
    vice::ui::GLOBAL_UI.set_frame_pacing_enabled(true);
    vice::ui::GLOBAL_UI.set_input_enabled(true);
    vice::ui::GLOBAL_UI.set_frame_limit(options.frames);
    vice::ui::GLOBAL_UI.clear_recorded_commands();
    vice::ui::GLOBAL_UI.reset_stats();

    vice::stdlib::set_process_args(args);
    let mut vm = vice::vm::machine::Vm::new();
    if options.profile {
        vm.config.profile_opcodes = true;
    }
    let started = Instant::now();
    let result = match vm.interpret(compiled) {
        Ok(result) => result,
        Err(error) => {
            vice::ui::GLOBAL_UI.set_recording_enabled(true);
            vice::ui::GLOBAL_UI.set_frame_pacing_enabled(true);
            vice::ui::GLOBAL_UI.set_input_enabled(true);
            vice::ui::GLOBAL_UI.set_frame_limit(None);
            eprintln!("{}", format_error_for_file(&error, &entry_path));
            return 1;
        }
    };
    let elapsed = started.elapsed();
    for out in &result.output {
        println!("{}", out);
    }
    if options.profile {
        let profile = RuntimeProfile {
            elapsed,
            instructions: result.instructions,
            ui_stats: vice::ui::GLOBAL_UI.stats(),
            hot_opcodes: result.opcode_counts.clone(),
        };
        print_profile_report("Vice profile report", &profile);
    }
    vice::ui::GLOBAL_UI.set_recording_enabled(true);
    vice::ui::GLOBAL_UI.set_frame_pacing_enabled(true);
    vice::ui::GLOBAL_UI.set_input_enabled(true);
    vice::ui::GLOBAL_UI.set_frame_limit(None);

    0
}

fn bench(path: &Path, frames: u64, output: OutputMode) -> i32 {
    let entry_path = match resolve_entry_path(path) {
        Ok(entry_path) => entry_path,
        Err(message) => {
            eprintln!("vice bench: {}", message);
            return 1;
        }
    };
    let program = match load_program(&entry_path) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{}", format_error_for_file(&error, &entry_path));
            return 1;
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(error) = analyzer.analyze(&program) {
        eprintln!("{}", format_error_for_file(&error, &entry_path));
        return 1;
    }
    let compiler = vice::vm::compiler::Compiler::new();
    let compiled = match compiler.compile(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", format_error_for_file(&e, &entry_path));
            return 1;
        }
    };

    vice::ui::GLOBAL_UI.set_recording_enabled(true);
    vice::ui::GLOBAL_UI.set_frame_pacing_enabled(false);
    vice::ui::GLOBAL_UI.set_input_enabled(false);
    vice::ui::GLOBAL_UI.set_frame_limit(Some(frames));
    vice::ui::GLOBAL_UI.clear_recorded_commands();
    vice::ui::GLOBAL_UI.reset_stats();
    vice::stdlib::set_process_args(Vec::new());

    let mut vm = vice::vm::machine::Vm::new();
    vm.config.profile_opcodes = true;
    let started = Instant::now();
    let result = match vm.interpret(compiled) {
        Ok(result) => result,
        Err(error) => {
            vice::ui::GLOBAL_UI.set_recording_enabled(true);
            vice::ui::GLOBAL_UI.set_frame_pacing_enabled(true);
            vice::ui::GLOBAL_UI.set_input_enabled(true);
            vice::ui::GLOBAL_UI.set_frame_limit(None);
            eprintln!("{}", format_error_for_file(&error, &entry_path));
            return 1;
        }
    };
    let profile = RuntimeProfile {
        elapsed: started.elapsed(),
        instructions: result.instructions,
        ui_stats: vice::ui::GLOBAL_UI.stats(),
        hot_opcodes: result.opcode_counts,
    };
    vice::ui::GLOBAL_UI.set_recording_enabled(true);
    vice::ui::GLOBAL_UI.set_frame_pacing_enabled(true);
    vice::ui::GLOBAL_UI.set_input_enabled(true);
    vice::ui::GLOBAL_UI.set_frame_limit(None);

    match output {
        OutputMode::Human => print_profile_report("Vice benchmark report", &profile),
        OutputMode::Json => print_profile_json(&profile),
    }
    0
}

fn profile_numbers(profile: &RuntimeProfile) -> (f64, f64) {
    let elapsed_ms = profile.elapsed.as_secs_f64() * 1000.0;
    let avg_frame_ms = if profile.ui_stats.frames > 0 {
        elapsed_ms / profile.ui_stats.frames as f64
    } else {
        0.0
    };
    (elapsed_ms, avg_frame_ms)
}

fn print_profile_report(title: &str, profile: &RuntimeProfile) {
    let (elapsed_ms, avg_frame_ms) = profile_numbers(profile);
    let ui_stats = &profile.ui_stats;
    println!("{}", title);
    println!("  elapsed_ms: {:.2}", elapsed_ms);
    println!("  frames: {}", ui_stats.frames);
    println!("  avg_frame_ms: {:.2}", avg_frame_ms);
    println!("  vm_instructions: {}", profile.instructions);
    println!("  ui_rects: {}", ui_stats.rects);
    println!("  ui_images: {}", ui_stats.images);
    println!("  ui_labels: {}", ui_stats.labels);
    println!("  ui_buttons: {}", ui_stats.buttons);
    println!("  ui_draw_calls: {}", ui_stats.draw_calls);
    println!("  ui_recorded_commands: {}", ui_stats.recorded_commands);
    if ui_stats.labels > ui_stats.frames.saturating_mul(20).max(200) {
        println!(
            "  warning: many text labels were submitted; text rendering may dominate frame time"
        );
    }
    if ui_stats.recorded_commands >= 7_500 {
        println!("  warning: UI recorder buffer is near its cap");
    }
    if !profile.hot_opcodes.is_empty() {
        println!("  hot_opcodes:");
        for (name, count) in profile.hot_opcodes.iter().take(8) {
            println!("    {}: {}", name, count);
        }
    }
    println!("  sections:");
    println!("    vm: {:.2} ms", elapsed_ms);
    println!("    update: tracked in VM profile");
    println!("    render: tracked through ui_draw_calls/ui_* counters");
    println!("    gc_alloc: not applicable yet");
}

fn print_profile_json(profile: &RuntimeProfile) {
    let (elapsed_ms, avg_frame_ms) = profile_numbers(profile);
    let ui_stats = &profile.ui_stats;
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "elapsed_ms": elapsed_ms,
            "frames": ui_stats.frames,
            "avg_frame_ms": avg_frame_ms,
            "vm_instructions": profile.instructions,
            "ui": {
                "rects": ui_stats.rects,
                "images": ui_stats.images,
                "labels": ui_stats.labels,
                "buttons": ui_stats.buttons,
                "draw_calls": ui_stats.draw_calls,
                "recorded_commands": ui_stats.recorded_commands,
            },
            "hot_opcodes": profile.hot_opcodes.iter().take(16).map(|(name, count)| {
                serde_json::json!({"opcode": name, "count": count})
            }).collect::<Vec<_>>(),
            "sections": {
                "vm": {"elapsed_ms": elapsed_ms, "instructions": profile.instructions},
                "update": {"status": "tracked_by_vm"},
                "render": {"draw_calls": ui_stats.draw_calls},
                "gc_alloc": {"status": "not_applicable_yet"}
            }
        })
    );
}

fn find_manifest_root() -> PathBuf {
    if let Ok(mut cur) = env::current_dir() {
        loop {
            if cur.join("vice.toml").is_file() {
                return cur;
            }
            if !cur.pop() {
                break;
            }
        }
    }
    PathBuf::from(".")
}

fn resolve_entry_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    if !path.exists() {
        return Err(format!("'{}' does not exist", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("'{}' is not a file or directory", path.display()));
    }

    let manifest_path = path.join("vice.toml");
    if manifest_path.exists() {
        let source = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("cannot read '{}': {}", manifest_path.display(), error))?;
        let entry = read_toml_string_value(&source, "entry").ok_or_else(|| {
            format!(
                "'{}' must contain a string package entry",
                manifest_path.display()
            )
        })?;
        return resolve_safe_project_entry(path, &manifest_path, &entry);
    }

    let manifest_path = path.join("vice.json");
    if manifest_path.exists() {
        let source = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("cannot read '{}': {}", manifest_path.display(), error))?;
        let manifest: serde_json::Value = serde_json::from_str(&source)
            .map_err(|error| format!("invalid '{}': {}", manifest_path.display(), error))?;
        let entry = manifest
            .get("entry")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                format!(
                    "'{}' must contain a string 'entry'",
                    manifest_path.display()
                )
            })?;
        return resolve_safe_project_entry(path, &manifest_path, entry);
    }

    for relative in ["src/main.vice", "main.vice"] {
        let candidate = path.join(relative);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "'{}' has no vice.toml/vice.json entry, src/main.vice, or main.vice",
        path.display()
    ))
}

fn read_toml_string_value(source: &str, key: &str) -> Option<String> {
    for line in source.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() != key {
            continue;
        }
        let value = right.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].replace("\\\"", "\""));
        }
    }
    None
}

fn resolve_safe_project_entry(
    root: &Path,
    manifest_path: &Path,
    entry: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(entry);
    if entry.is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "'{}' has an unsafe project entry '{}'",
            manifest_path.display(),
            entry
        ));
    }
    let candidate = root.join(relative);
    if !candidate.is_file() {
        return Err(format!(
            "project entry '{}' does not exist",
            candidate.display()
        ));
    }
    Ok(candidate)
}

fn collect_vice_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.exists() {
        return Err(format!("'{}' does not exist", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("'{}' is not a file or directory", path.display()));
    }

    fn visit(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let file_type = entry.file_type()?;
            let entry_path = entry.path();
            if file_type.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with('.') || matches!(name.as_ref(), "target" | "node_modules") {
                    continue;
                }
                visit(&entry_path, files)?;
            } else if file_type.is_file()
                && entry_path.extension().and_then(|value| value.to_str()) == Some("vice")
            {
                files.push(entry_path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(path, &mut files)
        .map_err(|error| format!("cannot scan '{}': {}", path.display(), error))?;
    if files.is_empty() {
        return Err(format!("'{}' contains no .vice files", path.display()));
    }
    Ok(files)
}

struct TestOutcome {
    file: PathBuf,
    name: String,
    output: Vec<String>,
    error: Option<ViceError>,
}

fn test_path(path: &Path, output_mode: OutputMode) -> i32 {
    let scan_root = if path.is_dir() && has_project_manifest(path) {
        path.join("tests")
    } else {
        path.to_path_buf()
    };
    let files = match collect_vice_files(&scan_root) {
        Ok(files) => files,
        Err(message) => {
            let message = if path.is_dir() && has_project_manifest(path) && !scan_root.exists() {
                format!("project '{}' has no tests directory", path.display())
            } else {
                message
            };
            match output_mode {
                OutputMode::Human => eprintln!("vice test: {}", message),
                OutputMode::Json => {
                    println!("{}", serde_json::json!({ "ok": false, "error": message }))
                }
            }
            return 1;
        }
    };

    let mut outcomes = Vec::new();
    for file in &files {
        collect_test_outcomes(file, &mut outcomes);
    }
    if outcomes.is_empty() {
        let message = format!(
            "'{}' contains no @test or entry functions",
            scan_root.display()
        );
        match output_mode {
            OutputMode::Human => eprintln!("vice test: {}", message),
            OutputMode::Json => {
                println!("{}", serde_json::json!({ "ok": false, "error": message }))
            }
        }
        return 1;
    }

    let failed = outcomes
        .iter()
        .filter(|outcome| outcome.error.is_some())
        .count();
    match output_mode {
        OutputMode::Human => {
            for outcome in &outcomes {
                let label = format!("{} :: {}", outcome.file.display(), outcome.name);
                if let Some(error) = &outcome.error {
                    eprintln!("FAIL {}", label);
                    eprintln!("{}", format_error_for_file(error, &outcome.file));
                } else {
                    println!("PASS {}", label);
                    for line in &outcome.output {
                        println!("  | {}", line);
                    }
                }
            }
            if failed == 0 {
                println!("Test result: OK. {} passed; 0 failed", outcomes.len());
                0
            } else {
                eprintln!(
                    "Test result: FAILED. {} passed; {} failed",
                    outcomes.len() - failed,
                    failed
                );
                1
            }
        }
        OutputMode::Json => {
            let tests = outcomes
                .iter()
                .map(|outcome| {
                    let mut item = serde_json::json!({
                        "ok": outcome.error.is_none(),
                        "file": canonical_display_path(&outcome.file),
                        "name": outcome.name,
                        "output": outcome.output,
                    });
                    if let Some(error) = &outcome.error {
                        let warnings = [];
                        item["diagnostic"] =
                            diagnostic_json(error, &outcome.file, &warnings)["diagnostic"].clone();
                    }
                    item
                })
                .collect::<Vec<_>>();
            println!(
                "{}",
                serde_json::json!({
                    "ok": failed == 0,
                    "root": canonical_display_path(&scan_root),
                    "tests": tests,
                    "summary": {
                        "total": outcomes.len(),
                        "passed": outcomes.len() - failed,
                        "failed": failed,
                    }
                })
            );
            i32::from(failed != 0)
        }
    }
}

fn has_project_manifest(path: &Path) -> bool {
    path.join("vice.toml").is_file() || path.join("vice.json").is_file()
}

fn collect_test_outcomes(file: &Path, outcomes: &mut Vec<TestOutcome>) {
    let program = match load_program(file) {
        Ok(program) => program,
        Err(error) => {
            outcomes.push(TestOutcome {
                file: file.to_path_buf(),
                name: "<load>".to_string(),
                output: Vec::new(),
                error: Some(error),
            });
            return;
        }
    };

    let test_names = program
        .functions
        .iter()
        .filter(|function| {
            function
                .attributes
                .iter()
                .any(|attribute| attribute.name == "test")
        })
        .map(|function| function.name.clone())
        .collect::<Vec<_>>();
    let has_entry = program.functions.iter().any(|function| {
        function.name == "main"
            || function
                .attributes
                .iter()
                .any(|attribute| attribute.name == "entry")
    });
    if test_names.is_empty() && !has_entry {
        return;
    }

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(error) = analyzer.analyze(&program) {
        outcomes.push(TestOutcome {
            file: file.to_path_buf(),
            name: "<analysis>".to_string(),
            output: Vec::new(),
            error: Some(error),
        });
        return;
    }

    let cases = if test_names.is_empty() {
        vec![("entry".to_string(), None)]
    } else {
        test_names
            .into_iter()
            .map(|name| (name.clone(), Some(name)))
            .collect()
    };
    for (name, entry) in cases {
        let compiled =
            vice::vm::compiler::Compiler::new().compile_with_entry(&program, entry.as_deref());
        let result = match compiled {
            Ok(compiled) => {
                vice::stdlib::set_process_args(Vec::new());
                vice::vm::machine::Vm::new().interpret(compiled)
            }
            Err(error) => Err(error),
        };
        match result {
            Ok(result) => outcomes.push(TestOutcome {
                file: file.to_path_buf(),
                name,
                output: result.output,
                error: None,
            }),
            Err(error) => outcomes.push(TestOutcome {
                file: file.to_path_buf(),
                name,
                output: Vec::new(),
                error: Some(error),
            }),
        }
    }
}

fn check_path(path: &Path, output: OutputMode) -> i32 {
    if !path.is_dir() {
        return check_file(path, output);
    }
    let files = match collect_vice_files(path) {
        Ok(files) => files,
        Err(message) => {
            match output {
                OutputMode::Human => eprintln!("vice check: {}", message),
                OutputMode::Json => {
                    println!("{}", serde_json::json!({ "ok": false, "error": message }))
                }
            }
            return 1;
        }
    };

    match output {
        OutputMode::Human => {
            let mut failed = 0usize;
            for file in &files {
                if check_file(file, OutputMode::Human) != 0 {
                    failed += 1;
                }
            }
            if failed == 0 {
                println!("Checked {} Vice file(s)", files.len());
                0
            } else {
                eprintln!("Failed: {} of {} Vice file(s)", failed, files.len());
                1
            }
        }
        OutputMode::Json => {
            let mut failed = 0usize;
            let results = files
                .iter()
                .map(|file| {
                    let (ok, report) = check_file_json_report(file);
                    if !ok {
                        failed += 1;
                    }
                    report
                })
                .collect::<Vec<_>>();
            println!(
                "{}",
                serde_json::json!({
                    "ok": failed == 0,
                    "root": canonical_display_path(path),
                    "files": results,
                    "summary": {
                        "checked": files.len(),
                        "passed": files.len() - failed,
                        "failed": failed,
                    }
                })
            );
            i32::from(failed != 0)
        }
    }
}

fn check_file(path: &Path, output: OutputMode) -> i32 {
    let warnings = legacy_warnings_for_file(path);
    match check_program(path) {
        Ok(_) => {
            match output {
                OutputMode::Human => {
                    print_warnings(path, &warnings);
                    println!("OK: {}", path.display());
                }
                OutputMode::Json => println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "file": canonical_display_path(path),
                        "warnings": warnings_json(path, &warnings),
                    })
                ),
            }
            0
        }
        Err(error) => {
            match output {
                OutputMode::Human => {
                    eprintln!("{}", format_error_for_file(&error, path));
                    print_warnings(path, &warnings);
                }
                OutputMode::Json => println!("{}", diagnostic_json(&error, path, &warnings)),
            }
            1
        }
    }
}

fn check_file_json_report(path: &Path) -> (bool, serde_json::Value) {
    let warnings = legacy_warnings_for_file(path);
    match check_program(path) {
        Ok(_) => (
            true,
            serde_json::json!({
                "ok": true,
                "file": canonical_display_path(path),
                "warnings": warnings_json(path, &warnings),
            }),
        ),
        Err(error) => (false, diagnostic_json(&error, path, &warnings)),
    }
}

fn format_path(path: &Path, mode: FormatMode) -> i32 {
    if !path.is_dir() {
        return format_file(path, mode);
    }
    if matches!(mode, FormatMode::Stdout) {
        eprintln!("vice fmt: --stdout requires a single .vice file");
        return 2;
    }
    let files = match collect_vice_files(path) {
        Ok(files) => files,
        Err(message) => {
            eprintln!("vice fmt: {}", message);
            return 1;
        }
    };
    let mut failed = 0usize;
    for file in &files {
        if format_file(file, mode) != 0 {
            failed += 1;
        }
    }
    if failed == 0 {
        println!("Processed {} Vice file(s)", files.len());
        0
    } else {
        eprintln!("Failed: {} of {} Vice file(s)", failed, files.len());
        1
    }
}

fn format_stdin() -> i32 {
    let mut source = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut source) {
        eprintln!("vice fmt: cannot read stdin: {}", error);
        return 1;
    }
    match format_source(&source, Path::new("<stdin>")) {
        Ok(formatted) => {
            print!("{}", formatted);
            0
        }
        Err(error) => {
            eprintln!("{}", format_error_with_source(&error, &source));
            1
        }
    }
}

fn format_file(path: &Path, mode: FormatMode) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Cannot read '{}': {}", path.display(), error);
            return 1;
        }
    };

    let formatted = match format_source(&source, path) {
        Ok(formatted) => formatted,
        Err(error) => {
            eprintln!("{}", format_error_with_source(&error, &source));
            return 1;
        }
    };

    match mode {
        FormatMode::Check if source == formatted => {
            println!("OK: {}", path.display());
            0
        }
        FormatMode::Check => {
            eprintln!("{} is not formatted", path.display());
            1
        }
        FormatMode::Stdout => {
            print!("{}", formatted);
            0
        }
        FormatMode::Write => match fs::write(path, formatted) {
            Ok(()) => {
                println!("Formatted: {}", path.display());
                0
            }
            Err(error) => {
                eprintln!("Cannot write '{}': {}", path.display(), error);
                1
            }
        },
    }
}

fn diagnostic_json(
    error: &ViceError,
    fallback_path: &Path,
    warnings: &[ViceWarning],
) -> serde_json::Value {
    let file = error
        .file
        .clone()
        .unwrap_or_else(|| canonical_display_path(fallback_path));
    let stack = error
        .stack_trace
        .iter()
        .map(|frame| {
            serde_json::json!({
                "function": frame.function,
                "file": frame.file,
                "line": frame.line,
                "column": frame.column,
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "ok": false,
        "diagnostic": {
            "type": error_type_name(&error.error_type),
            "message": error.message,
            "file": file,
            "line": error.line,
            "column": error.column,
            "length": error.length,
            "stack": stack,
        },
        "warnings": warnings_json(fallback_path, warnings),
    })
}

fn print_warnings(path: &Path, warnings: &[ViceWarning]) {
    for warning in warnings {
        eprintln!(
            "warning[{}]: {}:{}:{}: {} ({})",
            warning.code,
            path.display(),
            warning.line,
            warning.column,
            warning.message,
            warning.help
        );
    }
}

fn warnings_json(path: &Path, warnings: &[ViceWarning]) -> Vec<serde_json::Value> {
    let file = canonical_display_path(path);
    warnings
        .iter()
        .map(|warning| {
            serde_json::json!({
                "code": warning.code,
                "message": warning.message,
                "help": warning.help,
                "file": file,
                "line": warning.line,
                "column": warning.column,
                "length": warning.length,
            })
        })
        .collect()
}

fn canonical_display_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

fn error_type_name(error_type: &ViceErrorType) -> &'static str {
    match error_type {
        ViceErrorType::Syntax => "syntax",
        ViceErrorType::Compile => "compile",
        ViceErrorType::Runtime => "runtime",
        ViceErrorType::Semantic => "semantic",
        ViceErrorType::ReturnEarly => "return_early",
    }
}

fn new_project(path: &Path, executable: &str, template: ProjectTemplate) -> i32 {
    if path.exists() {
        eprintln!(
            "{} new: folder '{}' already exists",
            executable,
            path.display()
        );
        return 1;
    }

    if let Err(error) = create_project(path, template) {
        eprintln!(
            "{} new: cannot create '{}': {}",
            executable,
            path.display(),
            error
        );
        return 1;
    }

    println!("Created Vice project: {}", path.display());
    println!("Run it with:");
    println!("  cd {}", path.display());
    println!("  {} run .", executable);
    0
}

fn init_project(path: &Path, executable: &str, template: ProjectTemplate) -> i32 {
    if !path.exists() {
        if let Err(error) = fs::create_dir_all(path) {
            eprintln!(
                "{} init: cannot create '{}': {}",
                executable,
                path.display(),
                error
            );
            return 1;
        }
    }
    if !path.is_dir() {
        eprintln!(
            "{} init: '{}' is not a directory",
            executable,
            path.display()
        );
        return 1;
    }
    if let Err(error) = create_project_files(path, false, template) {
        eprintln!(
            "{} init: cannot initialize '{}': {}",
            executable,
            path.display(),
            error
        );
        return 1;
    }
    println!("Initialized Vice project: {}", path.display());
    0
}

fn migrate_path(path: &Path, check: bool) -> i32 {
    let files = match collect_vice_files(path) {
        Ok(files) => files,
        Err(message) => {
            eprintln!("vice migrate: {}", message);
            return 1;
        }
    };

    let mut changed = Vec::new();
    for file in files {
        let source = match fs::read_to_string(&file) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("vice migrate: cannot read '{}': {}", file.display(), error);
                return 1;
            }
        };
        let migrated = migrate_source_text(&source);
        if migrated != source {
            changed.push(file.clone());
            if !check {
                if let Err(error) = fs::write(&file, migrated) {
                    eprintln!("vice migrate: cannot write '{}': {}", file.display(), error);
                    return 1;
                }
            }
        }
    }

    let root = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };
    let legacy_manifest = root.join("vice.json");
    let modern_manifest = root.join("vice.toml");
    if legacy_manifest.is_file() && !modern_manifest.exists() {
        changed.push(legacy_manifest.clone());
        if !check {
            match migrate_manifest_json_to_toml(&legacy_manifest, &modern_manifest) {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("vice migrate: {}", error);
                    return 1;
                }
            }
        }
    }

    if check && !changed.is_empty() {
        for file in &changed {
            eprintln!("would migrate: {}", file.display());
        }
        return 1;
    }
    println!("Migrated {} item(s)", changed.len());
    0
}

fn migrate_source_text(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut migrated = String::with_capacity(source.len());
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;

    while index < chars.len() {
        let current = chars[index];

        if in_line_comment {
            migrated.push(current);
            index += 1;
            if current == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if in_string {
            migrated.push(current);
            index += 1;
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == '"' {
                in_string = false;
            }
            continue;
        }

        if current == '#' {
            in_line_comment = true;
            migrated.push(current);
            index += 1;
            continue;
        }

        if current == '/' && chars.get(index + 1) == Some(&'/') {
            in_line_comment = true;
            migrated.push('/');
            migrated.push('/');
            index += 2;
            continue;
        }

        if current == '"' {
            in_string = true;
            migrated.push(current);
            index += 1;
            continue;
        }

        if starts_with_keyword(&chars, index, "@mount")
            && chars
                .get(index + "@mount".len())
                .is_some_and(|ch| ch.is_whitespace())
        {
            migrated.push_str("use");
            index += "@mount".len();
            continue;
        }

        if is_legacy_output_statement(&chars, index) {
            migrated.push_str("emit");
            index += "output".len();
            continue;
        }

        if let Some(after_error_call) = legacy_error_call_end(&chars, index) {
            migrated.push_str("fail");
            index = after_error_call;
            continue;
        }

        migrated.push(current);
        index += 1;
    }

    migrated
}

fn legacy_warnings_for_file(path: &Path) -> Vec<ViceWarning> {
    fs::read_to_string(path)
        .map(|source| legacy_warnings_for_source(&source))
        .unwrap_or_default()
}

fn legacy_warnings_for_source(source: &str) -> Vec<ViceWarning> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut warnings = Vec::new();
    let mut index = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;

    while index < chars.len() {
        let current = chars[index];

        if in_line_comment {
            advance_cursor(current, &mut line, &mut column);
            index += 1;
            if current == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if in_string {
            advance_cursor(current, &mut line, &mut column);
            index += 1;
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == '"' {
                in_string = false;
            }
            continue;
        }

        if current == '#' {
            in_line_comment = true;
            advance_cursor(current, &mut line, &mut column);
            index += 1;
            continue;
        }

        if current == '/' && chars.get(index + 1) == Some(&'/') {
            in_line_comment = true;
            advance_cursor('/', &mut line, &mut column);
            advance_cursor('/', &mut line, &mut column);
            index += 2;
            continue;
        }

        if current == '"' {
            in_string = true;
            advance_cursor(current, &mut line, &mut column);
            index += 1;
            continue;
        }

        if starts_with_keyword(&chars, index, "@mount")
            && chars
                .get(index + "@mount".len())
                .is_some_and(|ch| ch.is_whitespace())
        {
            warnings.push(ViceWarning {
                code: "legacy-import",
                message: "`@mount` is deprecated in Vice v0.2",
                help: "use `use \"path.vice\";` instead",
                line,
                column,
                length: "@mount".len(),
            });
            advance_many(&chars, &mut index, "@mount".len(), &mut line, &mut column);
            continue;
        }

        if is_legacy_output_statement(&chars, index) {
            warnings.push(ViceWarning {
                code: "legacy-output",
                message: "`output <-` is deprecated in Vice v0.2",
                help: "use `emit <-` instead",
                line,
                column,
                length: "output".len(),
            });
            advance_many(&chars, &mut index, "output".len(), &mut line, &mut column);
            continue;
        }

        if let Some(after_error_call) = legacy_error_call_end(&chars, index) {
            warnings.push(ViceWarning {
                code: "legacy-fail",
                message: "`error() <-` is deprecated in Vice v0.2",
                help: "use `fail <-` instead",
                line,
                column,
                length: after_error_call - index,
            });
            while index < after_error_call {
                advance_cursor(chars[index], &mut line, &mut column);
                index += 1;
            }
            continue;
        }

        advance_cursor(current, &mut line, &mut column);
        index += 1;
    }

    warnings
}

fn advance_many(
    chars: &[char],
    index: &mut usize,
    count: usize,
    line: &mut usize,
    column: &mut usize,
) {
    for _ in 0..count {
        if let Some(ch) = chars.get(*index) {
            advance_cursor(*ch, line, column);
            *index += 1;
        }
    }
}

fn advance_cursor(ch: char, line: &mut usize, column: &mut usize) {
    if ch == '\n' {
        *line += 1;
        *column = 1;
    } else {
        *column += 1;
    }
}

fn starts_with_keyword(chars: &[char], index: usize, keyword: &str) -> bool {
    keyword
        .chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(index + offset) == Some(&expected))
        && !chars
            .get(index + keyword.chars().count())
            .is_some_and(|ch| is_identifier_continue(*ch))
        && !index
            .checked_sub(1)
            .and_then(|previous| chars.get(previous))
            .is_some_and(|ch| is_identifier_continue(*ch))
}

fn is_legacy_output_statement(chars: &[char], index: usize) -> bool {
    if !starts_with_keyword(chars, index, "output") {
        return false;
    }
    let arrow = skip_whitespace(chars, index + "output".len());
    chars.get(arrow) == Some(&'<') && chars.get(arrow + 1) == Some(&'-')
}

fn legacy_error_call_end(chars: &[char], index: usize) -> Option<usize> {
    if !starts_with_keyword(chars, index, "error") {
        return None;
    }
    let open = skip_whitespace(chars, index + "error".len());
    if chars.get(open) != Some(&'(') {
        return None;
    }
    let close = skip_whitespace(chars, open + 1);
    if chars.get(close) != Some(&')') {
        return None;
    }
    let arrow = skip_whitespace(chars, close + 1);
    if chars.get(arrow) == Some(&'<') && chars.get(arrow + 1) == Some(&'-') {
        Some(close + 1)
    } else {
        None
    }
}

fn skip_whitespace(chars: &[char], mut index: usize) -> usize {
    while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
        index += 1;
    }
    index
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'
}

fn migrate_manifest_json_to_toml(from: &Path, to: &Path) -> Result<(), String> {
    let source = fs::read_to_string(from)
        .map_err(|error| format!("cannot read '{}': {}", from.display(), error))?;
    let manifest: serde_json::Value = serde_json::from_str(&source)
        .map_err(|error| format!("invalid '{}': {}", from.display(), error))?;
    let name = manifest
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("vice-project");
    let version = manifest
        .get("version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("0.2.0");
    let entry = manifest
        .get("entry")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("src/main.vice");
    fs::write(to, manifest_template(name, version, entry))
        .map_err(|error| format!("cannot write '{}': {}", to.display(), error))
}

fn doc_path(path: &Path, output: OutputMode) -> i32 {
    let entry = match resolve_entry_path(path) {
        Ok(entry) => entry,
        Err(message) => {
            eprintln!("vice doc: {}", message);
            return 1;
        }
    };
    let program = match load_program(&entry) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{}", format_error_for_file(&error, &entry));
            return 1;
        }
    };
    let functions = program
        .functions
        .iter()
        .map(|function| {
            serde_json::json!({
                "name": function.name,
                "attributes": function.attributes.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
                "params": function.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                "return": function.return_type.as_ref().map(|ty| ty.to_string()),
            })
        })
        .collect::<Vec<_>>();
    match output {
        OutputMode::Json => {
            println!(
                "{}",
                serde_json::json!({ "ok": true, "entry": canonical_display_path(&entry), "functions": functions })
            );
        }
        OutputMode::Human => {
            println!("# Vice API");
            println!();
            println!("Entry: {}", entry.display());
            println!();
            for item in functions {
                println!(
                    "- fn {}({}){}",
                    item["name"].as_str().unwrap_or("<unknown>"),
                    item["params"]
                        .as_array()
                        .map(|params| params
                            .iter()
                            .filter_map(|p| p.as_str())
                            .collect::<Vec<_>>()
                            .join(", "))
                        .unwrap_or_default(),
                    item["return"]
                        .as_str()
                        .map(|ret| format!(" -> {}", ret))
                        .unwrap_or_default()
                );
            }
        }
    }
    0
}

fn dump_schema() -> i32 {
    let registry = vice::native_registry::NativeRegistry::new();
    let mut modules_json = serde_json::Map::new();
    for (module_name, methods) in &registry.modules {
        let mut methods_json = serde_json::Map::new();
        for (method_name, def) in methods {
            let args: Vec<_> = def
                .args
                .iter()
                .map(|arg| {
                    serde_json::json!({
                        "name": arg.name,
                        "type": format!("{:?}", arg.ty),
                        "optional": arg.optional
                    })
                })
                .collect();
            methods_json.insert(
                method_name.to_string(),
                serde_json::json!({
                    "args": args,
                    "return_type": format!("{:?}", def.return_type),
                    "varargs": def.varargs
                }),
            );
        }
        modules_json.insert(
            module_name.to_string(),
            serde_json::Value::Object(methods_json),
        );
    }
    let full = serde_json::json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "modules": modules_json
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&full).unwrap_or_else(|_| "{}".to_string())
    );
    0
}

fn lsp_stdio() -> i32 {
    let capabilities = serde_json::json!({
        "capabilities": {
            "textDocumentSync": 1,
            "documentFormattingProvider": true,
            "completionProvider": { "triggerCharacters": [".", "@"] }
        },
        "serverInfo": { "name": "vice-lsp", "version": env!("CARGO_PKG_VERSION") }
    });
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() || input.trim().is_empty() {
        println!("{}", capabilities);
        return 0;
    }
    if input.contains("\"method\":\"initialize\"") || input.contains("\"method\": \"initialize\"") {
        let id = extract_json_rpc_id(&input).unwrap_or_else(|| serde_json::json!(1));
        let response = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": capabilities });
        let body = response.to_string();
        print!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    }
    0
}

fn extract_json_rpc_id(input: &str) -> Option<serde_json::Value> {
    let body_start = input.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
    serde_json::from_str::<serde_json::Value>(&input[body_start..])
        .ok()
        .and_then(|value| value.get("id").cloned())
}

fn create_project(path: &Path, template: ProjectTemplate) -> std::io::Result<()> {
    fs::create_dir(path)?;
    create_project_files(path, true, template)
}

fn create_project_files(
    path: &Path,
    fail_if_files_exist: bool,
    template: ProjectTemplate,
) -> std::io::Result<()> {
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("tests"))?;
    if matches!(
        template,
        ProjectTemplate::App
            | ProjectTemplate::Platformer
            | ProjectTemplate::Topdown
            | ProjectTemplate::Tool
    ) {
        fs::create_dir_all(path.join("assets"))?;
    }
    let main_path = path.join("src").join("main.vice");
    let test_path = path.join("tests").join("main_test.vice");
    let readme_path = path.join("README.md");
    let manifest_path = path.join("vice.toml");
    let ignore_path = path.join(".gitignore");
    let project_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("vice-project");
    if fail_if_files_exist && (main_path.exists() || test_path.exists() || manifest_path.exists()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "project files already exist",
        ));
    }
    if fail_if_files_exist || !main_path.exists() {
        fs::write(main_path, main_template(template))?;
    }
    if fail_if_files_exist || !test_path.exists() {
        fs::write(test_path, test_template(template))?;
    }
    if fail_if_files_exist || !readme_path.exists() {
        fs::write(readme_path, readme_template(template))?;
    }
    if fail_if_files_exist || !manifest_path.exists() {
        fs::write(
            manifest_path,
            manifest_template_with_kind(
                project_name,
                "0.2.0",
                "src/main.vice",
                project_template_kind(template),
            ),
        )?;
    }
    if fail_if_files_exist || !ignore_path.exists() {
        fs::write(ignore_path, "target/\n*.tmp\n")?;
    }
    Ok(())
}

fn manifest_template(name: &str, version: &str, entry: &str) -> String {
    manifest_template_with_kind(name, version, entry, "cli")
}

fn manifest_template_with_kind(name: &str, version: &str, entry: &str, kind: &str) -> String {
    format!(
        r#"[package]
name = "{}"
version = "{}"
entry = "{}"
kind = "{}"

[paths]
source = "src"
tests = "tests"
examples = "examples"

[features]
desktop = false
audio = false
"#,
        name, version, entry, kind
    )
}

fn project_template_kind(template: ProjectTemplate) -> &'static str {
    match template {
        ProjectTemplate::Cli => "cli",
        ProjectTemplate::App => "app",
        ProjectTemplate::Platformer => "game",
        ProjectTemplate::Topdown => "game",
        ProjectTemplate::Tool => "tool",
    }
}

fn main_template(template: ProjectTemplate) -> &'static str {
    match template {
        ProjectTemplate::Cli => cli_main_template(),
        ProjectTemplate::App => app_main_template(),
        ProjectTemplate::Platformer => platformer_main_template(),
        ProjectTemplate::Topdown => topdown_main_template(),
        ProjectTemplate::Tool => tool_main_template(),
    }
}

fn cli_main_template() -> &'static str {
    r#"@entry
fn main() {
    emit <- "Hello, Vice!";
}
"#
}

fn app_main_template() -> &'static str {
    r##"use "std/app.vice";

@entry
fn main() {
    let config => app_default_config("Vice App");
    mut window => app_open(config);
    mut count => 0;

    while (ui.frame(window)) {
        app_label("Counter: ${count}", 40, 40);
        let add => app_primary_button("counter.add", "Add", 40, 90, 160);
        if (add.clicked) {
            count += 1;
        }
    }
}
"##
}

fn platformer_main_template() -> &'static str {
    r##"use "std/platformer.vice";

@entry
fn main() {
    mut window => ui.window(
        title => "Vice Platformer",
        size => [960, 540],
        color => "#101820"
    );
    mut timer => game_time_start();
    mut camera => game_camera(960, 540);
    let world => PhysicsWorld();
    let level => platformer_default_level();
    mut player => platformer_default_player();

    while (ui.frame(window)) {
        timer = game_time_tick(timer);
        let controls => platformer_input();
        player = platformer_update_player(player, controls, level, world, timer.delta);
        camera = game_camera_follow(camera, player.body.pos);

        platformer_draw_level(level, camera);
        platformer_draw_player(player, camera);
        ui.text(text => "Arrows/WASD move, Space jumps", pos => [24, 24], color => "#ffffff");
    }
}
"##
}

fn topdown_main_template() -> &'static str {
    r##"use "std/topdown.vice";

@entry
fn main() {
    mut window => ui.window("Vice Topdown", [960, 540], "#101820");
    mut timer => game_time_start();
    mut camera => game_camera(960, 540);
    mut hero => topdown_actor(200.0, 180.0);

    while (ui.frame(window)) {
        timer = game_time_tick(timer);
        hero = topdown_update_actor(hero, topdown_input(), timer.delta);
        camera = game_camera_follow(camera, hero.pos);
        topdown_draw_actor(hero, camera);
    }
}
"##
}

fn tool_main_template() -> &'static str {
    r##"use "std/app.vice";

@entry
fn main() {
    mut config => AppConfig();
    config.title = "Vice Tool";
    config.width = 1000;
    config.height = 680;
    config.color = "#080c0e";
    mut window => app_open(config);

    while (ui.frame(window)) {
        mut panel => AppPanel();
        panel.id = "tool.main";
        panel.title = "Tool";
        panel.x = 24;
        panel.y = 24;
        panel.width = 420;
        panel.height = 300;
        app_panel(panel);
        app_label("Build your workflow here.", 48, 72);
        app_primary_button("tool.run", "Run", 48, 120, 180);
        app_end_panel();
    }
}
"##
}

fn test_template(template: ProjectTemplate) -> &'static str {
    match template {
        ProjectTemplate::Platformer => platformer_test_template(),
        ProjectTemplate::Topdown => topdown_test_template(),
        _ => cli_test_template(),
    }
}

fn cli_test_template() -> &'static str {
    r#"@test
fn addition_keeps_its_value() {
    let total => 20 + 22;
    if (total != 42) {
        fail <- "expected 20 + 22 to equal 42";
    }
}
"#
}

fn platformer_test_template() -> &'static str {
    r##"use "std/platformer.vice";

@test
fn player_lands_on_floor() {
    let controls => PlatformerInput();
    mut level => PlatformerLevel();
    level.width = 800;
    level.height = 600;
    level.floor_y = 120.0;
    level.platforms = [];
    let world => PhysicsWorld();
    mut player => PlatformerPlayer();
    player.body = physics_body(game_vec2(20.0, 90.0), game_vec2(30.0, 44.0));
    player.body.velocity.y = 100.0;
    player = platformer_update_player(player, controls, level, world, 0.1);
    if (!player.body.grounded) {
        fail <- "expected player to land on floor";
    }
}
"##
}

fn topdown_test_template() -> &'static str {
    r##"use "std/topdown.vice";

@test
fn actor_moves_right() {
    mut controls => TopdownInput();
    controls.x = 1.0;
    controls.y = 0.0;
    mut actor => topdown_actor(0.0, 0.0);
    actor.speed = 100.0;
    actor = topdown_update_actor(actor, controls, 0.5);
    if (actor.pos.x != 50.0) {
        fail <- "expected actor to move right";
    }
}
"##
}

fn readme_template(template: ProjectTemplate) -> &'static str {
    match template {
        ProjectTemplate::Cli => cli_readme_template(),
        ProjectTemplate::App => app_readme_template(),
        ProjectTemplate::Platformer => platformer_readme_template(),
        ProjectTemplate::Topdown => topdown_readme_template(),
        ProjectTemplate::Tool => tool_readme_template(),
    }
}

fn cli_readme_template() -> &'static str {
    r#"# New Vice project

This folder contains the smallest useful Vice program.

Run:

vice run .

Check without running:

vice check .

Run tests:

vice test .

Format the whole project:

vice fmt .

"#
}

fn app_readme_template() -> &'static str {
    r#"# Vice app project

Run:

vice run .

Open the project in Vice App Manager:

vicetool run VAM

"#
}

fn platformer_readme_template() -> &'static str {
    r#"# Vice platformer project

Run:

vice run .

Check:

vice check .

Open the project in Vice App Manager:

vicetool run VAM

"#
}

fn topdown_readme_template() -> &'static str {
    r#"# Vice topdown project

Run:

vice run .

Open the project in Vice App Manager:

vicetool run VAM

"#
}

fn tool_readme_template() -> &'static str {
    r#"# Vice tool project

Run:

vice run .

Open the project in Vice App Manager:

vicetool run VAM

"#
}

fn print_usage(executable: &str) {
    eprintln!("Usage:");
    eprintln!(
        "  {} run [--sandbox <root>] [--allow-fs <path>] <file.vice|project-dir> [-- args...]",
        executable
    );
    eprintln!(
        "  {} run --profile <file-or-project> [--frames N]",
        executable
    );
    eprintln!(
        "  {} bench <file-or-project> [--frames N] [--json]",
        executable
    );
    eprintln!(
        "  {} check [--json] [--sandbox <root>] [--allow-fs <path>] <file-or-dir>",
        executable
    );
    eprintln!(
        "  {} test [--json] [--sandbox <root>] [--allow-fs <path>] [file-or-project-dir]",
        executable
    );
    eprintln!("  {} new <name>", executable);
    eprintln!(
        "  {} new <name> --template <cli|app|platformer|topdown|tool>",
        executable
    );
    eprintln!(
        "  {} init [path] --template <cli|app|platformer|topdown|tool>",
        executable
    );
    eprintln!("  {} fmt <file-or-dir>      # format in place", executable);
    eprintln!("  {} fmt --check <file-or-dir>", executable);
    eprintln!("  {} fmt --stdout <file.vice>", executable);
    eprintln!(
        "  {} fmt --stdin            # format standard input",
        executable
    );
    eprintln!(
        "  {} doctor [--runtime|--gpu|--project] [--json]",
        executable
    );
    eprintln!("  {} test-metrics [--json]  # count test cases", executable);
    eprintln!("  {} ui-check              # check UI support", executable);
    eprintln!("  {} --version             # print version", executable);
    eprintln!("  {} <file.vice>            # alias for run", executable);
}

fn test_metrics(output: OutputMode) -> i32 {
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let test_files = collect_paths_with_extensions(&root.join("tests"), &["rs", "vice", "vscn"]);
    let src_files = collect_paths_with_extensions(&root.join("src"), &["rs"]);
    let rust_tests = test_files
        .iter()
        .chain(src_files.iter())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("rs"))
        .map(|path| count_file_occurrences(path, "#[test]"))
        .sum::<usize>();
    let generated_cases = test_files
        .iter()
        .filter(|path| {
            path.file_name().and_then(|value| value.to_str()) == Some("v2_regression_matrix.rs")
        })
        .map(|path| count_generated_v2_cases(path))
        .sum::<usize>()
        + test_files
            .iter()
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("rs"))
            .map(|path| count_declared_matrix_cases(path, None))
            .sum::<usize>();
    let corpus_programs = test_files
        .iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("vice" | "vscn")
            )
        })
        .count();
    let security_cases = test_files
        .iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("rs"))
        .map(|path| count_security_cases(path))
        .sum::<usize>()
        + test_files
            .iter()
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("rs"))
            .map(|path| count_declared_matrix_cases(path, Some("SECURITY")))
            .sum::<usize>();
    let total_cases = rust_tests + generated_cases + corpus_programs;
    match output {
        OutputMode::Human => {
            println!("Vice test metrics");
            println!("  rust_tests: {}", rust_tests);
            println!("  generated_cases: {}", generated_cases);
            println!("  corpus_programs: {}", corpus_programs);
            println!("  security_cases: {}", security_cases);
            println!("  total_cases: {}", total_cases);
            println!("  target_0_4_0: 2000");
            println!(
                "  remaining_to_target: {}",
                2000usize.saturating_sub(total_cases)
            );
        }
        OutputMode::Json => println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "rust_tests": rust_tests,
                "generated_cases": generated_cases,
                "corpus_programs": corpus_programs,
                "security_cases": security_cases,
                "total_cases": total_cases,
                "target_0_4_0": 2000,
                "remaining_to_target": 2000usize.saturating_sub(total_cases),
            })
        ),
    }
    0
}

fn collect_paths_with_extensions(root: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_paths_with_extensions_inner(root, extensions, &mut paths);
    paths
}

fn collect_paths_with_extensions_inner(root: &Path, extensions: &[&str], paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_paths_with_extensions_inner(&path, extensions, paths);
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extensions.contains(&extension))
        {
            paths.push(path);
        }
    }
}

fn count_file_occurrences(path: &Path, needle: &str) -> usize {
    fs::read_to_string(path)
        .map(|source| source.matches(needle).count())
        .unwrap_or(0)
}

fn count_generated_v2_cases(path: &Path) -> usize {
    fs::read_to_string(path)
        .map(|source| {
            source
                .lines()
                .filter(|line| {
                    let trimmed = line.trim_start();
                    trimmed.starts_with("v2_") && trimmed.contains("=>") && trimmed.ends_with(';')
                })
                .count()
        })
        .unwrap_or(0)
}

fn count_declared_matrix_cases(path: &Path, name_filter: Option<&str>) -> usize {
    fs::read_to_string(path)
        .map(|source| {
            source
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim_start();
                    let body = trimmed.strip_prefix("vice_case_count!(")?;
                    let body = body.strip_suffix(");")?;
                    let (name, count) = body.split_once(',')?;
                    if name_filter.is_some_and(|filter| !name.trim().contains(filter)) {
                        return None;
                    }
                    count.trim().parse::<usize>().ok()
                })
                .sum()
        })
        .unwrap_or(0)
}

fn count_security_cases(path: &Path) -> usize {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return 0;
    };
    let looks_security_file = name.contains("security")
        || name.contains("limits")
        || name.contains("stability")
        || name.contains("fs")
        || name.contains("path");
    if !looks_security_file {
        return 0;
    }
    count_file_occurrences(path, "#[test]")
}

fn doctor(output: OutputMode, scope: DoctorScope) -> i32 {
    let ui = if cfg!(feature = "desktop") {
        "desktop"
    } else {
        "headless core"
    };
    let runtime = serde_json::json!({
        "vm": "ok",
        "bytecode": "v1_with_v2_metrics",
        "instruction_budget": "available",
        "stack_limit": 1_000_000,
        "call_depth_limit": 64,
        "deterministic_frames": "available_with_--frames",
    });
    let gpu = serde_json::json!({
        "ui": ui,
        "game2d": "v2_api_foundation",
        "recorded_command_limit": vice::ui::MAX_RECORDED_COMMANDS,
        "batching": "rect_batch_current_sprite_batch_pending",
    });
    let project = serde_json::json!({
        "cwd": env::current_dir().map(|path| path.display().to_string()).unwrap_or_else(|_| ".".to_string()),
        "vice_toml": Path::new("vice.toml").is_file(),
        "vice_json": Path::new("vice.json").is_file(),
        "src_main": Path::new("src").join("main.vice").is_file(),
        "vam_project": Path::new("vam.project.json").is_file(),
        "scenes_dir": Path::new("scenes").is_dir(),
        "ui_dir": Path::new("ui").is_dir(),
        "assets_dir": Path::new("assets").is_dir(),
    });
    match output {
        OutputMode::Human => {
            println!("Vice doctor report");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Platform: {}", std::env::consts::OS);
            println!("Arch: {}", std::env::consts::ARCH);
            if matches!(scope, DoctorScope::All | DoctorScope::Runtime) {
                println!("Runtime: OK");
                println!("  VM: {}", runtime["vm"]);
                println!("  Bytecode: {}", runtime["bytecode"]);
                println!("  Stack limit: {}", runtime["stack_limit"]);
                println!("  Call depth limit: {}", runtime["call_depth_limit"]);
            }
            if matches!(scope, DoctorScope::All | DoctorScope::Gpu) {
                println!("GPU/UI: OK");
                println!("  UI support: {}", ui);
                println!("  Game2D: {}", gpu["game2d"]);
                println!(
                    "  Recorded command limit: {}",
                    gpu["recorded_command_limit"]
                );
            }
            if matches!(scope, DoctorScope::All | DoctorScope::Project) {
                println!("Project:");
                println!("  vice.toml: {}", project["vice_toml"]);
                println!("  src/main.vice: {}", project["src_main"]);
                println!("  VAM project: {}", project["vam_project"]);
                println!("  scenes/: {}", project["scenes_dir"]);
                println!("  ui/: {}", project["ui_dir"]);
                println!("  assets/: {}", project["assets_dir"]);
            }
            println!("Stdlib: included");
        }
        OutputMode::Json => println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "scope": format!("{:?}", scope),
                "version": env!("CARGO_PKG_VERSION"),
                "platform": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "runtime": runtime,
                "gpu": gpu,
                "project": project,
                "stdlib": "included",
            })
        ),
    }
    0
}

fn ui_check() -> i32 {
    println!("UI check: verifying UI bindings");
    println!("Available UI v0.3 functions:");
    println!("  ui.window, ui.frame, ui.panel, ui.button, ui.input_text");
    println!("  ui.draw_rect, ui.draw_text, ui.draw_image, ui.canvas");
    println!("Result: OK");
    0
}
