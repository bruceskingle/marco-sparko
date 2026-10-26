
mod ordered_map;
pub use ordered_map::OrderedMap;

mod octopus;
mod util;
mod views;
pub mod components;
mod profile;

mod private_file;
pub mod data_home;

#[cfg(feature = "desktop")]
pub mod window_state;

mod data_set;
mod cache;
use cache::CacheConfig;

use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use anyhow::anyhow;
use async_trait::async_trait;
use dioxus::core::Element;
use serde::de::DeserializeOwned;
use serde::Serialize;
use clap::{Parser};

use crate::profile::ActiveProfile;

const NONE: &'static str = "None";
const NULL: &'static str = "Null";
const CACHE_DIRECTORY_NAME: &'static str = ".marco-sparko-cache";

/*
 * It is important that the names of all args here do not contain hyphens or underscores
 * as these are used to separate module args from main args.
 * 
 * For example, to pass an arg to the octopus module, use --octopus-<arg_name> and in the corresponding Arg struct in the octopus module
 * all field names must begin with octopus_ so octopus_arg_name in this example.
 * 
 * Underscores in the names of fields on this struct are translated into hyphens on the command line automatically by clap.
 */
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
pub struct MarcoSparkoArgs {
    /// Name of the config profile to use
    #[arg(short, long)]
    profile: Option<String>,
    #[arg(short, long, value_delimiter = ',', num_args = 1..)]
    modules: Vec<String>,
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long)]
    verbose: bool,
    /// The directory holding .marco-sparko and .marco-sparko-cache, instead of the home directory
    #[arg(long)]
    home: Option<PathBuf>,
    /// Fetch the latest data of every module for the last 30 days, without opening a window, then exit, e.g. from a
    /// scheduled task, see docs/ScheduledFetch.md
    #[arg(long)]
    fetch: bool,
}

pub struct Args {
    pub marco_sparko_args: MarcoSparkoArgs,
    pub module_args: HashMap<String, Vec<String>>,
}

impl Args {
    fn extract_module(s: &str) -> Option<&str> {
        // Only the name identifies the module, the value may contain hyphens, e.g. --home=/tmp/ms-test
        let s = s.trim_start_matches('-').split('=').next().unwrap_or_default();
        if let Some((module, _)) = s.split_once('-') {
            if module.trim().is_empty() {
                None
            }
            else {
                Some(module)
            }   
        }
        else {
            None
        }
    }

    pub fn ms_parse() -> Self {
        let mut main_only_args = Vec::new();
        let mut module_args: HashMap<String, Vec<String>> = HashMap::new();
        let mut module_id: Option<String> = None;

        let mut it = std::env::args();
        let arg0 = it.next().unwrap_or_else(|| "marco-sparko".to_string());
        main_only_args.push(arg0.clone());
        for arg in it {
            if arg.starts_with("-") {
                if let Some(module) = Args::extract_module(&arg) {
                    module_id = Some(module.to_string());
                }
            }
            if let Some(module) = &module_id {
                    module_args
                    .entry(module.clone())
                    .or_insert_with(|| { let mut v = Vec::new(); v.push(arg0.clone()); v})
                    .push(arg);
            }
            else {
                main_only_args.push(arg);
            }
        }

        let marco_sparko_args = MarcoSparkoArgs::parse_from(main_only_args);

        if marco_sparko_args.verbose {
            println!("Main args: {:?}", marco_sparko_args);
            // Only the names of the module args are shown as their values may be secrets, e.g. -o for an API key, which
            // modules show redacted once they are parsed.
            for (module_id, args) in &module_args {
                let names: Vec<&str> = args.iter()
                    .filter(|arg| arg.starts_with('-'))
                    .map(|arg| arg.split('=').next().unwrap_or_default())
                    .collect();
                println!("Module '{}' args: {:?}", module_id, names);
            }
        }

        Args {
            marco_sparko_args,
            module_args,
        }
    }

    pub fn module_args(&self, module_name: &str) ->  Option<Vec<String>> {
        self.module_args.get(module_name).cloned()
    }

    /// Whether --fetch was given, to fetch the latest data without a window, see fetch_latest_data.
    pub fn fetch(&self) -> bool {
        self.marco_sparko_args.fetch
    }
}

/// Fetches the latest data of each module of the profile, see Module::fetch_latest_data, without a window, for --fetch.
/// A module which is not logged in, or whose fetch fails, is reported and the others are still fetched. Returns whether
/// they all succeeded, for the exit status.
pub fn fetch_latest_data(args: Args) -> bool {
    let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Unable to start: {error}");
            return false;
        },
    };
    println!("Marco Sparko {} fetching the latest data at {} UTC", env!("CARGO_PKG_VERSION"), time::OffsetDateTime::now_utc());
    let verbose = args.marco_sparko_args.verbose;
    let result = runtime.block_on(async move {
        let context = MarcoSparkoContext::new(args)?;
        let registrations = ModuleRegistrations::new(verbose);
        let module_ids: Vec<String> = context.profile.active_profile.modules.keys().cloned().collect();
        if module_ids.is_empty() {
            return Err(anyhow!("Profile {} has no modules set up, open the app and log in first", context.profile.active_profile.name));
        }
        let mut all_ok = true;
        for module_id in module_ids {
            println!("Module {module_id}");
            let fetched = async {
                let factory = registrations.module_factory(&module_id, &context).await?;
                if !factory.is_ready().await? {
                    return Err(anyhow!("Not logged in, open the app and log in first"));
                }
                factory.build().await?.fetch_latest_data().await
            }.await;
            match fetched {
                Ok(()) => println!("Module {module_id}: done"),
                Err(error) => {
                    println!("Module {module_id}: FAILED: {error:#}");
                    all_ok = false;
                },
            }
        }
        anyhow::Ok(all_ok)
    });
    match result {
        Ok(all_ok) => {
            println!("Finished{}", if all_ok { "" } else { " with errors" });
            all_ok
        },
        Err(error) => {
            println!("FAILED: {error:#}");
            false
        },
    }
}


#[derive(Clone)]
pub struct PageInfo {
    label: &'static str,
    path: &'static str,
}

//  let x: fn(ModuleProps) -> std::result::Result<VNode, RenderError> = Module;
/// How many days back --fetch fetches, see Module::fetch_latest_data.
pub const FETCH_DAYS: i64 = 30;

// Not Send, as fetch_latest_data is run on one thread, see fetch_latest_data, and the data sets' futures are not Send.
#[async_trait(?Send)]
pub trait Module {
    fn get_page_list(&self) -> Vec<PageInfo>;
    fn module_id(&self) -> &'static str;
    fn get_component(&self, page_id: &str, path: Vec<String>) -> Element;
    fn cli_debug(&self) -> anyhow::Result<()>;
    /// Fetches the latest data of every page, as if each was shown with the last FETCH_DAYS days, printing its progress,
    /// for --fetch. Data which is already cached and up to date is not fetched again.
    async fn fetch_latest_data(&self) -> anyhow::Result<()>;
}

#[async_trait]
pub trait ModuleFactory: Send {
    async fn is_ready(&self) -> anyhow::Result<bool>;
    fn init_page(&self) -> Element;
    async fn build(&self) -> anyhow::Result<Box<dyn Module + Send>>;
}

pub type ModuleFactoryConstructor = dyn Fn(Arc<MarcoSparkoContext>, Option<serde_json::Value>) -> anyhow::Result<Arc<dyn ModuleFactory>>;

pub struct ModuleRegistration {
    pub module_id: String,
    pub constructor: Arc<ModuleFactoryConstructor>,
}

#[derive(Clone, Default)]
pub struct ModuleRegistrations(Arc<HashMap<String, ModuleRegistration>>);

impl std::fmt::Debug for ModuleRegistrations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for k in self.0.keys() {
             write!(f, "{}, ", k)?;
        }
         write!(f, "]")?;
         Ok(())
    }
}

impl PartialEq for ModuleRegistrations {
    fn eq(&self, other: &Self) -> bool {
        let s = self.0.keys();
        let o = other.0.keys();
        let mut it = o.into_iter();

        for k in s {
            if let Some(other_key) = it.next() {
                if k != other_key {
                    return false;
                }
            }
        }

        it.next().is_none()
    }
}

impl ModuleRegistrations {
    pub async fn module_factory(&self, module_id: &str, context: &Arc<MarcoSparkoContext>) -> anyhow::Result<Arc<dyn ModuleFactory + Send>> {
        if let Some(module_registration) = self.0.get(module_id) {
            let constructor = module_registration.constructor.as_ref();
            let profile = if let Some(value) = context.profile.active_profile.modules.get(module_id) {
                Some(value.clone())
            }
            else {
                None
            };

            if context.args.marco_sparko_args.verbose {
                let mut redacted_profile = profile.clone();
                if let Some(value) = &mut redacted_profile {
                    sparko_graphql::redact::redact_secrets(value);
                }
                println!("Initializing module '{}' with profile '{:?}'", module_id, redacted_profile);
            }
            let builder = constructor(context.clone(), profile)?;

            Ok(builder)
        }
        else {
            return Err(anyhow!(format!("Unknown module \"{}\"", module_id)))
        }
    }

    fn new(verbose: bool) -> ModuleRegistrations {


        let dir = std::env::current_dir().unwrap();

        if verbose {
            println!("Current directory is {}", dir.display());
        }
        //assert!(cfg!(debug_assertions));

        let mut module_registrations = HashMap::new();

        Self::load_module(&mut module_registrations, octopus::OctopusModule::registration(), verbose);

        if verbose {
            println!("Loaded {} modules", module_registrations.len());

            for (k, _v) in &module_registrations {
                println!(" Module {}", k);
            }
        }
        ModuleRegistrations(Arc::new(module_registrations))
    }

    fn load_module(module_registrations: &mut HashMap<String, ModuleRegistration> , registration: ModuleRegistration, verbose: bool) {
        if verbose {
            println!("Load module {}", &registration.module_id);
        }   
        module_registrations.insert(registration.module_id.clone(), registration);
    }
}

 pub struct MarcoSparkoContext {
    pub args: Args,
    pub profile: ActiveProfile,
}

#[derive(Clone, Copy, Default)]
pub struct UpgradeRequested(pub bool);

#[derive(Clone, Copy, Default)]
pub struct InitRequested(pub bool);

// impl PartialEq for MarcoSparkoContext {
//     fn eq(&self, other: &Self) -> bool {
//         self.args == other.args && self.profile == other.profile
//     }
// }

impl MarcoSparkoContext {
    /// The profile file, see data_home.
    pub fn get_file_path() -> anyhow::Result<PathBuf> {
        Ok(data_home::data_home()?.join(".marco-sparko"))
    }

    /// The directory holding the cache files of every profile, see data_home.
    pub fn get_cache_dir_path() -> anyhow::Result<PathBuf> {
        Ok(data_home::data_home()?.join(CACHE_DIRECTORY_NAME))
    }

    pub fn new(args: Args) -> anyhow::Result<Arc<MarcoSparkoContext>> {
        private_file::create_private_dir(&Self::get_cache_dir_path()?)?;

       
        let profile = crate::profile::fetch_active_profile(&args.marco_sparko_args.profile)?;
        

        Ok(Arc::new(MarcoSparkoContext {
            args,
            profile,
       }))
    }

    // pub fn with_profile(&self, profile_name: &String) -> anyhow::Result<Arc<MarcoSparkoContext>> {
    //     Ok(Arc::new(MarcoSparkoContext {
    //         args: self.args.clone(),
    //         profile: crate::profile::set_active_profile(profile_name)?,
    //    }))
    // }

    fn get_cache_file_path(&self, module_id: &str) -> anyhow::Result<PathBuf> {
        let profile_name = &self.profile.active_profile.name;
        Ok(Self::get_cache_dir_path()?.join(format!("{}-{}.json", profile_name, module_id)))
    }

    fn get_cache_data_dir_path(&self, module_id: &str) -> anyhow::Result<PathBuf> {
        let profile_name =&self.profile.active_profile.name;
        Ok(Self::get_cache_dir_path()?.join(format!("{}-{}", profile_name, module_id)))
    }
      
    fn create_cache_config(&self, module_id: &str, verbose: bool) -> anyhow::Result<Arc<CacheConfig>> {
        let dir_path = self.get_cache_data_dir_path(module_id)?;
        private_file::create_private_dir(&dir_path)?;

        Ok(Arc::new(CacheConfig::new(dir_path,verbose)))
    }

    pub fn read_cache<T>(&self, module_id: &str) -> Option<T>
    where
        T: DeserializeOwned
     {
        if let Ok(path) = self.get_cache_file_path(module_id) {
            if let Ok(reader) = fs::File::open(path) {
                match serde_json::from_reader(reader) {
                    Ok(result) => return Some(result),
                    Err(error) => println!("ERROR reading cached token: {:?}", error),
                }
            }
        }
        return None
    }

    pub fn update_cache<T>(&self, module_id: &str, profile: &T) -> anyhow::Result<()>
    where
        T: Serialize
    {
        let path = self.get_cache_file_path(module_id)?;

        serde_json::to_writer_pretty(private_file::create_private_file(path)?, &profile)?;

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::Args;

    #[test]
    fn the_module_comes_from_the_name_of_an_arg_not_its_value() {
        assert_eq!(Args::extract_module("--octopus-api-key=abc"), Some("octopus"));
        assert_eq!(Args::extract_module("--home=/tmp/ms-test"), None);
        assert_eq!(Args::extract_module("--profile"), None);
        assert_eq!(Args::extract_module("-v"), None);
    }
}
