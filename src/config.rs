use std::{fs, path::PathBuf};

pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
	//same folder as exe takes precedence, if exists
	let mut exe_path = std::env::current_exe()?;
	exe_path.pop(); //remove .exe file
	exe_path.push("config.toml");
	if exe_path.exists() {
		return Ok(exe_path);
	}

	let app_name = env!("CARGO_PKG_NAME");
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

pub fn load_config_toml<T>() -> Result<T, Box<dyn std::error::Error>>
where
    T: serde_core::de::DeserializeOwned,
{
	let config_file_path = get_config_path()?;
	let toml_content: String = fs::read_to_string(config_file_path)?;
    Ok(toml::from_str(&toml_content)?)
}
