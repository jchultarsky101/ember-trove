use uuid::Uuid;

/// Generate a URL-safe slug from a title with a short UUID suffix for uniqueness.
///
/// Example: `"My First Article!"` → `"my-first-article-a1b2c3d4"`
pub fn slugify(title: &str) -> String {
    let base: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse consecutive hyphens and trim leading/trailing.
    let mut slug = String::new();
    let mut prev_hyphen = true; // start true to trim leading hyphens
    for c in base.chars() {
        if c == '-' {
            if !prev_hyphen {
                slug.push('-');
            }
            prev_hyphen = true;
        } else {
            slug.push(c);
            prev_hyphen = false;
        }
    }

    // Trim trailing hyphen.
    if slug.ends_with('-') {
        slug.pop();
    }

    // Append short UUID suffix for uniqueness.
    let suffix = &Uuid::new_v4().to_string()[..8];
    if slug.is_empty() {
        suffix.to_string()
    } else {
        format!("{slug}-{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slugification() {
        let slug = slugify("My First Article!");
        assert!(slug.starts_with("my-first-article-"));
        // 8-char UUID suffix
        let parts: Vec<&str> = slug.rsplitn(2, '-').collect();
        assert_eq!(parts[0].len(), 8);
    }

    #[test]
    fn collapses_hyphens() {
        let slug = slugify("hello   world---test");
        assert!(slug.starts_with("hello-world-test-"));
    }

    #[test]
    fn empty_title() {
        let slug = slugify("");
        assert_eq!(slug.len(), 8); // just the UUID suffix
    }

    #[test]
    fn unicode_stripped() {
        let slug = slugify("Café résumé");
        assert!(slug.starts_with("caf-r-sum-"));
    }
}
