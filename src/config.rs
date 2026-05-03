use std::env;
use std::path::PathBuf;
use std::time::Duration;

pub struct Config {
    pub database_url: String,
    pub listen: String,
    pub jwt_secret: String,
    pub maintenance: MaintenanceConfig,
}

/// SQLite 维护：启动 VACUUM、定时物理清除软删行 + VACUUM（对齐 gin-vue-admin 习惯）。
#[derive(Clone, Copy)]
pub struct MaintenanceConfig {
    pub sqlite: bool,
    pub startup_vacuum: bool,
    pub scheduled: bool,
    pub interval: Duration,
}

fn is_sqlite_url(url: &str) -> bool {
    url.trim_start()
        .to_ascii_lowercase()
        .starts_with("sqlite:")
}

fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(s) => {
            let t = s.trim().to_ascii_lowercase();
            matches!(t.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => default,
    }
}

fn maintenance_interval() -> Duration {
    let secs: u64 = env::var("MAINTENANCE_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n >= 60)
        .unwrap_or(86400);
    Duration::from_secs(secs)
}

pub struct CliArgs {
    pub listen: Option<String>,
    pub database: Option<String>,
}

impl CliArgs {
    pub fn parse() -> Self {
        let mut listen = None;
        let mut database = None;
        let mut it = env::args().skip(1);
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--listen" | "-l" => {
                    if let Some(v) = it.next() {
                        listen = Some(v);
                    }
                }
                "--database" | "--db" | "-d" => {
                    if let Some(v) = it.next() {
                        database = Some(v);
                    }
                }
                _ => {}
            }
        }
        Self { listen, database }
    }
}

fn default_database_url() -> String {
    "sqlite:./rs-admin.db".to_string()
}

fn normalize_database_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return default_database_url();
    }
    if trimmed.starts_with("sqlite:") {
        return trimmed.to_string();
    }
    let p = PathBuf::from(trimmed).to_string_lossy().replace('\\', "/");
    format!("sqlite:{p}")
}

pub fn load(args: &CliArgs) -> Config {
    let database_url = args
        .database
        .as_deref()
        .map(normalize_database_url)
        .or_else(|| env::var("DATABASE_URL").ok().map(|v| normalize_database_url(&v)))
        .unwrap_or_else(default_database_url);
    let listen = args
        .listen
        .clone()
        .or_else(|| env::var("LISTEN").ok())
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "rs-admin-dev-change-me-in-production".to_string());

    let sqlite = is_sqlite_url(&database_url);
    let maintenance_off = env_bool("MAINTENANCE_DISABLE", false);
    let maintenance = MaintenanceConfig {
        sqlite: sqlite && !maintenance_off,
        startup_vacuum: sqlite && !maintenance_off && env_bool("MAINTENANCE_STARTUP_VACUUM", true),
        scheduled: sqlite && !maintenance_off && env_bool("MAINTENANCE_SCHEDULED", true),
        interval: maintenance_interval(),
    };

    Config {
        database_url,
        listen,
        jwt_secret,
        maintenance,
    }
}
