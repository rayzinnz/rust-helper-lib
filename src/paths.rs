use std::path::{Component, Path};

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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_agnostic_relative_windows() {
        let base: &Path = Path::new(r"C:\Users\hrag");
        let path: &Path = Path::new(r"C:\Users\hrag\five\eight\six.txt");
        assert_eq!(path_to_agnostic_relative(path.parent().unwrap(), base), "five/eight");
    }
    #[test]
    fn test_path_to_agnostic_relative_linux() {
        let base: &Path = Path::new(r"/home/ray");
        let path: &Path = Path::new(r"/home/ray/five/eight/six.txt");
        assert_eq!(path_to_agnostic_relative(path.parent().unwrap(), base), "five/eight");
    }
}
