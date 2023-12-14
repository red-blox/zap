use std::path::PathBuf;

use clap::Parser;
use lib::run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(default_value = "net.zap")]
	config: Option<PathBuf>,

	#[arg(short, long, default_value = "network")]
	output: Option<PathBuf>,
}

fn main() {
	let args = Args::parse();

	let config_path = args.config.unwrap();
	let output_dir_path = args.output.unwrap();

	let config = match std::fs::read_to_string(&config_path) {
		Ok(config) => config,
		Err(err) => {
			eprintln!("Failed to read config file: {}", err);
			return;
		}
	};

	match run(config.as_str()) {
		Ok(code) => {
			std::fs::create_dir_all(&output_dir_path).expect("Failed to create output directory!");

			std::fs::write(output_dir_path.join("server.luau"), code.server).expect("Failed to write server code!");
			std::fs::write(output_dir_path.join("client.luau"), code.client).expect("Failed to write client code!");
		}

		Err(err) => eprintln!("{}", err),
	}
}
