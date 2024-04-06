mod atom;
mod token;

use std::ops::Range;

use atom::{lex, Atom};
use lasso::Rodeo;
pub use token::*;
use zapc_meta::{FileId, Span};

fn atom_to_token(code: &str, atom: Atom, range: Range<usize>, rodeo: &mut Rodeo) -> Token {
	match atom {
		Atom::Word => Token::Word(rodeo.get_or_intern(&code[range])),
		Atom::Number => Token::Number(code[range].parse().unwrap()),

		// do not include the quotes because it makes it annoying to compare equality with words
		Atom::String => Token::String(rodeo.get_or_intern(&code[range.start + 1..range.end - 1])),

		Atom::OpenAngle => Token::Open(Delim::Angle),
		Atom::CloseAngle => Token::Close(Delim::Angle),

		Atom::OpenBrace => Token::Open(Delim::Brace),
		Atom::CloseBrace => Token::Close(Delim::Brace),

		Atom::OpenParen => Token::Open(Delim::Paren),
		Atom::CloseParen => Token::Close(Delim::Paren),

		Atom::Arrow => Token::Symbol(Symbol::Arrow),
		Atom::Comma => Token::Symbol(Symbol::Comma),
		Atom::Colon => Token::Symbol(Symbol::Colon),
		Atom::DotDot => Token::Symbol(Symbol::DotDot),
		Atom::Dot => Token::Symbol(Symbol::Dot),
		Atom::Equal => Token::Symbol(Symbol::Equal),
		Atom::Semicolon => Token::Symbol(Symbol::Semicolon),
		Atom::Question => Token::Symbol(Symbol::Question),
	}
}

pub fn tokenize<'a>(file: FileId, code: &'a str, rodeo: &'a mut Rodeo) -> impl Iterator<Item = (Token, Span)> + 'a {
	lex(code)
		.map(move |(result, range)| {
			let span = Span::from_range(file, range.clone());
			let token = match result {
				Ok(atom) => atom_to_token(code, atom, range, rodeo),
				Err(_) => Token::Error(code[range].chars().next().unwrap()),
			};

			(token, span)
		})
		.chain(std::iter::once((
			Token::Eof,
			Span::from_range(file, code.len()..code.len()),
		)))
}
