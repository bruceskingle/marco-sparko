pub mod octopus;
pub mod system;
pub mod util;

use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{BufReader, Lines, Write};
use std::io::BufRead;
use std::error::Error as StdError;
use std::path::Path;
use std::{collections::HashMap, fmt::{self, Display}, fs, path::PathBuf, sync::{Arc, Mutex}};
use async_trait::async_trait;
use dirs::home_dir;
use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use clap::{ArgMatches, Parser, Subcommand};

use reedline::{DefaultPrompt, DefaultPromptSegment, Prompt, Reedline, Signal};

#[derive(Debug)]
pub enum Error {
    OctopusError(octopus::error::Error),
    JsonError(serde_json::Error),
    IOError(std::io::Error),
    InternalError(String),
    UserError(String),
    WrappedError(Box<dyn StdError>),
    // AnyHowError(easy_repl::anyhow::Error),
    // ReplError(reedline_repl_rs::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OctopusError(err) => f.write_fmt(format_args!("OctopusError({})", err)),
            Error::IOError(err) => f.write_fmt(format_args!("IOError({})", err)),
            Error::JsonError(err) => f.write_fmt(format_args!("JsonError({})", err)),
            Error::InternalError(err) => f.write_fmt(format_args!("InternalError({})", err)),
            Error::UserError(err) => f.write_fmt(format_args!("{}", err)),
            Error::WrappedError(err) => f.write_fmt(format_args!("WrappedError({})", err)),
            // Error::AnyHowError(err) => f.write_fmt(format_args!("AnyHowError({})", err)),
            // Error::ReplError(err) => f.write_fmt(format_args!("ReplError({})", err)),
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

// impl From<easy_repl::anyhow::Error> for Error {
//     fn from(err: easy_repl::anyhow::Error) -> Error {
//         Error::AnyHowError(err)
//     }
// }

// impl From<reedline_repl_rs::Error> for Error {
//     fn from(err: reedline_repl_rs::Error) -> Error {
//         Error::ReplError(err)
//     }
// }

impl From<Box<dyn StdError>> for Error {
    fn from(err: Box<dyn StdError>) -> Error {
        Error::WrappedError(err)
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

pub async fn list_handler(context: &mut MarcoSparkoContext) {

}

type Handler<'a, T> = Box<dyn Fn(&'a mut T) -> Box<dyn Future<Output = ()>>>;
type Handler2<T> = dyn Fn(&mut T) -> Box<dyn Future<Output = ()>>;
type Handler3<'a,T> = Box<dyn Fn(&'a mut T) -> BoxFuture<'a, ()>>;
type Handler4<T> = Box<dyn Fn(&mut T) -> BoxFuture<()>>;
type Handler5 = Box<dyn Fn() -> BoxFuture<'static, ()>>;

pub struct ReplCommand {
    pub command: &'static str,
    pub description: &'static str,
    pub help: &'static str,
    // pub handler: Handler5,
}

// pub fn foo<'a>() {
//     let x: Handler<'a,MarcoSparkoContext> = Box::new(|context| Box::new(list_handler(context)));

//     // x(context);
// }

// pub fn foo2(context: &mut MarcoSparkoContext) {
//     let x: Handler2<MarcoSparkoContext> = Box::new(list_handler(context));

//     x(context);
// }

// pub fn foo3<'a>() {
//     let x: Handler3<'a,MarcoSparkoContext> = Box::new(|context| Box::pin(list_handler(context)));

//     // x(context);
// }

// pub fn foo4(context: &mut MarcoSparkoContext) {
//     let x: Handler4<MarcoSparkoContext> = Box::new(|context| Box::pin(list_handler(context)));
//     let y = ReplCommand {
//         command: "hello",
//         description: "Greeting",
//         help: "Say Hello",
//         // handler: Box::new(|context| Box::pin(list_handler(context))),
//         handler: Box::new(|| Box::pin(list_handler())),
//     };

//     x(context);
//     let yy = y.handler;
//     yy(context);
// }



// pub struct Repl {
//     pub commands: HashMap<&'static str, ReplCommand>,
// }

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
pub trait Module: CommandProvider {
    async fn summary(&mut self) -> Result<(), Error>;
    async fn bill(&mut self) -> Result<(), Error>;
    async fn test(&mut self) -> Result<(), Error>;
}

#[async_trait]
pub trait CommandProvider {
    // async fn repl(&mut self) -> Result<(), Error>;
    // async fn get_repl_commands(&mut self) -> Vec<ReplCommand<&mut Self>>;
    // async fn get_repl_commands<'a>(&'a mut self) -> Vec<ReplCommand<&'a mut Self>>;
    async fn get_repl_commands(&mut self) -> Vec<ReplCommand>;
    async fn exec_repl_command(&mut self, command: &str, args: std::str::SplitWhitespace<'_>) ->  Result<(), Error>;
}

#[async_trait]
pub trait ModuleBuilder {
     // fn with_init(&mut self, init: bool) -> Result<&mut Self, Error>;
     async fn build(self: Box<Self>, init: bool) -> Result<Box<dyn Module + Send>, Error>;
}
// #[async_trait]
// pub trait ModuleBuilder<T: Module> {
//      // fn with_init(&mut self, init: bool) -> Result<&mut Self, Error>;
//      async fn build(self: Box<Self>, init: bool) -> Result<Box<T>, Error>;
// }

type ModuleConstructor = dyn Fn(Arc<MarcoSparkoContext>, Option<serde_json::Value>) -> Result<Box<dyn ModuleBuilder>, Error>;
// type ModuleConstructor<T: Module> = dyn Fn(Arc<MarcoSparkoContext>, Option<serde_json::Value>) -> Result<Box<dyn ModuleBuilder<T>>, Error>;

/*
 * This context is shared with all modules and needs to be separate from MarcoSparko because that struct holds the list of modules.
 */
pub struct MarcoSparkoContext {
    pub args: Args,
    before_profiles: Vec<Profile>,
    active_profile: Profile,
    after_profiles: Vec<Profile>,
    updated_profile: Mutex<Profile>,
}

impl MarcoSparkoContext {
    fn new() -> Result<Arc<MarcoSparkoContext>, Error> {

        let args = Args::parse();
        let before_profiles;
        let opt_active_profile;
        let active_profile;
        let profile_name;
        let after_profiles;
        let updated_profile;

        if let Ok(file)= fs::File::open(Self::get_file_path()?) {
            let profiles: Vec<Profile> = serde_json::from_reader(file)?;
            
            (before_profiles, opt_active_profile, after_profiles) = Self::remove_active_profile(&args, profiles)?;

        }
        else {
            before_profiles = Vec::new();
            opt_active_profile = None;
            after_profiles = Vec::new();
        }


        if let Some(existing_profile) = opt_active_profile {
            profile_name = existing_profile.name.clone();
            active_profile = existing_profile;
        }
        else {
            profile_name = DEFAULT_PROFILE.to_string();
            active_profile = Profile {
                name: profile_name.clone(),
                modules: ModuleProfiles::new()
            }
        };

        updated_profile = Mutex::new(Profile {
            name: profile_name,
            modules: ModuleProfiles::new()
        });

        Ok(Arc::new(MarcoSparkoContext {
            args,
            before_profiles,
            active_profile,
            after_profiles,
            updated_profile,
       }))
    }

    fn remove_active_profile(args: &Args, mut profiles: Vec<Profile>) -> 
        Result<(Vec<Profile>, Option<Profile>, Vec<Profile>), Error> {
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

        if profiles.is_empty() {
            Ok((Vec::new(), None, profiles))
        }
        else {
            Ok((Vec::new(), Some(profiles.remove(0)), profiles))
        }
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
            
            serde_json::to_writer_pretty(fs::File::create(Self::get_file_path()?)?, &profiles)?;
            
            return Ok(())
        }
    }

    fn get_file_path() -> Result<PathBuf, Error> {
        let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;

        path.push(".marco-sparko");

        println!("Path is <{:?}>", path.to_str());
        Ok(path)
    }

    fn get_cache_file_path(&self, module_id: &str) -> Result<PathBuf, Error> {
        let profile_name = &self.active_profile.name;
        let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;
        path.push(".marco-sparko-cache");
        path.push(format!("{}-{}.json", profile_name, module_id));

        println!("Path is {:?}", &path);
                Ok(path)
    }

    fn get_cache_data_dir_path(&self, module_id: &str) -> Result<PathBuf, Error> {
        let profile_name = &self.active_profile.name;

        let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;
        path.push(".marco-sparko-cache");
        path.push(format!("{}-{}", profile_name, module_id));

        println!("Path is {:?}", &path);
        Ok(path)
    }
      
    fn create_cache_manager(&self, module_id: &str) -> Result<CacheManager, Error> {
        let dir_path = self.get_cache_data_dir_path(module_id)?;
        fs::create_dir_all(&dir_path)?;

        Ok(CacheManager {
            dir_path,
        })
    }
    
    pub fn update_profile<T>(&self, module_id: &str, profile: T) -> Result<(), Error>
    where
        T: Serialize
    {
        let mutex = self.updated_profile.lock()?;
        let mut updated_profile = mutex;

        updated_profile.modules.insert(module_id.to_string(), serde_json::to_value(profile)?);
        Ok(())
    }

    pub fn read_cache<T>(&self, module_id: &str) -> Option<T>
    where
        T: DeserializeOwned
     {
        if let Ok(path) = self.get_cache_file_path(module_id) {
            if let Ok(reader) = fs::File::open(path) {
                if let Ok(result) = serde_json::from_reader(reader) {
                    return Some(result)
                }
            }
        }
        return None
    }

    pub fn update_cache<T>(&self, module_id: &str, profile: &T) -> Result<(), Error>
    where
        T: Serialize
    {
        let path = self.get_cache_file_path(module_id)?;

        serde_json::to_writer_pretty(fs::File::create(path)?, &profile)?;
            

        Ok(())
    }
}

struct SparkoPrompt {
    prompt: std::borrow::Cow<'static, str>,
    empty: std::borrow::Cow<'static, str>,
}

// impl SparkoPrompt {
//     fn new() -> Self {
//         SparkoPrompt {
//             prompt: std::borrow::Cow::from("> "),
//             empty: std::borrow::Cow::from(" <"),
//         }
//     }
// }

// impl Prompt for SparkoPrompt {
//     fn render_prompt_left(&self) -> std::borrow::Cow<str> {
//         self.prompt.clone()
//     }

//     fn render_prompt_right(&self) -> std::borrow::Cow<str> {
//         self.empty.clone()
//     }

//     fn render_prompt_indicator(&self, prompt_mode: reedline::PromptEditMode) -> std::borrow::Cow<str> {
//         self.prompt.clone()
//     }

//     fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
//         self.prompt.clone()
//     }

//     fn render_prompt_history_search_indicator(
//         &self,
//         history_search: reedline::PromptHistorySearch,
//     ) -> std::borrow::Cow<str> {
//         self.prompt.clone()
//     }
// }

pub struct MarcoSparko {
    context: Arc<MarcoSparkoContext>,
    module_registrations: HashMap<String, Box<ModuleConstructor>>,
    modules: HashMap<String, Box<dyn Module>>,
    current_module: Option<String>,
}

impl MarcoSparko {

    async fn exec_repl_command(&mut self, command: &str, args: std::str::SplitWhitespace<'_>) ->  Result<(), Error> {
        match command {
            "list" => {
                self.list_handler(args).await
            },
            _ => Err(Error::UserError(format!("Invalid command '{}'", command)))
        }
    }

    async fn get_repl_commands(&mut self) -> Vec<ReplCommand> {
        vec!(
            ReplCommand {
                command:"list",
                description: "Greeting",
                help: "Help yourself",
                // handler: Box::new(|| Box::pin(self.list_handler())),    
            }
        )
    }
    pub async fn list_handler(&self, mut args: std::str::SplitWhitespace<'_>) -> Result<(), Error> {
        if let Some(target) = args.next() {

            match target.as_ref() {
                "modules" => {
                    for reg in &self.module_registrations {
                        let status = if let Some(_module) = self.modules.get(reg.0) {
                            "Active"
                        }
                        else {
                            "Uninitialized"
                        };
                        println!("{} [{}]", reg.0, status);
                    }
                },
                "profiles" => {
                    for profile in &self.context.before_profiles {
                        println!("{}", profile.name);
                    }
                    println!("{} [Active]", &self.context.active_profile.name);

                    for profile in &self.context.after_profiles {
                        println!("{}", profile.name);
                    }
                },
                _ => {
                    println!("ERROR: usage: list modules|profiles");
                },
            };
        }
        else {
            println!("ERROR: usage: list modules|profiles");
        }

        Ok(())
    }

    pub async fn new() -> Result<MarcoSparko, Error> {

        let mut marco_sparko_manager = MarcoSparko {
            context: MarcoSparkoContext::new()?,
            module_registrations: HashMap::new(),
            modules: HashMap::new(),
            current_module: None,
       };

    //    let active_profile = marco_sparko_manager.marco_sparko.get_active_profile();

       let init = marco_sparko_manager.context.args.init;

       marco_sparko_manager.load_modules();

       for module_id in  marco_sparko_manager.get_module_list() {
            marco_sparko_manager.initialize(module_id, init).await?;
       }

       Ok(marco_sparko_manager)
    }

    // pub fn new_config() -> Result<MarcoSparko, Error> {
    //     MarcoSparko::new(ContextImpl::load()?)
    // }

    // pub fn new_cli() -> Result<MarcoSparko, Error> {
    //     MarcoSparko::new(ContextImpl::load_cli()?)
    // }

    fn get_module_list(&self) -> Vec<String> {
        self.context.args.modules.clone()
    }

    pub fn args(&self) -> &Args {
        &self.context.args
    }

    fn load_modules(&mut self) {
        self.load_module(octopus::Client::registration());
    }

    fn load_module(&mut self, registration: (String, Box<ModuleConstructor>)) {
        self.module_registrations.insert(registration.0, registration.1);
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        if let Some(command) =  &self.args().command {
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

            
        }
        else {
            // match &self.current_module {
            //     Some(name) => {
            //         match self.modules.get_mut(name) {
            //             Some(module) => {
            //                 module.repl().await?;
            //             },
            //             None => return Err(Error::UserError(format!("Unable to find current module {}", name))),
            //         }
            //     },
            //     None => {
            //         self.repl().await?;
            //     },
            // }

            self.repl().await?;
        }
        self.context.save_updated_profile()?;

            return Ok(())
            // Err(Error::UserError(String::from("No command given - try 'Summary'")))
        }
    
        async fn repl(&mut self) -> Result<(), crate::Error> {
            let marco_sparko_prompt = "Marco Sparko".to_string();
            let module_id = if let Some(module_id) = &self.current_module {
                module_id
            }
            else {
                &marco_sparko_prompt
            };

            let mut line_editor = Reedline::create();
            let prompt = //SparkoPrompt::new(); //DefaultPrompt::default();
            DefaultPrompt {
                left_prompt: DefaultPromptSegment::Basic(module_id.clone()),
                right_prompt: DefaultPromptSegment::CurrentDateTime,
            };

            loop {
                let out = line_editor.read_line(&prompt).unwrap();
                match out {
                    Signal::Success(content) => {
                        // process content
                        println!("GOT <{}>", content);

                        let mut arg_iterator = content.split_whitespace();
                        if let Some(command) = arg_iterator.next() {
                            match command {
                                "quit" => break,
                                _ => {

                                    let result = if let Some(module_id) = &self.current_module {
                                        let module: &mut Box<dyn Module> = self.modules.get_mut(module_id).unwrap();
                                        module.exec_repl_command(&command, arg_iterator).await
                                    }
                                    else {
                                        self.exec_repl_command(&command, arg_iterator).await
                                    };
                                    if let Err(error) = result {
                                        println!("{}", error);
                                    }
                                }
                            }

                        }
                    }
                    Signal::CtrlD => break,
                    _ => {
                        eprintln!("Entry aborted!");

                    }
                }
            }

            Ok(())
        }
    
    // async fn zzrepl(&mut self) -> Result<(), crate::Error> {
    //     // let exec = self.exec_repl_command;

    //     // Self::zzdo_repl(self.get_repl_commands(), |command| self.exec_repl_command(command), "Marco Sparko".to_string()).await

        
    //     // Self::do_repl(self, "Marco Sparko".to_string()).await

    //     if let Some(module_id) = &self.current_module {
    //         let module: &mut Box<dyn Module> = self.modules.get_mut(module_id).unwrap();
    //         Self::do_repl(module, module_id.clone()).await
    //     }
    //     else {
    //         Self::do_repl(Box::new(self), "Marco Sparko".to_string()).await
    //         // Err(Error::InternalError(format!("No current module")))
    //     }
    // }


    
    // // async fn zzdo_repl(get_repl_commands: impl Future<Output = Vec<ReplCommand>>, exec_repl_command: impl FnMut(&str) -> impl Future<Output = Result<(), Error>>, command: String) -> Result<(), Error> {
    // //     todo!()
    // // }

    // // async fn do_repl(command_provider: &mut dyn CommandProvider, module_id: String) -> Result<(), crate::Error> {
    // async fn do_repl(command_provider: &mut Box<dyn Module>, module_id: String) -> Result<(), crate::Error> {
    //     // self.do_repl().await?;

    //     let mut line_editor = Reedline::create();
    //     let prompt = //SparkoPrompt::new(); //DefaultPrompt::default();
    //     DefaultPrompt {
    //         left_prompt: DefaultPromptSegment::Basic(module_id),
    //         right_prompt: DefaultPromptSegment::CurrentDateTime,
    //     };

    //     loop {
    //         let out = line_editor.read_line(&prompt).unwrap();
    //         match out {
    //             Signal::Success(content) => {
    //                 // process content
    //                 println!("GOT <{}>", content)
    //             }
    //             Signal::CtrlD => break,
    //             _ => {
    //                 eprintln!("Entry aborted!");

    //             }
    //         }
    //     }

    //     Ok(())
    // }

    // async fn hello<T>(args: ArgMatches, _context: &mut T) -> reedline_repl_rs::Result<Option<String>> {
    //     Ok(Some(format!(
    //         "Hello, {}",
    //         args.get_one::<String>("who").unwrap()
    //     )))
    // }
    
    // /// Called after successful command execution, updates prompt with returned Option
    // async fn update_prompt<T>(_context: &mut T) -> reedline_repl_rs::Result<Option<String>> {
    //     Ok(Some("updated".to_string()))
    // }

    // async fn do_repl(&mut self) -> reedline_repl_rs::Result<()> {

    //     use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
    //     use reedline_repl_rs::{Repl, Result};

    //     let mut repl = Repl::new(())
    //         .with_name("MyApp")
    //         .with_version("v0.1.0")
    //         .with_command_async(
    //             Command::new("hello")
    //                 .arg(Arg::new("who").required(true))
    //                 .about("Greetings!"),
    //             |args, context| Box::pin(Self::hello(args, context)),
    //         )
    //         .with_on_after_command_async(|context| Box::pin(Self::update_prompt(context)));
    //     repl.run_async().await

    //     // // let marko_sparko = RefCell::new(self);
    //     // // let self1 = &marko_sparko;
    //     // // let self2 = &marko_sparko;
    
    //     // let mut repl = easy_repl::Repl::builder()
    //     //     .add("list", command! {
    //     //         "List modules or profiles",
    //     //         (target: String) => |target: String| {
    //     //             let this = self1.borrow();
    //     //             println!("List {}!", target);

    //     //             match target.as_ref() {
    //     //                 "modules" => {
    //     //                     for reg in &this.module_registrations {
    //     //                         let status = if let Some(_module) = this.modules.get(reg.0) {
    //     //                             "Active"
    //     //                         }
    //     //                         else {
    //     //                             "Uninitialized"
    //     //                         };
    //     //                         println!("{} [{}]", reg.0, status);
    //     //                     }
    //     //                 },
    //     //                 "profiles" => {
    //     //                     for profile in &this.context.before_profiles {
    //     //                         println!("{}", profile.name);
    //     //                     }
    //     //                     println!("{} [Active]", &this.context.active_profile.name);

    //     //                     for profile in &this.context.after_profiles {
    //     //                         println!("{}", profile.name);
    //     //                     }
    //     //                 },
    //     //                 _ => {
    //     //                     println!("ERROR: usage: list modules|profiles");
    //     //                 },
    //     //             };

    //     //             Ok(CommandStatus::Done)
    //     //         }
    //     //     })
    //     //     .add("init", command! {
    //     //         "initialize a module",
    //     //         (module_name: String) => |module_name: String| {
    //     //             let mut this = self2.borrow_mut();
    //     //             println!("Initialize {}!", module_name);

    //     //             if let Some(_) = &this.modules.get(&module_name) {
    //     //                 println!("ERROR: module {} is already active", module_name);
    //     //             }
    //     //             else {
    //     //                 if let Err(error) = &this.initialize(module_name, true).await {
    //     //                     println!("ERROR {}", error);
    //     //                 }
    //     //             }

    //     //             Ok(CommandStatus::Done)
    //     //         }
    //     //     })
    //     //     .add("add", command! {
    //     //         "Add X to Y",
    //     //         (X:i32, Y:i32) => |x, y| {
    //     //             println!("{} + {} = {}", x, y, x + y);
    //     //             Ok(CommandStatus::Done)
    //     //         }
    //     //     })
    //     //     .build().context("Failed to create repl")?;
    
    //     // repl.run().context("Critical REPL error")?;
    
    //     // Ok(())
    // }

    // async fn do_repl(&mut self) -> easy_repl::anyhow::Result<()> {

    //     use easy_repl::command;
    //     use easy_repl::{anyhow::Context, CommandStatus};

    //     let marko_sparko = RefCell::new(self);
    //     let self1 = &marko_sparko;
    //     let self2 = &marko_sparko;
    
    //     let mut repl = easy_repl::Repl::builder()
    //         .add("list", command! {
    //             "List modules or profiles",
    //             (target: String) => |target: String| {
    //                 let this = self1.borrow();
    //                 println!("List {}!", target);

    //                 match target.as_ref() {
    //                     "modules" => {
    //                         for reg in &this.module_registrations {
    //                             let status = if let Some(_module) = this.modules.get(reg.0) {
    //                                 "Active"
    //                             }
    //                             else {
    //                                 "Uninitialized"
    //                             };
    //                             println!("{} [{}]", reg.0, status);
    //                         }
    //                     },
    //                     "profiles" => {
    //                         for profile in &this.context.before_profiles {
    //                             println!("{}", profile.name);
    //                         }
    //                         println!("{} [Active]", &this.context.active_profile.name);

    //                         for profile in &this.context.after_profiles {
    //                             println!("{}", profile.name);
    //                         }
    //                     },
    //                     _ => {
    //                         println!("ERROR: usage: list modules|profiles");
    //                     },
    //                 };

    //                 Ok(CommandStatus::Done)
    //             }
    //         })
    //         .add("init", command! {
    //             "initialize a module",
    //             (module_name: String) => |module_name: String| {
    //                 let mut this = self2.borrow_mut();
    //                 println!("Initialize {}!", module_name);

    //                 if let Some(_) = &this.modules.get(&module_name) {
    //                     println!("ERROR: module {} is already active", module_name);
    //                 }
    //                 else {
    //                     if let Err(error) = &this.initialize(module_name, true).await {
    //                         println!("ERROR {}", error);
    //                     }
    //                 }

    //                 Ok(CommandStatus::Done)
    //             }
    //         })
    //         .add("add", command! {
    //             "Add X to Y",
    //             (X:i32, Y:i32) => |x, y| {
    //                 println!("{} + {} = {}", x, y, x + y);
    //                 Ok(CommandStatus::Done)
    //             }
    //         })
    //         .build().context("Failed to create repl")?;
    
    //     repl.run().context("Critical REPL error")?;
    
    //     Ok(())
    // }

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

    // fn init_module_repl<T: Module + ?Sized>(&mut self, module_id: &str, module_box: Box<T>, repl_commands: Vec<ReplCommand>) {


    //     use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
    //     use reedline_repl_rs::{Repl, Result};

    //     // let module = *module_box;
    //     let mut repl = Repl::new(module_box)
    //         .with_name(&module_id)
    //         .with_version("v0.1.0");

    //     for repl_command in repl_commands {
    //         repl = repl.with_command_async(
    //             Command::new(repl_command.command)
    //                 // .arg(Arg::new("who").required(true))
    //                 .about(repl_command.description),
    //             |args, context| Box::pin(
    //                 context.exec_repl_command("repl_command.command")
    //                 // Self::hello(args, context)
    //             ),
    //         )
    //     }
    //         // .with_on_after_command_async(|context| Box::pin(Self::update_prompt(context)));
    //     // repl.run_async().await
    // }
    
    async fn initialize(&mut self, module_id: String, init: bool) -> Result<(), Error> {
        if let Some(module_registration) = self.module_registrations.get(&module_id) {
            let constructor = module_registration.as_ref();
            let profile = if let Some(value) = self.context.active_profile.modules.get(&module_id) {
                Some(value.clone())
            }
            else {
                None
            };
            let builder = constructor(self.context.clone(), profile)?;
            // let x = builder.build(init);
            // let modules = &mut self.modules;
            let module = builder.build(init).await?;
            // self.init_module_repl(&module_id, module, module.get_repl_commands().await);
            self.modules.insert(module_id.clone(),module);

            // let r: Vec<ReplCommand> = module.get_repl_commands().await;

        



            if self.current_module.is_none() {
                self.current_module = Some(module_id.clone());

                // let m = modules.get_mut(&module_id).unwrap();
                // m.exec_repl_command("list").await?;
            }
            
            

            Ok(())
        }
        else {
            return Err(Error::UserError(format!("Unknown module \"{}\"", module_id)))
        }
    }
}

pub struct CacheManager {
    pub dir_path: PathBuf,
}

impl CacheManager {
    pub fn write<T: Serialize>(&self, hash_key: &str, vec: &Vec<(String, T)>, cached_cnt: usize) -> Result<(), Error> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        if cached_cnt == 0 {
            let mut out = fs::File::create(path)?;
            for (key, value) in vec {
                writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
                println!("WRITE {}", key);
            }
        }
        else {
            let mut out = OpenOptions::new().append(true).open(path)?;

            let mut i = cached_cnt;
            while i < vec.len() {
                let (key, value) = vec.get(i).unwrap();
                writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
                i += 1;
            }
        }

        Ok(())
    }

    // The output is wrapped in a Result to allow matching on errors.
    // Returns an Iterator to the Reader of the lines of the file.
    fn read_lines<P>(filename: P) -> std::io::Result<Lines<BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(BufReader::new(file).lines())
    }

    pub fn read<T: DeserializeOwned>(&self, hash_key: &str, vec: &mut Vec<(String, T)>) -> Result<(), Error> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        match Self::read_lines(path) {
            Ok(lines) => {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.map_while(Result::ok) {
                    println!("READ {}", line);

                    match line.split_once('\t') {
                        Some((key, value)) => vec.push((key.to_string(), serde_json::from_str(value)?)),
                        None => return Err(Error::InternalError(format!("Invalid cached object <{}>", line))),
                    }
                }
            },

            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    println!("ERROR {:?}", error);
                    return Err(Error::IOError(error))
                }
                
            },
        }

        // for (key, value) in map {
        //     writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
        //     println!("WRITE {}", key);
        // }

        Ok(())
    }
}

// #[derive(Clone)]
// pub struct Context {
//     marco_sparko: Arc<ContextImpl>,
// }

// impl Context {
//     fn new(marco_sparko: Arc<ContextImpl>) -> Context {
//         Context {
//             marco_sparko
//         }
//     }

//     // pub fn new_test() -> Context {
//     //     Context {
//     //         marco_sparko: Arc::new(ContextImpl::new(None))
//     //     }
//     // }

//     pub fn args(&self) -> &Option<Args> {
//         &self.marco_sparko.args
//     }

//     pub fn get_active_profile(&self) -> &Option<Profile> {
//         self.marco_sparko.get_active_profile()
//     }

//     pub fn update_cache<T>(&self, module_id: &str, profile: &T) -> Result<(), Error>
//     where
//         T: Serialize
//      {
//         let path = self.marco_sparko.get_cache_file_path(module_id)?;

//         serde_json::to_writer_pretty(fs::File::create(path)?, &profile)?;
            

//         Ok(())
//      }



//     pub fn read_cache<T>(&self, module_id: &str) -> Option<T>
//     where
//         T: DeserializeOwned
//      {
//         if let Ok(path) = self.marco_sparko.get_cache_file_path(module_id) {
//             if let Ok(reader) = fs::File::open(path) {
//                 if let Ok(result) = serde_json::from_reader(reader) {
//                     return Some(result)
//                 }
//             }
//         }
//         return None
//     }

//     //     let path = ContextImpl::get_cache_file_path(module_id).unwrap();
//     //     let reader = fs::File::open(path).unwrap();
//     //     let result = serde_json::from_reader(reader).unwrap();
        
//     //                 return Some(result)
//     // }
       

//      pub fn update_profile<T>(&mut self, module_id: &str, profile: T) -> Result<(), Error>
//      where
//          T: Serialize
//       {
//          let mutex = self.marco_sparko.updated_profile.lock()?;
//          let mut updated_profile = mutex;
 
//          updated_profile.modules.insert(module_id.to_string(), serde_json::to_value(profile)?);
//          Ok(())
//       }
      
//     fn create_cache_manager(&self, module_id: &str) -> Result<CacheManager, Error> {
//         let dir_path = self.marco_sparko.get_cache_data_dir_path(module_id)?;
//         fs::create_dir_all(&dir_path)?;

//         Ok(CacheManager {
//             dir_path,
//         })
//     }

//     fn get_cache_data_dir_path(&self, module_id: &str) -> Result<PathBuf, Error> {
//         let profile_name = if let Some(active_profile) = &self.active_profile {
//             &active_profile.name
//         }
//         else {
//             return Err(Error::InternalError("No Active Profile".to_string()))
//         };

//         let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;
//         path.push(".marco-sparko-cache");
//         path.push(format!("{}-{}", profile_name, module_id));

//         println!("Path is {:?}", &path);
//         Ok(path)
//     }
// }

// #[derive(Debug)]
// struct ContextImpl {
//     pub args: Option<Args>,
//     before_profiles: Vec<Profile>,
//     active_profile: Option<Profile>,
//     after_profiles: Vec<Profile>,
//     updated_profile: Mutex<Profile>,
// }

// impl ContextImpl {
//     fn new(args: Option<Args>) -> ContextImpl {
        
//         ContextImpl{
//             args,
//             before_profiles: Vec::new(),
//             active_profile: None,
//             after_profiles: Vec::new(),
//             updated_profile: Mutex::new(Profile {
//                 name: DEFAULT_PROFILE.to_string(),
//                 modules: ModuleProfiles::new()
//             })
//         }
//     }

//     fn get_active_profile(&self) -> &Option<Profile> {
//         &self.active_profile
//     }

//     fn get_cache_file_path(&self, module_id: &str) -> Result<PathBuf, Error> {
//         let profile_name = if let Some(active_profile) = &self.active_profile {
//             &active_profile.name
//         }
//         else {
//             return Err(Error::InternalError("No Active Profile".to_string()))
//         };

//         let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;
//         path.push(".marco-sparko-cache");
//         path.push(format!("{}-{}.json", profile_name, module_id));

//         println!("Path is {:?}", &path);
//                 Ok(path)
//      }

//      fn get_cache_data_dir_path(&self, module_id: &str) -> Result<PathBuf, Error> {
//         let profile_name = if let Some(active_profile) = &self.active_profile {
//             &active_profile.name
//         }
//         else {
//             return Err(Error::InternalError("No Active Profile".to_string()))
//         };

//         let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;
//         path.push(".marco-sparko-cache");
//         path.push(format!("{}-{}", profile_name, module_id));

//         println!("Path is {:?}", &path);
//         Ok(path)
//       }

//     fn get_file_path() -> Result<PathBuf, Error> {
//         let mut path = home_dir().ok_or(Error::InternalError("Unable to locate home directory".to_string()))?;

//         path.push(".marco-sparko");

//         println!("Path is <{:?}>", path.to_str());
//         Ok(path)
//     }

//     fn save_updated_profile(&self) -> Result<(), Error> {
//         let updated_profile = self.updated_profile.lock()?;

//         if updated_profile.modules.is_empty() {
//             Ok(())
//         }
//         else {

//             let mut profiles = Vec::new();
            
//             profiles.extend(&self.before_profiles);
//             profiles.push(&updated_profile);
//             profiles.extend(&self.after_profiles);
            
//             serde_json::to_writer_pretty(fs::File::create(ContextImpl::get_file_path()?)?, &profiles)?;
            
//             return Ok(())
//         }
//     }

//     fn load_cli() -> Result<ContextImpl, Error> {
//         Ok(ContextImpl::do_load(Some(Args::parse()))?)
//     }


//     // pub fn get_args(&self) -> Result<&Option<Args>, Error> {
//     //     Ok(&self.args)
//     // }

//     fn remove_active_profile(args: &Option<Args>, mut profiles: Vec<Profile>) -> 
//         Result<(Vec<Profile>, Option<Profile>, Vec<Profile>), Error> {
        
        
//         if let Some(args) = args {
//             if let Some(profile_name) = &args.profile {
//                 let mut active_profile = None;
//                 let mut before_profiles = Vec::new();
//                 let mut after_profiles = Vec::new();
//                 let mut after = false;
                
                
//                 for profile in profiles {
//                     if profile.name.eq(profile_name) {
//                         active_profile = Some(profile);
//                         after = true;
//                     }
//                     else {
//                         if after {
//                             after_profiles.push(profile);
//                         }
//                         else {
//                             before_profiles.push(profile);
//                         }
//                     }
//                 }

//                 if let None = active_profile {
//                     return Result::Err(Error::UserError(format!("No profile called {}", profile_name)))
//                 }
//                 return Ok((before_profiles, active_profile, after_profiles))
//             }
//         }

//         if profiles.is_empty() {
//             Ok((Vec::new(), None, profiles))
//         }
//         else {
//             Ok((Vec::new(), Some(profiles.remove(0)), profiles))
//         }
//     }


//     pub fn load() -> Result<ContextImpl, Error> {
//         ContextImpl::do_load(None)
//     }


//     fn do_load(args: Option<Args>) -> Result<ContextImpl, Error> {

//         if let Ok(file)= fs::File::open(ContextImpl::get_file_path()?) {
//             let profiles: Vec<Profile> = serde_json::from_reader(file)?;
            
//             let (before_profiles, active_profile, after_profiles) = ContextImpl::remove_active_profile(&args, profiles)?;

//             let profile_name = if let Some(existing_profile) = &active_profile {
//                 existing_profile.name.clone()
//             }
//             else {
//                 DEFAULT_PROFILE.to_string()
//             };

//             return Ok(ContextImpl {
//                 args, 
//                 before_profiles,
//                 active_profile,
//                 after_profiles,
//                 updated_profile: Mutex::new(Profile {
//                     name: profile_name,
//                     modules: ModuleProfiles::new()
//                 })
//              })
//         }
//         else {
//             Ok(ContextImpl::new(args))
//         }

//     }
// }

// // pub trait Token {
// //     fn fetch(&self) -> Arc<String>;
// //     fn has_expired(&self) -> bool;
// // }
