use std::{fs, path::PathBuf};

use full_moon::LuaVersion;
use selene_lib::{standard_library::StandardLibrary, Checker};

pub struct Selene {
	pub linter: Checker<toml::Value>,
	pub lua_version: LuaVersion,
}

fn resolve_standard_library_bases(base_name: &str) -> Vec<StandardLibrary> {
	let mut resolved_libs: Vec<StandardLibrary> = vec![];

	if let Some(base_library) = StandardLibrary::from_name(base_name) {
		let base = base_library.base.clone();

		resolved_libs.push(base_library);

		if let Some(next_base) = base {
			resolved_libs.append(&mut resolve_standard_library_bases(&next_base))
		}
	}

	resolved_libs
}

pub fn initialise_selene() -> Selene {
	let working_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

	let standard_library_path = working_dir.join("polyfill.yml");
	let standard_library_file = fs::read_to_string(standard_library_path).expect("Unable to read the standard library");
	let mut standard_library: StandardLibrary =
		serde_yml::from_str(&standard_library_file).expect("Unable to parse the standard library");

	if let Some(base_name) = &standard_library.base {
		let base_libs = resolve_standard_library_bases(base_name);
		for base in base_libs {
			standard_library.extend(base);
		}
	}

	let config_path = working_dir.join("selene.toml");
	let config_file = fs::read_to_string(config_path).expect("Unable to read selene.toml");
	let config = toml::from_str(&config_file).expect("Unable to decode selene.toml");

	let (lua_version, lua_version_errors) = standard_library.lua_version();
	assert!(lua_version_errors.is_empty());

	let linter = Checker::new(config, standard_library).expect("Unable to initialise selene!");

	Selene { linter, lua_version }
}
