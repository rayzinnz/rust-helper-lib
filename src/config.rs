use std::{fs, path::PathBuf};

/// gets a full path to the config.toml. e.g.
/// ```rust
/// let config_path = get_config_path(app_name!());
/// ```
/// app_name! is a macro to get the app_name of the calling application
pub fn get_config_path(app_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
	//same folder as exe takes precedence, if exists
	let mut exe_path = std::env::current_exe()?;
	exe_path.pop(); //remove .exe file
	exe_path.push("config.toml");
	if exe_path.exists() {
		return Ok(exe_path);
	}

	//config dir is otherwise the default
    match dirs::config_dir() {
        Some(config_dir) => {
		    let app_config_path = config_dir.join(app_name);
            fs::create_dir_all(&app_config_path)?;
            Ok(app_config_path.join("config.toml"))
        },
        None => Err("Could not resolve config dir".into())
    }
}

///loads the standard config.toml path into a struct.
/// ```rust
/// use helper_lib::{app_name, config::load_config_toml};
/// use serde::Deserialize;
/// 
/// #[derive(Debug, Deserialize)]
/// pub struct Config {
/// 	pub image_save_root: String,
/// 	pub db_path: String,
/// }
/// 
/// impl Default for Config {
///     fn default() -> Self {
///         Self { 
/// 			image_save_root: "~".to_string(),
/// 			db_path: "~/detection_events.db".to_string(),
/// 		}
///     }
/// }
/// 
/// pub fn load_config() -> Config {
/// 	load_config_toml(app_name!()).unwrap_or_default()
/// }
/// ```
pub fn load_config_toml<T>(app_name: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde_core::de::DeserializeOwned,
{
	let config_file_path = get_config_path(app_name)?;
	let toml_content: String = fs::read_to_string(config_file_path)?;
    Ok(toml::from_str(&toml_content)?)
}
