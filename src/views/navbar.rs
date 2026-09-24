use std::sync::Arc;

use crate::{ MarcoSparkoContext, UpgradeRequested, components::app::Route};
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
    let mut context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let mut upgrade_requested = use_context_provider::<Signal<UpgradeRequested>>(|| Signal::new(UpgradeRequested(false)));
    let context = context_signal.read().as_ref().unwrap().clone();


    // State for the dropdown menu
    let mut menu_open = use_signal(|| false);
    let mut new_profile_open = use_signal(|| false);
    let mut new_profile_generation = use_signal(|| 0u32);
    let mut new_profile_name = use_signal(String::new);
    let mut new_profile_error = use_signal(|| None::<String>);

    let mut all = Vec::new();
    for name in &context.profile.all_profiles {
        all.push(name.clone());
    }
    
    // get the current route so we can mark the active nav item
    let current_route = use_route::<Route>();
    let navigator = use_navigator();
    
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        nav { class: "nav",
            div { class: "nav-left",

                Link {
                    class: if matches!(&current_route, Route::Home {}) { "nav-item active" } else { "nav-item" },
                    to: Route::Home {},
                    "Home"
                }

                for module_id in (context.profile.active_profile.modules.clone().keys()) {
                    Link {
                        class: match &current_route {
                            Route::Module { module_id: mid } if mid == module_id => "nav-item active",
                            _ => "nav-item",
                        },
                        to: Route::Module {
                            module_id: module_id.clone(),
                        },
                        "{module_id}"
                    }
                }
            }

            div { class: "spacer" }

            div { class: "nav-right",
                div { class: "dropdown",
                    button {
                        key: "{new_profile_generation}",
                        class: "dropdown-toggle",
                        onclick: move |_| {
                            if !*new_profile_open.read() {
                                let is_open = *menu_open.read();
                                menu_open.set(!is_open);
                            }
                        },
                        // ARIA roles for accessibility
                        // aria_has_popup: "true",
                        aria_expanded: "{menu_open}",

                        if *new_profile_open.read() {
                            if let Some(error) = &*new_profile_error.read() {
                                span { class: "invalid", "{error}" }
                            }
                            input {
                                class: "profile-rename",
                                r#type: "text",
                                placeholder: "Profile name",
                                autofocus: true,
                                onmounted: move |event| async move {
                                    let _ = event.data().set_focus(true).await;
                                },
                                value: "{new_profile_name}",
                                oninput: move |event| new_profile_name.set(event.value().clone()),
                                onclick: move |event| event.stop_propagation(),
                                onkeydown: move |event| {
                                    match event.key().to_string().as_str() {
                                        "Enter" => {
                                            let profile_name = new_profile_name.read().clone();
                                            match crate::profile::create_profile(&profile_name) {
                                                Ok(active_profile) => {

                                                    // start
                                                    // let new_context = Arc::new(MarcoSparkoContext {
                                                    //     args: crate::Args::ms_parse(),
                                                    //     profile: active_profile,
                                                    // });
                                                    // context_signal.set(Some(new_context));
                                                    // end
                                                    let new_context = Arc::new(MarcoSparkoContext {
                                                        args: crate::Args::ms_parse(),
                                                        profile: active_profile,
                                                    });
                                                    context_signal.set(Some(new_context));
                                                    //next
                                                    upgrade_requested.set(UpgradeRequested(false));
                                                    new_profile_open.set(false);
                                                    new_profile_name.set(String::new());
                                                    new_profile_error.set(None);
                                                    navigator.replace(Route::Home {});
                                                }
                                                Err(error) => new_profile_error.set(Some(error.to_string())),
                                            }
                                        }
                                        "Escape" => {
                                            new_profile_open.set(false);
                                            new_profile_name.set(String::new());
                                            new_profile_error.set(None);
                                        }
                                        _ => {}
                                    }
                                },
                            }
                        } else {
                            "Profile {context.profile.active_profile.name} "
                            svg {
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M6 9l6 6 6-6",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                }
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
                    div { class: if *menu_open.read() { "menu open" } else { "menu" },

                        for name in all {
                            div {
                                class: "menu-item",
                                onclick: move |_| {

                                    menu_open.set(false);
                                    let new_context = Arc::new(MarcoSparkoContext {
                                        args: crate::Args::ms_parse(),
                                        profile: crate::profile::set_active_profile(&name)?,
                                    });

                                    context_signal.set(Some(new_context));
                                    upgrade_requested.set(UpgradeRequested(false));
                                    navigator.replace(Route::Home {});
                                    Ok(())
                                },
                                "{&name}"
                            }
                        }

                        div {
                            class: "menu-item create-new",
                            onclick: move |_| {
                                menu_open.set(false);
                                new_profile_open.set(true);
                                let generation = *new_profile_generation.read() + 1;
                                new_profile_generation.set(generation);
                                new_profile_error.set(None);
                                new_profile_name.set(String::new());
                            },
                            "Create New Profile"
                        }
                    }
                }
            }
        }

        Outlet::<Route> {}
    }
}

