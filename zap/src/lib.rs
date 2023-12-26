mod config;
mod irgen;
mod output;
mod parser;

use codespan_reporting::diagnostic::Diagnostic;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
pub struct Output {
	pub path: PathBuf,
	pub contents: String,
	pub definitions: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(getter_with_clone)]
pub struct Output {
	pub contents: String,
	pub definitions: Option<String>,
}

#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct Code {
	pub server: Output,
	pub client: Output,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run(input: &str) -> (Option<Code>, Vec<Diagnostic<()>>) {
	//let file = parser::parse(config)?;

	/*
	let server_contents = output::server::code(&file);
	let client_contents = output::client::code(&file);

	Ok(Code {
		server: Output {
			path: file.server_output,
			contents: server_contents,
			definitions: server_definitions,
		},
		client: Output {
			path: file.client_output,
			contents: client_contents,
			definitions: client_definitions,
		},
	})
	*/

	let (config, errors) = parser::parse(input);

	if config.is_none() {}

	todo!()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run(config: &str) -> Result<Code, JsError> {
	let file = parser::parse(config)?;

	let server_contents = output::luau::server::code(&file);
	let server_definitions = output::typescript::server::code(&file);

	let client_contents = output::luau::client::code(&file);
	let client_definitions = output::typescript::client::code(&file);

	Ok(Code {
		server: Output {
			contents: server_contents,
			definitions: server_definitions,
		},
		client: Output {
			contents: client_contents,
			definitions: client_definitions,
		},
	})
}
