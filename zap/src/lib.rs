mod irgen;
mod output;
mod parser;
mod util;

use thiserror::Error;
use wasm_bindgen::prelude::*;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Config parser error: {0}")]
	ParseError(String),
	#[error("File System error: {0}")]
	FSError(#[from] std::io::Error),
	#[error("Unknown type referenced: `{0}`")]
	UnknownTypeReference(String),
	#[error("Duplicate type referenced: `{0}`")]
	DuplicateType(String),
	#[error("Duplicate Event referenced: `{0}`")]
	DuplicateEvent(String),
}

#[wasm_bindgen(getter_with_clone)]
pub struct Code {
	pub server: String,
	pub client: String,
}

pub fn run(config: &str) -> Result<Code, Error> {
	let file = parser::parse(config)?;

	Ok(Code {
		server: output::server::code(&file),
		client: output::client::code(&file),
	})
}

#[wasm_bindgen]
pub fn run_wasm(config: &str) -> Result<Code, JsError> {
	Ok(run(config)?)
}
