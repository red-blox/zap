use std::{fs, path::PathBuf};

use zapc_meta::FileDatabase;

pub fn run(project_path: PathBuf) {
	let mut files = FileDatabase::new();
	add_files(&mut files, project_path);
}

fn add_files(files: &mut FileDatabase, path: PathBuf) {
	if path.is_dir() {
		for entry in fs::read_dir(path).expect("Failed to read directory") {
			let entry = entry.expect("Failed to read entry");
			add_files(files, entry.path());
		}
	} else if path.extension().map_or(false, |ext| ext == "zap") {
		files.add(
			path.display().to_string(),
			fs::read_to_string(path).expect("Failed to read file"),
		);
	}
}
