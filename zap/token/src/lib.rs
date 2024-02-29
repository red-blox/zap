// This crate exists to resolve a cyclic dependency.
// Report needs access to tokens to implement errors,
// and the lexer needs access to reports to report errors.

use std::fmt::Display;

use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Logos)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"--\[\[[^(\]\])]*\]\]")]
#[logos(skip r"--[^\n]*")]
pub enum Token<'a> {
	#[regex(r"[_a-zA-Z][_a-zA-Z0-9]*")]
	Word(&'a str),

	#[regex(r"-?[0-9]+(\.[0-9]+)?")]
	Number(&'a str),

	#[regex(r#""[^"]*""#)]
	#[regex(r"'[^']*'")]
	String(&'a str),

	#[token("{")]
	OpenBrace,

	#[token("}")]
	CloseBrace,

	#[token("[")]
	OpenBracket,

	#[token("]")]
	CloseBracket,

	#[token("(")]
	OpenParen,

	#[token(")")]
	CloseParen,

	#[token(",")]
	Comma,

	#[token(":")]
	Colon,

	#[token("=")]
	Equal,

	#[token(";")]
	Semicolon,

	#[token("?")]
	Question,

	#[token(".")]
	Dot,

	#[token("->")]
	Arrow,

	#[token("..")]
	DotDot,
}

impl<'a> Token<'a> {
	pub fn lexer(input: &'a str) -> impl Iterator<Item = (Result<Token, ()>, logos::Span)> + 'a {
		Logos::lexer(input).spanned()
	}
}

impl<'a> Display for Token<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Word(word) => write!(f, "{word}"),
			Self::Number(number) => write!(f, "{number}"),
			Self::String(string) => write!(f, "{string}"),

			Self::OpenBrace => write!(f, "{{"),
			Self::CloseBrace => write!(f, "}}"),

			Self::OpenBracket => write!(f, "["),
			Self::CloseBracket => write!(f, "]"),

			Self::OpenParen => write!(f, "("),
			Self::CloseParen => write!(f, ")"),

			Self::Comma => write!(f, ","),
			Self::Colon => write!(f, ":"),
			Self::Equal => write!(f, "="),
			Self::Semicolon => write!(f, ";"),
			Self::Question => write!(f, "?"),
			Self::Dot => write!(f, "."),
			Self::Arrow => write!(f, "->"),
			Self::DotDot => write!(f, ".."),
		}
	}
}
