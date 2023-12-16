mod irgen;
mod output;
mod parser;
mod util;

use thiserror::Error;
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_wasm(config: &str) -> Result<Code, JsError> {
	Ok(run(config)?)
}
