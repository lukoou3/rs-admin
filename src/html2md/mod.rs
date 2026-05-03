//! HTML → Markdown，对齐 gin-vue-admin `server/service/tools/html2md_util.go`。

mod download;
mod format_tpl;
mod render;

use serde::Deserialize;

pub use download::markdown_use_local_img;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html2mdReq {
    #[serde(default)]
    pub title_prefix: String,
    #[serde(default)]
    pub line_append: String,
    #[serde(default)]
    pub default_lang: String,
    #[serde(default)]
    pub img_path: String,
    pub download_img: i8,
    pub r#type: i8,
    #[serde(default)]
    pub html: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Html2mdResp {
    pub md: String,
    pub img_cnt: usize,
}

pub struct Html2mdUtil {
    pub title_prefix: String,
    pub line_append: String,
    pub default_lang: String,
    pub download_img: i8,
    pub img_path: String,
}

impl Default for Html2mdUtil {
    fn default() -> Self {
        Self {
            title_prefix: "##".into(),
            line_append: "\n\n".into(),
            default_lang: "go".into(),
            download_img: 0,
            img_path: String::new(),
        }
    }
}

impl Html2mdUtil {
    pub fn html_to_md(&self, html: &str) -> anyhow::Result<String> {
        let mut text = render::html_fragment_to_md(
            html,
            &self.title_prefix,
            &self.default_lang,
            &self.line_append,
        );
        text = text.replace('\u{a0}', "");
        Ok(text)
    }

    pub fn md_to_md(&self, text: &str) -> String {
        text.to_string()
    }
}

pub async fn run_html2md(
    client: &reqwest::Client,
    req: &Html2mdReq,
) -> anyhow::Result<Html2mdResp> {
    let mut util = Html2mdUtil::default();
    apply_req(&mut util, req);

    if req.r#type == 0 {
        if req.download_img == 1 && req.img_path.trim().is_empty() {
            anyhow::bail!("imgPath is empty");
        }
        let md = util.html_to_md(&req.html)?;
        let (md, img_cnt) = if req.download_img == 1 {
            markdown_use_local_img(client, md, &req.img_path).await?
        } else {
            (md, 0usize)
        };
        Ok(Html2mdResp { md, img_cnt })
    } else {
        if req.img_path.trim().is_empty() {
            anyhow::bail!("imgPath is empty");
        }
        let text = util.md_to_md(&req.html);
        let (md, img_cnt) = markdown_use_local_img(client, text, &req.img_path).await?;
        Ok(Html2mdResp { md, img_cnt })
    }
}

pub fn apply_req(util: &mut Html2mdUtil, req: &Html2mdReq) {
    if !req.line_append.is_empty() {
        util.line_append = req.line_append.clone();
    }
    if !req.title_prefix.is_empty() {
        util.title_prefix = req.title_prefix.clone();
    }
    if !req.default_lang.is_empty() {
        util.default_lang = req.default_lang.clone();
    }
    util.download_img = req.download_img;
    util.img_path = req.img_path.clone();
}

#[cfg(test)]
mod tests {
    use super::Html2mdUtil;

    #[test]
    fn html_to_md_paragraph() {
        let u = Html2mdUtil::default();
        let r = u.html_to_md("<p>hello</p>").unwrap();
        assert!(r.contains("hello"));
    }

    #[test]
    fn html_to_md_list_inline_markdown() {
        let u = Html2mdUtil::default();
        let r = u.html_to_md("<ul><li>a <strong>b</strong></li></ul>").unwrap();
        assert!(r.contains("**b**"));
    }
}
