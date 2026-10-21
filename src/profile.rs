use std::collections::HashSet;
use std::{collections::HashMap};
use std::fs;
use indexmap::IndexMap;
use semver::Version;
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::{MarcoSparkoContext, private_file};

const DEFAULT_PROFILE: &str = "default";
pub const CURRENT_VERSION: &str = "0.4.3";

#[derive(PartialEq)]
pub struct ActiveProfile {
    pub all_profiles:   Vec<String>,
    pub active_profile: Profile,
    pub profile_version_state: ProfileVersionState,
}

fn profile_names_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

impl ActiveProfile {
    // Create empty default profile
    fn new() -> ActiveProfile {
        ActiveProfile {
            all_profiles: vec!(String::from(DEFAULT_PROFILE)),
            active_profile: Profile::new(),
            profile_version_state: ProfileVersionState::Current,
        }
    }
}

pub fn fetch_active_profile(profile_name: &Option<String>) -> anyhow::Result<ActiveProfile> {
    let mut all_profiles: Vec<String> = Vec::new();
    let mut set: HashSet<String> = HashSet::new();
    let mut active_profile = None;

    if let Ok(file)= fs::File::open(&MarcoSparkoContext::get_file_path()?) {
        let profile_file: ProfileFile = serde_json::from_reader(file)?;

        for profile in profile_file {
            if all_profiles.iter().any(|name| profile_names_equal(name, &profile.name)) {
                return Err(anyhow!("Duplicate profile \"{}\"", &profile.name));
            }
            set.insert(profile.name.clone());
            
            let got_it = if let Some(name) = profile_name {
                profile_names_equal(name, &profile.name)
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

    Ok(
        
        if let Some(active_profile) = active_profile
        {
            let profile_version_state = ProfileVersionState::from(&active_profile.version)?;
            ActiveProfile {
                all_profiles,
                active_profile: active_profile,
                profile_version_state
            }
        }
        else
        {
            let active_profile = ActiveProfile::new();
            let mut profiles = Vec::new();

            profiles.push(&active_profile.active_profile);

            let f = private_file::create_private_file(&MarcoSparkoContext::get_file_path()?)?;
            serde_json::to_writer_pretty(f, &profiles)?;

            active_profile
        }
    )
}

pub fn set_active_profile(profile_name: &String) -> anyhow::Result<ActiveProfile> {

    let mut all_profiles: Vec<String> = Vec::new();
    let mut map: IndexMap<String, Profile> = IndexMap::new();
    

    if let Ok(file)= fs::File::open(&MarcoSparkoContext::get_file_path()?) {
        let profile_file: ProfileFile = serde_json::from_reader(file)?;

        for profile in profile_file {
            if map.keys().any(|name| profile_names_equal(name, &profile.name)) {
                return Err(anyhow!("Duplicate profile \"{}\"", &profile.name));
            }
            all_profiles.push(profile.name.clone());
            map.insert(profile.name.clone(), profile);
        }
        let active_profile = if let Some(existing_name) = map.keys().find(|name| profile_names_equal(name, profile_name)).cloned() {
            map.shift_remove(&existing_name).unwrap()
        }
        else {
            return Err(anyhow!("No such profile \"{}\"", profile_name)); 
        };

        let mut profiles = Vec::new();

        profiles.push(&active_profile);
        profiles.extend(map.values());

        serde_json::to_writer_pretty(private_file::create_private_file(&MarcoSparkoContext::get_file_path()?)?, &profiles)?;
        let profile_version_state = ProfileVersionState::from(&active_profile.version)?;

        Ok(ActiveProfile {
            all_profiles,
            active_profile,
            profile_version_state,
        })
    }
    else {
        Err(anyhow!("No such profile \"{}\" (no profiles at all, in fact)", profile_name))
    }
}

pub fn create_profile(profile_name: &str) -> anyhow::Result<ActiveProfile> {
    let profile_name = profile_name.trim();
    if profile_name.is_empty() {
        return Err(anyhow!("Profile name cannot be empty"));
    }
    if !profile_name.chars().all(char::is_alphanumeric) {
        return Err(anyhow!("Profile name must contain only alphanumeric characters"));
    }

    let mut profiles: ProfileFile = if let Ok(file) = fs::File::open(&MarcoSparkoContext::get_file_path()?) {
        serde_json::from_reader(file)?
    } else {
        Vec::new()
    };

    if profiles.iter().any(|profile| profile_names_equal(&profile.name, profile_name)) {
        return Err(anyhow!("A profile named \"{}\" already exists", profile_name));
    }

    let mut active_profile = Profile::new();
    active_profile.name = profile_name.to_string();
    profiles.insert(0, active_profile.clone());

    serde_json::to_writer_pretty(
        private_file::create_private_file(&MarcoSparkoContext::get_file_path()?)?,
        &profiles,
    )?;

    let all_profiles = profiles.into_iter().map(|profile| profile.name).collect();
    let profile_version_state = ProfileVersionState::from(&active_profile.version)?;

    Ok(ActiveProfile {
        all_profiles,
        active_profile,
        profile_version_state,
    })
}

pub fn update_profile<T>(profile_name: &String, module_id: &str, module_profile: &T) -> anyhow::Result<()>
    where
    T: Serialize
{
    if let Ok(file)= fs::File::open(&MarcoSparkoContext::get_file_path()?) {
        let mut profile_file: ProfileFile = serde_json::from_reader(file)?;


        let mut found = false;

        for profile in profile_file.iter_mut() {
            if profile_names_equal(&profile.name, profile_name) {
                profile.modules.insert(module_id.to_string(), serde_json::to_value(module_profile)?);
                found = true;

            }
        }
        if ! found{
            return Err(anyhow!("No such profile \"{}\"", profile_name)); 
        };
    
        serde_json::to_writer_pretty(private_file::create_private_file(&MarcoSparkoContext::get_file_path()?)?, &profile_file)?;

        Ok(())
    }
    else {
        Err(anyhow!("No such profile \"{}\" (no profiles at all, in fact)", profile_name))
    }
}

pub type ModuleProfiles = HashMap<String, serde_json::Value>;
pub type ProfileFile = Vec<Profile>;

#[derive(PartialEq)]
pub enum ProfileVersionState {
    OlderThanCurrent,
    Current,
    NewerThanCurrent,
    Invalid
}

impl ProfileVersionState {
    pub fn from(version: &str) -> anyhow::Result<ProfileVersionState> {
        let version = version.trim();

        if version.len() == 0 {
            return Ok(ProfileVersionState::OlderThanCurrent);
        }


        let current_version = Version::parse(CURRENT_VERSION).unwrap();
        Ok(if let Ok(version) = Version::parse(version) {
            match version.cmp(&current_version) {
                std::cmp::Ordering::Less => Self::OlderThanCurrent,
                std::cmp::Ordering::Equal => Self::Current,
                std::cmp::Ordering::Greater => Self::NewerThanCurrent,
            }
        }
        else {
            Self::Invalid
        })
    }
}

fn default_profile_version() ->String {
    "0.0.0".to_string()
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    #[serde(default = "default_profile_version")]
    pub version: String,
    pub modules: ModuleProfiles
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            name: DEFAULT_PROFILE.to_string(),
            version: CURRENT_VERSION.to_string(),
            modules: ModuleProfiles::new(),
        }
    }
}