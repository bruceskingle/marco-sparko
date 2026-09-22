use std::collections::HashSet;
use std::{collections::HashMap};
use std::fs;
use std::sync::Mutex;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::{Cli, private_file};

const DEFAULT_PROFILE: &str = "default";

#[derive(PartialEq)]
pub struct ActiveProfile {
    pub all_profiles:   Vec<String>,
    pub active_profile: Profile,
}

impl ActiveProfile {
    // Create empty default profile
    fn new() -> ActiveProfile {
        ActiveProfile {
            all_profiles: vec!(String::from(DEFAULT_PROFILE)),
            active_profile: Profile::new(),
        }
    }
}

pub fn fetch_active_profile(profile_name: &Option<String>) -> anyhow::Result<ActiveProfile> {
    let mut all_profiles = Vec::new();
    let mut set = HashSet::new();
    let mut active_profile = None;

    if let Ok(file)= fs::File::open(&Cli::get_file_path()?) {
        let profile_file: ProfileFile = serde_json::from_reader(file)?;

        for profile in profile_file {
            if set.contains(&profile.name) {
                return Err(anyhow!("Duplicate profile \"{}\"", &profile.name));
            }
            set.insert(profile.name.clone());
            
            let got_it = if let Some(name) = profile_name {
                name == &profile.name
            }
            else {
                active_profile.is_none()
            };

            if got_it {
                all_profiles.push(profile.name.clone());
                active_profile = Some(profile)
            }
            else {
                all_profiles.push(profile.name);
            }
        }
    }

    if let Some(name) = profile_name {
        if active_profile.is_none() {
            return Err(anyhow!("No such profile \"{}\"", name));
        }
    }

    Ok(if active_profile.is_none() {
        let active_profile = ActiveProfile::new();
        let mut profiles = Vec::new();

        profiles.push(&active_profile.active_profile);

        let f = private_file::create_private_file(&Cli::get_file_path()?)?;
        println!("About to write to private file {:?}...", f);
        serde_json::to_writer_pretty(f, &profiles)?;

        println!("Done");
        active_profile
    }
    else {
        ActiveProfile {
            all_profiles,
            active_profile: active_profile.unwrap(),
        }
    })
}

pub fn set_active_profile(profile_name: &String) -> anyhow::Result<ActiveProfile> {

    let mut all_profiles = Vec::new();
    let mut map = IndexMap::new();
    

    if let Ok(file)= fs::File::open(&Cli::get_file_path()?) {
        let profile_file: ProfileFile = serde_json::from_reader(file)?;

        for profile in profile_file {
            if map.contains_key(&profile.name) {
                return Err(anyhow!("Duplicate profile \"{}\"", &profile.name));
            }
            all_profiles.push(profile.name.clone());
            map.insert(profile.name.clone(), profile);
        }
        let active_profile = if let Some(p) = map.shift_remove(profile_name) {
            p
        }
        else {
            return Err(anyhow!("No such profile \"{}\"", profile_name)); 
        };

        let mut profiles = Vec::new();

        profiles.push(&active_profile);
        profiles.extend(map.values());

        serde_json::to_writer_pretty(private_file::create_private_file(&Cli::get_file_path()?)?, &profiles)?;

        Ok(ActiveProfile {
            all_profiles,
            active_profile,
        })
    }
    else {
        Err(anyhow!("No such profile \"{}\" (no profiles at all, in fact)", profile_name))
    }
}
    
pub fn update_profile<T>(profile_name: &String, module_id: &str, module_profile: &T) -> anyhow::Result<()>
    where
    T: Serialize
{
    if let Ok(file)= fs::File::open(&Cli::get_file_path()?) {
        let mut profile_file: ProfileFile = serde_json::from_reader(file)?;

        println!("existing profile_file <{:?}>", &profile_file);

        let mut found = false;
        // let mut profiles = Vec::new();

        for profile in profile_file.iter_mut() {
            if &profile.name == profile_name {
                println!("existing profile <{:?}>", &profile);
                profile.modules.insert(module_id.to_string(), serde_json::to_value(module_profile)?);
                found = true;

                println!("updated profile <{:?}>", &profile);
            }
            // profiles.push(profile);
        }
        if ! found{
            return Err(anyhow!("No such profile \"{}\"", profile_name)); 
        };

         println!("saving profiles <{:?}>", &profile_file);

            serde_json::to_writer_pretty(private_file::create_private_file(&Cli::get_file_path()?)?, &profile_file)?;

        Ok(())
    }
    else {
        Err(anyhow!("No such profile \"{}\" (no profiles at all, in fact)", profile_name))
    }
}

pub type ModuleProfiles = HashMap<String, serde_json::Value>;
pub type ProfileFile = Vec<Profile>;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub modules: ModuleProfiles
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            name: DEFAULT_PROFILE.to_string(),
            modules: ModuleProfiles::new(),
        }
    }
}

struct ProfileMap {
    map: IndexMap<String, Profile>,
    updated: bool,
}

pub struct ProfileManager {
    profile_map:    Mutex<ProfileMap>,
    pub active_profile: Profile,
    pub profile_names: Vec<String>,



    // pub before_profiles: Vec<Profile>,
    // pub after_profiles: Vec<Profile>,
    // updated_profile: Mutex<Profile>,
    // // file_path: &Path,
}

impl PartialEq for ProfileManager {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl ProfileManager {
    pub fn new(selector: &Option<String>) -> anyhow::Result<ProfileManager>  {
        let mut map = IndexMap::new();
        let mut profile_names = Vec::new();

        if let Ok(file)= fs::File::open(&Cli::get_file_path()?) {
            let profile_file: ProfileFile = serde_json::from_reader(file)?;

            for profile in profile_file {
                if map.contains_key(&profile.name) {
                    return Err(anyhow!("Duplicate profile \"{}\"", &profile.name));
                }
                profile_names.push(profile.name.clone());
                map.insert(profile.name.clone(), profile);
            }
        }

        if map.is_empty() {
            let profile = Profile::new();
            map.insert(profile.name.clone(), profile);
        }

        // we know its not empty
        // let first_profile_name = map.keys().into_iter().next().unwrap();
        // let default = if let Some(name) = opt_default_name { name.clone() } else { first_profile_name.clone()};
        // let last_used = if let Some(name) = opt_last_name { name.clone() } else { first_profile_name.clone()};

        let active_name = if let Some(name) = selector {
            &name
        }
        else {
            DEFAULT_PROFILE
        };

        // match selector {
        //     ProfileSelector::Default => default.clone(),
        //     ProfileSelector::Last => last_used.clone(),
        //     ProfileSelector::Named(name) => name.clone(),
        // };

        let active_profile = if let Some(profile) = map.get(active_name) {
            profile.clone()
        }
        else {
            return Err(anyhow!("No such profile \"{}\"", active_name));
        };
            
        //     let mut it = profiles.into_iter();

        //     if let Some(profile) = it.next() {
        //         let default_profile = &profile;
        //         map.insert(profile.name.clone(), default_profile.clone());

                

        //         active_profile = if let Some(name) = active_profile_name {
        //             if let Some(profile) = map.get(name) {
        //                 profile.clone()
        //             }
        //             else {
        //                 return Err(anyhow!("No such profile \"{}\"", name));
        //             }
        //         }
        //         else {
        //             default_profile.clone()
        //         };
        //     }
        //     else {
        //         active_profile = Profile::new();
        //         map.insert(active_profile.name.clone(), active_profile.clone());
                
        //         if let Some(name) = active_profile_name {
        //             if name != &active_profile.name {
        //                 return Err(anyhow!("No such profile \"{}\"", name));
        //             }
        //         }
        //     }
        // }
        // else {
        //     active_profile = Profile::new();
        //     map.insert(active_profile.name.clone(), active_profile.clone());
            
        //     if let Some(name) = active_profile_name {
        //         if name != &active_profile.name {
        //             return Err(anyhow!("No such profile \"{}\"", name));
        //         }
        //     }
        // }
        

        Ok(ProfileManager {
            profile_map: Mutex::new(ProfileMap {
                map,
                updated: true,
            }),
            active_profile,
            profile_names,
        })








        // // let x: i32 = "3.14".parse()?;

        // // panic!("TEST");
        // let before_profiles;
        // let opt_active_profile;
        // let active_profile;
        // let profile_name;
        // let after_profiles;
        // let updated_profile;

        // if let Ok(file)= fs::File::open(&MarcoSparko::get_file_path()?) {
        //     let profiles: Vec<Profile> = serde_json::from_reader(file)?;
            
        //     (before_profiles, opt_active_profile, after_profiles) = Self::remove_active_profile(active_profile_name, profiles)?;

        // }
        // else {
        //     before_profiles = Vec::new();
        //     opt_active_profile = None;
        //     after_profiles = Vec::new();
        // }


        // if let Some(existing_profile) = opt_active_profile {
        //     profile_name = existing_profile.name.clone();
        //     active_profile = existing_profile;
        // }
        // else {
        //     profile_name = DEFAULT_PROFILE.to_string();
        //     active_profile = Profile {
        //         name: profile_name.clone(),
        //         modules: ModuleProfiles::new()
        //     }
        // };

        // updated_profile = Mutex::new(Profile {
        //     name: profile_name,
        //     modules: ModuleProfiles::new()
        // });

        // Ok(ProfileManager{
        //     before_profiles,
        //     active_profile,
        //     after_profiles,
        //     updated_profile,
        //     // file_path
        // })
    }


    // fn remove_active_profile(active_profile_name: &Option<String>, mut profiles: Vec<Profile>) -> 
    //     anyhow::Result<(Vec<Profile>, Option<Profile>, Vec<Profile>)> {
    //     if let Some(profile_name) = active_profile_name {
    //         let mut active_profile = None;
    //         let mut before_profiles = Vec::new();
    //         let mut after_profiles = Vec::new();
    //         let mut after = false;
            
            
    //         for profile in profiles {
    //             if profile.name.eq(profile_name) {
    //                 active_profile = Some(profile);
    //                 after = true;
    //             }
    //             else {
    //                 if after {
    //                     after_profiles.push(profile);
    //                 }
    //                 else {
    //                     before_profiles.push(profile);
    //                 }
    //             }
    //         }

    //         if let None = active_profile {
    //             active_profile = Some(Profile {
    //                 name: profile_name.clone(),
    //                 modules: ModuleProfiles::new()
    //             });
    //             // return Result::Err(Error::UserError(format!("No profile called {}", profile_name)))
    //         }
    //         return Ok((before_profiles, active_profile, after_profiles))
    //     }

    //     if profiles.is_empty() {
    //         Ok((Vec::new(), None, profiles))
    //     }
    //     else {
    //         Ok((Vec::new(), Some(profiles.remove(0)), profiles))
    //     }
    // }

    pub fn save_updated_profile(&self) -> anyhow::Result<()> {
        let profile_map = match self.profile_map.lock() {
            Ok(p) => p,
            Err(_) => return Err(anyhow!("Profile manager mutex is poisoned")),
        };

        if profile_map.updated  {
            let mut profiles = Vec::new();
            
            profiles.extend(profile_map.map.values().map(|p| p.clone()));

            // let profile_file = ProfileFile {
            //     default: Some(profile_map.default.clone()),
            //     last_used: Some(profile_map.last_used.clone()),
            //     profiles,
            // };

           
            serde_json::to_writer_pretty(private_file::create_private_file(&Cli::get_file_path()?)?, &profiles)?;
            
            return Ok(())
        }
        else {
            Ok(())
        }


















        // let updated_profile = match self.updated_profile.lock() {
        //     Ok(p) => p,
        //     Err(_) => return Err(anyhow!("Profile manager mutex is poisoned")),
        // };

        // if updated_profile.modules.is_empty() {
        //     Ok(())
        // }
        // else {

        //     let mut profiles = Vec::new();
            
        //     profiles.extend(&self.before_profiles);
        //     profiles.push(&updated_profile);
        //     profiles.extend(&self.after_profiles);
            
        //     serde_json::to_writer_pretty(private_file::create_private_file(&MarcoSparko::get_file_path()?)?, &profiles)?;
            
        //     return Ok(())
        // }
    }
    
    // pub fn update_profile<T>(&self, module_id: &str, module_profile: T) -> anyhow::Result<()>
    // where
    //     T: Serialize
    // {
    //     let mut profile_map = match self.profile_map.lock() {
    //         Ok(p) => p,
    //         Err(_) => return Err(anyhow!("Profile manager mutex is poisoned")),
    //     };

    //     if let Some(profile) = profile_map.map.get_mut(&self.active_profile.name) {
    //         profile.modules.insert(module_id.to_string(), serde_json::to_value(module_profile)?);
    //     }
    //     else {
    //         return Err(anyhow!("Internal error, failed to find active profile {}", &self.active_profile.name))
    //     }
    //     Ok(())
    // }
}