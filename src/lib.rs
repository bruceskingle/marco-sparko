pub mod octopus;
pub mod system;
pub mod util;
pub mod views;
pub mod components;
pub mod profile;

pub mod private_file;

mod cache_manager;
pub use cache_manager::CacheManager;


use std::collections::BTreeMap;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use anyhow::anyhow;
use async_trait::async_trait;
use dioxus::core::Element;
use dirs::home_dir;
use serde::de::DeserializeOwned;
use serde::Serialize;
use clap::{Parser};

use reedline::{Emacs, ExampleHighlighter, FileBackedHistory, MenuBuilder, ReedlineMenu};
use reedline::{default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultPrompt, DefaultPromptSegment, KeyCode, KeyModifiers, Reedline, ReedlineEvent, Signal};
use crate::profile::ActiveProfile;

use {
    nu_ansi_term::{Color, Style},
    reedline::{DefaultValidator, DefaultHinter},
  };

pub const CHECK_FOR_UPDATES: bool = true;
const CACHE_DIRECTORY_NAME: &'static str = ".marco-sparko-cache";

pub struct ReplCommand {
    pub command: &'static str,
    pub description: &'static str,
    pub help: &'static str,
}

#[derive(Parser, Debug, Clone, PartialEq)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
pub struct MarcoSparkoArgs {
    /// Name of the config profile to use
    #[arg(short, long)]
    profile: Option<String>,
    #[arg(short, long, value_delimiter = ',', num_args = 1..)]
    modules: Vec<String>,
    #[arg(short, long)]
    pub cli: bool,
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
        // s.split_once('-').map(|(head, _)| head).or(Some(s))
    }

    pub fn ms_parse() -> Self {
        // let args = Args::parse();

        // // filter out plugin args from main
        // let main_only_args: Vec<String> = std::env::args()
        //     .filter(|s| !s.contains('_'))
        //     // .cloned()
        //     .collect();



        let mut main_only_args = Vec::new();
        let mut module_args: HashMap<String, Vec<String>> = HashMap::new();
        let mut module_id: Option<String> = None;

        let mut it = std::env::args();
        let arg0 = it.next().unwrap_or_else(|| "marco-sparko".to_string());
        main_only_args.push(arg0.clone());
        for arg in it {
            if arg.starts_with("-") {
                println!("Processing arg: {}", arg);
                if let Some(module) = Args::extract_module(&arg) {
                    module_id = Some(module.to_string());
                    println!("  identified module arg for module '{}'", module);
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
        println!("Main args (ignoring plugin args): {:?}", &main_only_args);

        let marco_sparko_args = MarcoSparkoArgs::parse_from(main_only_args);


       

        println!("Module args : {:?}", &module_args);

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
pub trait Module: CommandProvider {
    fn get_page_list(&self) -> Vec<PageInfo>;
    fn module_id(&self) -> &'static str;
    fn get_component<'a>(&'a self, page_id: &'a str, path: Vec<String>) -> Box<dyn Fn() -> Element + 'a>;
    fn cli_debug(&self) -> anyhow::Result<()>;
}

#[async_trait(?Send)]
pub trait CommandProvider {
    fn get_repl_commands(&self) -> Vec<ReplCommand>;
    async fn exec_repl_command(&mut self, command: &str, args: std::str::SplitWhitespace<'_>) ->  anyhow::Result<()>;
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
    fn new() -> ModuleRegistrations {


        let dir = std::env::current_dir().unwrap();
        println!("Current directory is {}", dir.display());
println!("No assertion failure here");
        //assert!(cfg!(debug_assertions));


        let mut module_registrations = HashMap::new();

        Self::load_module(&mut module_registrations, octopus::OctopusModule::registration());

        println!("Loaded {} modules", module_registrations.len());

        for (k, _v) in &module_registrations {
            println!(" Module {}", k);
        }
        ModuleRegistrations(Arc::new(module_registrations))
    }

    fn load_module(module_registrations: &mut HashMap<String, ModuleRegistration> , registration: ModuleRegistration) {
        println!("Load module {}", &registration.module_id);
        module_registrations.insert(registration.module_id.clone(), registration);
    }
}

 pub struct MarcoSparkoContext {
    pub args: Args,
    pub profile: ActiveProfile,
}

// impl PartialEq for MarcoSparkoContext {
//     fn eq(&self, other: &Self) -> bool {
//         self.args == other.args && self.profile == other.profile
//     }
// }

impl MarcoSparkoContext {
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

    fn get_history_file_path(&self, module_id: &Option<String>) -> anyhow::Result<PathBuf> {
        let profile_name =&self.profile.active_profile.name;
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(CACHE_DIRECTORY_NAME);
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
        path.push(CACHE_DIRECTORY_NAME);
        path.push(format!("{}-{}", profile_name, module_id));
        Ok(path)
    }
      
    fn create_cache_manager(&self, module_id: &str, verbose: bool) -> anyhow::Result<Arc<CacheManager>> {
        let dir_path = self.get_cache_data_dir_path(module_id)?;
        private_file::create_private_dir(&dir_path)?;

        Ok(Arc::new(CacheManager {
            dir_path,
            verbose,
        }))
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

pub struct Cli {
    context: Arc<MarcoSparkoContext>,
    module_registrations: ModuleRegistrations,
    modules: HashMap<String, Box<dyn Module>>,
    current_module: Option<String>,
}


impl Cli {

    pub fn get_file_path() -> anyhow::Result<PathBuf> {
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;

        path.push(".marco-sparko");
        Ok(path)
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
                    let constructor = module_registration.constructor.as_ref();
                    let profile = if let Some(value) = self.context.profile.active_profile.modules.get(module_id) {
                        Some(value.clone())
                    }
                    else {
                        None
                    };
                    let builder = constructor(self.context.clone(), profile)?;
                    let module = builder.build().await?;
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

    pub async fn new(args: Args) -> anyhow::Result<Cli> {
        let mut cli = Cli {
            context: MarcoSparkoContext::new(args)?,
            module_registrations: ModuleRegistrations::new(), //Self::load_modules(),
            modules: HashMap::new(),
            current_module: None,
        };

        let list = cli.get_module_list();

        if list.is_empty() {
            let mut keys = Vec::new();
            for module_id in cli.context.profile.active_profile.modules.keys() {
                keys.push(module_id.to_string());
            }
            for module_id in &keys {
                cli.initialize(module_id).await?;
            }
        }
        else {
            for module_id in &list {
                cli.initialize(module_id).await?;
            }
        }

        Ok(cli)
    }

    fn get_module_list(&self) -> Vec<String> {
        self.context.args.marco_sparko_args.modules.clone()
    }

    pub fn args(&self) -> &Args {
        &self.context.args
    }



    pub async fn run(&mut self) -> anyhow::Result<()> {


        if self.context.args.marco_sparko_args.debug {
            println!("Args: {:?}", self.context.args.marco_sparko_args);
            for (module_id, module) in &self.modules {
                println!("Module '{}':", module_id);
                module.cli_debug()?;
            }
        }
        self.repl().await?;

        return Ok(())
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
            let validator = Box::new(DefaultValidator);
            // Use the interactive menu to select options from the completer
            let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
            // Set up the required keybindings
            let mut keybindings = default_emacs_keybindings();
            keybindings.add_binding(
                KeyModifiers::NONE,
                KeyCode::Tab,
                ReedlineEvent::Menu("completion_menu".to_string()),
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

                                    let result = if let Some(module_id) = &self.current_module {
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
                                    if let Err(error) = result {
                                        println!("\n\n\nERROR============================================================\n{}", error);
                                    }
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
    
    async fn initialize(&mut self, module_id: &String) -> anyhow::Result<()> {
        let module = Self::do_initialize(module_id, &self.module_registrations, &self.context).await?;

        self.modules.insert(module_id.clone(),module);

        if self.current_module.is_none() {
            self.current_module = Some(module_id.clone());
        }

        Ok(())
    }

    pub async fn do_initialize(module_id: &str, module_registrations: &ModuleRegistrations, context: &Arc<MarcoSparkoContext>) -> anyhow::Result<Box<dyn Module + Send>> {
        if let Some(module_registration) = module_registrations.0.get(module_id) {
            let constructor = module_registration.constructor.as_ref();
            let profile = if let Some(value) = context.profile.active_profile.modules.get(module_id) {
                Some(value.clone())
            }
            else {
                None
            };

            println!("Initializing module '{}' with profile '{:?}'", module_id, profile);
            let builder = constructor(context.clone(), profile)?;
            let module = builder.build().await?;
            
            
            

            Ok(module)
        }
        else {
            return Err(anyhow!(format!("Unknown module \"{}\"", module_id)))
        }
    }

    pub async fn do_construct(module_id: &str, module_registrations: &ModuleRegistrations, context: &Arc<MarcoSparkoContext>) -> anyhow::Result<Arc<dyn ModuleFactory + Send>> {
        if let Some(module_registration) = module_registrations.0.get(module_id) {
            let constructor = module_registration.constructor.as_ref();
            let profile = if let Some(value) = context.profile.active_profile.modules.get(module_id) {
                Some(value.clone())
            }
            else {
                None
            };

            println!("Initializing module '{}' with profile '{:?}'", module_id, profile);
            let builder = constructor(context.clone(), profile)?;

            Ok(builder)
        }
        else {
            return Err(anyhow!(format!("Unknown module \"{}\"", module_id)))
        }
    }
}
