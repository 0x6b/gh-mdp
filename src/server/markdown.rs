use std::{collections::HashMap, fs::read_to_string, path::Path, time::Instant};

use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag, TagEnd, html::push_html};
use serde_yaml::{Value, from_str};
use tracing::info;

const OPTIONS: Options = Options::ENABLE_GFM
    .union(Options::ENABLE_TABLES)
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_SMART_PUNCTUATION)
    .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

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

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn yaml_value_to_html(value: &Value) -> String {
    match value {
        serde_yaml::Value::Null => String::new(),
        serde_yaml::Value::Bool(b) => format!("<div>{b}</div>"),
        serde_yaml::Value::Number(n) => format!("<div>{n}</div>"),
        serde_yaml::Value::String(s) => format!("<div>{}</div>", escape_html(s)),
        serde_yaml::Value::Sequence(seq) => {
            let mut html = String::from("<div><table><tbody>");
            for item in seq {
                html.push_str("<tr><td>");
                html.push_str(&yaml_value_to_html(item));
                html.push_str("</td></tr>");
            }
            html.push_str("</tbody></table></div>");
            html
        }
        serde_yaml::Value::Mapping(map) => {
            let mut html = String::from("<div><table><thead><tr>");
            for (key, _) in map {
                html.push_str("<th>");
                html.push_str(&yaml_value_to_html(key));
                html.push_str("</th>");
            }
            html.push_str("</tr></thead><tbody><tr>");
            for (_, val) in map {
                html.push_str("<td>");
                html.push_str(&yaml_value_to_html(val));
                html.push_str("</td>");
            }
            html.push_str("</tr></tbody></table></div>");
            html
        }
        serde_yaml::Value::Tagged(tagged) => yaml_value_to_html(&tagged.value),
    }
}

fn render_front_matter(yaml: &str) -> String {
    let value: Value = match from_str(yaml) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    let mapping = match value.as_mapping() {
        Some(m) => m,
        None => return String::new(),
    };

    if mapping.is_empty() {
        return String::new();
    }

    let mut html = String::from("<table><thead><tr>");
    for (key, _) in mapping {
        let key_str = match key.as_str() {
            Some(s) => escape_html(s),
            None => format!("{key:?}"),
        };
        html.push_str(&format!("<th>{key_str}</th>"));
    }
    html.push_str("</tr></thead><tbody><tr>");
    for (_, value) in mapping {
        html.push_str("<td>");
        html.push_str(&yaml_value_to_html(value));
        html.push_str("</td>");
    }
    html.push_str("</tr></tbody></table>\n");
    html
}

pub fn render(path: &Path) -> String {
    let start = Instant::now();
    let content = read_to_string(path).unwrap_or_else(|e| format!("Error reading file: {e}"));
    let parser = Parser::new_ext(&content, OPTIONS);

    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut heading: Option<String> = None;
    let mut task_index: usize = 0;
    let mut in_metadata = false;
    let mut metadata_text = String::new();

    let events: Vec<Event> = parser
        .flat_map(|event| match (&event, &mut heading) {
            (Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)), _) => {
                in_metadata = true;
                metadata_text.clear();
                vec![]
            }
            (Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)), _) => {
                in_metadata = false;
                vec![]
            }
            _ if in_metadata => {
                if let Event::Text(t) = &event {
                    metadata_text.push_str(t);
                }
                vec![]
            }
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
                let html = format!("<input type=\"checkbox\" data-task-index=\"{i}\"{c}/>\n");
                vec![Event::Html(html.into())]
            }
            _ => vec![event],
        })
        .collect();

    let mut html = String::new();
    if !metadata_text.is_empty() {
        html.push_str(&render_front_matter(&metadata_text));
    }
    push_html(&mut html, events.into_iter());
    info!(latency = ?start.elapsed(), "Markdown rendered");
    html
}
