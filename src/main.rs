use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_rolling_file::{RollingConditionBase, RollingFileAppenderBase};
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, format::Writer, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// 日志行内时间戳：本地时间（与 SQLite `localtime` 等业务一致）。
#[derive(Clone, Copy)]
struct LocalTimeFmt;

impl FormatTime for LocalTimeFmt {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(
            w,
            "{}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
        )
    }
}

fn init_env() {
    dotenvy::dotenv().ok();
    let env_file = rs_admin::app_dir().join(".env");
    dotenvy::from_path(env_file).ok();
}

fn init_logging() -> Result<Option<WorkerGuard>> {
    let log_dir_raw = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    let log_dir = rs_admin::resolve_relative_path(&log_dir_raw);
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("无法创建日志目录 {}: {e}", log_dir.display());
    }

    let max_file_size = std::env::var("LOG_MAX_FILE_SIZE")
        .unwrap_or_else(|_| "52428800".to_string())
        .parse::<u64>()
        .unwrap_or(50 * 1024 * 1024);

    let max_file_count = std::env::var("LOG_MAX_FILE_COUNT")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<usize>()
        .unwrap_or(3);

    let log_filename = log_dir.join("rs-admin.log");

    let timer = LocalTimeFmt;
    let mirror_stderr = std::env::var("LOG_MIRROR_STDERR")
        .map(|v| {
            let t = v.trim().to_ascii_lowercase();
            matches!(t.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(true);

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn"));

    match RollingFileAppenderBase::new(
        log_filename.clone(),
        RollingConditionBase::new().max_size(max_file_size),
        max_file_count,
    ) {
        Ok(file_appender) => {
            let (non_blocking, log_guard) = file_appender.get_non_blocking_appender();
            let file_layer = fmt::layer()
                .with_timer(timer)
                .with_writer(non_blocking)
                .with_ansi(false);
            if mirror_stderr {
                Registry::default()
                    .with(env_filter)
                    .with(file_layer)
                    .with(
                        fmt::layer()
                            .with_timer(timer)
                            .with_writer(std::io::stderr)
                            .with_ansi(true),
                    )
                    .init();
            } else {
                Registry::default().with(env_filter).with(file_layer).init();
            }
            return Ok(Some(log_guard));
        }
        Err(e) => {
            eprintln!(
                "日志文件初始化失败，退回到标准错误输出\n  path: {}\n  error: {e}",
                log_filename.display()
            );
            Registry::default()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_timer(timer)
                        .with_writer(std::io::stderr)
                        .with_ansi(true),
                )
                .init();
            return Ok(None);
        }
    }
}

fn main() -> Result<()> {
    init_env();
    let _log_guard = init_logging()?;

    let args = rs_admin::CliArgs::parse();

    match args.mode {
        rs_admin::AppMode::Service(rs_admin::ServiceCommand::Install) => {
            rs_admin::service::install(&args)?;
            tracing::info!("Windows Service installed");
            return Ok(());
        }
        rs_admin::AppMode::Service(rs_admin::ServiceCommand::Uninstall) => {
            rs_admin::service::uninstall()?;
            tracing::info!("Windows Service uninstalled");
            return Ok(());
        }
        rs_admin::AppMode::Service(rs_admin::ServiceCommand::Start) => {
            rs_admin::service::start()?;
            tracing::info!("Windows Service started");
            return Ok(());
        }
        rs_admin::AppMode::Service(rs_admin::ServiceCommand::Stop) => {
            rs_admin::service::stop()?;
            tracing::info!("Windows Service stopped");
            return Ok(());
        }
        rs_admin::AppMode::Run => {}
    }

    if rs_admin::service::try_run_dispatcher()? {
        return Ok(());
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.worker_threads)
        .thread_stack_size(args.thread_stack_size)
        .enable_all()
        .build()
        .context("创建 Tokio runtime 失败")?;
    tracing::info!(
        worker_threads = args.worker_threads,
        thread_stack_size = args.thread_stack_size,
        "Tokio runtime configured"
    );
    rt.block_on(rs_admin::run(args))
}
