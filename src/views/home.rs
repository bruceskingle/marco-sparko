use std::{collections::HashMap, sync::Arc};

use crate::{MarcoSparkoContext, ModuleRegistrations, components::app::Route};
use dioxus::prelude::*;


include!(concat!(env!("OUT_DIR"), "/crate_info.rs"));

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home(

    // xid: i32,
    // modules_signal: ModuleRegistrations
) -> Element {
    println!("TRace Home 1");
    let context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let opt_context = &*context_signal.read();
    let context = opt_context.as_ref().unwrap();
    let module_registrations = use_context::<ModuleRegistrations>();
    let mut modules = HashMap::new();

    println!("TRace Home 2");

    // println!("ZZ2 start id={} ", xid);
    // println!("ZZ2 start i={} {:?}", xid, modules_signal);
    for module_id in module_registrations.0.keys() {
        println!("ZZ2 module {}", module_id);
        let active = if context.profile.active_profile.modules.contains_key(module_id) {
            "Active"
        }
        else {
            "inactive"
        };

        modules.insert(module_id.clone(),active);
    }
    //  for (module_id, active) in &modules {
    //         println!("ZZ3 module {}", module_id);
    //     }

    
    let branch = if create_info::GIT_BRANCH.is_empty() {
        "N/A".to_string()
    } else {
        let mut s = create_info::GIT_BRANCH.to_string();
        if create_info::GIT_DIRTY  {
            
            s.push_str("-dirty");
        }
        if create_info::GIT_STAGED {
             s.push_str("-staged");
        }
        s
    };
    rsx! {
        div {
            // h1 { "This is Home #{xid}!" }
            h1 { "Modules" }
            for (module_id , active) in modules {
                Link {
                    class: "nav-item",
                    to: Route::Module {
                        module_id: module_id.clone(),
                    },
                    "{module_id}"
                }
                " [{active}]"
            }
            h2 { "Build Info" }
            table {
                // tr {
                //     td { "Package Name:" }
                //     td { "{create_info::PACKAGE_NAME}" }
                // }
                // tr {
                //     td { "Package Version:" }
                //     td { "{create_info::PACKAGE_VERSION}" }
                // }
                // tr {
                //     td { "User Agent:" }
                //     td { "{create_info::USER_AGENT}" }
                // }
                tr {
                    td { "Build Timestamp (UTC):" }
                    td { "{create_info::BUILD_TIMESTAMP}" }
                }
                // tr {
                //     td { "Git Repository:" }
                //     td { "{create_info::GIT_REPOSITORY}" }
                // }
                tr {
                    td { "Git Branch:" }
                    td { "{branch}" }
                }
                        // tr {
            //     td { "Git Dirty:" }
            //     td { "{create_info::GIT_DIRTY}" }
            // }
            // tr {
            //     td { "Git Staged:" }
            //     td { "{create_info::GIT_STAGED}" }
            // }
            // tr {
            //     td { "Git Stash:" }
            //     td { "{create_info::GIT_STASH}" }
            // }
            }
        }
    }
}
