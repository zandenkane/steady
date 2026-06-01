use clap::Parser;
use steady::cli::{Cli, Commands};
use steady::config::Config;
use steady::engine::pipeline::{FilterPipeline, FilterResult};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            config: config_path,
        } => {
            let config = match config_path {
                Some(path) => Config::load_from(&path).unwrap_or_else(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }),
                None => Config::load_default(),
            };

            eprintln!("steady: starting with configuration:");
            eprintln!(
                "  tremor: enabled={}, threshold={}px",
                config.tremor.enabled, config.tremor.threshold
            );
            eprintln!(
                "  smoothing: enabled={}, min_cutoff={}, beta={}",
                config.smoothing.enabled, config.smoothing.min_cutoff, config.smoothing.beta
            );
            eprintln!(
                "  dwell: enabled={}, time={}ms, radius={}px",
                config.dwell.enabled, config.dwell.time_ms, config.dwell.radius
            );
            eprintln!(
                "  clickzones: enabled={}, count={}",
                config.clickzones.enabled,
                config.clickzones.zones.len()
            );
            eprintln!("  active filters: {}", config.enabled_filter_count());

            run_daemon(config);
        }
        Commands::Defaults => {
            print!("{}", Config::default_toml());
        }
        Commands::Status => match Config::default_path() {
            Some(path) => {
                println!("config path: {}", path.display());
                if path.exists() {
                    println!("config file: found");
                    match Config::load_from(&path) {
                        Ok(c) => {
                            println!("config valid: yes");
                            println!("active filters: {}", c.enabled_filter_count());
                        }
                        Err(e) => {
                            println!("config valid: no ({})", e);
                        }
                    }
                } else {
                    println!("config file: not found (will use defaults)");
                }
            }
            None => {
                println!("config path: unable to determine config directory");
            }
        },
        Commands::Validate {
            config: config_path,
        } => {
            let path = match config_path {
                Some(p) => p,
                None => match Config::default_path() {
                    Some(p) => p,
                    None => {
                        eprintln!("error: unable to determine default config path");
                        std::process::exit(1);
                    }
                },
            };

            if !path.exists() {
                eprintln!("error: file not found: {}", path.display());
                std::process::exit(1);
            }

            match Config::load_from(&path) {
                Ok(c) => {
                    println!("valid: {}", path.display());
                    println!(
                        "  tremor: enabled={}, threshold={}px",
                        c.tremor.enabled, c.tremor.threshold
                    );
                    println!(
                        "  smoothing: enabled={}, min_cutoff={}, beta={}",
                        c.smoothing.enabled, c.smoothing.min_cutoff, c.smoothing.beta
                    );
                    println!(
                        "  dwell: enabled={}, time={}ms, radius={}px",
                        c.dwell.enabled, c.dwell.time_ms, c.dwell.radius
                    );
                    println!(
                        "  clickzones: enabled={}, zones={}",
                        c.clickzones.enabled,
                        c.clickzones.zones.len()
                    );
                }
                Err(e) => {
                    eprintln!("invalid: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn run_daemon(config: Config) {
    use steady::backend::types::BackendAction;
    use steady::backend::windows::WindowsBackend;

    let mut pipeline = FilterPipeline::from_config(&config);
    let mut backend = WindowsBackend::new();

    // Handle Ctrl+C for graceful shutdown.
    let stop_flag = backend.stop_flag.clone();
    ctrlc_handler(move || {
        stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    let result = backend.run(move |event| {
        let results = pipeline.process(event.x, event.y, event.timestamp);
        let mut actions = Vec::new();

        for r in results {
            match r {
                FilterResult::Modified(x, y) => {
                    actions.push(BackendAction::MoveTo(x, y));
                }
                FilterResult::DwellClick(x, y) => {
                    actions.push(BackendAction::Click(x, y));
                }
                FilterResult::Suppressed => {}
            }
        }

        actions
    });

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(target_os = "linux")]
fn run_daemon(config: Config) {
    use steady::backend::linux::LinuxBackend;
    use steady::backend::types::{BackendAction, InputBackend};

    let mut pipeline = FilterPipeline::from_config(&config);
    let mut backend = LinuxBackend::new();

    let result = backend.run(move |event| {
        let results = pipeline.process(event.x, event.y, event.timestamp);
        let mut actions = Vec::new();
        for r in results {
            match r {
                FilterResult::Modified(x, y) => actions.push(BackendAction::MoveTo(x, y)),
                FilterResult::DwellClick(x, y) => actions.push(BackendAction::Click(x, y)),
                FilterResult::Suppressed => {}
            }
        }
        actions
    });

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(target_os = "macos")]
fn run_daemon(config: Config) {
    use steady::backend::macos::MacOSBackend;
    use steady::backend::types::{BackendAction, InputBackend};

    let mut pipeline = FilterPipeline::from_config(&config);
    let mut backend = MacOSBackend::new();

    let result = backend.run(move |event| {
        let results = pipeline.process(event.x, event.y, event.timestamp);
        let mut actions = Vec::new();
        for r in results {
            match r {
                FilterResult::Modified(x, y) => actions.push(BackendAction::MoveTo(x, y)),
                FilterResult::DwellClick(x, y) => actions.push(BackendAction::Click(x, y)),
                FilterResult::Suppressed => {}
            }
        }
        actions
    });

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn run_daemon(_config: Config) {
    eprintln!("error: no backend available for this platform");
    std::process::exit(1);
}

#[cfg(target_os = "windows")]
fn ctrlc_handler<F: FnMut() + Send + 'static>(handler: F) {
    use std::sync::Mutex;
    use windows::core::BOOL;
    use windows::Win32::System::Console::{SetConsoleCtrlHandler, CTRL_C_EVENT};

    static HANDLER: std::sync::OnceLock<Mutex<Box<dyn FnMut() + Send>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| Mutex::new(Box::new(handler)));

    unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> BOOL {
        if ctrl_type == CTRL_C_EVENT {
            if let Some(h) = HANDLER.get() {
                if let Ok(mut f) = h.lock() {
                    f();
                }
            }
            BOOL(1)
        } else {
            BOOL(0)
        }
    }

    unsafe {
        let _ = SetConsoleCtrlHandler(Some(ctrl_handler), true);
    }
}
