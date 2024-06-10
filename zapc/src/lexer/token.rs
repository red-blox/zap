use lasso::{Resolver, Spur};

#[derive(Clone, Copy, PartialEq)]
pub enum Delim {
	Angle,
	Brace,
	Paren,
}

impl Delim {
	pub fn open(&self) -> char {
		match self {
			Delim::Angle => '<',
			Delim::Brace => '{',
			Delim::Paren => '(',
		}
	}

	pub fn close(&self) -> char {
		match self {
			Delim::Angle => '>',
			Delim::Brace => '}',
			Delim::Paren => ')',
		}
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Symbol {
	Arrow,
	Comma,
	Colon,
	DotDot,
	Dot,
	Equal,
	Semicolon,
	Question,
}

impl Symbol {
	pub fn text(&self) -> &'static str {
		match self {
			Symbol::Arrow => "->",
			Symbol::Comma => ",",
			Symbol::Colon => ":",
			Symbol::DotDot => "..",
			Symbol::Dot => ".",
			Symbol::Equal => "=",
			Symbol::Semicolon => ";",
			Symbol::Question => "?",
		}
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Token {
	Word(Spur),
	Number(f64),
	String(Spur),

	Open(Delim),
	Close(Delim),

	Symbol(Symbol),

	Error(char),
	Eof,
}

impl Token {
	pub fn error_text(&self, rodeo: &impl Resolver) -> String {
		match self {
			Self::Word(spur) => {
				let text = rodeo.resolve(spur);

				if text.is_empty() {
					"Word".to_string()
				} else {
					format!("`{text}`")
				}
			}

			Self::Number(_) => "Number".to_string(),
			Self::String(_) => "String".to_string(),

			Self::Open(delim) => format!("`{}`", delim.open()),
			Self::Close(delim) => format!("`{}`", delim.close()),

			Self::Symbol(symbol) => format!("`{}`", symbol.text()),

			Self::Error(c) => format!("`{}`", c),
			Self::Eof => "Eof".to_string(),
		}
	}
}
