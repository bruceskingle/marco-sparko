pub mod octopus;
pub mod system;
pub mod util;
pub mod views;
pub mod components;
pub mod profile;

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Lines, Write};
use std::io::BufRead;
use std::path::Path;
use std::{collections::HashMap, fs, path::PathBuf, sync::{Arc, Mutex}};
use anyhow::anyhow;
use async_trait::async_trait;
use dirs::home_dir;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};

use reedline::{Emacs, ExampleHighlighter, FileBackedHistory, MenuBuilder, ReedlineMenu};
use reedline::{default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultPrompt, DefaultPromptSegment, KeyCode, KeyModifiers, Reedline, ReedlineEvent, Signal};
use sparko_graphql::types::Date;
use time::Month;
use {
    nu_ansi_term::{Color, Style},
    reedline::{DefaultValidator, DefaultHinter},
  };

use profile::ProfileManager;

pub const CHECK_FOR_UPDATES: bool = true;
pub struct ReplCommand {
    pub command: &'static str,
    pub description: &'static str,
    pub help: &'static str,
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



#[async_trait]
pub trait Module: CommandProvider {
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

type ModuleConstructor = dyn Fn(Arc<MarcoSparkoContext>, Option<serde_json::Value>) -> anyhow::Result<Box<dyn ModuleBuilder>>;

/*
 * This context is shared with all modules and needs to be separate from MarcoSparko because that struct holds the list of modules.
 */
pub struct MarcoSparkoContext {
    pub args: Args,
    pub profile_manager: ProfileManager,
}

impl MarcoSparkoContext {
    fn new() -> anyhow::Result<Arc<MarcoSparkoContext>> {

        let args = Args::parse();
        let profile_manager = Self::create_profile_manager(&args.profile)?;
        

        Ok(Arc::new(MarcoSparkoContext {
            args,
            profile_manager,
       }))
    }

    fn create_profile_manager(active_profile: &Option<String>) -> anyhow::Result<ProfileManager>  {
        match ProfileManager::new(active_profile) {
            Ok(p) => Ok(p),
            Err(_) => Err(anyhow!("FAILED")),
        }
    }

    fn save_updated_profile(&self) -> anyhow::Result<()> {
        match self.profile_manager.save_updated_profile() {
            Ok(p) => Ok(p),
            Err(_) => Err(anyhow!("FAILED")),
        }
    }

    

    fn get_cache_file_path(&self, module_id: &str) -> anyhow::Result<PathBuf> {
        let profile_name = &self.profile_manager.active_profile.name;
        let mut path = home_dir().ok_or(anyhow!("Unable to locate home directory"))?;
        path.push(".marco-sparko-cache");
        path.push(format!("{}-{}.json", profile_name, module_id));
                Ok(path)
    }

    fn get_history_file_path(&self, module_id: &Option<String>) -> anyhow::Result<PathBuf> {
        let profile_name =&self.profile_manager.active_profile.name;
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
        let profile_name =&self.profile_manager.active_profile.name;

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
    
    pub fn update_profile<T>(&self, module_id: &str, profile: T) -> anyhow::Result<()>
    where
        T: Serialize
    {
        match self.profile_manager.update_profile(module_id, profile) {
            Ok(p) => Ok(p),
            Err(_) => Err(anyhow!("FAILED")),
        }
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
    module_registrations: HashMap<String, Box<ModuleConstructor>>,
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
            if let Some(module_registration) = self.module_registrations.get(module_id) {
                if self.modules.contains_key(module_id) {
                    println!("ERROR: module '{}' is already active", module_id); 
                }
                else {
                    let constructor = module_registration.as_ref();
                    let profile = if let Some(value) = self.context.profile_manager.active_profile.modules.get(module_id) {
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
                    for profile in &self.context.profile_manager.before_profiles {
                        println!("{}", profile.name);
                    }
                    println!("{} [Active]", &self.context.profile_manager.active_profile.name);

                    for profile in &self.context.profile_manager.after_profiles {
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

pub async fn new() -> anyhow::Result<MarcoSparko> {

    let mut marco_sparko_manager = MarcoSparko {
        context: MarcoSparkoContext::new()?,
        module_registrations: HashMap::new(),
        modules: HashMap::new(),
        current_module: None,
    };

    //    let active_profile = marco_sparko_manager.marco_sparko.get_active_profile();

    let init = marco_sparko_manager.context.args.init;

    marco_sparko_manager.load_modules();

    let list = marco_sparko_manager.get_module_list();

    if list.is_empty() {
        let mut keys = Vec::new();
        for module_id in marco_sparko_manager.context.profile_manager.active_profile.modules.keys() {
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

    fn load_modules(&mut self) {
        self.load_module(octopus::Client::registration());
    }

    fn load_module(&mut self, registration: (String, Box<ModuleConstructor>)) {
        self.module_registrations.insert(registration.0, registration.1);
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


        self.context.save_updated_profile()?;

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
                                        if self.module_registrations.contains_key(new_module) {
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
        if let Some(module_registration) = self.module_registrations.get(module_id) {
            let constructor = module_registration.as_ref();
            let profile = if let Some(value) = self.context.profile_manager.active_profile.modules.get(module_id) {
                Some(value.clone())
            }
            else {
                None
            };
            let builder = constructor(self.context.clone(), profile)?;
            let module = builder.build(init).await?;
            self.modules.insert(module_id.clone(),module);

            if self.current_module.is_none() {
                self.current_module = Some(module_id.clone());
            }
            
            

            Ok(())
        }
        else {
            return Err(anyhow!(format!("Unknown module \"{}\"", module_id)))
        }
    }
}

pub struct CacheManager {
    pub dir_path: PathBuf,
    pub verbose: bool,
}

impl CacheManager {
    fn path_for_date(path: &mut PathBuf, date: &Date) {
        path.push(date.year().to_string());
        // path.push(date.month().to_string());
    }

    fn path_hash_key_for_date(path: &mut PathBuf, date: &Date, hash_key: &str) {
        // path.push(format!("{}#{}", date.day(), hash_key));
        path.push(format!("{}#{}", date.month(), hash_key));
    }

    pub fn write<T: Serialize>(&self, hash_key: &str, vec: &Vec<(String, T)>, cached_cnt: usize) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        self.do_write(path, vec, cached_cnt)
    }

    pub fn write_for_date<T: Serialize>(&self, date: &Date, hash_key: &str, vec: &Vec<(String, T)>, cached_cnt: usize) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();

        Self::path_for_date(&mut path, date);

        fs::create_dir_all(&path)?;

        Self::path_hash_key_for_date(&mut path, date, hash_key);

        self.do_write(path, vec, cached_cnt)
    }

    fn do_write<T: Serialize>(&self, path: PathBuf, vec: &Vec<(String, T)>, cached_cnt: usize) -> anyhow::Result<()> {

        if cached_cnt == 0 {
            let mut out = fs::File::create(path)?;
            for (key, value) in vec {
                writeln!(out, "{}\t{}", key, serde_json::to_string(&value)?)?;
                if self.verbose 
                {
                    println!("WRITE {}", key);
                }
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

    pub fn read<T: DeserializeOwned>(&self, hash_key: &str, vec: &mut Vec<(String, T)>) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        self.do_read(path, vec)
    }

    pub fn read_for_date<T: DeserializeOwned>(&self, date: &Date, hash_key: &str, vec: &mut Vec<(String, T)>) -> anyhow::Result<(Date, Date)> {
        let start_date = Date::from_calendar_date(date.year(), date.month(), 1)?;
        let end_date = if date.month() == Month::December {
            Date::from_calendar_date(date.year() + 1, Month::January, 1)?
        }
        else {
            Date::from_calendar_date(date.year(), date.month().next(), 1)?
        };
        let mut path = self.dir_path.clone();

        Self::path_for_date(&mut path, date);
        Self::path_hash_key_for_date(&mut path, date, hash_key);

        self.do_read(path, vec)?;

        Ok((start_date, end_date))
    }

    fn do_read<T: DeserializeOwned>(&self, path: PathBuf, vec: &mut Vec<(String, T)>) -> anyhow::Result<()> {
        match Self::read_lines(path) {
            Ok(lines) => {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.map_while(Result::ok) {
                    if self.verbose 
                    {
                        println!("READ {}", line);
                    }

                    match line.split_once('\t') {
                        Some((key, value)) => vec.push((key.to_string(), serde_json::from_str(value)?)),
                        None => return Err(anyhow!(format!("Invalid cached object <{}>", line))),
                    }
                }
            },

            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    println!("ERROR {:?}", error);
                    return Err(anyhow!(error))
                }
                
            },
        }

        Ok(())
    }


    pub fn write_one<T: Serialize>(&self, hash_key: &str, value: &T) -> anyhow::Result<()> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        let mut out = fs::File::create(path)?;
        writeln!(out, "{}", serde_json::to_string(&value)?)?;

        Ok(())
    }

    pub fn read_one<T: DeserializeOwned>(&self, hash_key: &str) -> anyhow::Result<Option<T>> {
        let mut path = self.dir_path.clone();
        path.push(hash_key);

        Ok(match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Some(serde_json::from_reader(reader)?)
            },
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    println!("ERROR {:?}", error);
                    return Err(anyhow!(error))
                }
                None
            },
        })
    }
}