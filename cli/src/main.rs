use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use codespan_reporting::{
	diagnostic::Severity,
	files::SimpleFile,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use zap::run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(default_value = "net.zap")]
	config: Option<PathBuf>,
	/// Convert all warnings to errors
	#[clap(long)]
	no_warnings: bool,
}

fn main() -> Result<()> {
	let args = Args::parse();

	let config_path = args.config.unwrap();

	let config = std::fs::read_to_string(&config_path)?;

	let ret = run(config.as_str(), args.no_warnings);

	let code = ret.code;
	let diagnostics = ret.diagnostics;

	if let Some(code) = code {
		let server_path = config_path.parent().unwrap().join(code.server.path);
		let client_path = config_path.parent().unwrap().join(code.client.path);

		if let Some(parent) = server_path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		if let Some(parent) = client_path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		std::fs::write(server_path.clone(), code.server.code)?;
		std::fs::write(client_path.clone(), code.client.code)?;

		if let Some(defs) = code.server.defs {
			let file_path = if server_path.file_stem().unwrap() == "init" {
				server_path.with_file_name("index.d.ts")
			} else {
				server_path.with_extension("d.ts")
			};

			std::fs::write(file_path, defs)?;
		}

		if let Some(defs) = code.client.defs {
			let file_path = if client_path.file_stem().unwrap() == "init" {
				client_path.with_file_name("index.d.ts")
			} else {
				client_path.with_extension("d.ts")
			};

			std::fs::write(file_path, defs)?;
		}

		if let Some(types_output) = code.types {
			let types_path = config_path.parent().unwrap().join(types_output.path);

			if let Some(parent) = types_path.parent() {
				std::fs::create_dir_all(parent)?;
			}

			if let Some(defs) = types_output.defs {
				let defs_path = if types_path.file_stem().unwrap() == "init" {
					types_path.with_file_name("index.d.ts")
				} else {
					types_path.with_extension("d.ts")
				};

				std::fs::write(defs_path, defs)?;
			}

			std::fs::write(types_path, types_output.code)?;
		}

		if let Some(tooling) = code.tooling {
			let tooling_path = config_path.parent().unwrap().join(tooling.path);

			if let Some(parent) = tooling_path.parent() {
				std::fs::create_dir_all(parent)?;
			}

			std::fs::write(tooling_path, tooling.code)?;
		}
	}

	if diagnostics.is_empty() {
		return Ok(());
	}

	let file = SimpleFile::new(config_path.to_str().unwrap(), config);

	let writer = StandardStream::stderr(ColorChoice::Auto);
	let config_term = codespan_reporting::term::Config::default();

	for diagnostic in &diagnostics {
		term::emit(&mut writer.lock(), &config_term, &file, diagnostic)?;
	}

	if diagnostics.iter().any(|diag| diag.severity == Severity::Error) {
		std::process::exit(1)
	}

	Ok(())
}
