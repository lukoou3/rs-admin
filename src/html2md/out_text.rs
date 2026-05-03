//! 对应 Go `outTagText`。
use nipper::Node;
use std::collections::HashMap;

pub fn out_tag_text(node: &Node<'_>, sep: &str, strip: bool, line_append: &str) -> String {
    if node.is_text() {
        let data = node.text().to_string();
        if strip {
            if data == "\n" {
                return data;
            }
            return data.trim().to_string();
        }
        return data;
    }

    let mut texts: Vec<String> = Vec::new();
    let mut line_map: HashMap<&'static str, i32> = [
        ("br", 1),
        ("p", 1),
        ("div", 1),
        ("ul", 1),
        ("ol", 1),
        ("h1", 1),
        ("h2", 1),
        ("h3", 1),
        ("h4", 1),
        ("h5", 1),
    ]
    .into_iter()
    .collect();
    let mut line_map2: HashMap<&'static str, i32> = [
        ("p", 1),
        ("div", 1),
        ("ul", 1),
        ("ol", 1),
        ("h1", 1),
        ("h2", 1),
        ("h3", 1),
        ("h4", 1),
        ("h5", 1),
    ]
    .into_iter()
    .collect();

    let mut child = node.first_child();
    let mut pre_child: Option<Node<'_>> = None;

    while let Some(c) = child {
        let append_sep = {
            let pre_tag = pre_child.as_ref().and_then(|p| p.node_name());
            let cur_tag = c.node_name();
            let pre_hit = pre_tag
                .as_ref()
                .map(|t| line_map.contains_key(t.as_ref()))
                .unwrap_or(false);
            let cur_hit = cur_tag
                .as_ref()
                .map(|t| line_map2.contains_key(t.as_ref()))
                .unwrap_or(false);
            pre_hit || cur_hit
        };

        if append_sep {
            let length = texts.len();
            let start = length.saturating_sub(4);
            let joined = texts[start..length].join("");
            let has_nn = joined.ends_with("\n\n");
            let has_nnn = joined.ends_with("\n\n\n");
            if has_nn && line_append.contains('\n') && !has_nnn {
                texts.push("\n".to_string());
            } else {
                texts.push(line_append.to_string());
            }
        }

        pre_child = Some(c.clone());

        let name = c.node_name();
        let tag = name.as_ref().map(|s| s.as_ref()).unwrap_or("");
        if tag == "script" || tag == "style" {
            child = c.next_sibling();
            continue;
        }

        if c.is_element() {
            texts.push(out_tag_text(&c, sep, strip, line_append));
        } else if c.is_text() {
            let data = c.text().to_string();
            if strip && data != "\n" {
                texts.push(data.trim().to_string());
            } else {
                texts.push(data);
            }
        }

        child = c.next_sibling();
    }

    texts.join(sep)
}
