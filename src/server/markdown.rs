use std::{collections::HashMap, fs::read_to_string, path::Path, time::Instant};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html::push_html};
use tracing::info;

const OPTIONS: Options = Options::ENABLE_GFM
    .union(Options::ENABLE_TABLES)
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_SMART_PUNCTUATION);

fn slugify(text: &str) -> String {
    text.chars()
        .filter_map(|c| match c {
            _ if c.is_alphanumeric() => Some(c.to_ascii_lowercase()),
            ' ' | '-' | '_' => Some('-'),
            _ => None,
        })
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

    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut heading: Option<String> = None;
    let mut task_index: usize = 0;

    let events: Vec<Event> = parser
        .flat_map(|event| match (&event, &mut heading) {
            (Event::Start(Tag::Heading { .. }), h) => {
                *h = Some(String::new());
                vec![event]
            }
            (Event::Text(t) | Event::Code(t), Some(text)) => {
                text.push_str(t);
                vec![event]
            }
            (Event::End(TagEnd::Heading(_)), h @ Some(_)) => {
                let text = h.take().unwrap();
                let base = slugify(&text);
                let slug = match slug_counts.get_mut(&base) {
                    Some(n) => {
                        *n += 1;
                        format!("{base}-{n}")
                    }
                    None => {
                        slug_counts.insert(base.clone(), 0);
                        base
                    }
                };
                let anchor = format!("<a id=\"{slug}\" class=\"anchor\" href=\"#{slug}\"></a>");
                vec![Event::Html(anchor.into()), event]
            }
            (Event::TaskListMarker(checked), _) => {
                let i = task_index;
                task_index += 1;
                let c = if *checked { " checked=\"\"" } else { "" };
                let html = format!(
                    "<input type=\"checkbox\" data-task-index=\"{i}\"{c}/>\n"
                );
                vec![Event::Html(html.into())]
            }
            _ => vec![event],
        })
        .collect();

    let mut html = String::new();
    push_html(&mut html, events.into_iter());
    info!(latency = ?start.elapsed(), "Markdown rendered");
    html
}
