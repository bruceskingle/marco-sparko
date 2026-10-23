
mod ordered_map;
pub use ordered_map::OrderedMap;

mod octopus;
mod util;
mod views;
pub mod components;
mod profile;

mod private_file;

#[cfg(feature = "desktop")]
pub mod window_state;

mod data_set;
mod cache;
use cache::CacheConfig;

use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use anyhow::anyhow;
use async_trait::async_trait;
use dioxus::core::Element;
use dirs::home_dir;
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
}

pub struct Args {
    pub marco_sparko_args: MarcoSparkoArgs,
    pub module_args: HashMap<String, Vec<String>>,
}

impl Args {
    fn extract_module(s: &str) -> Option<&str> {
        let s = s.trim_start_matches('-');
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
}


#[derive(Clone)]
pub struct PageInfo {
    label: &'static str,
    path: &'static str,
}

//  let x: fn(ModuleProps) -> std::result::Result<VNode, RenderError> = Module;
#[async_trait]
pub trait Module {
    fn get_page_list(&self) -> Vec<PageInfo>;
    fn module_id(&self) -> &'static str;
    fn get_component(&self, page_id: &str, path: Vec<String>) -> Element;
    fn cli_debug(&self) -> anyhow::Result<()>;
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
    pub fn get_file_path() -> anyhow::Result<PathBuf> {
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;

        path.push(".marco-sparko");
        Ok(path)
    }

    /// The directory holding the cache files of every profile.
    pub fn get_cache_dir_path() -> anyhow::Result<PathBuf> {
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(CACHE_DIRECTORY_NAME);
        Ok(path)
    }

    pub fn new(args: Args) -> anyhow::Result<Arc<MarcoSparkoContext>> {
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(CACHE_DIRECTORY_NAME);
        private_file::create_private_dir(&path)?;

       
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
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(CACHE_DIRECTORY_NAME);
        path.push(format!("{}-{}.json", profile_name, module_id));
                Ok(path)
    }

    fn get_cache_data_dir_path(&self, module_id: &str) -> anyhow::Result<PathBuf> {
        let profile_name =&self.profile.active_profile.name;

        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(CACHE_DIRECTORY_NAME);
        path.push(format!("{}-{}", profile_name, module_id));
        Ok(path)
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