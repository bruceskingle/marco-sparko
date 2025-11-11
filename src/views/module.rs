
use std::{collections::HashMap, sync::Arc};

use dioxus::prelude::*;

use crate::{MarcoSparko, MarcoSparkoContext, ModuleRegistrations, PageInfo};

const PAGE_CONTENT_CSS: Asset = asset!("/assets/styling/page_content.css");


// #[component]
// pub fn SidebarMenu() -> Element {
//     let nav = use_navigator();
//     let pages = use_context::<RuntimePages>();

//     // let pages = runtime_pages.read();

//     rsx! {
//         nav {
//             ul {
//                 for (path, info) in pages.iter() {
//                     // Skip hidden or home pages if desired
//                     if path.is_empty() { continue; }

//                     li {
//                         button {
//                             onclick: {
//                                 let path = path.clone();
//                                 move |_| nav.push(format!("/{path}"))
//                             },
//                             "{info.label}"
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }


// #[component]
// fn DynamicRouter() -> Element {
//     // Read the current path from the window (desktop/web)
//     let path = match cfg!(target_arch = "wasm32") {
//         true => web_sys::window().unwrap().location().pathname().unwrap(),
//         false => std::env::args().nth(1).unwrap_or_default(), // simple desktop fallback
//     };
//     let path = path.trim_matches('/');

//     let parts: Vec<String> = if path.is_empty() {
//         vec![]
//     } else {
//         path.split('/').map(|s| s.to_string()).collect()
//     };

//     let runtime_pages = use_context::<RuntimePages>();

//     // Select the first segment as page key
//     let Some(page_key) = parts.first() else {
//         // No path → home page
//         if let Some(renderer) = runtime_pages.read().get("") {
//             return renderer(vec![]);
//         }
//         return rsx!("404 - Home page not registered");
//     };

//     let params = parts[1..].to_vec();

//     if let Some(renderer) = runtime_pages.read().get(page_key) {
//         renderer(params)
//     } else {
//         rsx!("404 - Page Not Found")
//     }
// }



fn get_page<'a>(module: &'a Box<dyn crate::Module + Send>, path: &'a Vec<&str>, page_list: &'a Vec<PageInfo>) -> (&'a str, Box<dyn Fn() -> Element + 'a>) {
    let mut page_id = "";
    let mut it = path.into_iter();

    if let Some(p) = it.next() {
        page_id = p;
    }

    let mut remaining_path = Vec::new();

    loop {
        if let Some(s) = it.next() {
            remaining_path.push(*s);
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

        let x1 = Box::new(move || rsx!(
            "Module page list is empty {module_id}"
        ));
        return ("", x1);
    }

    for page_info in page_list {
        if page_id == "" || page_id == page_info.path {
            return(page_info.path, module.get_component(page_info.path, remaining_path))
        }
    }



    let msg = format!("{:?}", path);
    let x2 = Box::new(move || rsx!(
        "Unknown page in path {msg}"
    ));
    ("", x2)











    // let default_page = if page_list.len() < 1 {
    //     let module_id = module.module_id();
    //     return ("", rsx!(
    //         "Module page list is empty {module_id}"
    //     ));
    // }
    // else {
    //     page_list.get(0).unwrap()
    // };

    // let page_id = if path.len()<1 {
    //     ""
    // }
    // else {
    //      *path.get(0).unwrap()
    // };

    // if page_id == "" {
    //     return (default_page.path,  module.get_page(default_page.path))
    // }

    // for page_info in page_list {
    //     if page_id == page_info.path {
    //         return(page_info.path, module.get_page(page_info.path))
    //     }
    // }

    // let msg = format!("{:?}", path);
    // ("", rsx!(
    //     "Unknown page in path {msg}"
    // ))
}

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn Module(module_id: String) -> Element {
    println!("ZZ2 Blog start i={}", module_id);

    let mut path_signal = use_signal(|| vec!(""));
    let path = &*path_signal.read();
    let context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let opt_context = (&*context_signal.read());
    let context = opt_context.as_ref().unwrap();
    let module_registrations = use_context::<ModuleRegistrations>();

    let mut call_signal = use_signal::<bool>(|| true);
    let a          = move |module_registrations: ModuleRegistrations, marco_sparko_context: std::sync::Arc<crate::MarcoSparkoContext>, module_id: String|  async move { MarcoSparko::do_initialize(&module_id, false, &module_registrations, &marco_sparko_context).await};
    let mut action: Action<(ModuleRegistrations, Arc<MarcoSparkoContext>, String), Box<dyn crate::Module + Send>> = use_action( move |module_registrations: ModuleRegistrations, marco_sparko_context: std::sync::Arc<crate::MarcoSparkoContext>, module_id: String|  async move { MarcoSparko::do_initialize(&module_id, false, &module_registrations, &marco_sparko_context).await});

    if *call_signal.read() {
        call_signal.set(false);
        // let t = 
        action.call(module_registrations.clone(), context.clone(), module_id.clone());
    }


    // let xx = move | 
    //     // cm: Arc<crate::CacheManager>, 
    //     // rm: Arc<sparko_graphql::AuthenticatedRequestManager<crate::octopus::token::OctopusTokenManager> >, 
    //     account_id: String, 
    //     check_for_updates: bool|  
    //                     //move | cache_manager: CacheManager, request_manager: AuthenticatedRequestManager<OctopusTokenManager>, account_number: String, check_for_updates: bool |  
    //                     async move { 
    //                         Ok("BillList::new(&cm, &rm, &account_id, check_for_updates).await");
    //                     };
    // // let mut action2 = use_action(xx);

    if let Some(result) = action.value() {
        let module_signal = result?;
        let module = &*module_signal.read();
        // let x = z.as_component();


        let page_list = module.get_page_list();
        let (active_page_id, content) = get_page(module, path, &page_list);
        // let sub_menu = rsx!(
        //     div { class: "nav-left",
        //         for page_info in page_list {
        //             div {
        //                 class: "nav_item",
        //                 "{page_info.label}"
        //             }

        //         }
        //     }
        // );

        // Signal for sidebar visibility
    let mut sidebar_open = use_signal(|| true);

    // Toggle sidebar
    let toggle_sidebar = {
        let b = !*sidebar_open.read();
        move |_| {
            sidebar_open.set(b);
        }
    };

    // let zgoto_page = |p: &str| {
    //         let x= vec!(p);
    //         path_signal.set(x);
    // };

    rsx! {
        document::Link { rel: "stylesheet", href: PAGE_CONTENT_CSS }
        div { class: "layout-root",
            // // Topbar
            // div { class: "topbar",
            //     // button { class: "hamburger", onclick: toggle_sidebar,if *sidebar_open.read() { "<<" } else { ">>" } }
                
            //     // h2 { class: "title", "My App" }
            //     "Top Bar"
            // }

            // Main container
            div { class: "container",
                // Sidebar
                nav {
                    class: format_args!("sidebar {}", if *sidebar_open.read() { "open" } else { "closed" }),
                    style: "width: 240px;",  // fixed width
                    aria_label: "secondary navigation",

                    div { class: "sidebar-inner",
                        if *sidebar_open.read() {button { class: "hamburger", onclick: toggle_sidebar, "<<" }} 
                        // h3 { "Secondary Nav" }
                        // ul {
                        //     li { "Item 1" }
                        //     li { "Item 2" }
                        //     li { "Item 3" }
                        //     li { "Item 4" }
                        // }
                        // {sub_menu}
                        // div { class: "nav-left",
                            for page_info in page_list.clone() {
                                div {
                                    // class: "nav_item",
                                    class: format_args!("sidebar-item {}", if active_page_id == page_info.path { "active" } else { "inactive" }),
                                    onclick: move |_| {
                                        path_signal.set(vec!(page_info.path))
                                    }, 
                                    "{page_info.label}"
                                }

                            }
                        // }
                    }
                }

                // Main content
                main { class: "main",
                    // button { class: "hamburger", onclick: toggle_sidebar,if *sidebar_open.read() { "<<" } else { ">>" } }
                    if !*sidebar_open.read() {button { class: "hamburger", onclick: toggle_sidebar, ">>" }} 
                    // button { class: "hamburger", onclick: toggle_sidebar, "☰" }
                    // div { class: "filler", "2Content goes here..." }
                    div { class: "filler", {content()} }
                }
            }
        }
    }
    }
    else {
         rsx!{
            "Loading {module_id}..."
        }
    }















    
    // let content = if let Some(result) = action.value() {
    //     let module_signal = result?;
    //     let module = &*module_signal.read();
    //     // let x = z.as_component();


    // let page_list = module.get_page_list();
        
    //     get_page(module, path, &page_list)
    //     // let page_info = get_page(module, &page_list, path);
    //     // //module.get_page_list().into_iter().next().unwrap();
    //     // let page_renderer = module.get_page(page_info.path);
    //     // page_renderer

    //     // let pages = use_signal(HashMap::<String, PageInfo>::new);

    //     // {
    //     //     let mut p = pages.write();
    //     //     for page_info in module.get_page_list() {
    //     //         p.insert(String::from(page_info.path), page_info);
    //     //     }
    //     // }
    //     // use_context_provider(|| RuntimePages { pages: pages.clone() });

    //     // rsx! {
    //     //     Router {
    //     //         div { class: "layout",
    //     //             SidebarMenu {}
    //     //             DynamicRouter {}
    //     //         }
    //     //     }
    //     // }
    // }
    // else {
    //      rsx!{
    //         "Loading {module_id}..."
    //     }
    // }?;

    // // Signal for sidebar visibility
    // let mut sidebar_open = use_signal(|| true);

    // // Toggle sidebar
    // let toggle_sidebar = {
    //     let b = !*sidebar_open.read();
    //     move |_| {
    //         sidebar_open.set(b);
    //     }
    // };

    // rsx! {
    //     document::Link { rel: "stylesheet", href: PAGE_CONTENT_CSS }
    //     div { class: "layout-root",
    //         // // Topbar
    //         // div { class: "topbar",
    //         //     // button { class: "hamburger", onclick: toggle_sidebar,if *sidebar_open.read() { "<<" } else { ">>" } }
                
    //         //     // h2 { class: "title", "My App" }
    //         //     "Top Bar"
    //         // }

    //         // Main container
    //         div { class: "container",
    //             // Sidebar
    //             nav {
    //                 class: format_args!("sidebar {}", if *sidebar_open.read() { "open" } else { "closed" }),
    //                 style: "width: 240px;",  // fixed width
    //                 aria_label: "secondary navigation",

    //                 div { class: "sidebar-inner",
    //                     if *sidebar_open.read() {button { class: "hamburger", onclick: toggle_sidebar, "<<" }} 
    //                     h3 { "Secondary Nav" }
    //                     // ul {
    //                     //     li { "Item 1" }
    //                     //     li { "Item 2" }
    //                     //     li { "Item 3" }
    //                     //     li { "Item 4" }
    //                     // }
    //                     sub_menu
    //                 }
    //             }

    //             // Main content
    //             main { class: "main",
    //                 // button { class: "hamburger", onclick: toggle_sidebar,if *sidebar_open.read() { "<<" } else { ">>" } }
    //                 if !*sidebar_open.read() {button { class: "hamburger", onclick: toggle_sidebar, ">>" }} 
    //                 // button { class: "hamburger", onclick: toggle_sidebar, "☰" }
    //                 div { class: "filler", "2Content goes here..." }
    //                 div { class: "filler", {content} }
    //             }
    //         }
    //     }
    // }
}
