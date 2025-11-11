use dioxus::prelude::*;

const PAGE_CONTENT_CSS: Asset = asset!("/assets/styling/page_content.css");

#[component]
pub fn PageContent() -> Element {
    // Signal for sidebar visibility
    let mut sidebar_open = use_signal(|| true);

    // Toggle sidebar
    let toggle_sidebar = {
        let b = !*sidebar_open.read();
        move |_| {
            sidebar_open.set(b);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: PAGE_CONTENT_CSS }
        div { class: "layout-root",
            // Topbar
            div { class: "topbar",
                // button { class: "hamburger", onclick: toggle_sidebar,if *sidebar_open.read() { "<<" } else { ">>" } }
                
                // h2 { class: "title", "My App" }
            }

            // Main container
            div { class: "container",
                // Sidebar
                nav {
                    class: format_args!("sidebar {}", if *sidebar_open.read() { "open" } else { "closed" }),
                    style: "width: 240px;",  // fixed width
                    aria_label: "secondary navigation",

                    div { class: "sidebar-inner",
                        if *sidebar_open.read() {button { class: "hamburger", onclick: toggle_sidebar, "<<" }} 
                        h3 { "Secondary Nav" }
                        ul {
                            li { "Item 1" }
                            li { "Item 2" }
                            li { "Item 3" }
                            li { "Item 4" }
                        }
                    }
                }

                // Main content
                main { class: "main",
                    // button { class: "hamburger", onclick: toggle_sidebar,if *sidebar_open.read() { "<<" } else { ">>" } }
                    if !*sidebar_open.read() {button { class: "hamburger", onclick: toggle_sidebar, ">>" }} 
                    // button { class: "hamburger", onclick: toggle_sidebar, "☰" }
                    div { class: "filler", "Content goes here..." }
                }
            }
        }
    }
}
