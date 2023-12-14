use crate::parser::Casing;

pub mod luau;

pub struct Output {
	base: String,
	lines: Vec<String>,
	tabs: usize,
}

impl Output {
	pub fn new(base: String) -> Output {
		Output {
			base,
			lines: Vec::new(),
			tabs: 0,
		}
	}

	pub fn line(&mut self, line: String) -> &mut Self {
		self.lines.push(format!("{}{}", "\t".repeat(self.tabs), line));
		self
	}

	pub fn tab(&mut self) -> &mut Self {
		self.tabs += 1;
		self
	}

	pub fn untab(&mut self) -> &mut Self {
		self.tabs -= 1;
		self
	}

	pub fn get(self) -> String {
		format!("{}{}", self.base, self.lines.join("\n"))
	}
}

pub fn casing(casing: Casing, pascal: &'static str, camel: &'static str, snake: &'static str) -> &'static str {
	match casing {
		Casing::Pascal => pascal,
		Casing::Camel => camel,
		Casing::Snake => snake,
	}
}

#[macro_export]
macro_rules! line {
	($output:expr, $($arg:tt)*) => {
		$output.line(format!($($arg)*))
	};
}
