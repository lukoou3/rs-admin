//! DOM 转换步骤，顺序与 Go `Html2Md` 一致。
use crate::html2md::format_tpl::format_str;
use nipper::{Document, Selection};
use regex::Regex;
use std::sync::LazyLock;

fn wrap_md_fragment(md: &str) -> String {
    format!(
        "<span>{}</span>",
        md.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
    )
}

pub fn convert_figure(doc: &mut Document) {
    while doc.select("figure").exists() {
        let mut sel = doc.select("figure").first();
        let nos = sel.select("noscript");
        if !nos.exists() {
            sel.remove();
            continue;
        }
        let text = nos.text().trim().to_string();
        if text.to_ascii_lowercase().starts_with("<img") {
            sel.replace_with_html(text);
        } else {
            sel.remove();
        }
    }
}

pub fn convert_br(doc: &mut Document) {
    while doc.select("br").exists() {
        let mut sel = doc.select("br").first();
        sel.replace_with_html("\n");
    }
}

pub fn convert_link(doc: &mut Document) {
    while doc.select("a").exists() {
        let mut sel = doc.select("a").first();
        let href = sel
            .attr("href")
            .map(|t| t.trim().to_string())
            .unwrap_or_default();
        let text = sel.text().trim().to_string();
        if href.is_empty() && text.is_empty() {
            sel.remove();
            continue;
        }
        if href.contains("so.csdn.net/so/search") {
            sel.replace_with_html(wrap_md_fragment(&text));
        } else {
            let md = format_str(
                "[{1}]({0} \"{1}\")",
                &[href.clone(), text.clone()],
            );
            sel.replace_with_html(wrap_md_fragment(&md));
        }
    }
}

pub fn convert_img(doc: &mut Document) {
    while doc.select("img.look-more-preCode").exists() {
        doc.select("img.look-more-preCode").first().remove();
    }
    while doc.select("img").exists() {
        let mut sel = doc.select("img").first();
        let mut url = sel.attr("src").map(|t| t.to_string()).unwrap_or_default();
        url = url.trim().to_string();
        if url.starts_with("//") {
            url = format!("https:{url}");
        }
        let md = format_str(" ![]({0}) ", &[url]);
        sel.replace_with_html(wrap_md_fragment(&md));
    }
}

pub fn convert_htag(doc: &mut Document, title_prefix: &str) {
    let hs = doc.select("h1, h2, h3, h4, h5, h6");
    if !hs.exists() {
        return;
    }
    let mut nums: Vec<i32> = Vec::new();
    for s in hs.iter() {
        let Some(name) = s.nodes().first().and_then(|n| n.node_name()) else {
            continue;
        };
        let name = name.trim();
        let num_str = name.trim_start_matches('h');
        if let Ok(num) = num_str.parse::<i32>() {
            nums.push(num);
        }
    }
    nums.sort_unstable();
    let mut h_level: std::collections::HashMap<i32, usize> =
        std::collections::HashMap::new();
    let mut max_level = 0usize;
    for num in nums {
        h_level.entry(num).or_insert_with(|| {
            let l = max_level;
            max_level += 1;
            l
        });
    }

    while doc
        .select("h1, h2, h3, h4, h5, h6")
        .exists()
    {
        let mut sel = doc.select("h1, h2, h3, h4, h5, h6").first();
        let Some(name) = sel.nodes().first().and_then(|n| n.node_name()) else {
            sel.remove();
            continue;
        };
        let num_str = name.trim().trim_start_matches('h');
        let Ok(num) = num_str.parse::<i32>() else {
            sel.remove();
            continue;
        };
        let extra = h_level.get(&num).copied().unwrap_or(0);
        let hashes = "#".repeat(extra);
        let inner = sel.text().trim().to_string();
        let text = format!("{title_prefix}{hashes} {inner}");
        sel.replace_with_html(wrap_md_fragment(&text));
    }
}

pub fn convert_btag(doc: &mut Document) {
    for tag in ["b", "strong"] {
        while doc.select(tag).exists() {
            let mut sel = doc.select(tag).first();
            let inner = sel.text().trim().to_string();
            let md = format!("**{inner}**");
            sel.replace_with_html(wrap_md_fragment(&md));
        }
    }
}

static RE_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());

pub fn convert_ul(doc: &mut Document) {
    for list_tag in ["ul", "ol"] {
        while doc.select(list_tag).exists() {
            let mut sel = doc.select(list_tag).first();
            let mut not_digit = false;
            let mut texts: Vec<String> = Vec::new();
            let Some(ul) = sel.nodes().first() else {
                sel.remove();
                continue;
            };
            for child in ul.children() {
                if child.node_name().map(|n| n.trim().to_string()).as_deref() != Some("li") {
                    continue;
                }
                let mut li_sel = Selection::from(child);
                let t = li_sel.text().trim().to_string();
                if !not_digit && !RE_NUM.is_match(&t) {
                    not_digit = true;
                }
                texts.push(format!("* {t}"));
            }
            if not_digit {
                let joined = texts.join("    \n");
                sel.replace_with_html(wrap_md_fragment(&joined));
            } else {
                sel.remove();
            }
        }
    }
}

fn lines_all_numeric(lines: &[String]) -> bool {
    lines.iter().all(|line| {
        let line = line.trim();
        line.is_empty() || RE_NUM.is_match(line)
    })
}

pub fn convert_code(doc: &mut Document, default_lang: &str) {
    while doc.select("code, pre").exists() {
        let mut sel = doc.select("code, pre").first();

        while sel.select("code.hljs-line-numbers").exists() {
            let mut inner = sel.select("code.hljs-line-numbers").first();
            let inner_txt = inner.text().to_string();
            let lines: Vec<String> = inner_txt.split('\n').map(|s| s.to_string()).collect();
            if lines_all_numeric(&lines) {
                inner.remove();
            } else {
                break;
            }
        }

        while sel.select("ul.pre-numbering").exists() {
            sel.select("ul.pre-numbering").first().remove();
        }

        let mut code = sel.text().to_string();

        let ol_text = sel.select("ol li").text().trim().to_string();
        if sel.select("ol li").exists() && ol_text == code.trim() {
            let mut sb = String::new();
            let mut i = 0usize;
            while sel.select("ol li").exists() {
                let mut li = sel.select("ol li").first();
                if i != 0 {
                    sb.push('\n');
                }
                sb.push_str(&li.text());
                li.remove();
                i += 1;
            }
            code = sb;
        }

        let md = if code.contains('\n') {
            format!("```{default_lang}\n{code}\n```")
        } else {
            format!("`{code}`")
        };

        sel.replace_with_html(wrap_md_fragment(&md));
    }
}

pub fn convert_blockquote(doc: &mut Document) {
    while doc.select("blockquote").exists() {
        let mut sel = doc.select("blockquote").first();
        let inner = sel.text().trim().to_string();
        let md = format!("> {inner}\n\n");
        sel.replace_with_html(wrap_md_fragment(md.trim()));
    }
}

pub fn convert_table(doc: &mut Document) {
    while doc.select("table").exists() {
        let mut table_sel = doc.select("table").first();
        let mut txts: Vec<String> = Vec::new();
        let mut row_i = 0usize;
        while table_sel.select("tr").exists() {
            let mut tr = table_sel.select("tr").first();
            let texts_row: Vec<String> = tr
                .select("td, th")
                .iter()
                .map(|mut cell| cell.text().trim().replace('|', "&#124;"))
                .collect();
            let line = format!("| {} |", texts_row.join(" | "));
            txts.push(line);
            if row_i == 0 {
                let sep_cells: Vec<&str> = texts_row.iter().map(|_| "--").collect();
                txts.push(format!("| {} |", sep_cells.join(" | ")));
            }
            row_i += 1;
            tr.remove();
        }
        let table_md = txts.join("\n");
        table_sel.replace_with_html(wrap_md_fragment(&table_md));
    }
}
