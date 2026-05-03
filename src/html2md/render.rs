//! 使用 scraper 遍历 DOM 直接生成 Markdown（不再做 DOM 变异）。
use crate::html2md::format_tpl::format_str;
use regex::Regex;
use scraper::{CaseSensitivity, ElementRef, Html, Node, Selector};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static SEL_H: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h1, h2, h3, h4, h5, h6").unwrap());
static SEL_NOSCRIPT: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("noscript").unwrap());
static SEL_IMG: LazyLock<Selector> = LazyLock::new(|| Selector::parse("img").unwrap());
static SEL_OL_LI: LazyLock<Selector> = LazyLock::new(|| Selector::parse("ol li").unwrap());
static RE_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());

pub struct RenderCtx<'a> {
    pub title_prefix: &'a str,
    pub default_lang: &'a str,
    pub line_append: &'a str,
    pub h_level: HashMap<i32, usize>,
}

fn trim_space_data(s: &str, strip: bool) -> String {
    if strip {
        if s == "\n" {
            s.to_string()
        } else {
            s.trim().to_string()
        }
    } else {
        s.to_string()
    }
}

/// 与原 `outTagText` 的 line_map / line_map2 一致（不含 h6）。
fn line_maps() -> (HashSet<&'static str>, HashSet<&'static str>) {
    let line_map: HashSet<_> = [
        "br", "p", "div", "ul", "ol", "h1", "h2", "h3", "h4", "h5",
    ]
    .into_iter()
    .collect();
    let line_map2: HashSet<_> = ["p", "div", "ul", "ol", "h1", "h2", "h3", "h4", "h5"]
        .into_iter()
        .collect();
    (line_map, line_map2)
}

/// 对齐原 `outTagText(contentDiv.Nodes[0], "", true, lineAppend)`。
pub fn out_tag_text_root(root: ElementRef<'_>, ctx: &RenderCtx<'_>) -> String {
    let (line_map, line_map2) = line_maps();
    let strip = true;
    let sep = "";
    let mut texts: Vec<String> = Vec::new();
    let mut pre_child_tag: Option<String> = None;

    for child in root.children() {
        let append_sep = {
            let pre = pre_child_tag.as_deref();
            let cur_tag = match child.value().as_element() {
                Some(e) => Some(e.name()),
                None => None,
            };
            let pre_hit = pre.map(|t| line_map.contains(t)).unwrap_or(false);
            let cur_hit = cur_tag.map(|t| line_map2.contains(t)).unwrap_or(false);
            pre_hit || cur_hit
        };

        if append_sep {
            let length = texts.len();
            let start = length.saturating_sub(4);
            let joined = texts[start..length].join("");
            let has_nn = joined.ends_with("\n\n");
            let has_nnn = joined.ends_with("\n\n\n");
            if has_nn && ctx.line_append.contains('\n') && !has_nnn {
                texts.push("\n".to_string());
            } else {
                texts.push(ctx.line_append.to_string());
            }
        }

        match child.value() {
            Node::Text(t) => {
                texts.push(trim_space_data(t, strip));
                pre_child_tag = None;
            }
            Node::Element(_) => {
                let Some(el) = ElementRef::wrap(child) else {
                    continue;
                };
                let tag = el.value().name().to_string();
                pre_child_tag = Some(tag.clone());
                if tag == "script" || tag == "style" {
                    continue;
                }
                texts.push(render_element(el, ctx, strip));
            }
            _ => {}
        }
    }
    texts.join(sep)
}

fn render_children_inline(el: ElementRef<'_>, ctx: &RenderCtx<'_>, strip: bool) -> String {
    let mut out = String::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => out.push_str(&trim_space_data(t, strip)),
            Node::Element(_) => {
                if let Some(ce) = ElementRef::wrap(child) {
                    let tag = ce.value().name();
                    if tag == "script" || tag == "style" {
                        continue;
                    }
                    out.push_str(&render_element(ce, ctx, strip));
                }
            }
            _ => {}
        }
    }
    out
}

fn normalize_img_url(url: &str) -> String {
    let mut url = url.trim().to_string();
    if url.starts_with("//") {
        url = format!("https:{url}");
    }
    url
}

fn collect_heading_levels(html: &Html) -> HashMap<i32, usize> {
    let mut nums: Vec<i32> = Vec::new();
    for el in html.select(&SEL_H) {
        let name = el.value().name();
        if let Ok(n) = name.trim_start_matches('h').parse::<i32>() {
            nums.push(n);
        }
    }
    nums.sort_unstable();
    let mut h_level = HashMap::new();
    let mut max_level = 0usize;
    for num in nums {
        h_level.entry(num).or_insert_with(|| {
            let l = max_level;
            max_level += 1;
            l
        });
    }
    h_level
}

pub fn html_fragment_to_md(
    html: &str,
    title_prefix: &str,
    default_lang: &str,
    line_append: &str,
) -> String {
    let document = Html::parse_fragment(html);
    let h_level = collect_heading_levels(&document);
    let ctx = RenderCtx {
        title_prefix,
        default_lang,
        line_append,
        h_level,
    };
    let root = document.root_element();
    out_tag_text_root(root, &ctx)
}

fn render_element(el: ElementRef<'_>, ctx: &RenderCtx<'_>, strip: bool) -> String {
    let tag = el.value().name();

    match tag {
        "figure" => {
            if let Some(ns) = el.select(&SEL_NOSCRIPT).next() {
                let raw = ns.inner_html();
                let t = raw.trim();
                if t.to_ascii_lowercase().starts_with("<img") {
                    let frag = Html::parse_fragment(t);
                    if let Some(img) = frag.select(&SEL_IMG).next() {
                        return render_img_el(&img);
                    }
                }
            }
            out_tag_text_root(el, ctx)
        }
        "br" => "\n".to_string(),
        "a" => {
            let href = el.attr("href").unwrap_or("").trim().to_string();
            let text = render_children_inline(el, ctx, strip)
                .trim()
                .to_string();
            if href.is_empty() && text.is_empty() {
                return String::new();
            }
            if href.contains("so.csdn.net/so/search") {
                text
            } else {
                format_str(
                    "[{1}]({0} \"{1}\")",
                    &[href.clone(), text.clone()],
                )
            }
        }
        "img" => render_img_el(&el),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let num: i32 = tag.trim_start_matches('h').parse().unwrap_or(1);
            let extra = ctx.h_level.get(&num).copied().unwrap_or(0);
            let hashes = "#".repeat(extra);
            let inner = render_children_inline(el, ctx, strip)
                .trim()
                .to_string();
            format!("{}{hashes} {inner}", ctx.title_prefix)
        }
        "b" | "strong" => {
            let inner = render_children_inline(el, ctx, strip)
                .trim()
                .to_string();
            format!("**{inner}**")
        }
        "blockquote" => {
            let inner = el
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            format!("> {inner}\n\n")
        }
        "ul" | "ol" => render_list(el, ctx, strip),
        "table" => render_table(el),
        "pre" => render_pre(el, ctx),
        "code" => {
            if inside_parent(el, "pre") {
                String::new()
            } else {
                let code = render_children_inline(el, ctx, false);
                if code.contains('\n') {
                    format!("```{}\n{code}\n```", ctx.default_lang)
                } else {
                    format!("`{code}`")
                }
            }
        }
        "head" => String::new(),
        _ => {
            if is_blockish(tag) {
                out_tag_text_root(el, ctx)
            } else {
                render_children_inline(el, ctx, strip)
            }
        }
    }
}

fn inside_parent(el: ElementRef<'_>, parent_tag: &str) -> bool {
    let mut cur = el.parent();
    while let Some(p) = cur {
        if let Some(pe) = ElementRef::wrap(p) {
            if pe.value().name() == parent_tag {
                return true;
            }
            cur = pe.parent();
        } else {
            break;
        }
    }
    false
}

fn is_blockish(tag: &str) -> bool {
    matches!(
        tag,
        "html"
            | "body"
            | "article"
            | "section"
            | "nav"
            | "aside"
            | "main"
            | "div"
            | "p"
            | "form"
            | "fieldset"
            | "header"
            | "footer"
            | "address"
            | "dl"
            | "dt"
            | "dd"
    )
}

fn render_img_el(el: &ElementRef<'_>) -> String {
    if el
        .value()
        .classes()
        .any(|c| c == "look-more-preCode")
    {
        return String::new();
    }
    let url = el.attr("src").unwrap_or("");
    let url = normalize_img_url(url);
    format_str(" ![]({0}) ", &[url.to_string()])
}

fn render_list(el: ElementRef<'_>, ctx: &RenderCtx<'_>, strip: bool) -> String {
    let mut not_digit = false;
    let mut texts: Vec<String> = Vec::new();
    for li in el.child_elements() {
        if li.value().name() != "li" {
            continue;
        }
        let plain_check = li
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        if !not_digit && !RE_NUM.is_match(&plain_check) {
            not_digit = true;
        }
        let t = render_children_inline(li, ctx, strip).trim().to_string();
        texts.push(format!("* {t}"));
    }
    if not_digit {
        texts.join("    \n")
    } else {
        String::new()
    }
}

fn render_table(el: ElementRef<'_>) -> String {
    let sel_tr = Selector::parse("tr").unwrap();
    let sel_cell = Selector::parse("td, th").unwrap();
    let mut txts: Vec<String> = Vec::new();
    for (i, tr) in el.select(&sel_tr).enumerate() {
        let cells: Vec<String> = tr
            .select(&sel_cell)
            .map(|cell| {
                cell.text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .replace('|', "&#124;")
            })
            .collect();
        let line = format!("| {} |", cells.join(" | "));
        txts.push(line);
        if i == 0 {
            let sep = cells.iter().map(|_| "--").collect::<Vec<_>>().join(" | ");
            txts.push(format!("| {} |", sep));
        }
    }
    txts.join("\n")
}

fn lines_all_numeric(lines: &[String]) -> bool {
    lines.iter().all(|line| {
        let line = line.trim();
        line.is_empty() || RE_NUM.is_match(line)
    })
}

fn el_has_class(el: ElementRef<'_>, class: &str) -> bool {
    el.value()
        .has_class(class, CaseSensitivity::CaseSensitive)
}

fn collect_pre_text(el: ElementRef<'_>) -> String {
    let mut s = String::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => s.push_str(t),
            Node::Element(_) => {
                let ce = ElementRef::wrap(child).unwrap();
                let tag = ce.value().name();
                if tag == "ul" && el_has_class(ce, "pre-numbering") {
                    continue;
                }
                if tag == "code" && el_has_class(ce, "hljs-line-numbers") {
                    let inner = ce.text().collect::<Vec<_>>().join("\n");
                    let lines: Vec<String> =
                        inner.split('\n').map(|s| s.to_string()).collect();
                    if lines_all_numeric(&lines) {
                        continue;
                    }
                }
                s.push_str(&collect_pre_text(ce));
            }
            _ => {}
        }
    }
    s
}

fn try_rewrite_pre_from_ol(el: ElementRef<'_>, code: &str) -> Option<String> {
    let lis: Vec<_> = el.select(&SEL_OL_LI).collect();
    if lis.is_empty() {
        return None;
    }
    let ol_text = lis
        .iter()
        .map(|li| li.text().collect::<Vec<_>>().join(""))
        .collect::<String>()
        .trim()
        .to_string();
    if ol_text != code.trim() {
        return None;
    }
    let mut sb = String::new();
    for (i, li) in lis.iter().enumerate() {
        if i != 0 {
            sb.push('\n');
        }
        sb.push_str(&li.text().collect::<Vec<_>>().join(""));
    }
    Some(sb)
}

fn render_pre(el: ElementRef<'_>, ctx: &RenderCtx<'_>) -> String {
    let mut code = collect_pre_text(el);
    if let Some(rewritten) = try_rewrite_pre_from_ol(el, &code) {
        code = rewritten;
    }
    let code = code.trim_end().to_string();
    format!("```{}\n{code}\n```", ctx.default_lang)
}
