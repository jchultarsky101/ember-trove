/// Extract all wiki-link target titles from a markdown body.
///
/// Handles both `[[title]]` and `[[title|display text]]` — only the target
/// title (before the `|`) is returned. Titles are trimmed of whitespace.
/// Duplicate titles are preserved; deduplication is the caller's concern.
pub fn parse_wikilink_titles(content: &str) -> Vec<String> {
    let mut titles = Vec::new();
    let mut remaining = content;

    while let Some(open) = remaining.find("[[") {
        remaining = &remaining[open + 2..];
        if let Some(close) = remaining.find("]]") {
            let inner = &remaining[..close];
            // Only look at the part before a pipe: [[target|display]]
            let target = inner.split('|').next().unwrap_or("").trim();
            if !target.is_empty() {
                titles.push(target.to_string());
            }
            remaining = &remaining[close + 2..];
        } else {
            // Unclosed [[  — stop scanning
            break;
        }
    }

    titles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_wikilink() {
        let titles = parse_wikilink_titles("See [[Rust Language]] for details.");
        assert_eq!(titles, vec!["Rust Language"]);
    }

    #[test]
    fn test_piped_wikilink() {
        let titles = parse_wikilink_titles("Check [[Rust Language|Rust]] out.");
        assert_eq!(titles, vec!["Rust Language"]);
    }

    #[test]
    fn test_multiple_wikilinks() {
        let titles = parse_wikilink_titles("[[Alpha]] and [[Beta|B]] and [[Gamma]].");
        assert_eq!(titles, vec!["Alpha", "Beta", "Gamma"]);
    }

    #[test]
    fn test_unclosed_wikilink() {
        // Unclosed [[ should not panic and should stop scanning.
        let titles = parse_wikilink_titles("[[Alpha]] then [[unclosed");
        assert_eq!(titles, vec!["Alpha"]);
    }

    #[test]
    fn test_empty_body() {
        assert!(parse_wikilink_titles("").is_empty());
    }
}
