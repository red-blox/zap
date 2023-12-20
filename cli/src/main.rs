use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
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

	let config = std::fs::read_to_string(config_path)?;

	let code = run(config.as_str())?;

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

	Ok(())
}
