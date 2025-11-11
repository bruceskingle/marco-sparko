//! The views module contains the components for all Layouts and Routes for our app. Each layout and route in our [`Route`]
//! enum will render one of these components.
//!
//!
//! The [`Home`] and [`Blog`] components will be rendered when the current route is [`Route::Home`] or [`Route::Blog`] respectively.
//!
//!
//! The [`Navbar`] component will be rendered on all pages of our app since every page is under the layout. The layout defines
//! a common wrapper around all child routes.

mod home;
use std::collections::HashMap;

use dioxus::signals::Signal;
pub use home::Home;

mod module;
pub use module::Module;

mod navbar;
pub use navbar::Navbar;

use crate::PageInfo;

pub mod profile;
pub mod page_content;
// pub mod sidebar_menu;
// // pub mod dynamic_router;



// #[derive(Clone)]
// struct RuntimePages {
//     pages: Signal<HashMap<String, PageInfo>>,
// }
