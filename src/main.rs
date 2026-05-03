use anyhow::Result;
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("无法创建日志目录 {log_dir}: {e}");
    }

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn"));

    let max_file_size = std::env::var("LOG_MAX_FILE_SIZE")
        .unwrap_or_else(|_| "52428800".to_string())
        .parse::<u64>()
        .unwrap_or(50 * 1024 * 1024);

    let max_file_count = std::env::var("LOG_MAX_FILE_COUNT")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<usize>()
        .unwrap_or(3);

    let log_filename = std::path::PathBuf::from(&log_dir).join("rs-admin.log");

    let file_appender = RollingFileAppenderBase::new(
        log_filename,
        RollingConditionBase::new().max_size(max_file_size),
        max_file_count,
    )
    .unwrap();

    let (non_blocking, _log_guard) = file_appender.get_non_blocking_appender();

    let timer = LocalTimeFmt;

    let file_layer = fmt::layer()
        .with_timer(timer)
        .with_writer(non_blocking)
        .with_ansi(false);

    let stderr_layer = fmt::layer()
        .with_timer(timer)
        .with_writer(std::io::stderr)
        .with_ansi(true);

    let mirror_stderr = std::env::var("LOG_MIRROR_STDERR")
        .map(|v| {
            let t = v.trim().to_ascii_lowercase();
            matches!(t.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(true);

    let subscriber = Registry::default().with(env_filter).with(file_layer);
    if mirror_stderr {
        subscriber.with(stderr_layer).init();
    } else {
        subscriber.init();
    }

    let args = rs_admin::CliArgs::parse();
    rs_admin::run(args).await
}
