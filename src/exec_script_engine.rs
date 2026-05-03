use crate::crud::exec_script;
use crate::error::{AppError, AppResult};
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecScriptRunBody {
    pub id: i64,
    pub cate: i64,
    pub interpreter: String,
    pub params: String,
    #[serde(default)]
    pub stdin: String,
    pub content: String,
    #[serde(default)]
    pub encoding: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInfoResponse {
    pub finished: i32,
    pub rst_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub params: String,
    pub stdin: String,
    pub out_str: String,
    pub err_str: String,
}

struct RunSlot {
    finished: i32,
    rst_code: i32,
    err: Option<String>,
    args_tail_display: String,
    stdin: String,
    start_time: Option<chrono::NaiveDateTime>,
    end_time: Option<chrono::NaiveDateTime>,
    stdout: Arc<Mutex<Vec<u8>>>,
    stderr: Arc<Mutex<Vec<u8>>>,
}

#[derive(Clone)]
pub struct ExecScriptEngine {
    runs: Arc<RwLock<HashMap<i64, Arc<RwLock<RunSlot>>>>>,
}

impl ExecScriptEngine {
    pub fn new() -> Self {
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn try_run(&self, req: ExecScriptRunBody) -> AppResult<()> {
        if req.interpreter.trim().is_empty() {
            return Err(AppError::BadRequest("interpreter 不能为空".into()));
        }

        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&req.params).map_err(
            |_| AppError::BadRequest("params 必须是合法 JSON 对象".into()),
        )?;

        {
            let m = self.runs.read().await;
            if let Some(existing) = m.get(&req.id) {
                let r = existing.read().await;
                if r.finished == 0 {
                    return Err(AppError::BadRequest("has other run".into()));
                }
            }
        }

        let params_map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&req.params).unwrap();

        let (script_path, temp_keep): (String, Option<tempfile::NamedTempFile>) =
            if req.cate == 1 {
                let p = req.content.trim().to_string();
                if p.is_empty() {
                    return Err(AppError::BadRequest("content 作为路径时不能为空".into()));
                }
                if !Path::new(&p).exists() {
                    return Err(AppError::BadRequest("脚本路径不存在".into()));
                }
                (p, None)
            } else {
                let mut f = tempfile::Builder::new()
                    .suffix(".exec_script")
                    .tempfile()
                    .map_err(|e| AppError::BadRequest(format!("创建临时文件失败: {e}")))?;
                use std::io::Write;
                f.write_all(req.content.as_bytes())
                    .map_err(|e| AppError::BadRequest(format!("写入临时文件失败: {e}")))?;
                f.flush().ok();
                let path = f.path().to_string_lossy().into_owned();
                (path, Some(f))
            };

        let mut argv: Vec<String> = vec![script_path];
        for (k, v) in params_map {
            argv.push(format!("--{}", k.trim()));
            argv.push(match v {
                serde_json::Value::String(s) => s,
                other => format!("{other}"),
            });
        }

        let args_tail_display = argv.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
        let stdin_display = req.stdin.trim().to_string();
        let stdin_bytes = string_to_bytes(stdin_display.as_str(), &req.encoding);
        let interpreter = req.interpreter.trim().to_string();

        let stdout_buf = Arc::new(Mutex::new(Vec::new()));
        let stderr_buf = Arc::new(Mutex::new(Vec::new()));
        let start = chrono::Utc::now().naive_utc();

        let slot = Arc::new(RwLock::new(RunSlot {
            finished: 0,
            rst_code: 0,
            err: None,
            args_tail_display,
            stdin: stdin_display,
            start_time: Some(start),
            end_time: None,
            stdout: stdout_buf.clone(),
            stderr: stderr_buf.clone(),
        }));

        {
            let mut m = self.runs.write().await;
            m.insert(req.id, slot.clone());
        }

        tokio::spawn(async move {
            let _keep_temp = temp_keep;

            let mut cmd = Command::new(&interpreter);
            cmd.args(&argv);
            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let mut w = slot.write().await;
                    w.finished = 1;
                    w.rst_code = 1;
                    w.err = Some(format!("启动进程失败: {e}"));
                    w.end_time = Some(chrono::Utc::now().naive_utc());
                    return;
                }
            };

            if let Some(mut sin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = sin.write_all(&stdin_bytes).await;
            }

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let h1 = tokio::spawn(drain_reader_opt(
                stdout,
                stdout_buf.clone(),
            ));
            let h2 = tokio::spawn(drain_reader_opt(
                stderr,
                stderr_buf.clone(),
            ));

            let status = child.wait().await;
            let _ = tokio::join!(h1, h2);

            let mut w = slot.write().await;
            w.end_time = Some(chrono::Utc::now().naive_utc());
            w.finished = 1;
            match status {
                Ok(st) => {
                    w.rst_code = st.code().unwrap_or(-1);
                }
                Err(e) => {
                    w.err = Some(e.to_string());
                    w.rst_code = -1;
                }
            }
        });

        Ok(())
    }

    pub async fn get_run_info(
        &self,
        pool: &SqlitePool,
        id: i64,
        encoding: &str,
    ) -> AppResult<RunInfoResponse> {
        let slot_arc = {
            let m = self.runs.read().await;
            m.get(&id).cloned()
        };
        let Some(slot_arc) = slot_arc else {
            return Err(AppError::BadRequest("has not run command".into()));
        };

        let slot = slot_arc.read().await;
        let out = slot.stdout.lock().await.clone();
        let err_b = slot.stderr.lock().await.clone();
        let out_str = bytes_to_string(&out, encoding);
        let err_str = bytes_to_string(&err_b, encoding);

        let finished = slot.finished;
        let rst_code = slot.rst_code;
        let err = slot.err.clone();
        let params = slot.args_tail_display.clone();
        let stdin = slot.stdin.clone();
        let start_time = slot.start_time.map(|t| t.to_string());
        let end_time = slot.end_time.map(|t| t.to_string());
        let start_t = slot.start_time;
        let end_t = slot.end_time;
        drop(slot);

        let resp = RunInfoResponse {
            finished,
            rst_code,
            err,
            start_time,
            end_time,
            params: params.clone(),
            stdin: stdin.clone(),
            out_str,
            err_str,
        };

        if finished == 1 && id > 0 {
            let exec_info = serde_json::json!({
                "stdin": stdin,
                "outStr": resp.out_str.clone(),
                "errStr": resp.err_str.clone(),
            });
            if let (Some(st), Some(et)) = (start_t, end_t) {
                let _ = exec_script::update_last_exec(
                    pool,
                    id,
                    st,
                    et,
                    &params,
                    &exec_info.to_string(),
                )
                .await;
            }
            let mut m = self.runs.write().await;
            m.remove(&id);
        }

        Ok(resp)
    }

    pub async fn is_running(&self, id: i64) -> bool {
        let m = self.runs.read().await;
        if let Some(slot_arc) = m.get(&id) {
            let slot = slot_arc.read().await;
            return slot.finished == 0;
        }
        false
    }
}

fn bytes_to_string(b: &[u8], encoding: &str) -> String {
    if encoding.eq_ignore_ascii_case("gbk") {
        encoding_rs::GBK.decode(b).0.into_owned()
    } else {
        String::from_utf8_lossy(b).into_owned()
    }
}

fn string_to_bytes(s: &str, encoding: &str) -> Vec<u8> {
    if encoding.eq_ignore_ascii_case("gbk") {
        let (cow, _, _) = encoding_rs::GBK.encode(s);
        cow.into_owned().into()
    } else {
        s.as_bytes().to_vec()
    }
}

async fn drain_reader_opt(
    reader: Option<impl tokio::io::AsyncRead + Unpin>,
    buf: Arc<Mutex<Vec<u8>>>,
) {
    let Some(mut r) = reader else {
        return;
    };
    let mut local = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match r.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                local.extend_from_slice(&chunk[..n]);
                let mut g = buf.lock().await;
                *g = local.clone();
            }
            Err(_) => break,
        }
    }
}

impl Default for ExecScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}
