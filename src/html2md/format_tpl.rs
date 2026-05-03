//! 对应 Go `formatStr`，占位符 `{0}`、`{1}` 或 `{}` 顺序填充。
use regex::Regex;
use std::sync::LazyLock;

static RE_FMT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{(\d*)\}").unwrap());

pub fn format_str(template: &str, args: &[String]) -> String {
    let mut i = 0usize;
    RE_FMT
        .replace_all(template, |caps: &regex::Captures| {
            let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let out = if inner.is_empty() {
                let idx = i;
                i += 1;
                args.get(idx).cloned().unwrap_or_default()
            } else if let Ok(idx) = inner.parse::<usize>() {
                args.get(idx).cloned().unwrap_or_default()
            } else {
                String::new()
            };
            out
        })
        .into_owned()
}
