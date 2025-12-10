use std::path::{Component, Path, PathBuf};

pub fn format_bytes(bytes:u64) -> String {
	if bytes < 1_024 {
		return format!("{}B", bytes)
	}
	else if bytes < 1_048_576 {
		return format!("{:.1}KB", bytes as f64 / 1024.0)
	}
	else if bytes < 1_073_741_824 {
		return format!("{:.1}MB", bytes as f64 / 1_048_576.0)
	}
	else if bytes < 1_099_511_627_776 {
		return format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
	}
	else {
		return format!("{:.1}TB", bytes as f64 / 1_099_511_627_776.0)
	}
}

/// Takes a path and a base path from Windows or Linux, and outputs a path relative to the base path
/// using "/" as the seperator irrespective of the OS
pub fn path_to_agnostic_relative(path: &Path, base: &Path) -> String {
	// println!("path {:?}", path);
	// println!("base {:?}", base);
	let path_components = path.components();
	let base_components: Vec<Component> = base.components().collect();
	let mut rtn = String::new();
	for (icomp, path_component) in path_components.enumerate() {
		// println!("{:?}", path_component);
		if icomp >= base_components.len() {
			let mut sep = "";
			if !rtn.is_empty() {
				sep = "/";
			}
			match path_component {
				Component::Normal(c) => {
					rtn.push_str(&format!("{}{}", sep, c.to_string_lossy()));
				},
				_ => {},
			}
		}


	}

	
	return rtn;
}


/// a function to append an extension
/// as at writing this, `PathBuf::add_extension` fn is blocked as unstable
/// 	https://github.com/rust-lang/rust/issues/127292
pub fn add_extension(path:&Path, extension:&str) -> PathBuf {
	let mut out_pathbuf = PathBuf::new();
	// get components
	let path_components = path.components();
	let num_components = path_components.clone().count();
	for (icomponent, component) in path_components.enumerate() {
		if icomponent == num_components - 1 {
			let mut component_str = component.as_os_str().to_string_lossy().to_string();
			component_str.push('.');
			component_str.push_str(extension);
			out_pathbuf.push(component_str);
		} else {
			out_pathbuf.push(component);
		}
	}
	
	out_pathbuf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
	#[test]
    fn test_path_to_agnostic_relative_windows() {
        let base: &Path = Path::new(r"C:\Users\hrag");
        let path: &Path = Path::new(r"C:\Users\hrag\five\eight\six.txt");
        assert_eq!(path_to_agnostic_relative(path.parent().unwrap(), base), "five/eight");
    }
	
	#[cfg(target_os = "linux")]
    #[test]
    fn test_path_to_agnostic_relative_linux() {
        let base: &Path = Path::new("/home/ray");
        let path: &Path = Path::new("/home/ray/five/eight/six.txt");
        assert_eq!(path_to_agnostic_relative(path.parent().unwrap(), base), "five/eight");
    }

    #[test]
    fn test_add_extension() {
        let path = Path::new("/home/ray/five/eight/six.txt");
		let extension = "newext";
		let expected = PathBuf::from("/home/ray/five/eight/six.txt.newext");
		let result = add_extension(path, extension);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_extension_from_none() {
		//note this can also be done using the built-in PathBuf::set_extension
        let path = Path::new("/home/ray/five/eight/six");
		let extension = "newext";
		let expected = PathBuf::from("/home/ray/five/eight/six.newext");
		let result = add_extension(path, extension);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_bytes_kb() {
		let expected = String::from("976.6KB");
		let result = format_bytes(1_000_000);
        assert_eq!(result, expected);
    }
    #[test]
    fn test_format_bytes_mb() {
		let expected = String::from("953.7MB");
		let result = format_bytes(1_000_000_000);
        assert_eq!(result, expected);
    }
}
