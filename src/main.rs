
use std::{collections::HashMap, sync::Arc};

use dioxus::prelude::*;
use marco_sparko::{DioxusContext, MarcoSparko, MarcoSparkoContext, ModuleRegistrations, components::app::App};
// use marco_sparko::PROFILE_MANAGER;



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

    Ok(())
}