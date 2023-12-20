mod parser;
mod util;

use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Unable to parse config file: {0}")]
	ParseError(String),
	#[error("File System error: {0}")]
	FSError(#[from] std::io::Error),
	#[error("Unknown type referenced: `{0}`")]
	UnknownTypeRef(String),
	#[error("Duplicate type declared: `{0}`")]
	DuplicateType(String),
	#[error("Duplicate event declared: `{0}`")]
	DuplicateEvent(String),
}

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
pub fn run(config: &str) -> Result<Code, Error> {
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
