
use std::sync::Arc;

use dioxus::prelude::*;

use crate::{Cli, MarcoSparkoContext, ModuleRegistrations, ModuleFactory, PageInfo};

const PAGE_CONTENT_CSS: Asset = asset!("/assets/styling/page_content.css");

// Simple HTML escaper
fn escape_html(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}

fn get_page<'a>(module: &'a Box<dyn crate::Module + Send>, path: &'a Vec<String>, page_list: &'a Vec<PageInfo>) -> (&'a str, Box<dyn Fn() -> Element + 'a>) {
    let mut page_id = "";
    let mut it = path.into_iter();

    if let Some(p) = it.next() {
        page_id = p;
    }

    let mut remaining_path = Vec::new();

    loop {
        if let Some(s) = it.next() {
            remaining_path.push((*s).clone());
        }
        else {
            break;
        }
    }
    if page_list.len() < 1 {
        let module_id = module.module_id();

        // let x = || rsx!(
        //     "Module page list is empty {module_id}"
        // );
        // let y: impl Fn() -> Element = x;

        let x1 = Box::new(move || rsx!( "Module page list is empty {module_id}" ));
        return ("", x1);
    }

    for page_info in page_list {
        if page_id == "" || page_id == page_info.path {
            return(page_info.path, module.get_component(page_info.path, remaining_path))
        }
    }



    let msg = format!("{:?}", path);
    let x2 = Box::new(move || rsx!( "Unknown page in path {msg}" ));
    ("", x2)


}

#[component]
pub fn Module(module_id: String) -> Element {
    let mut path_signal = use_signal(|| vec!(String::from("")));
    let path = (&*path_signal.read()).clone();

    use_context_provider::<Signal<Vec<String>>>(move || path_signal);

    let context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let opt_context = &*context_signal.read();
    let context = opt_context.as_ref().unwrap();
    let module_registrations = use_context::<ModuleRegistrations>();

    let mut call_construct_module_signal = use_signal::<bool>(|| true);
    let mut construct_module_action: Action<(ModuleRegistrations, Arc<MarcoSparkoContext>, String), Arc<dyn crate::ModuleFactory + Send>> = use_action( move |module_registrations: ModuleRegistrations, marco_sparko_context: std::sync::Arc<crate::MarcoSparkoContext>, module_id: String|  async move { Cli::do_construct(&module_id, &module_registrations, &marco_sparko_context).await});


    let mut call_check_ready_module_signal = use_signal::<bool>(|| true);
    let mut check_ready_module_action: Action<(Arc<dyn ModuleFactory>, bool), bool> = use_action( move |builder: Arc<dyn ModuleFactory>, _dummy: bool|  async move { builder.is_ready().await});

    let mut call_build_module_signal = use_signal::<bool>(|| true);
    let mut build_module_action: Action<(Arc<dyn ModuleFactory>, bool), Box<dyn crate::Module + Send>> = use_action( move |builder: Arc<dyn ModuleFactory>, _dummy: bool|  async move { builder.build().await});

    if *call_construct_module_signal.read() {
        call_construct_module_signal.set(false);
        construct_module_action.call(module_registrations.clone(), context.clone(), module_id.clone());
    }

    if let Some(result) = construct_module_action.value() {
        let builder_signal = result?;
        let builder = (*builder_signal.read()).clone();
        
        if *call_check_ready_module_signal.read() {
            call_check_ready_module_signal.set(false);
            check_ready_module_action.call(builder.clone(), true);
        }

        if let Some(result) = check_ready_module_action.value() {
            let is_ready_signal = result?;
            let is_ready = *is_ready_signal.read();

            if is_ready {
                if *call_build_module_signal.read() {
                    call_build_module_signal.set(false);
                    // let t = 
                    build_module_action.call(builder.clone(), true);
                }

                if let Some(result) = build_module_action.value() {
                    let module_signal = result?;
                    let module = &*module_signal.read();
                    let page_list = module.get_page_list();
                    let (active_page_id, content) = get_page(&module, &path, &page_list);
                    
                    // Signal for sidebar visibility
                    let mut sidebar_open = use_signal(|| true);

                    // Toggle sidebar
                    let toggle_sidebar = {
                        let b = !*sidebar_open.read();
                        move |_| {
                            sidebar_open.set(b);
                        }
                    };

                    let body = match content() {
                        Ok(element) => rsx!(
                            div { class: "filler", {element} }
                        )?,
                        Err(error) => {
                            let escaped_error = escape_html(&error.to_string());

                            let html = format!("<div class=\"error\">Failed to load page content: <pre>{}</pre></div>", escaped_error);
                            rsx! {
                                div { dangerous_inner_html: "{html}" }
                            }?
                        },
                    };

                    rsx! {
                        document::Link { rel: "stylesheet", href: PAGE_CONTENT_CSS }
                        div { class: "layout-root",
                            // Main container
                            div { class: "container",
                                // Sidebar
                                nav {
                                    class: format_args!("sidebar {}", if *sidebar_open.read() { "open" } else { "closed" }),
                                    style: "width: 240px;", // fixed width
                                    aria_label: "secondary navigation",

                                    div { class: "sidebar-inner",
                                        if *sidebar_open.read() {
                                            button {
                                                class: "hamburger",
                                                onclick: toggle_sidebar,
                                                "<<"
                                            }
                                        }
                                        for page_info in page_list.clone() {
                                            div {
                                                class: format_args!(
                                                    "sidebar-item {}",
                                                    if active_page_id == page_info.path { "active" } else { "inactive" },
                                                ),
                                                onclick: move |_| { path_signal.set(vec![String::from(page_info.path)]) },
                                                "{page_info.label}"
                                            }
                                        }
                                                                        // }
                                    }
                                }

                                // Main content
                                main { class: "main",
                                    if !*sidebar_open.read() {
                                        button {
                                            class: "hamburger",
                                            onclick: toggle_sidebar,
                                            ">>"
                                        }
                                    }
                                    {body}
                                }
                            }
                        }
                    }
                }
                else {
                    rsx!{ "Building {module_id}..." }
                }
            }
            else {
                builder.init_page()
            }
        }
        else {
            rsx!{ "Loading {module_id}..." }
        }
    }
    else {
        rsx!{ "Loading {module_id}..." }
    }
}