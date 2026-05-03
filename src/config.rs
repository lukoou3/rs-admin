pub struct Config {
    pub database_url: String,
    pub listen: String,
    pub jwt_secret: String,
}

pub fn load() -> Config {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "sqlite:D:/apps/gin-vue-admin/simple/server/sqlite.db".to_string()
    });
    let listen = std::env::var("LISTEN").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "rs-admin-dev-change-me-in-production".to_string());
    Config {
        database_url,
        listen,
        jwt_secret,
    }
}
