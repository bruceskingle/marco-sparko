


use std::sync::Arc;
use clap::Parser;

use crate::{ MarcoSparkoContext, components::app::Route};
use dioxus::prelude::*;

// use crate::PROFILE_MANAGER;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
#[component]
/// A navigation bar with left-aligned items and a right-aligned dropdown.
/// Designed for Dioxus Desktop (but works for web too).
pub fn Navbar() -> Element {
    println!("Trace 1");

    let mut context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let opt_context = &*context_signal.read();
    let context = opt_context.as_ref().unwrap();


    // State for the dropdown menu
    let mut menu_open = use_signal(|| false);

    // Toggle function
    let toggle_menu = {
        let x = *menu_open.read();
        // let menu_open = menu_open.clone();
        move |_| menu_open.set(!x)
    };

    let mut all = Vec::new();
    for name in &context.profile.all_profiles {
        all.push(name.clone());
    }
    
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        nav { class: "nav",
            div { class: "nav-left",

                        Link {
                            class: "nav-item", 
                            to: Route::Home {},
                            "Home"
                        }
                        
                        for module_id in (context.profile.active_profile.modules.clone().keys()) {
                            Link {
                                class: "nav-item", 
                                to: Route::Module { module_id: module_id.clone() },
                                "{module_id}"
                            }
                        }
            }

            div { class: "spacer" }

            div { class: "nav-right",
                div { class: "dropdown",
                    button {
                        class: "dropdown-toggle",
                        onclick: toggle_menu,
                        // ARIA roles for accessibility
                        // aria_has_popup: "true",
                        aria_expanded: "{menu_open}" ,
                        
                        "Profile {context.profile.active_profile.name} "
                        svg {
                            width: "14", height: "14", view_box: "0 0 24 24",
                            path {
                                d: "M6 9l6 6 6-6",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                            }
                        }
                    }

                    // If menu is open, render overlay
                    if *menu_open.read() {
                        div {
                            class: "overlay",
                            onclick: move |_| menu_open.set(false),
                        }
                    }

                    

                    // Dropdown menu
                    div {
                        class: if *menu_open.read() { "menu open" } else { "menu" },

                        for name in all {
                            div { class: "menu-item", onclick:  move |_| {
                                
                                menu_open.set(false);
                                let new_context = Arc::new(MarcoSparkoContext {
                                        args: crate::Args::parse(),
                                        profile: crate::profile::set_active_profile(&name)?,
                                });

                                context_signal.set(Some(new_context));
                                Ok(())
                            }, "{&name}" }
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}

