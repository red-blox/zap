pub mod config;
pub mod irgen;
pub mod output;
pub mod parser;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use codespan_reporting::diagnostic::{Diagnostic, Severity};
#[cfg(target_arch = "wasm32")]
use codespan_reporting::{
	diagnostic::Severity,
	files::SimpleFile,
	term::{self, termcolor::Buffer},
};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
pub struct Output {
	pub path: PathBuf,
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
	pub types: Option<Output>,
	pub tooling: Option<Output>,
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
pub fn run(input: &str, no_warnings: bool) -> Return {
	let (config, reports) = parser::parse(input);
	let diagnostics = reports
		.into_iter()
		.map(|report| report.to_diagnostic(no_warnings))
		.collect::<Vec<Diagnostic<()>>>();

	if !diagnostics.iter().any(|diag| diag.severity == Severity::Error)
		&& let Some(config) = config
	{
		return Return {
			code: Some(Code {
				server: Output {
					path: config.server_output.into(),
					code: output::luau::server::code(&config),
					defs: output::typescript::server::code(&config),
				},
				client: Output {
					path: config.client_output.into(),
					code: output::luau::client::code(&config),
					defs: output::typescript::client::code(&config),
				},
				types: config.types_output.map(|types_output| Output {
					path: types_output.into(),
					code: output::luau::types::code(&config),
					defs: output::typescript::types::code(&config),
				}),
				tooling: output::tooling::code(&config),
			}),
			diagnostics,
		};
	}

	Return {
		code: None,
		diagnostics,
	}
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run(input: &str, no_warnings: bool, ansi_output_supported: bool) -> Return {
	let (config, reports) = parser::parse(input);

	let mut writer = if ansi_output_supported {
		Buffer::ansi()
	} else {
		Buffer::no_color()
	};

	let file = SimpleFile::new("input.zap", input);
	let term_config = term::Config::default();

	let mut no_errors = true;

	for report in reports {
		let diagnostic = report.to_diagnostic(no_warnings);

		if diagnostic.severity == Severity::Error {
			no_errors = false;
		}

		term::emit(&mut writer, &term_config, &file, &diagnostic).unwrap();
	}

	let diagnostics = String::from_utf8(writer.into_inner()).unwrap();

	if no_errors {
		if let Some(config) = config {
			return Return {
				code: Some(Code {
					server: Output {
						code: output::luau::server::code(&config),
						defs: output::typescript::server::code(&config),
					},
					client: Output {
						code: output::luau::client::code(&config),
						defs: output::typescript::client::code(&config),
					},
					types: config.types_output.map(|_| Output {
						code: output::luau::types::code(&config),
						defs: output::typescript::types::code(&config),
					}),
					tooling: output::tooling::code(&config),
				}),
				diagnostics,
			};
		}
	}

	Return {
		code: None,
		diagnostics,
	}
}
