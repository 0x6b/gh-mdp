use std::{collections::HashMap, fs::read_to_string, path::Path, time::Instant};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html::push_html};
use tracing::info;

const OPTIONS: Options = Options::ENABLE_GFM
    .union(Options::ENABLE_TABLES)
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_SMART_PUNCTUATION);

/// Convert heading text to a URL-friendly slug
fn slugify(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn render(path: &Path) -> String {
    let start = Instant::now();
    let content = read_to_string(path).unwrap_or_else(|e| format!("Error reading file: {e}"));
    let parser = Parser::new_ext(&content, OPTIONS);

    // Track heading slugs to handle duplicates
    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut in_heading = false;
    let mut heading_text = String::new();
    let mut heading_level = 0;

    // Collect events and inject heading IDs
    let events: Vec<Event> = parser
        .flat_map(|event| match &event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_text.clear();
                heading_level = *level as usize;
                vec![event]
            }
            Event::Text(text) if in_heading => {
                heading_text.push_str(text);
                vec![event]
            }
            Event::Code(code) if in_heading => {
                heading_text.push_str(code);
                vec![event]
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let base_slug = slugify(&heading_text);
                let slug = if let Some(count) = slug_counts.get_mut(&base_slug) {
                    *count += 1;
                    format!("{}-{}", base_slug, count)
                } else {
                    slug_counts.insert(base_slug.clone(), 0);
                    base_slug
                };

                // Insert an anchor element before the heading closes
                let anchor = format!("<a id=\"{slug}\" class=\"anchor\" href=\"#{slug}\"></a>");
                vec![Event::Html(anchor.into()), event]
            }
            _ => vec![event],
        })
        .collect();

    let mut html_output = String::new();
    push_html(&mut html_output, events.into_iter());
    info!(latency = ?start.elapsed(), "Markdown rendered");
    html_output
}
