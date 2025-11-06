


use std::sync::Arc;
use clap::Parser;

use crate::{ MarcoSparko, MarcoSparkoContext, ModuleBuilder, ModuleRegistrations, components::app::Route, profile::ProfileManager, views::profile::Profile};
use dioxus::{CapturedError, prelude::*};

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
    let opt_context = (&*context_signal.read());
    let context = opt_context.as_ref().unwrap();
    let module_registrations = use_context::<ModuleRegistrations>();

    // let modules_signal: Signal<ModuleRegistrations> = use_signal(|| MarcoSparkoC::load_modules());

   


    // for module_id in ((*modules_signal.read()).0.keys()) {
    //     println!("ZZ module {}", module_id);
    // }

    //     for module_id in ((*modules_signal.read()).0.keys()) {
    //     println!("ZZb module {}", module_id);
    // }

    // let mut context_signal: Signal<Option<Arc<MarcoSparkoContext>>>  = use_signal(|| None);
    // // let opt_context = &*context_signal.read();
    // if (&*context_signal.read()).is_none() {
    //     let context = MarcoSparkoContext::new()?;
    //     context_signal.set(Some(context));
    //     return rsx!("Loading...");
    // };

    // let opt_context = &*context_signal.read();
    // let context = opt_context.as_ref().unwrap();
    // let profile = &context.marco_sparko_context.profile;

    // let mut active_profile: Signal<Option<crate::profile::ActiveProfile>> = use_signal(|| None);


    // println!("Trace 2");
    
    // let the_profile = crate::profile::fetch_active_profile()?;
    // let update = if let Some(p) = &*active_profile.read() {
    //     &p.active_profile.name != &the_profile.active_profile.name
    // }
    // else {
    //     true
    // };
    // println!("Trace 3");

    // if update {
    //     active_profile.set(Some(the_profile));
    // }
    println!("Trace 4");

   

    // // let profile_manager_signal = use_signal(move || ProfileManager::new(&None));

    // // let x = &*profile_manager_signal.read();

    // // let mut profile_manager = match x {
    // //     Ok(p) => p,
    // //     Err(e) => 
    // //         return Err(RenderError::Error(CapturedError(Arc::new(anyhow!("{e}"))))),
    // // };

    // // let y = (&profile_manager.active_profile.name).clone();
    // // let mut active_profile = use_signal(move || (&profile_manager.active_profile.name).clone());
    // // let profile_label = &*active_profile.read().clone();
    // // // let mut active_profile_name = use_signal(profile_manager.read())

    // let profile_manager = crate::PROFILE_MANAGER.get().unwrap();
    // let mut active_profile: Signal<String> = use_signal(|| profile_manager.active_profile.name.clone());

    // // let mut active_profile: Signal<Option<String>> = use_signal(|| None);
    // // // let x = &*active_profile.read();
    // // let profile_manager = ProfileManager::new(&*active_profile.read())?;
    // let profile_label = profile_manager.active_profile.name.clone();

    // let mut profile_names = Vec::new();

    // for name in &profile_manager.profile_names {
    //     profile_names.push(name.clone());
    // }

    // // for item in profile_manager.before_profiles {
    // //     profile_names.push(item.name);
    // // }
    
    // // profile_names.push(profile_label.clone());

    // // for item in profile_manager.after_profiles {
    // //     profile_names.push(item.name);
    // // }

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
    
    println!("Trace 5");


    // Toggle function
    let toggle_menu = {
        let x = *menu_open.read();
        // let menu_open = menu_open.clone();
        move |_| menu_open.set(!x)
    };

    // let opt_profile = (&*active_profile.read());
    // let profile = opt_profile.as_ref().unwrap();
    // let all_profiles = profile.all_profiles.clone();
    // let modules = &profile.active_profile.modules;


    // // ✅ Deriving state with use_memo
    // let first_signal = use_signal(|| 0);
    // // Memos are specifically designed for derived state. If your state fits this pattern, use it.
    // let second_signal = use_memo(move || {
    //     let opt_profile = active_profile();
    //     let profile = opt_profile.as_ref().unwrap();
    //     let all_profiles = profile.all_profiles.clone();
    //     let modules = &profile.active_profile.modules;
    // });

    // for name in modules.keys() {
    // }
    //  let m: ModuleRegistrations = (*modules_signal.read()).clone();


    //     let mm = m.clone();
    //      for module_id in (mm.0.keys()) {
    //     println!("ZZz2 module {}", module_id);
    // }

        //  for module_id in (m.0.keys()) {
        // println!("ZZz module {}", module_id);
        //  }
// let new_profile = crate::profile::set_active_profile(&String::from("default"))?;
    let mut all = Vec::new();
    for name in &context.profile.all_profiles {
        all.push(name.clone());
    }
    
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
                            // for module_id in (m.0.keys()) {
                            //     "-{module_id}"
                            // }
                        }
                        
            //             for name in &context.marco_sparko_context.profile.active_profile.modules.keys() {
                            for module_id in (context.profile.active_profile.modules.clone().keys()) {
                        //     "ZZz module {module_id}"
                        // }
                            Link {
                                class: "nav-item", 
                                to: Route::Blog { module_id: module_id.clone() },
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

                                // let new_profile = crate::profile::set_active_profile(&name);

                                // pub fn set_active_profile(profile_name: &String) -> anyhow::Result<ActiveProfile>

//                                 let n =  {

//     let mut all_profiles = Vec::new();
//     let mut map = indexmap::IndexMap::new();
    

//     if let Ok(file)= std::fs::File::open(&MarcoSparko::get_file_path()?) {
//         let profile_file: crate::profile::ProfileFile = serde_json::from_reader(file)?;

//         for profile in profile_file {
//             // if map.contains_key(&profile.name) {
//             //     return Err(anyhow!("Duplicate profile \"{}\"", &profile.name));
//             // }
//             all_profiles.push(profile.name.clone());
//             map.insert(profile.name.clone(), profile);
//         }
//         let active_profile = if let Some(p) = map.shift_remove(&name) {
//             p
//         }
//         else {
//             panic!("Cant happen");
//             // return Err(anyhow!("No such profile \"{}\"", profile_name)); 
//         };

//         let mut profiles = Vec::new();

//         profiles.push(&active_profile);
//         profiles.extend(map.values());

//         serde_json::to_writer_pretty(std::fs::File::create(&MarcoSparko::get_file_path()?)?, &profiles)?;

//         crate::profile::ActiveProfile {
//             all_profiles,
//             active_profile,
//         }
//     }
//     else {
//         panic!("Cant happen");
//         // Err(anyhow!("No such profile \"{}\" (no profiles at all, in fact)", profile_name))
//     }
// };
                                
                                menu_open.set(false);
                                // let new_context = context.with_profile(&name)?;

                                // let new_profile = crate::profile::set_active_profile(&name)?;
                                let new_context = Arc::new(MarcoSparkoContext {
                                        args: crate::Args::parse(),
                                        profile: crate::profile::set_active_profile(&name)?,
                                });

                                context_signal.set(Some(new_context));
                                // use_context_provider::<Arc<MarcoSparkoContext>>(move || new_context);



                                
                                // let args = ((&*context_signal.read()).as_ref().unwrap()).args.clone();
                                // let new_profile = crate::profile::set_active_profile(&name)?;
                                // let new_context = Some(Arc::new(MarcoSparkoContext {
                                //     args,
                                //     profile: new_profile,
                                // }));
                                // context_signal.set(new_context);
                                // active_profile.set(Some(new_profile));
                                Ok(())
                            }, "{&name}" }
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
