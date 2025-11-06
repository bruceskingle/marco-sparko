
use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{DioxusContext, MarcoSparko, ModuleRegistrations, components::app::Route};

const BLOG_CSS: Asset = asset!("/assets/styling/blog.css");

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn Blog(module_id: String) -> Element {
    println!("ZZ2 Blog start i={}", module_id);

    let context = use_context::<DioxusContext>();

    let mut call_signal = use_signal::<bool>(|| true);
    let mut action = use_action( move |module_registrations: ModuleRegistrations, marco_sparko_context: std::sync::Arc<crate::MarcoSparkoContext>, module_id: String|  async move { MarcoSparko::do_initialize(&module_id, false, &module_registrations, &marco_sparko_context).await});

    if *call_signal.read() {
        call_signal.set(false);
        let t = action.call(context.module_registrations.clone(), context.marco_sparko_context.clone(), module_id.clone());
    }
    
    if let Some(result) = action.value() {
        let r = result?;
        let z = r.read();
        let x = z.as_component();
        // let x = module.get_component();
        x()
    }
    else {
         rsx!{
            "Loading {module_id}..."
        }
    }



//     // let context_ref = context.marco_sparko_context.clone();
//     // let reg = context.module_registrations.clone();
//     let mut action = use_action( move |module_registrations: ModuleRegistrations, marco_sparko_context: std::sync::Arc<crate::MarcoSparkoContext>, module_id: String|  async move { MarcoSparko::do_initialize(&module_id, false, &module_registrations, &marco_sparko_context).await});

//     let x = action.pending()
//     // let t = action.call(context.module_registrations.clone(), context.marco_sparko_context.clone(), module_id.clone());

//    if let Some(result) = action.value() {
//         let x = result?.read().get_component();
//         // let x = module.get_component();
//         x()
//     }
//     else {
//          rsx!{
//         "Loading {module_id}..."
//     }
    // }



    // // let mut modules = HashMap::new();
    // // for module_id in modules_signal.0.keys() {
    // //     println!("ZZ2 module {}", module_id);
    // //     modules.insert(module_id.clone(), "Inactive");
    // // }
    // //  for (module_id, active) in &modules {
    // //         println!("ZZ3 module {}", module_id);
    // //     }
    // rsx! {
    //     document::Link { rel: "stylesheet", href: BLOG_CSS }

    //     div {
    //         id: "blog",

    //         // Content
    //         h1 { "This is blog #{id}!" }
    //         p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

    //         // Navigation links
    //         // The `Link` component lets us link to other routes inside our app. It takes a `to` prop of type `Route` and
    //         // any number of child nodes.
    //         // Link {
    //         //     // The `to` prop is the route that the link should navigate to. We can use the `Route` enum to link to the
    //         //     // blog page with the id of -1. Since we are using an enum instead of a string, all of the routes will be checked
    //         //     // at compile time to make sure they are valid.
    //         //     to: Route::Blog { id: id - 1, modules_signal: modules_signal.clone() },
    //         //     "Previous"
    //         // }
    //         // span { " <---> " }
    //         // Link {
    //         //     to: Route::Blog { id: id + 1, modules_signal: modules_signal.clone() },
    //         //     "Next"
    //         // }
    //     }
    // }
}
