mod config;
mod irgen;
mod output;
mod parser;

use codespan_reporting::diagnostic::Diagnostic;

use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct Output {
	pub path: Option<PathBuf>,
	pub code: String,
	pub defs: Option<String>,
}

#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct Code {
	pub server: Output,
	pub client: Output,
}

#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct Return {
	pub code: Option<Code>,
	pub diagnostics: Vec<Diagnostic<()>>,
}

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
