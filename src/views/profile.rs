use crate::components::app::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");
const SELECT_PROFILE: &str = "Select profile...";

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
#[component]
pub fn Profile() -> Element {  
    let mut pet = use_signal(|| String::from(""));
    rsx!(
        div {
            id: "profile",
            // h1 { "pet : {pet}" }
            if pet.read().len() == 0 {
                select { onchange: move |e| pet.set(e.value()),
                    option { disabled: true, "Select profile..." }
                    option { "dog" }
                    option { "cat" }
                    option { "others" }
                }
            } else {
                select { onchange: move |e| pet.set(e.value()),
                    option { "dog" }
                    option { "cat" }
                    option { "others" }
                }
            }
            
        }
    )
}

