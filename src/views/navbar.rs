

use crate::{components::app::Route, profile::ProfileManager, views::profile::Profile};
use dioxus::{CapturedError, prelude::*};


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
    // let profile_manager_signal = use_signal(move || ProfileManager::new(&None));

    // let x = &*profile_manager_signal.read();

    // let mut profile_manager = match x {
    //     Ok(p) => p,
    //     Err(e) => 
    //         return Err(RenderError::Error(CapturedError(Arc::new(anyhow!("{e}"))))),
    // };

    // let y = (&profile_manager.active_profile.name).clone();
    // let mut active_profile = use_signal(move || (&profile_manager.active_profile.name).clone());
    // let profile_label = &*active_profile.read().clone();
    // // let mut active_profile_name = use_signal(profile_manager.read())




    let mut active_profile: Signal<Option<String>> = use_signal(|| None);
    // let x = &*active_profile.read();
    let profile_manager = ProfileManager::new(&*active_profile.read())?;
    let profile_label = profile_manager.active_profile.name;

    let mut profile_names = Vec::new();

    for item in profile_manager.before_profiles {
        profile_names.push(item.name);
    }
    
    profile_names.push(profile_label.clone());

    for item in profile_manager.after_profiles {
        profile_names.push(item.name);
    }

    // State for the dropdown menu
    let mut menu_open = use_signal(|| false);
    // let mut current_profile: Signal<&crate::profile::Profile> = use_signal(|| &profile_manager.active_profile);
    // let profile_label = (current_profile.read()).name.clone();
    
    //  let profile_label = String::from("TEST");
    
    // if let Some(profile) = &*current_profile.read() {
    //     profile.clone()
    // }
    // else {
    //     String::from("Select Profile")
    // };
    


    // Toggle function
    let toggle_menu = {
        let x = *menu_open.read();
        // let menu_open = menu_open.clone();
        move |_| menu_open.set(!x)
    };

    
    
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        nav { class: "nav",
            div { class: "nav-left",
                // a { class: "brand", href: "#", "MySite" }
                // a { class: "nav-item", href: "#", "Home" }
                // a { class: "nav-item", href: "#", "Features" }
                // a { class: "nav-item", href: "#", "Pricing" }
                // a { class: "nav-item", href: "#", "Docs" }
                // a { class: "nav-item", href: "#", "Blog" }

                        Link {
                            class: "brand", 
                            to: Route::Home {},
                            "Home"
                        }
                        Link {
                            class: "nav-item", 
                            to: Route::Blog { id: 1 },
                            "Blog"
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
                        
                        "Profile {profile_label} "
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

                        for profile_name in profile_names {
                            div { class: "menu-item", onclick:  move |_| {
                               menu_open.set(false);
                                active_profile.set(Some(profile_name.clone()))
                            }, "{&profile_name}" }
                        }
                        // div { class: "menu-item", onclick: move |_| menu_open.set(false), "{profile_label}" }
                        // for item in &profile_manager.after_profiles {
                        //     let item_name = ""; //(&item.name).clone();
                        //     div { class: "menu-item", onclick: move |_| {
                        //         menu_open.set(false);
                        //         active_profile.set(item_name)
                        //     }, "{item.name}" }
                        // }
                        // // div { class: "menu-item", onclick: move |_| {menu_open.set(false); current_profile.set(Some(String::from("default")));} , "default" }
                        // div { class: "menu-item", onclick: move |_| menu_open.set(false), "prof2" }
                        // div { class: "menu-item", onclick: move |_| menu_open.set(false), "another" }

                        // // a { class: "menu-item", href: "#", onclick: move |_| menu_open.write(), "Profile" }
                        // // a { class: "menu-item", href: "#", onclick: move |_| menu_open.set(false), "Settings" }
                        // // a { class: "menu-item", href: "#", onclick: move |_| menu_open.set(false), "Sign out" }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}


// pub fn Navbar() -> Element {
//     // rsx! {
//     //     document::Link { rel: "stylesheet", href: NAVBAR_CSS }

//     //     div {
//     //         id: "navbar",
//     //         Link {
//     //             to: Route::Home {},
//     //             "Home"
//     //         }
//     //         Link {
//     //             to: Route::Blog { id: 1 },
//     //             "Blog"
//     //         }

//     //         Profile {}
//     //     }

//     //     nav {
//     //         class: "nav",
//     //         aria-label: "Main navigation",

//     //         Link {
//     //             class: "nav-left"
//     //             to: Route::Home {},
//     //             "Home"
//     //         }
//     //         Link {
//     //             class: "nav-left"
//     //             to: Route::Blog { id: 1 },
//     //             "Blog"
//     //         }
//     //         div {
//     //             class: "spacer",
//     //             aria-hidden: "true",
//     //         }

//     //         div {
//     //             class: "nav-right",
//     //             div {
//     //                 class: "dropdown",
//     //                 button {
//     //                     class: "dropdown-toggle",
//     //                     id: "accountToggle",
//     //                     aria-haspopup: "true",
//     //                     aria-expanded: "false"
//     //                     Account
//     //                     svg {
//     //                         width: "14",
//     //                         height: "14",
//     //                         viewBox: "0 0 24 24",
//     //                         fill: "none",
//     //                         aria-hidden: "true"
//     //                         path {
//     //                             d: "M6 9l6 6 6-6",
//     //                             stroke: "currentColor",
//     //                             stroke-width: "2",
//     //                             stroke-linecap: "round",
//     //                             stroke-linejoin: "round"
//     //                         }
//     //                     }
//     //                 }

//     //             <div class: "menu" id: "accountMenu" role: "menu" aria-labelledby: "accountToggle">
//     //             <a class: "menu-item" href: "#" role: "menuitem">Profile</a>
//     //             <a class: "menu-item" href: "#" role: "menuitem">Settings</a>
//     //             <a class: "menu-item" href: "#" role: "menuitem">Sign out</a>
//     //             </div>
//     //         </div>
//     //         }
//     //     }


//         // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
//         // the [`Home`] or [`Blog`] component depending on the current route.
//         Outlet::<Route> {}
//     }
// }
