use crate::components::Hero;
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}

    }
}



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
