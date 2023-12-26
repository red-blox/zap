use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use codespan_reporting::{
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
}

fn main() -> Result<()> {
	let args = Args::parse();

	let config_path = args.config.unwrap();

	let config = std::fs::read_to_string(&config_path)?;

	let (code, diagnostics) = run(config.as_str());

	if let Some(code) = code {
		if let Some(definitions) = code.server.definitions {
			let mut path = code.server.path.clone();
			path.set_extension("d.ts");

			std::fs::write(path, definitions)?
		}

		if let Some(definitions) = code.client.definitions {
			let mut path = code.client.path.clone();
			path.set_extension("d.ts");

			std::fs::write(path, definitions)?
		}

		std::fs::write(code.server.path, code.server.contents)?;
		std::fs::write(code.client.path, code.client.contents)?;
	}

	if diagnostics.is_empty() {
		return Ok(());
	}

	let file = SimpleFile::new(config_path.to_str().unwrap(), config);

	let writer = StandardStream::stderr(ColorChoice::Always);
	let config_term = codespan_reporting::term::Config::default();

	for diagnostic in diagnostics {
		term::emit(&mut writer.lock(), &config_term, &file, &diagnostic)?;
	}

	todo!()
}
