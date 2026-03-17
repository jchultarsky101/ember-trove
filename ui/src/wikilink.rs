use std::collections::HashMap;

use common::id::NodeId;

/// Preprocess markdown content, converting `[[title]]` and `[[title|display]]`
/// wiki-links into HTML that the Leptos click handler can intercept.
///
/// - **Resolved** links → `<a class="wikilink" data-node-id="<uuid>">display</a>`
/// - **Unresolved** links → `<span class="wikilink-unresolved">display</span>`
///
/// The returned string is passed to `pulldown_cmark` before HTML rendering.
/// Inline HTML is preserved by pulldown-cmark and then sanitized by ammonia
/// (which must be configured to allow `class` and `data-node-id` on `<a>`).
pub fn preprocess_wikilinks(content: &str, title_map: &HashMap<String, NodeId>) -> String {
    let mut result = String::with_capacity(content.len() + 64);
    let mut remaining = content;

    while let Some(open) = remaining.find("[[") {
        result.push_str(&remaining[..open]);
        remaining = &remaining[open + 2..];

        if let Some(close) = remaining.find("]]") {
            let inner = &remaining[..close];
            let (target, display) = if let Some(pipe) = inner.find('|') {
                (inner[..pipe].trim(), inner[pipe + 1..].trim())
            } else {
                let t = inner.trim();
                (t, t)
            };

            if let Some(node_id) = title_map.get(target) {
                result.push_str(&format!(
                    r#"<a class="wikilink" data-node-id="{}">{}</a>"#,
                    node_id.0,
                    html_escape(display),
                ));
            } else {
                result.push_str(&format!(
                    r#"<span class="wikilink-unresolved">{}</span>"#,
                    html_escape(display),
                ));
            }
            remaining = &remaining[close + 2..];
        } else {
            // Unclosed [[ — emit as-is and stop
            result.push_str("[[");
            result.push_str(remaining);
            return result;
        }
    }

    result.push_str(remaining);
    result
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
