//! The data directory, which holds the profile file .marco-sparko and the cache directory .marco-sparko-cache. It is
//! the home directory unless it has been moved elsewhere, e.g. to keep the credentials in them on another drive.

use std::{ffi::OsString, fs, path::{Path, PathBuf}, sync::OnceLock};

use anyhow::anyhow;

/// The environment variable which can give the data directory.
pub const HOME_ENV_VAR: &str = "MARCO_SPARKO_HOME";
/// The file in the home directory which can give the data directory, it contains just the path.
pub const POINTER_FILE_NAME: &str = ".marco-sparko-home";

/// The data directory and where it was set, None for the home directory.
#[derive(Clone, Debug, PartialEq)]
struct DataHome {
    path: PathBuf,
    source: Option<String>,
}

static DATA_HOME: OnceLock<Result<DataHome, String>> = OnceLock::new();

/// The data directory, which is the first of
/// 1. the --home command line option
/// 2. the MARCO_SPARKO_HOME environment variable
/// 3. the path in the file ~/.marco-sparko-home
/// 4. the home directory
///
/// It is worked out once. A data directory other than the home directory is created if its parent exists. If it does
/// not, e.g. because it is on a drive which is not connected, this is an error rather than falling back to the home
/// directory, so that credentials are never written somewhere the user did not choose.
pub fn data_home() -> anyhow::Result<PathBuf> {
    let data_home = DATA_HOME
        .get_or_init(|| {
            let option = crate::Args::ms_parse().marco_sparko_args.home;
            resolve(option, std::env::var_os(HOME_ENV_VAR), dirs::home_dir()).map_err(|error| error.to_string())
        })
        .clone()
        .map_err(|error| anyhow!(error))?;

    if let Some(source) = &data_home.source {
        ensure_dir(&data_home.path, source)?;
    }
    Ok(data_home.path)
}

/// The data directory there would be without the --home option, e.g. where the profiles normally are when another
/// data directory is used for screenshots, see screenshots.rs.
pub fn usual_data_home() -> anyhow::Result<PathBuf> {
    Ok(resolve(None, std::env::var_os(HOME_ENV_VAR), dirs::home_dir())?.path)
}

/// Where the data directory was set, for display, e.g. "the MARCO_SPARKO_HOME environment variable", or None if it is
/// the home directory.
pub fn data_home_source() -> Option<String> {
    DATA_HOME.get()?.as_ref().ok()?.source.clone()
}

fn resolve(option: Option<PathBuf>, env: Option<OsString>, home: Option<PathBuf>) -> anyhow::Result<DataHome> {
    if let Some(path) = option {
        return Ok(DataHome { path: std::path::absolute(path)?, source: Some(String::from("the --home option")) });
    }

    if let Some(path) = env.filter(|path| !path.is_empty()) {
        return Ok(DataHome { path: std::path::absolute(path)?, source: Some(format!("the {} environment variable", HOME_ENV_VAR)) });
    }

    let home = home.ok_or(anyhow!("Unable to locate home directory"))?;
    let pointer_file = home.join(POINTER_FILE_NAME);
    match fs::read_to_string(&pointer_file) {
        Ok(contents) => {
            let source = format!("the file {}", pointer_file.display());
            let line = contents.trim();
            // ~/ at the start is the home directory.
            let path = match line.strip_prefix("~/").or_else(|| line.strip_prefix("~\\")) {
                Some(rest) => home.join(rest),
                None => PathBuf::from(line),
            };
            if line.is_empty() || !path.is_absolute() {
                return Err(anyhow!("{} must contain the full path of the data directory, but it contains \"{}\"", source, line));
            }
            Ok(DataHome { path, source: Some(source) })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DataHome { path: home, source: None }),
        Err(error) => Err(anyhow!("Unable to read {}: {}", pointer_file.display(), error)),
    }
}

/// Creates the data directory if its parent exists, and warns if it can be read by others.
fn ensure_dir(path: &Path, source: &str) -> anyhow::Result<()> {
    if !path.is_dir() {
        if path.parent().is_some_and(Path::is_dir) {
            crate::private_file::create_private_dir(path)?;
        }
        else {
            return Err(anyhow!(
                "The data directory {} (set by {}) does not exist. If it is on a drive which is not connected, connect it and restart Marco Sparko.",
                path.display(), source));
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(path)?.permissions().mode() & 0o077 != 0 {
            crate::log_warning!("The data directory {} can be accessed by other users, it should only be accessible by its owner (chmod 700)", path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{POINTER_FILE_NAME, resolve};

    #[test]
    fn the_option_then_the_environment_then_the_pointer_file_then_the_home_directory_is_used() {
        let home = std::env::temp_dir().join(format!("marco-sparko-home-test-{}", std::process::id()));
        fs::create_dir_all(&home).unwrap();
        // Full paths on every platform, Windows ones need a drive.
        let option = home.join("option");
        let env = home.join("env");
        let pointer = home.join("pointer");

        let path = |option: Option<PathBuf>, env: Option<PathBuf>| resolve(option, env.map(Into::into), Some(home.clone())).map(|data_home| data_home.path);

        assert_eq!(path(None, None).unwrap(), home);

        fs::write(home.join(POINTER_FILE_NAME), format!("  {}\n", pointer.display())).unwrap();
        assert_eq!(path(None, None).unwrap(), pointer);
        assert_eq!(path(None, Some(env.clone())).unwrap(), env);
        assert_eq!(path(Some(option.clone()), Some(env)).unwrap(), option);

        fs::write(home.join(POINTER_FILE_NAME), "~/elsewhere").unwrap();
        assert_eq!(path(None, None).unwrap(), home.join("elsewhere"));

        fs::write(home.join(POINTER_FILE_NAME), "relative").unwrap();
        assert!(path(None, None).is_err());

        fs::remove_dir_all(&home).unwrap();
    }
}
