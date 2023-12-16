use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use zap::run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(default_value = "net.zap")]
	config: Option<PathBuf>,

	#[arg(short, long, default_value = "network")]
	output: Option<PathBuf>,
}

fn main() -> Result<()> {
	let args = Args::parse();

	let config_path = args.config.unwrap();
	let output_dir_path = args.output.unwrap();

	let config = std::fs::read_to_string(config_path)?;

	let code = run(config.as_str())?;

	std::fs::create_dir_all(&output_dir_path)?;
	std::fs::write(output_dir_path.join("server.luau"), code.server)?;
	std::fs::write(output_dir_path.join("client.luau"), code.client)?;

	Ok(())
}
