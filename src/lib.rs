pub mod octopus;
pub mod system;
pub mod util;
pub mod views;
pub mod components;
pub mod profile;

mod cache_manager;
pub use cache_manager::CacheManager;


use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Lines, Write};
use std::io::BufRead;
use std::path::Path;
use std::sync::OnceLock;
use std::{collections::HashMap, fs, path::PathBuf, sync::{Arc, Mutex}};
use anyhow::anyhow;
use async_trait::async_trait;
use dioxus::core::Element;
use dirs::home_dir;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};

use reedline::{Emacs, ExampleHighlighter, FileBackedHistory, MenuBuilder, ReedlineMenu};
use reedline::{default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultPrompt, DefaultPromptSegment, KeyCode, KeyModifiers, Reedline, ReedlineEvent, Signal};
use sparko_graphql::types::Date;
use time::Month;
use crate::profile::ActiveProfile;

use {
    nu_ansi_term::{Color, Style},
    reedline::{DefaultValidator, DefaultHinter},
  };

use profile::ProfileManager;

// pub static PROFILE_MANAGER: OnceLock<Arc<ProfileManager>> = OnceLock::new();

// type Component = Box<dyn Fn() -> dioxus::core::Element>;
//fn() -> std::result::Result<dioxus::core::VNode, dioxus::core::RenderError>;

pub const CHECK_FOR_UPDATES: bool = true;
pub struct ReplCommand {
    pub command: &'static str,
    pub description: &'static str,
    pub help: &'static str,
}

#[derive(Parser, Debug, Clone, PartialEq)]
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

    // #[command(subcommand)]
    // command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Summary,
    Bill,
    /// does testing things
    Test,
}

#[derive(Clone)]
pub struct PageInfo {
    label: &'static str,
    path: &'static str,
}

//  let x: fn(ModuleProps) -> std::result::Result<VNode, RenderError> = Module;
#[async_trait]
pub trait Module: CommandProvider {
    // fn get_page(&self, page_id: &String) -> Component;
    // fn as_component<'a>(&'a self) -> Box<dyn Fn() -> dioxus::core::Element + 'a>;
    // fn as_component<'a>(&'a self) -> Element;
    fn get_page_list(&self) -> Vec<PageInfo>;
    // fn get_page(&self, page_id: &str) -> Element;
    fn module_id(&self) -> &'static str;
    fn get_component<'a>(&'a self, page_id: &'a str, path: Vec<String>) -> Box<dyn Fn() -> Element + 'a>;
    // fn get_pages<'a>(&'a self) -> HashMap<&str, Box<dyn Fn() -> dioxus::core::Element + 'a>>;
    // fn get_pages<'a>(&'a self) -> HashMap<&str, Box<impl FnOnce() -> dioxus::core::Element + 'a>>;
    // fn get_component(&self) -> Component;




    // async fn summary(&mut self) -> anyhow::Result<()>;
    // async fn bill(&mut self) -> anyhow::Result<()>;
    // async fn test(&mut self) -> anyhow::Result<()>;
}

#[async_trait(?Send)]
pub trait CommandProvider {
    fn get_repl_commands(&self) -> Vec<ReplCommand>;
    async fn exec_repl_command(&mut self, command: &str, args: std::str::SplitWhitespace<'_>) ->  anyhow::Result<()>;
}

#[async_trait]
pub trait ModuleBuilder {
     // fn with_init(&mut self, init: bool) -> anyhow::Result<&mut Self>;
     async fn build(self: Box<Self>, init: bool) -> anyhow::Result<Box<dyn Module + Send>>;
}

pub type ModuleConstructor = dyn Fn(Arc<MarcoSparkoContext>, Option<serde_json::Value>) -> anyhow::Result<Box<dyn ModuleBuilder>>;

#[derive(Clone, Default)]
pub struct ModuleRegistrations(Arc<HashMap<String, Box<ModuleConstructor>>>);

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
    fn new() -> ModuleRegistrations {
        let mut module_registrations = HashMap::new();

        Self::load_module(&mut module_registrations, octopus::Client::registration());

        println!("Loaded {} modules", module_registrations.len());

        for (k,v) in &module_registrations {
            println!(" Module {}", k);
        }
        ModuleRegistrations(Arc::new(module_registrations))
    }

    fn load_module(module_registrations: &mut HashMap<String, Box<ModuleConstructor>> , registration: (String, Box<ModuleConstructor>)) {
        println!("Load module {}", registration.0);
        module_registrations.insert(registration.0, registration.1);
    }
}

// #[derive(Clone, PartialEq)]
// pub struct DioxusContext {
//     marco_sparko_context: Arc<MarcoSparkoContext>,
//     module_registrations:   ModuleRegistrations,
// }

// impl Default for DioxusContext {
//     fn default() -> Self {
//         let marco_sparko_context = MarcoSparkoContext::new().unwrap();
//         Self { 
//             marco_sparko_context,
//             module_registrations: Default::default() }
//     }
// }

// impl DioxusContext {
//     pub fn new() -> anyhow::Result<DioxusContext> {
//         let marco_sparko_context = MarcoSparkoContext::new()?;
//         let module_registrations = ModuleRegistrations::new();

//         Ok(DioxusContext {
//             marco_sparko_context,
//             module_registrations,
//         })
//     }
// }

/*
 * This context is shared with all modules and needs to be separate from MarcoSparko because that struct holds the list of modules.
 */
 pub struct MarcoSparkoContext {
    pub args: Args,
    pub profile: ActiveProfile,
}

impl PartialEq for MarcoSparkoContext {
    fn eq(&self, other: &Self) -> bool {
        self.args == other.args && self.profile == other.profile
    }
}

impl MarcoSparkoContext {
    pub fn new() -> anyhow::Result<Arc<MarcoSparkoContext>> {

        let args = Args::parse();
        let profile = crate::profile::fetch_active_profile(&args.profile)?;
        

        Ok(Arc::new(MarcoSparkoContext {
            args,
            profile,
       }))
    }

    pub fn with_profile(&self, profile_name: &String) -> anyhow::Result<Arc<MarcoSparkoContext>> {
        Ok(Arc::new(MarcoSparkoContext {
            args: self.args.clone(),
            profile: crate::profile::set_active_profile(profile_name)?,
       }))
    }

    fn create_profile_manager(active_profile: &Option<String>) -> anyhow::Result<ProfileManager>  {
        // let selector = if let Some(name) = active_profile {
        //     profile::ProfileSelector::Named(name.clone())
        // } else {
        //     profile::ProfileSelector::Default
        // };
        match ProfileManager::new(active_profile) {
            Ok(p) => Ok(p),
            Err(_) => Err(anyhow!("FAILED")),
        }
    }

    // fn save_updated_profile(&self) -> anyhow::Result<()> {
    //     match self.profile_manager.save_updated_profile() {
    //         Ok(p) => Ok(p),
    //         Err(_) => Err(anyhow!("FAILED")),
    //     }
    // }

    

    fn get_cache_file_path(&self, module_id: &str) -> anyhow::Result<PathBuf> {
        let profile_name = &self.profile.active_profile.name;
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(".marco-sparko-cache");
        path.push(format!("{}-{}.json", profile_name, module_id));
                Ok(path)
    }

    fn get_history_file_path(&self, module_id: &Option<String>) -> anyhow::Result<PathBuf> {
        let profile_name =&self.profile.active_profile.name;
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(".marco-sparko-cache");
        if let Some(module_id) = module_id {
            path.push(format!("{}-{}-history.txt", profile_name, module_id));
        }
        else {
            path.push(format!("{}-history.txt", profile_name));
        }
                Ok(path)
    }

    fn get_cache_data_dir_path(&self, module_id: &str) -> anyhow::Result<PathBuf> {
        let profile_name =&self.profile.active_profile.name;

        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(".marco-sparko-cache");
        path.push(format!("{}-{}", profile_name, module_id));
        Ok(path)
    }
      
    fn create_cache_manager(&self, module_id: &str, verbose: bool) -> anyhow::Result<Arc<CacheManager>> {
        let dir_path = self.get_cache_data_dir_path(module_id)?;
        fs::create_dir_all(&dir_path)?;

        Ok(Arc::new(CacheManager {
            dir_path,
            verbose,
        }))
    }
    
    // pub fn update_profile<T>(&self, module_id: &str, profile: T) -> anyhow::Result<()>
    // where
    //     T: Serialize
    // {
    //     match self.profile_manager.update_profile(module_id, profile) {
    //         Ok(p) => Ok(p),
    //         Err(_) => Err(anyhow!("FAILED")),
    //     }
    // }

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

    pub fn update_cache<T>(&self, module_id: &str, profile: &T) -> anyhow::Result<()>
    where
        T: Serialize
    {
        let path = self.get_cache_file_path(module_id)?;

        serde_json::to_writer_pretty(fs::File::create(path)?, &profile)?;
            

        Ok(())
    }
}

pub struct MarcoSparko {
    context: Arc<MarcoSparkoContext>,
    module_registrations: ModuleRegistrations,
    modules: HashMap<String, Box<dyn Module>>,
    current_module: Option<String>,
}


impl MarcoSparko {

    pub fn get_file_path() -> anyhow::Result<PathBuf> {
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;

        path.push(".marco-sparko");
        Ok(path)
    }

    async fn exec_repl_command(&mut self, command: &str, args: std::str::SplitWhitespace<'_>) ->  anyhow::Result<()> {
        match command {
            "list" => self.list_handler(args).await,
            "init" => self.init_handler(args).await,
            _ => Err(anyhow!(format!("Invalid command '{}'", command)))
        }
    }

    fn get_repl_commands(&self) -> Vec<ReplCommand> {
        vec!(
            ReplCommand {
                command:"list",
                description: "List modules or profiles",
                help:
r#"
usage: list modules|profiles

"modules" lists all known modules and whether they are activated.
"profiles" lists all known profiles (run configurations) and indicates the active one.
"#,
            },
            ReplCommand {
                command:"init",
                description: "Initialize a module",
                help:
r#"
usage: init module_id

Initialize (activate) the given module.
"#,
            }
        )
    }

    fn get_global_repl_commands(&self) -> Vec<ReplCommand> {
        vec!(
            ReplCommand {
                command:"quit",
                description: "Quit Marco Sparko (also Ctrl-D)",
                help: "Terminates the application",
            },

            ReplCommand {
                command:"home",
                description: "Return to the main command context (outside any module)",
                help: 
r#"
usage: home

The main command context allows you to manage modules and the application as a whole. Each module has its own command context 
which provides access to the features of that module.
"#,
            },
            ReplCommand {
                command:"module",
                description: "Switch to the command context of an active module",
                help:
r#"
usage: module module_id

Switch to the command context of the given active module. To activate an inactive module use the init command.
"#,
            },
            ReplCommand {
                command:"help",
                description: "Print this message (try \"help help\" for more detail).",
                help: 
r#"
usage: help [command]

Without any arguments lists all the currently available commands, with a single command parameter,
prints more detailed help on that specific command.
"#,
            }
        )
    }

    pub async fn init_handler(&mut self, mut args: std::str::SplitWhitespace<'_>) -> anyhow::Result<()> {
        if let Some(module_id) = args.next() {
            if let Some(module_registration) = self.module_registrations.0.get(module_id) {
                if self.modules.contains_key(module_id) {
                    println!("ERROR: module '{}' is already active", module_id); 
                }
                else {
                    let constructor = module_registration.as_ref();
                    let profile = if let Some(value) = self.context.profile.active_profile.modules.get(module_id) {
                        Some(value.clone())
                    }
                    else {
                        None
                    };
                    let builder = constructor(self.context.clone(), profile)?;
                    let module = builder.build(true).await?;
                    self.modules.insert(module_id.to_string(),module);

                    if self.current_module.is_none() {
                        self.current_module = Some(module_id.to_string());
                    }
                }
            }
            else {
                println!("ERROR: unknown module '{}'", module_id); 
            }
        }
        Ok(())
    }

    pub async fn list_handler(&self, mut args: std::str::SplitWhitespace<'_>) -> anyhow::Result<()> {
        if let Some(target) = args.next() {

            match target.as_ref() {
                "modules" => {
                    for reg in &*self.module_registrations.0 {
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
                    for profile_name in &self.context.profile.all_profiles {
                        if profile_name == &self.context.profile.active_profile.name {
                            println!("{} [Active]", profile_name);
                        }
                        else {
                            println!("{}", profile_name);
                        }
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

    // pub async fn initialize_modules(registrations: ModuleRegistrations, profiles: HashMap<String, serde_json::Value>, init: bool) -> anyhow::Result<()> {

    //     let modules = HashMap::new();

    //     for (module_id, module_profile) in profiles {
    //         if let Some(module_registration) = registrations.get(&module_id) {
    //             let constructor = module_registration.as_ref();
    //             // let profile = if let Some(value) = self.context.profile_manager.active_profile.modules.get(module_id) {
    //             //     Some(value.clone())
    //             // }
    //             // else {
    //             //     None
    //             // };
    //             let builder = constructor(self.context.clone(), profile)?;
    //             let module = builder.build(init).await?;
    //             self.modules.insert(module_id.clone(),module);

    //             if self.current_module.is_none() {
    //                 self.current_module = Some(module_id.clone());
    //             }
                
                

    //             Ok(())
    //         }
    //         else {
    //             return Err(anyhow!(format!("Unknown module \"{}\"", module_id)))
    //         }
    //     }

    //     Ok(())
    // }

pub async fn new() -> anyhow::Result<MarcoSparko> {

    let mut marco_sparko_manager = MarcoSparko {
        context: MarcoSparkoContext::new()?,
        module_registrations: ModuleRegistrations::new(), //Self::load_modules(),
        modules: HashMap::new(),
        current_module: None,
    };

    let x = &marco_sparko_manager.context.profile.active_profile.modules;

    //    let active_profile = marco_sparko_manager.marco_sparko.get_active_profile();

    let init = marco_sparko_manager.context.args.init;

    // marco_sparko_manager.load_modules();

    let list = marco_sparko_manager.get_module_list();

    if list.is_empty() {
        let mut keys = Vec::new();
        for module_id in marco_sparko_manager.context.profile.active_profile.modules.keys() {
            keys.push(module_id.to_string());
        }
        for module_id in &keys {
            marco_sparko_manager.initialize(module_id, false).await?;
        }
    }
    else {
        for module_id in &list {
            marco_sparko_manager.initialize(module_id, init).await?;
        }
    }

    Ok(marco_sparko_manager)
}

    fn get_module_list(&self) -> Vec<String> {
        self.context.args.modules.clone()
    }

    pub fn args(&self) -> &Args {
        &self.context.args
    }



    pub async fn run(&mut self) -> anyhow::Result<()> {
        // if let Some(command) =  &self.args().command {
        //     match command {
        //         Commands::Summary => {
        //             self.summary().await?; 
                    
        //         }
        //         Commands::Bill => {
        //             self.bill().await?; 
                    
        //         }
        //         Commands::Test => {
        //             self.test().await?; 
                    
        //         },
        //     };

            
        // }
        // else {
        //     // match &self.current_module {
        //     //     Some(name) => {
        //     //         match self.modules.get_mut(name) {
        //     //             Some(module) => {
        //     //                 module.repl().await?;
        //     //             },
        //     //             None => return Err(Error::UserError(format!("Unable to find current module {}", name))),
        //     //         }
        //     //     },
        //     //     None => {
        //     //         self.repl().await?;
        //     //     },
        //     // }

        //     self.repl().await?;
        // }


        self.repl().await?;
        // LaunchBuilder::desktop().launch(App);
        // dioxus::launch(App);


        // self.context.save_updated_profile()?;

        return Ok(())
        // Err(Error::UserError(String::from("No command given - try 'Summary'")))
    }

    async fn repl(&mut self) -> anyhow::Result<()> {
        let marco_sparko_prompt = "Marco Sparko".to_string();

        

        

        loop {
            let (module_id, commands) = if let Some(module_id) = &self.current_module {
                let commands = if let Some(module) = self.modules.get(module_id) {
                    module.get_repl_commands()
                }
                else {
                    Vec::new()
                };
                (&module_id.clone(), commands)
            }
            else {
                (&marco_sparko_prompt, self.get_repl_commands())
            };

            let mut command_map = BTreeMap::new();
            let mut command_list = Vec::new();
            let mut max_command_len = 0;

            for cmd in commands {
                if cmd.command.len() > max_command_len {
                    max_command_len = cmd.command.len();
                }
                command_list.push(cmd.command.to_string());
                command_map.insert(cmd.command, cmd);
            }

            for cmd in self.get_global_repl_commands() {
                if cmd.command.len() > max_command_len {
                    max_command_len = cmd.command.len();
                }
                command_list.push(cmd.command.to_string());
                command_map.insert(cmd.command, cmd);
            }


            let completer = Box::new(DefaultCompleter::new_with_wordlen(command_list.clone(), 2));

            // let completer = Box::new(completer::ReplCompleter::new(command_list.clone()));

            let validator = Box::new(DefaultValidator);
            // Use the interactive menu to select options from the completer
            let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
            // Set up the required keybindings
            let mut keybindings = default_emacs_keybindings();
            keybindings.add_binding(
                KeyModifiers::NONE,
                KeyCode::Tab,
                ReedlineEvent::Menu("completion_menu".to_string()),
                // ReedlineEvent::UntilFound(vec![
                //     ReedlineEvent::Menu("completion_menu".to_string()),
                //     ReedlineEvent::MenuNext,
                // ]),
            );
            
            let edit_mode = Box::new(Emacs::new(keybindings));
            let history = Box::new(
                FileBackedHistory::with_file(50, self.context.get_history_file_path(&self.current_module)?)
                    .expect("Error configuring history with file"),
                );

            let mut line_editor = //Reedline::create();
                Reedline::create()
                    .with_hinter(Box::new(
                        DefaultHinter::default()
                            .with_style(Style::new().italic().fg(Color::LightGray)),
                )).with_validator(validator)

                .with_highlighter(Box::new(ExampleHighlighter::new(command_list.clone())))
                .with_completer(completer)
                .with_partial_completions(true)
                .with_quick_completions(true)
                .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
                .with_edit_mode(edit_mode)
                .with_history(history)
                ;

            let prompt = //SparkoPrompt::new(); //DefaultPrompt::default();
            DefaultPrompt {
                left_prompt: DefaultPromptSegment::Basic(module_id.clone()),
                right_prompt: DefaultPromptSegment::CurrentDateTime,
            };

            loop {
                let out = line_editor.read_line(&prompt).unwrap();
                match out {
                    Signal::Success(content) => {

                        let mut arg_iterator = content.split_whitespace();
                        if let Some(command) = arg_iterator.next() {
                            match command {
                                "quit" => return Ok(()),
                                "home" => {
                                    if self.current_module.is_none() {
                                        println!("You are already in the main command context.")
                                    }
                                    else {
                                        self.current_module = None;
                                        break;
                                    }
                                },
                                "module" => {
                                    if let Some(new_module) = arg_iterator.next() {
                                        if let Some(module_id) = &self.current_module {
                                            if module_id == new_module {
                                                println!("You are already in the '{}' command context.", new_module);
                                                continue;
                                            }
                                        }
                                        if self.module_registrations.0.contains_key(new_module) {
                                            if self.modules.contains_key(new_module) {
                                                self.current_module = Some(new_module.to_string());
                                                break;
                                            }
                                            else {
                                                println!("Module '{}' is inactive", new_module);
                                            }
                                        }
                                        else {
                                            println!("Unknown module '{}'", new_module);
                                        }
                                        break;
                                    }
                                    else {
                                        println!("usage: module module_id");
                                    }
                                },
                                "help" => {
                                    if let Some(param) = arg_iterator.next() {
                                        if let Some(cmd) = command_map.get(param) {
                                            println!("{}", cmd.help);
                                        }
                                        else {
                                            println!("Unrecognized command '{}'", param);
                                        }
                                    }
                                    else {
                                        for (name, command) in &command_map {
                                            println!("{:l$} {}", name, command.description, l = max_command_len);
                                        }
                                    }
                                },
                                _ => {

                                    let _result = if let Some(module_id) = &self.current_module {
                                        let module: &mut Box<dyn Module> = self.modules.get_mut(module_id).unwrap();
                                        module.exec_repl_command(&command, arg_iterator).await
                                    }
                                    else {
                                        // self.exec_repl_command(&command, arg_iterator).await
                                        match command {
                                            "list" => self.list_handler(arg_iterator).await,
                                            "init" => {
                                                self.init_handler(arg_iterator).await?;
                                                break;
                                            },
                                            _ => Err(anyhow!(format!("Invalid command '{}'", command)))
                                        }
                                    };
                                    // if let Err(error) = result {
                                    //     println!("{}", error);
                                    // }
                                }
                            }

                        }
                    }
                    Signal::CtrlD => return Ok(()),
                    _ => {
                        eprintln!("Entry aborted!");

                    }
                }
            }
        }
    }

    // async fn summary(&mut self) -> anyhow::Result<()> {
    //     for (_module_id, module) in self.modules.iter_mut() {
    //         println!("Summary {}", _module_id);
    //         module.summary().await?;
    //     }

    //     Ok(())
    // }

    // async fn bill(&mut self) -> anyhow::Result<()> {
    //     for (_module_id, module) in self.modules.iter_mut() {
    //         println!("Bill {}", _module_id);
    //         module.bill().await?;
    //     }

    //     Ok(())
    // }

    // async fn test(&mut self) -> anyhow::Result<()> {
    //     for (_module_id, module) in self.modules.iter_mut() {
    //         println!("Test {}", _module_id);
    //         module.test().await?;
    //     }

    //     Ok(())
    // }
    
    async fn initialize(&mut self, module_id: &String, init: bool) -> anyhow::Result<()> {
        // let module_registrations: &ModuleRegistrations = &self.module_registrations;
        // let profile: &profile::Profile = &self.context.profile.active_profile;
        // let context: Arc<MarcoSparkoContext> = self.context.clone();
        let module = Self::do_initialize(module_id, init, &self.module_registrations, &self.context).await?;

        self.modules.insert(module_id.clone(),module);

        if self.current_module.is_none() {
            self.current_module = Some(module_id.clone());
        }

        Ok(())
    }

    pub async fn do_initialize(module_id: &str, init: bool, module_registrations: &ModuleRegistrations, context: &Arc<MarcoSparkoContext>) -> anyhow::Result<Box<dyn Module + Send>> {
        if let Some(module_registration) = module_registrations.0.get(module_id) {
            let constructor = module_registration.as_ref();
            let profile = if let Some(value) = context.profile.active_profile.modules.get(module_id) {
                Some(value.clone())
            }
            else {
                None
            };
            let builder = constructor(context.clone(), profile)?;
            let module = builder.build(init).await?;
            
            
            

            Ok(module)
        }
        else {
            return Err(anyhow!(format!("Unknown module \"{}\"", module_id)))
        }
    }
}
