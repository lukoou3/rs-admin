use std::env;
use std::path::PathBuf;

pub struct Config {
    pub database_url: String,
    pub listen: String,
    pub jwt_secret: String,
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
    Config {
        database_url,
        listen,
        jwt_secret,
    }
}
