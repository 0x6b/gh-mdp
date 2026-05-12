use std::{collections::HashMap, time::Instant};

use pulldown_cmark::{
    BlockQuoteKind, Event, MetadataBlockKind, Options, Parser, Tag, TagEnd, html::push_html,
};
use serde_yaml::{Value, from_str};
use tracing::info;

const OPTIONS: Options = Options::ENABLE_GFM
    .union(Options::ENABLE_TABLES)
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_SMART_PUNCTUATION)
    .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

const ICON_NOTE: &str = r#"<svg class="octicon mr-2" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path></svg>"#;
const ICON_TIP: &str = r#"<svg class="octicon mr-2" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M8 1.5c-2.363 0-4 1.69-4 3.75 0 .984.424 1.625.984 2.304l.214.253c.223.264.47.556.673.848.284.411.537.896.621 1.49a.75.75 0 0 1-1.484.211c-.04-.282-.163-.547-.37-.847a8.456 8.456 0 0 0-.542-.68c-.084-.1-.173-.205-.268-.32C3.201 7.75 2.5 6.766 2.5 5.25 2.5 2.31 4.863 0 8 0s5.5 2.31 5.5 5.25c0 1.516-.701 2.5-1.328 3.259-.095.115-.184.22-.268.319-.207.245-.383.453-.541.681-.208.3-.33.565-.37.847a.751.751 0 0 1-1.485-.212c.084-.593.337-1.078.621-1.489.203-.292.45-.584.673-.848.075-.088.147-.173.213-.253.561-.679.985-1.32.985-2.304 0-2.06-1.637-3.75-4-3.75ZM5.75 12h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM6 15.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 0 1.5h-2.5a.75.75 0 0 1-.75-.75Z"></path></svg>"#;
const ICON_IMPORTANT: &str = r#"<svg class="octicon mr-2" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v9.5A1.75 1.75 0 0 1 14.25 13H8.06l-2.573 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.189l2.72-2.719a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25Zm7 2.25v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path></svg>"#;
const ICON_WARNING: &str = r#"<svg class="octicon mr-2" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path></svg>"#;
const ICON_CAUTION: &str = r#"<svg class="octicon mr-2" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M4.47.22A.749.749 0 0 1 5 0h6c.199 0 .39.079.53.22l4.25 4.25c.141.14.22.331.22.53v6a.749.749 0 0 1-.22.53l-4.25 4.25A.749.749 0 0 1 11 16H5a.749.749 0 0 1-.53-.22L.22 11.53A.749.749 0 0 1 0 11V5c0-.199.079-.39.22-.53Zm.84 1.28L1.5 5.31v5.38l3.81 3.81h5.38l3.81-3.81V5.31L10.69 1.5ZM8 4a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path></svg>"#;

fn alert_open(kind: BlockQuoteKind) -> String {
    let (cls, title, icon) = match kind {
        BlockQuoteKind::Note => ("note", "Note", ICON_NOTE),
        BlockQuoteKind::Tip => ("tip", "Tip", ICON_TIP),
        BlockQuoteKind::Important => ("important", "Important", ICON_IMPORTANT),
        BlockQuoteKind::Warning => ("warning", "Warning", ICON_WARNING),
        BlockQuoteKind::Caution => ("caution", "Caution", ICON_CAUTION),
    };
    format!(
        "<div class=\"markdown-alert markdown-alert-{cls}\">\n<p class=\"markdown-alert-title\">{icon}{title}</p>\n"
    )
}

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
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn yaml_value_to_html(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => format!("<div>{b}</div>"),
        Value::Number(n) => format!("<div>{n}</div>"),
        Value::String(s) => format!("<div>{}</div>", escape_html(s)),
        Value::Sequence(seq) => {
            let mut html = String::from("<div><table><tbody>");
            for item in seq {
                html.push_str("<tr><td>");
                html.push_str(&yaml_value_to_html(item));
                html.push_str("</td></tr>");
            }
            html.push_str("</tbody></table></div>");
            html
        }
        Value::Mapping(map) => {
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
        Value::Tagged(tagged) => yaml_value_to_html(&tagged.value),
    }
}

fn render_front_matter(yaml: &str) -> String {
    let Ok(Value::Mapping(mapping)) = from_str::<Value>(yaml) else {
        return String::new();
    };

    if mapping.is_empty() {
        return String::new();
    }

    use std::fmt::Write;
    let mut html = String::from("<table><thead><tr>");
    for (key, _) in &mapping {
        let key_str = match key.as_str() {
            Some(s) => escape_html(s),
            None => format!("{key:?}"),
        };
        let _ = write!(html, "<th>{key_str}</th>");
    }
    html.push_str("</tr></thead><tbody><tr>");
    for (_, value) in &mapping {
        html.push_str("<td>");
        html.push_str(&yaml_value_to_html(value));
        html.push_str("</td>");
    }
    html.push_str("</tr></tbody></table>\n");
    html
}

pub fn render(content: &str) -> String {
    let start = Instant::now();
    let parser = Parser::new_ext(content, OPTIONS);

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
                let slug = if let Some(n) = slug_counts.get_mut(&base) {
                    *n += 1;
                    format!("{base}-{n}")
                } else {
                    slug_counts.insert(base.clone(), 0);
                    base
                };
                let anchor = format!("<a id=\"{slug}\" class=\"anchor\" href=\"#{slug}\"></a>");
                vec![Event::Html(anchor.into()), event]
            }
            (Event::Start(Tag::BlockQuote(Some(kind))), _) => {
                vec![Event::Html(alert_open(*kind).into())]
            }
            (Event::End(TagEnd::BlockQuote(Some(_))), _) => {
                vec![Event::Html("</div>\n".into())]
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
