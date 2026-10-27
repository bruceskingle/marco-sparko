//! The help shown beside each screen, see views/help_panel.rs.
//!
//! The help is Markdown in docs/help, so that it is also part of the documentation, and is built into the app. Each
//! screen has a page, see the Navbar and Module::help_topic, which can link to other pages for more detail. Links to
//! other documents open on GitHub, and links to web sites in the browser. See docs/WritingHelp.md for how to add and
//! change pages.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Where the documentation is on GitHub, for links to documents other than the help pages.
const DOCS_URL: &str = "https://github.com/bruceskingle/marco-sparko/blob/main/";
/// Where the help pages are, relative to the repository and to DOCS_URL.
const HELP_DIR: &str = "docs/help/";

pub struct HelpPage {
    /// Relative to docs/help, e.g. octopus/meters.md
    pub path: &'static str,
    pub markdown: &'static str,
}

macro_rules! help_pages {
    ($($path:literal),* $(,)?) => {
        &[$(HelpPage { path: $path, markdown: include_str!(concat!("../docs/help/", $path)) }),*]
    };
}

/// All the help pages. A page added to docs/help must be added here too.
static PAGES: &[HelpPage] = help_pages![
    "index.md",
    "home.md",
    "profile-size.md",
    "settings.md",
    "log.md",
    "date-range.md",
    "graphs.md",
    "octopus/index.md",
    "octopus/login.md",
    "octopus/user.md",
    "octopus/account.md",
    "octopus/bills.md",
    "octopus/bill.md",
    "octopus/properties.md",
    "octopus/property.md",
    "octopus/meters.md",
    "octopus/meter.md",
    "octopus/home-mini.md",
    "octopus/electricity.md",
    "octopus/ev-charging.md",
    "octopus/charging-sessions.md",
];

macro_rules! help_icons {
    ($($name:literal),* $(,)?) => {
        &[$((concat!("icons/", $name, ".svg"), include_str!(concat!("../docs/help/icons/", $name, ".svg")))),*]
    };
}

/// The icons of the app's buttons, which the help refers to as images, e.g. ![Help](icons/help.svg), and are drawn
/// inline in the app, by their path relative to docs/help.
static ICONS: &[(&str, &str)] = help_icons![
    "help", "log", "settings", "profile", "close", "back", "hide-pages", "show-pages", "reload", "export", "categories",
    "calendar", "external-link", "previous", "next",
];

/// The help page at path, relative to docs/help.
pub fn page(path: &str) -> Option<&'static HelpPage> {
    PAGES.iter().find(|page| page.path == path)
}

/// Where a link on a help page goes.
#[derive(Debug, PartialEq)]
pub enum HelpLink {
    /// Another help page, by its path relative to docs/help.
    Page(String),
    /// A web page, e.g. another document on GitHub.
    Web(String),
    /// Somewhere on the same page, which is not followed.
    None,
}

/// Where href, a link on the help page at from, goes.
pub fn follow(from: &str, href: &str) -> HelpLink {
    if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("mailto:") {
        return HelpLink::Web(href.to_string());
    }
    let href = href.split('#').next().unwrap_or_default();
    if href.is_empty() {
        return HelpLink::None;
    }
    let path = resolve(from, href);
    match path.strip_prefix(HELP_DIR) {
        Some(help_path) if page(help_path).is_some() => HelpLink::Page(help_path.to_string()),
        _ => HelpLink::Web(format!("{DOCS_URL}{path}")),
    }
}

/// Where href, relative to the help page at from, is in the repository, so that links out of docs/help can go to
/// GitHub.
fn resolve(from: &str, href: &str) -> String {
    let from_path = format!("{HELP_DIR}{from}");
    let mut parts: Vec<&str> = from_path.split('/').collect();
    // The directory the page is in.
    parts.pop();
    for part in href.split('/') {
        match part {
            "" | "." => {},
            ".." => {
                parts.pop();
            },
            part => parts.push(part),
        }
    }
    parts.join("/")
}

/// The help page at path as HTML, to show in the help panel, with the icons of buttons drawn inline.
pub fn to_html(path: &str, markdown: &str) -> String {
    use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};

    let mut events = Vec::new();
    // The icon of the image being read, and its alt text, which comes as text before its end.
    let mut icon: Option<(&str, String)> = None;
    for event in Parser::new_ext(markdown, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH) {
        match (&mut icon, event) {
            (None, Event::Start(Tag::Image { dest_url, .. })) => {
                let icon_path = resolve(path, &dest_url);
                match icon_path.strip_prefix(HELP_DIR).and_then(|icon_path| ICONS.iter().find(|(path, _)| *path == icon_path)) {
                    Some((_, svg)) => icon = Some((svg, String::new())),
                    None => events.push(Event::Start(Tag::Image {
                        link_type: pulldown_cmark::LinkType::Inline, dest_url, title: CowStr::Borrowed(""), id: CowStr::Borrowed(""),
                    })),
                }
            },
            (Some((_, alt)), Event::Text(text)) => alt.push_str(&text),
            (Some((svg, alt)), Event::End(TagEnd::Image)) => {
                events.push(Event::InlineHtml(CowStr::from(format!("<span class=\"help-icon\" title=\"{alt}\">{}</span>", svg.trim()))));
                icon = None;
            },
            (Some(_), _) => {},
            (None, event) => events.push(event),
        }
    }
    let mut html = String::new();
    html::push_html(&mut html, events.into_iter());
    html
}

/// The help page of the screen being shown in a module, set by the module's view with use_help_topic, see the Navbar.
#[derive(Clone, Copy)]
pub struct ModuleHelpTopic(pub Signal<Option<String>>);

/// Sets the help page of the module screen being shown, if it has one. Does nothing outside the main window.
pub fn use_help_topic(topic: Option<String>) {
    let help = try_use_context::<ModuleHelpTopic>();
    use_effect(use_reactive((&topic,), move |(topic,)| {
        if let Some(mut help) = help
            && topic.is_some()
            && *help.0.peek() != topic
        {
            help.0.set(topic);
        }
    }));
}

/// The help settings of a profile, see Settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelpSettings {
    /// Whether the help panel is shown.
    #[serde(default = "shown")]
    pub show: bool,
}

fn shown() -> bool {
    true
}

impl Default for HelpSettings {
    fn default() -> Self {
        HelpSettings { show: true }
    }
}

#[cfg(test)]
mod tests {
    use super::{HelpLink, PAGES, follow};

    #[test]
    fn links_go_to_help_pages_documents_or_web_sites() {
        assert_eq!(follow("octopus/meters.md", "home-mini.md"), HelpLink::Page(String::from("octopus/home-mini.md")));
        assert_eq!(follow("octopus/meters.md", "../graphs.md#zooming"), HelpLink::Page(String::from("graphs.md")));
        assert_eq!(follow("home.md", "octopus/index.md"), HelpLink::Page(String::from("octopus/index.md")));
        assert_eq!(follow("home.md", "../ScheduledFetch.md"),
            HelpLink::Web(String::from("https://github.com/bruceskingle/marco-sparko/blob/main/docs/ScheduledFetch.md")));
        assert_eq!(follow("home.md", "https://octopus.energy"), HelpLink::Web(String::from("https://octopus.energy")));
        assert_eq!(follow("home.md", "#top"), HelpLink::None);
    }

    #[test]
    fn icons_are_drawn_inline() {
        let html = super::to_html("octopus/meters.md", "Click ![Help](../icons/help.svg) to show it");
        assert!(html.starts_with("<p>Click <span class=\"help-icon\" title=\"Help\"><svg "), "{html}");
        assert!(html.ends_with("</svg></span> to show it</p>\n"), "{html}");
    }

    /// Every link on every help page goes somewhere which exists, as far as can be checked here.
    #[test]
    fn links_between_help_pages_are_not_broken() {
        for page in PAGES {
            for href in markdown_links(page.markdown) {
                if let HelpLink::Web(url) = follow(page.path, &href)
                    && let Some(path) = url.strip_prefix("https://github.com/bruceskingle/marco-sparko/blob/main/")
                {
                    assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path).exists(), "{} links to {href} which does not exist", page.path);
                }
            }
        }
    }

    /// The targets of the Markdown links in text.
    fn markdown_links(text: &str) -> Vec<String> {
        text.match_indices("](").filter_map(|(index, _)| {
            let rest = &text[index + 2..];
            rest.find(')').map(|end| rest[..end].to_string())
        }).collect()
    }
}
