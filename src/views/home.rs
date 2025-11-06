use std::{collections::HashMap, sync::Arc};

use crate::{MarcoSparkoContext, ModuleBuilder, ModuleRegistrations, components::Hero};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home(

    // xid: i32,
    // modules_signal: ModuleRegistrations
) -> Element {
    println!("TRace Home 1");
    let context_signal = use_context::<Signal<Option<Arc<MarcoSparkoContext>>>>();
    let opt_context = (&*context_signal.read());
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
    rsx! {
        div {
            // h1 { "This is Home #{xid}!" }
            h1 { "Modules"}
            for (module_id, active) in modules {
                "{module_id} [{active}]"
            }
        }
    }
}



// pub fn Home(
//     modules_signal: Signal<ModuleRegistrations>
// ) -> Element {
//     let mut modules = HashMap::new();

//     println!("ZZ2 start");
//     for module_id in ((*modules_signal.read()).0.keys()) {
//         println!("ZZ2 module {}", module_id);
//         modules.insert(module_id.clone(), "Inactive");
//     }
//      for (module_id, active) in &modules {
//             println!("ZZ3 module {}", module_id);
//         }
//     rsx! {
//         "Modules"
//         for (module_id, active) in modules {
//             "{module_id} [{active}]"
//         }

//     }
// }



// pub fn Home(mut resource: Resource<Result<MarcoSparko, Error>>) -> Element {
//     let x = &*resource.read();
//     match x {
//         Some(ms) => {
//             match ms {
//                 Ok(mms) => todo!(),
//                 Err(_) => todo!(),
//             }
//         },
//         None => todo!(),
//     }
//     rsx! {
//         match &mut *resou.read() {
//             Some(ms) => todo!(),
//             None => "Loading...",
//         }
//         //Hero {}

//     }
// }
