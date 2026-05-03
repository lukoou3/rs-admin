use sqlx::SqlitePool;

async fn exec(pool: &SqlitePool, sql: &str) -> Result<(), sqlx::Error> {
    sqlx::query(sql).execute(pool).await?;
    Ok(())
}

async fn run_statements(pool: &SqlitePool, statements: &[&str]) -> Result<(), sqlx::Error> {
    for sql in statements {
        exec(pool, sql).await?;
    }
    Ok(())
}

fn current_tables() -> [&'static str; 8] {
    [
        r#"
        CREATE TABLE IF NOT EXISTS code_shellcode (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            name text UNIQUE,
            code text,
            "desc" text,
            created_by integer,
            updated_by integer,
            deleted_by integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS exec_script (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            name text,
            cate integer,
            interpreter text,
            encoding text,
            default_params text,
            content text,
            "desc" text,
            last_exec_start_time datetime,
            last_exec_end_time datetime,
            last_exec_params text,
            last_exec_info text,
            created_by integer,
            updated_by integer,
            deleted_by integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sql_datasource (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            name text UNIQUE,
            alias text,
            cate integer,
            introduction text,
            sql text,
            created_by integer,
            updated_by integer,
            deleted_by integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sql_querysql (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            name text UNIQUE,
            cate integer,
            sql text,
            "desc" text,
            created_by integer,
            updated_by integer,
            deleted_by integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_dictionaries (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            name text,
            type text,
            status numeric,
            "desc" text,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_dictionary_details (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            label text,
            value integer,
            status numeric,
            sort integer,
            sys_dictionary_id integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_operation_records (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            ip text,
            method text,
            path text,
            status integer,
            latency integer,
            agent text,
            error_message text,
            body text,
            resp text,
            user_id integer,
            PRIMARY KEY (id)
        )
        "#,
        r##"
        CREATE TABLE IF NOT EXISTS sys_users (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            uuid text,
            username text,
            password text,
            nick_name text DEFAULT "系统用户",
            side_mode text DEFAULT "dark",
            header_img text DEFAULT "https://qmplusimg.henrongyi.top/gva_header.jpg",
            base_color text DEFAULT "#fff",
            active_color text DEFAULT "#1890ff",
            authority_id integer DEFAULT 888,
            phone text,
            email text,
            enable integer DEFAULT 1,
            PRIMARY KEY (id)
        )
        "##,
    ]
}

fn legacy_tables() -> [&'static str; 19] {
    [
        r#"
        CREATE TABLE IF NOT EXISTS casbin_rule (
            id integer,
            ptype text,
            v0 text,
            v1 text,
            v2 text,
            v3 text,
            v4 text,
            v5 text,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS code_template (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            name text UNIQUE,
            engine integer,
            cate integer,
            default_params text,
            temp text,
            "desc" text,
            created_by integer,
            updated_by integer,
            deleted_by integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS exa_customers (
            id INTEGER,
            created_at VARCHAR(50),
            updated_at VARCHAR(50),
            deleted_at VARCHAR(50),
            customer_name INTEGER,
            customer_phone_data INTEGER,
            sys_user_id INTEGER,
            sys_user_authority_id INTEGER
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS exa_file_chunks (
            id VARCHAR(50),
            created_at VARCHAR(50),
            updated_at VARCHAR(50),
            deleted_at VARCHAR(50),
            exa_file_id VARCHAR(50),
            file_chunk_number VARCHAR(50),
            file_chunk_path VARCHAR(50)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS exa_file_upload_and_downloads (
            id INTEGER,
            created_at VARCHAR(50),
            updated_at VARCHAR(50),
            deleted_at VARCHAR(50),
            name VARCHAR(50),
            url VARCHAR(64),
            tag VARCHAR(50),
            "key" VARCHAR(50)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS exa_files (
            id VARCHAR(50),
            created_at VARCHAR(50),
            updated_at VARCHAR(50),
            deleted_at VARCHAR(50),
            file_name VARCHAR(50),
            file_md5 VARCHAR(50),
            file_path VARCHAR(50),
            chunk_total VARCHAR(50),
            is_finish VARCHAR(50)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS jwt_blacklists (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            jwt text,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_apis (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            path text,
            description text,
            api_group text,
            method text DEFAULT "POST",
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_authorities (
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            authority_id integer NOT NULL UNIQUE,
            authority_name text,
            parent_id integer,
            default_router text DEFAULT "dashboard",
            PRIMARY KEY (authority_id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_authority_btns (
            authority_id integer,
            sys_menu_id integer,
            sys_base_menu_btn_id integer
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_authority_menus (
            sys_base_menu_id integer,
            sys_authority_authority_id integer NOT NULL,
            PRIMARY KEY (sys_base_menu_id, sys_authority_authority_id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_auto_code_histories (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            package text,
            business_db text,
            table_name text,
            request_meta text,
            auto_code_path text,
            injection_meta text,
            struct_name text,
            struct_cn_name text,
            api_ids text,
            flag integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_auto_codes (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            package_name text,
            label text,
            "desc" text,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_base_menu_btns (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            name text,
            "desc" text,
            sys_base_menu_id integer,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_base_menu_parameters (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            sys_base_menu_id integer,
            type text,
            "key" text,
            value text,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_base_menus (
            id integer,
            created_at datetime,
            updated_at datetime,
            deleted_at datetime,
            menu_level integer,
            parent_id text,
            path text,
            name text,
            hidden numeric,
            component text,
            sort integer,
            active_name text,
            keep_alive numeric,
            default_menu numeric,
            title text,
            icon text,
            close_tab numeric,
            PRIMARY KEY (id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_chat_gpt_options (
            sk VARCHAR(50)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_data_authority_id (
            sys_authority_authority_id integer NOT NULL,
            data_authority_id_authority_id integer NOT NULL,
            PRIMARY KEY (sys_authority_authority_id, data_authority_id_authority_id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS sys_user_authority (
            sys_user_id integer,
            sys_authority_authority_id integer NOT NULL,
            PRIMARY KEY (sys_user_id, sys_authority_authority_id)
        )
        "#,
    ]
}

/// 当前项目的 SQLite 表初始化入口。
/// 分组依据：
/// - `current_tables()`: 本 Rust 项目直接使用的表
/// - `legacy_tables()`: 当前数据库里保留的旧项目/兼容表
pub async fn ensure_all_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    run_statements(pool, &current_tables()).await?;
    run_statements(pool, &legacy_tables()).await?;
    Ok(())
}
