//! 下载 Markdown 中的远程图片并替换为本地 `assets/文件名`，对齐 Go `markdownUseLocalImg`。
use regex::Regex;
use reqwest::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

static RE_IMG1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\[\]\((http.*?)\)").unwrap());
static RE_IMG2: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[image-\d*\]\((http.*?)\)").unwrap());

fn bytes_md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

async fn download_img(
    client: &Client,
    url: &str,
    img_path: &std::path::Path,
) -> anyhow::Result<String> {
    if !img_path.exists() {
        std::fs::create_dir_all(img_path)?;
    }
    let mut suffix = ".png";
    if let Some(pos) = url.rfind('.') {
        let suf = &url[pos..];
        if suf == ".jpg" || suf == ".png" || suf == ".gif" {
            suffix = suf;
        }
    }
    let mut last_err = None;
    for _ in 0..5 {
        match client.get(url).send().await {
            Ok(res) => match res.bytes().await {
                Ok(bytes) => {
                    let name = format!("{}{}", bytes_md5_hex(&bytes), suffix);
                    let full = img_path.join(&name);
                    if !full.exists() {
                        std::fs::write(&full, &bytes)?;
                    }
                    return Ok(name);
                }
                Err(e) => last_err = Some(e.to_string()),
            },
            Err(e) => last_err = Some(e.to_string()),
        }
    }
    Err(anyhow::anyhow!(
        "download failed after retries: {:?}",
        last_err
    ))
}

pub async fn markdown_use_local_img(
    client: &Client,
    mut text: String,
    img_path: &str,
) -> anyhow::Result<(String, usize)> {
    let base = PathBuf::from(img_path);
    let mut url_map: HashMap<String, ()> = HashMap::new();
    for caps in RE_IMG1.captures_iter(&text) {
        if let Some(u) = caps.get(1) {
            url_map.insert(u.as_str().to_string(), ());
        }
    }
    for caps in RE_IMG2.captures_iter(&text) {
        if let Some(u) = caps.get(1) {
            url_map.insert(u.as_str().to_string(), ());
        }
    }
    let urls: Vec<String> = url_map.into_keys().collect();
    let cnt = urls.len();

    for url in urls {
        match download_img(client, &url, &base).await {
            Ok(name) => {
                let local = format!("assets/{name}");
                text = text.replace(&url, &local);
            }
            Err(e) => tracing::warn!("html2md img download: {} {}", url, e),
        }
    }

    Ok((text, cnt))
}
