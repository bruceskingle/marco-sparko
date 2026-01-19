
use std::sync::Arc;

use dioxus::prelude::*;

use crate::{MarcoSparkoContext, ModuleRegistrations, views::*};


// use crate::views::{Blog, Home, Navbar};


/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
/// 
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    // The layout attribute defines a wrapper for all routes under the layout. Layouts are great for wrapping
    // many routes with a common UI like a navbar.
    #[layout(Navbar)]
        // The route attribute defines the URL pattern that a specific route matches. If that pattern matches the URL,
        // the component for that route will be rendered. The component name that is rendered defaults to the variant name.
        #[route("/")]
        Home {},
        // The route attribute can include dynamic parameters that implement [`std::str::FromStr`] and [`std::fmt::Display`] with the `:` syntax.
        // In this case, id will match any integer like `/blog/123` or `/blog/-456`.
        #[route("/blog/:module_id")]
        // Fields of the route variant will be passed to the component as props. In this case, the blog component must accept
        // an `id` prop of type `i32`.
        Module { module_id: String },
}

// We can import assets in dioxus with the `asset!` macro. This macro takes a path to an asset relative to the crate root.
// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
const FAVICON: Asset = asset!("/icons/icon.ico");
// The asset macro also minifies some assets like CSS and JS to make bundled smaller
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
// pub fn App(profile_manager: Arc<ProfileManager>) -> Element {
pub fn App() -> Element {
    println!("Rendering App component MAIN_CSS={}", MAIN_CSS);
    let mut init_signal = use_signal::<bool>(|| true);
    let init = *init_signal.read();
    let mut context_signal = use_signal::<Option<Arc<MarcoSparkoContext>>>(|| None);
    
    // Store the init signal in context so it can be accessed and reset from anywhere
    use_context_provider::<Signal<bool>>(move || init_signal);
    
    if init {
        let marco_sparko_context = MarcoSparkoContext::new()?;
        context_signal.set(Some(marco_sparko_context));
        // let x: Signal<Arc<MarcoSparkoContext>>;
        use_context_provider::<Signal<Option<Arc<MarcoSparkoContext>>>>(move || context_signal);

        let module_registrations = ModuleRegistrations::new();
        use_context_provider::<ModuleRegistrations>(move || module_registrations);

        init_signal.set(false);
        
        return rsx!{ "Loading..." };
    }
    rsx! {
        ErrorBoundary {
            handle_error: |errors: ErrorContext| {
                rsx! {

        

                    div { "Oops, we encountered an error. Please report this to the developer of this application" }
        
                    pre { "{errors:?}" }
                }
            },

            // In addition to element and text (which we will see later), rsx can contain other components. In this case,
            // we are using the `document::Link` component to add a link to our favicon and main CSS file into the head of our app.
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS }

            Router::<Route> {}
        }
    }
}

