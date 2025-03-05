use askama_escape::{ escape, Html };
use linkify::{ LinkFinder, LinkKind };
use regex::Regex;

/**
 * Escapes the special characters injected inside the text of an XML tag or attribute value.
 */
pub fn escape_xml_special_characters(input: &str) -> String {
    input.replace("&", "&amp;")
         .replace("<", "&lt;")
         .replace(">", "&gt;")
         .replace("\"", "&quot;")
         .replace("'", "&apos;")
}

/**
 * Convert text in comment that looks like a link to an actual anchor tag.
 * e.g. "Hello https://supercarband.com/" -> "Hello <a href="...">https://supercarband.com/</a>"
 */
pub fn make_content_links(comment: &str) -> String {
    let escaped_content = escape(comment, Html).to_string();
    let finder = LinkFinder::new();

    let mut result = String::new();
    let mut last_pos = 0;

    for link in finder.links(&escaped_content) {
        let start = link.start();
        let end = link.end();
        result.push_str(&escaped_content[last_pos..start]);

        let url: String = if link.kind() == &LinkKind::Email {
            format!("mailto:{}", link.as_str())
        } else {
            String::from(link.as_str())
        };

        result.push_str(&format!(
            r#"<a href="{}" target="_blank">{}</a>"#,
            url,
            link.as_str()
        ));

        last_pos = end;
    }

    result.push_str(&escaped_content[last_pos..]);

    result
}

/**
 * Convert an arbitrary string to kebab-case, generally used for URL transforms.
 */
pub fn to_kebab_case(input: &str) -> String {
    let non_kebab_characters_regex = Regex::new(r"[()\[\]!?'&#.,/\\~+]").unwrap();
    non_kebab_characters_regex.replace_all(input, "")
        .split_whitespace()
        .flat_map(|s| s.split('_'))
        .flat_map(|s| s.split('-'))
        .filter(|s| !s.is_empty())
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

/**
 * Convert an arbitrary string to snake_case, generally used for reversing URL transforms.
 */
pub fn to_snake_case(input: &str) -> String {
    let non_kebab_characters_regex = Regex::new(r"[()\[\]!?'&#.,/\\~]").unwrap();
    non_kebab_characters_regex.replace_all(input, "")
        .split_whitespace()
        .flat_map(|s| s.split('_'))
        .flat_map(|s| s.split('-'))
        .filter(|s| !s.is_empty())
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}