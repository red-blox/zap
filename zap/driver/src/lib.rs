use meta::{write_stdout, FileDatabase};
use std::{fs::read_to_string, path::PathBuf};

pub fn main(root_file_path: PathBuf, _opts: Option<String>) {
	let mut file_db = FileDatabase::new();

	let root_file_id = file_db.add(
		root_file_path.clone().into_os_string().to_string_lossy().into_owned(),
		read_to_string(&root_file_path).expect("Failed to read file"),
	);

	let lexer_result = lexer::lex(root_file_id, file_db.get(root_file_id).unwrap().content());

	if let Err(reports) = lexer_result {
		write_stdout(&file_db, &reports);
		return;
	}

	let (tokens, eoi_span) = lexer_result.unwrap();
	let ast_result = ast_builder::build(eoi_span, tokens);

	if let Err(reports) = ast_result {
		write_stdout(&file_db, &reports);
		return;
	}

	let ast = ast_result.unwrap();
	let reports = ast_analyzer::analyze(&ast);

	if !reports.is_empty() {
		write_stdout(&file_db, &reports);
	} else {
		println!("No errors found");
	}
}
