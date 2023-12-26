mod config;
mod irgen;
mod output;
mod parser;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use codespan_reporting::diagnostic::Diagnostic;
#[cfg(target_arch = "wasm32")]
use codespan_reporting::{
	files::SimpleFile,
	term::{self, termcolor::NoColor},
};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
pub struct Output {
	pub path: Option<PathBuf>,
	pub code: String,
	pub defs: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct Output {
	pub code: String,
	pub defs: Option<String>,
}

#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone), derive(Clone))]
pub struct Code {
	pub server: Output,
	pub client: Output,
}

#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
pub struct Return {
	pub code: Option<Code>,
	pub diagnostics: Vec<Diagnostic<()>>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
#[wasm_bindgen(getter_with_clone)]
pub struct Return {
	pub code: Option<Code>,
	pub diagnostics: String,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run(input: &str) -> Return {
	let (config, diagnostics) = parser::parse(input);

	if let Some(config) = config {
		Return {
			code: Some(Code {
				server: Output {
					path: config.server_output.map(|p| p.into()),
					code: output::luau::server::code(&config),
					defs: output::typescript::server::code(&config),
				},
				client: Output {
					path: config.client_output.map(|p| p.into()),
					code: output::luau::client::code(&config),
					defs: output::typescript::client::code(&config),
				},
			}),
			diagnostics,
		}
	} else {
		Return {
			code: None,
			diagnostics,
		}
	}
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run(input: &str) -> Return {
	let (config, diagnostics) = parser::parse(input);

	let mut writer = NoColor::new(Vec::new());

	let file = SimpleFile::new("input.zap", input);
	let term_config = term::Config::default();

	for diagnostic in diagnostics {
		term::emit(&mut writer, &term_config, &file, &diagnostic).unwrap();
	}

	let diagnostics = String::from_utf8(writer.into_inner()).unwrap();

	if let Some(config) = config {
		Return {
			code: Some(Code {
				server: Output {
					code: output::luau::server::code(&config),
					defs: output::typescript::server::code(&config),
				},
				client: Output {
					code: output::luau::client::code(&config),
					defs: output::typescript::client::code(&config),
				},
			}),
			diagnostics,
		}
	} else {
		Return {
			code: None,
			diagnostics,
		}
	}
}
