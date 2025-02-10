/*****************************************************************************
MIT License

Copyright (c) 2024 Bruce Skingle

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
******************************************************************************/

// pub mod gql;
pub mod octopus;
pub mod system;
pub mod request_manager;
pub use request_manager::RequestManager;
pub mod authenticated_request_manager;
pub use authenticated_request_manager::AuthenticatedRequestManager;

use std::error::Error as StdError;
use std::{collections::HashMap, fmt::{self, Display}, fs, path::PathBuf, sync::{Arc, Mutex}};
use async_trait::async_trait;
use dirs::home_dir;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};

#[derive(Debug)]
pub enum Error {
    OctopusError(octopus::error::Error),
    JsonError(serde_json::Error),
    IOError(std::io::Error),
    InternalError(String),
    UserError(String),
    WrappedError(Box<dyn StdError>),
    GraphQLError(Vec<graphql_client::Error>)
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OctopusError(err) => f.write_fmt(format_args!("OctopusError({})", err)),
            Error::IOError(err) => f.write_fmt(format_args!("IOError({})", err)),
            Error::JsonError(err) => f.write_fmt(format_args!("JsonError({})", err)),
            Error::InternalError(err) => f.write_fmt(format_args!("InternalError({})", err)),
            Error::UserError(err) => f.write_fmt(format_args!("UserError({})", err)),
            Error::WrappedError(err) => f.write_fmt(format_args!("WrappedError({})", err)),
            Error::GraphQLError(err) => {
                match  serde_json::to_string_pretty(err)  {
                    Ok(s) => f.write_str(&s),
                    Err(e) => f.write_fmt(format_args!("Failed to parse JSON: {}", e)),
                }
            }
        }
    }
}

impl StdError for Error {

}

// impl From<time::error::ComponentRange> for Error {
//     fn from(err: time::error::ComponentRange) -> Error {
//         Error::WrappedError(Box::new(err))
//     }
// }

impl From<Box<dyn StdError>> for Error {
    fn from(err: Box<dyn StdError>) -> Error {
        Error::WrappedError(err)
    }
}

impl From<Vec<graphql_client::Error>>  for Error {
    fn from(err: Vec<graphql_client::Error>) -> Error {
        Error::GraphQLError(err)
    }
}

impl From<reqwest::Error> for Error {
        fn from(err: reqwest::Error) -> Error {
            Error::WrappedError(Box::new(err))
        }
}

impl From<crate::octopus::error::Error> for Error {
    fn from(err: crate::octopus::error::Error) -> Error {
        Error::OctopusError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(err: std::sync::PoisonError<T>) -> Error {
        Error::InternalError(format!("Mutex poison error {:?}", err))
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Args {
    /// Name of the config profile to use
    #[arg(short, long)]
    profile: Option<String>,
    #[arg(short, long, value_delimiter = ',', num_args = 1..)]
    modules: Vec<String>,
    #[arg(short, long)]
    init: bool,
    #[arg(short, long)]
    verbose: bool,

    #[clap(flatten)]
    pub octopus: octopus::OctopusArgs,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Summary,
    Bill,
    /// does testing things
    Test,
}

const DEFAULT_PROFILE: &str = "default";

type ModuleProfiles = HashMap<String, serde_json::Value>;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    name: String,
    modules: ModuleProfiles
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            name: DEFAULT_PROFILE.to_string(),
            modules: ModuleProfiles::new()
        }
    }
}

#[async_trait]
pub trait Module {
    async fn summary(&mut self) -> Result<(), Error>;
    async fn bill(&mut self) -> Result<(), Error>;
    async fn test(&mut self) -> Result<(), Error>;
}

pub trait ModuleBuilder {
     // fn with_init(&mut self, init: bool) -> Result<&mut Self, Error>;
     fn build(self: Box<Self>, init: bool) -> Result<Box<dyn Module + Send>, Error>;
}

type ModuleConstructor = dyn Fn(Box<&Context>, Option<serde_json::Value>) -> Result<Box<dyn ModuleBuilder>, Error>;

pub struct MarcoSparko {
    marco_sparko: Context,
    module_registrations: HashMap<String, Box<ModuleConstructor>>,
    modules: HashMap<String, Box<dyn Module>>,
}

impl MarcoSparko {
    fn new(marco_sparko: ContextImpl) -> Result<MarcoSparko, Error> {
        let mut marco_sparko_manager = MarcoSparko {
            marco_sparko: Context::new(Arc::new(marco_sparko)),
            module_registrations: HashMap::new(),
            modules: HashMap::new()
       };

       let init = if let Some(args) = &marco_sparko_manager.marco_sparko.args() {
            args.init
       }
       else {
            false
       };

       marco_sparko_manager.load_modules();

       for module_id in  marco_sparko_manager.get_module_list() {
            if let Some(module_registration) = marco_sparko_manager.module_registrations.get(&module_id) {
                let constructor = module_registration.as_ref();
                let profile = if let Some(active_profile) = &marco_sparko_manager.marco_sparko.marco_sparko.active_profile {
                    if let Some(value) = active_profile.modules.get(&module_id) {
                        Some(value.clone())
                    }
                    else {
                        None
                    }
                }
                else {
                    None
                };
                let builder = constructor(Box::new(&marco_sparko_manager.marco_sparko), profile)?;
                // let x = builder.build(init);
                let modules = &mut marco_sparko_manager.modules;
                
                modules.insert(module_id.clone(),builder.build(init)?);
            }
            else {
                return Err(Error::UserError(format!("Unknown module \"{}\"", module_id)))
            }
       }

       Ok(marco_sparko_manager)
    }

    pub fn new_config() -> Result<MarcoSparko, Error> {
        MarcoSparko::new(ContextImpl::load()?)
    }

    pub fn new_cli() -> Result<MarcoSparko, Error> {
        MarcoSparko::new(ContextImpl::load_cli()?)
    }

    fn get_module_list(&self) -> Vec<String> {
        if let Some(args) = &self.marco_sparko.marco_sparko.args {
            args.modules.clone()
        }
        else {
            Vec::new()
        }
    }

    pub fn args(&self) -> &Option<Args> {
        &self.marco_sparko.marco_sparko.args
    }

    fn load_modules(&mut self) {
        self.load_module(octopus::Client::registration());
    }

    fn load_module(&mut self, registration: (String, Box<ModuleConstructor>)) {
        self.module_registrations.insert(registration.0, registration.1);
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        if let Some(args) = &self.args() {
            if let Some(command) =  &args.command {
                match command {
                    Commands::Summary => {
                        self.summary().await?; 
                        
                    }
                    Commands::Bill => {
                        self.bill().await?; 
                        
                    }
                    Commands::Test => {
                        self.test().await?; 
                        
                    },
                };

                self.marco_sparko.marco_sparko.save_updated_profile()?;

                return Ok(())
            }
        };

        Err(Error::UserError(String::from("No command given - try 'Summary'")))
    }

    async fn summary(&mut self) -> Result<(), Error> {
        for (_module_id, module) in self.modules.iter_mut() {
            println!("Summary {}", _module_id);
            module.summary().await?;
        }

        Ok(())
    }

    async fn bill(&mut self) -> Result<(), Error> {
        for (_module_id, module) in self.modules.iter_mut() {
            println!("Bill {}", _module_id);
            module.bill().await?;
        }

        Ok(())
    }

    async fn test(&mut self) -> Result<(), Error> {
        for (_module_id, module) in self.modules.iter_mut() {
            println!("Test {}", _module_id);
            module.test().await?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Context {
    marco_sparko: Arc<ContextImpl>,
}

impl Context {
    fn new(marco_sparko: Arc<ContextImpl>) -> Context {
        Context {
            marco_sparko
        }
    }

    pub fn new_test() -> Context {
        Context {
            marco_sparko: Arc::new(ContextImpl::new(None))
        }
    }

    pub fn args(&self) -> &Option<Args> {
        &self.marco_sparko.args
    }

    pub fn update_cache<T>(&self, module_id: &str, profile: &T) -> Result<(), Error>
    where
        T: Serialize
     {
        let path = self.marco_sparko.get_cache_file_path(module_id)?;

        serde_json::to_writer_pretty(fs::File::create(path)?, &profile)?;
            

        Ok(())
     }



    pub fn read_cache<T>(&self, module_id: &str) -> Option<T>
    where
        T: DeserializeOwned
     {
        if let Ok(path) = self.marco_sparko.get_cache_file_path(module_id) {
            if let Ok(reader) = fs::File::open(path) {
                if let Ok(result) = serde_json::from_reader(reader) {
                    return Some(result)
                }
            }
        }
        return None
    }

    //     let path = ContextImpl::get_cache_file_path(module_id).unwrap();
    //     let reader = fs::File::open(path).unwrap();
    //     let result = serde_json::from_reader(reader).unwrap();
        
    //                 return Some(result)
    // }
       

     pub fn update_profile<T>(&mut self, module_id: &str, profile: T) -> Result<(), Error>
     where
         T: Serialize
      {
         let mutex = self.marco_sparko.updated_profile.lock()?;
         let mut updated_profile = mutex;
 
         updated_profile.modules.insert(module_id.to_string(), serde_json::to_value(profile)?);
         Ok(())
      }
}

#[derive(Debug)]
struct ContextImpl {
    pub args: Option<Args>,
    before_profiles: Vec<Profile>,
    active_profile: Option<Profile>,
    after_profiles: Vec<Profile>,
    updated_profile: Mutex<Profile>,
}

impl ContextImpl {
    fn new(args: Option<Args>) -> ContextImpl {
        
        ContextImpl{
            args,
            before_profiles: Vec::new(),
            active_profile: None,
            after_profiles: Vec::new(),
            updated_profile: Mutex::new(Profile {
                name: DEFAULT_PROFILE.to_string(),
                modules: ModuleProfiles::new()
            })
        }
    }

    fn get_cache_file_path(&self, module_id: &str) -> Result<PathBuf, Error> {
        let profile_name = if let Some(active_profile) = &self.active_profile {
            &active_profile.name
        }
        else {
            return Err(Error::InternalError("No Active Profile".to_string()))
        };

        let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;
        path.push(".marco-sparko-cache");
        path.push(format!("{}-{}.json", profile_name, module_id));

        println!("Path is {:?}", &path);
                Ok(path)
     }

    fn get_file_path() -> Result<PathBuf, Error> {
        let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;

        path.push(".marco-sparko");

        println!("Path is <{:?}>", path.to_str());
        Ok(path)
    }

    fn save_updated_profile(&self) -> Result<(), Error> {
        let updated_profile = self.updated_profile.lock()?;

        if updated_profile.modules.is_empty() {
            Ok(())
        }
        else {

            let mut profiles = Vec::new();
            
            profiles.extend(&self.before_profiles);
            profiles.push(&updated_profile);
            profiles.extend(&self.after_profiles);
            
            serde_json::to_writer_pretty(fs::File::create(ContextImpl::get_file_path()?)?, &profiles)?;
            
            return Ok(())
        }
    }

    fn load_cli() -> Result<ContextImpl, Error> {
        Ok(ContextImpl::do_load(Some(Args::parse()))?)
    }


    // pub fn get_args(&self) -> Result<&Option<Args>, Error> {
    //     Ok(&self.args)
    // }

    fn remove_active_profile(args: &Option<Args>, mut profiles: Vec<Profile>) -> 
        Result<(Vec<Profile>, Option<Profile>, Vec<Profile>), Error> {
        
        
        if let Some(args) = args {
            if let Some(profile_name) = &args.profile {
                let mut active_profile = None;
                let mut before_profiles = Vec::new();
                let mut after_profiles = Vec::new();
                let mut after = false;
                
                
                for profile in profiles {
                    if profile.name.eq(profile_name) {
                        active_profile = Some(profile);
                        after = true;
                    }
                    else {
                        if after {
                            after_profiles.push(profile);
                        }
                        else {
                            before_profiles.push(profile);
                        }
                    }
                }

                if let None = active_profile {
                    return Result::Err(Error::UserError(format!("No profile called {}", profile_name)))
                }
                return Ok((before_profiles, active_profile, after_profiles))
            }
        }

        if profiles.is_empty() {
            Ok((Vec::new(), None, profiles))
        }
        else {
            Ok((Vec::new(), Some(profiles.remove(0)), profiles))
        }
    }


    pub fn load() -> Result<ContextImpl, Error> {
        ContextImpl::do_load(None)
    }


    fn do_load(args: Option<Args>) -> Result<ContextImpl, Error> {

        if let Ok(file)= fs::File::open(ContextImpl::get_file_path()?) {
            let profiles: Vec<Profile> = serde_json::from_reader(file)?;
            
            let (before_profiles, active_profile, after_profiles) = ContextImpl::remove_active_profile(&args, profiles)?;

            let profile_name = if let Some(existing_profile) = &active_profile {
                existing_profile.name.clone()
            }
            else {
                DEFAULT_PROFILE.to_string()
            };

            return Ok(ContextImpl {
                args, 
                before_profiles,
                active_profile,
                after_profiles,
                updated_profile: Mutex::new(Profile {
                    name: profile_name,
                    modules: ModuleProfiles::new()
                })
             })
        }
        else {
            Ok(ContextImpl::new(args))
        }

    }
}

// pub trait Token {
//     fn fetch(&self) -> Arc<String>;
//     fn has_expired(&self) -> bool;
// }
