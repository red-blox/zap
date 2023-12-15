mod irgen;
mod output;
mod parser;
mod util;

pub struct Code {
	pub server: String,
	pub client: String,
}

pub fn run(config: &str) -> Result<Code, String> {
	let file = parser::parse(config)?;

	/*
	Ok(Code {
		server: output::luau::server(&file),
		client: output::luau::client(&file),
	})
	*/

	todo!()
}