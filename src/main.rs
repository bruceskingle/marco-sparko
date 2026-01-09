use std::any::Any;

use marco_sparko::{ components::app::App};
// use marco_sparko::PROFILE_MANAGER;

use dioxus_desktop::{
    launch::launch,
    Config,
    WindowBuilder,
};

// struct AppProps {
//     profile_manager: Arc<ProfileManager>,
// }
// const MODULE_REGISTRATIONS: ModuleRegistrations = MarcoSparko::load_modules();


fn main() -> anyhow::Result<()> {
    // println!("main...");
    // let profile_manager: Arc<ProfileManager> = Arc::new(ProfileManager::new(ProfileSelector::Last)?);

    // let x = PROFILE_MANAGER.get_or_init(move || profile_manager);
    // let props = AppProps {
    //     profile_manager
    // };

    // let app_func = move || App(props);


    dioxus::launch(App);



//     /*
//     pub fn launch(
//     root: fn() -> Element,
//     contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
//     platform_config: Vec<Box<dyn Any>>,
// ) -> !
//      */

// // create a boxed Config for platform_config
//     let window_config: Box<dyn Any> = Box::new(
//         Config::new().with_window(
//             WindowBuilder::new().with_url("dioxus://localhost/"),
//         )
//     );

//     launch(
//         App,
//         Vec::<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>::new(), // empty contexts
//         vec![window_config], // must be a Vec<Box<dyn Any>>
//     );

    Ok(())
}