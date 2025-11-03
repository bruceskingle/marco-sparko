use std::{collections::HashMap, path::PathBuf};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::{MarcoSparko};

const DEFAULT_PROFILE: &str = "default";

type ModuleProfiles = HashMap<String, serde_json::Value>;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub modules: ModuleProfiles
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            name: DEFAULT_PROFILE.to_string(),
            modules: ModuleProfiles::new()
        }
    }
}


pub struct ProfileManager {
    pub before_profiles: Vec<Profile>,
    pub active_profile: Profile,
    pub after_profiles: Vec<Profile>,
    updated_profile: Mutex<Profile>,
    // file_path: &Path,
}

impl ProfileManager {
     pub fn get_file_path() -> anyhow::Result<PathBuf> {
        let mut path = dirs::home_dir().ok_or(anyhow!("Unable to locate home directory"))?;

        path.push(".marco-sparko");
        Ok(path)
    }

    pub fn new(active_profile_name: &Option<String>) -> anyhow::Result<ProfileManager>  {

        // let x: i32 = "3.14".parse()?;

        // panic!("TEST");
        let before_profiles;
        let opt_active_profile;
        let active_profile;
        let profile_name;
        let after_profiles;
        let updated_profile;

        if let Ok(file)= fs::File::open(&Self::get_file_path()?) {
            let profiles: Vec<Profile> = serde_json::from_reader(file)?;
            
            (before_profiles, opt_active_profile, after_profiles) = Self::remove_active_profile(active_profile_name, profiles)?;

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

        Ok(ProfileManager{
            before_profiles,
            active_profile,
            after_profiles,
            updated_profile,
            // file_path
        })
    }


    fn remove_active_profile(active_profile_name: &Option<String>, mut profiles: Vec<Profile>) -> 
        anyhow::Result<(Vec<Profile>, Option<Profile>, Vec<Profile>)> {
        if let Some(profile_name) = active_profile_name {
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
                active_profile = Some(Profile {
                    name: profile_name.clone(),
                    modules: ModuleProfiles::new()
                });
                // return Result::Err(Error::UserError(format!("No profile called {}", profile_name)))
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

    pub fn save_updated_profile(&self) -> anyhow::Result<()> {
        let updated_profile = match self.updated_profile.lock() {
            Ok(p) => p,
            Err(_) => return Err(anyhow!("Profile manager mutex is poisoned")),
        };

        if updated_profile.modules.is_empty() {
            Ok(())
        }
        else {

            let mut profiles = Vec::new();
            
            profiles.extend(&self.before_profiles);
            profiles.push(&updated_profile);
            profiles.extend(&self.after_profiles);
            
            serde_json::to_writer_pretty(fs::File::create(&Self::get_file_path()?)?, &profiles)?;
            
            return Ok(())
        }
    }
    
    pub fn update_profile<T>(&self, module_id: &str, profile: T) -> anyhow::Result<()>
    where
        T: Serialize
    {
        let mut updated_profile = match self.updated_profile.lock() {
            Ok(p) => p,
            Err(_) => return Err(anyhow!("Profile manager mutex is poisoned")),
        };

        updated_profile.modules.insert(module_id.to_string(), serde_json::to_value(profile)?);
        Ok(())
    }
}