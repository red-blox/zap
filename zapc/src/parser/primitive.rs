use chumsky::primitive::custom;

use crate::{
	ast::primitive::{AstNumber, AstString, AstWord},
	lexer::{Delim, Symbol, Token},
	meta::Span,
};

use super::{error::Error, Parser, State};

pub fn word<'a>() -> impl Parser<'a, AstWord> {
	custom(|input| {
		let before = input.offset();
		let state: &mut State<'a> = input.state();
		let empty_spur = state.rodeo.get_or_intern_static("");

		match input.next() {
			Some(Token::Word(spur)) => Ok(AstWord::new(spur, input.span_since(before))),
			token => Err(Error::new(
				&[Token::Word(empty_spur)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

pub fn keyword<'a>(keyword: &'static str) -> impl Parser<'a, AstWord> {
	custom(move |input| {
		let before = input.offset();
		let state: &mut State<'a> = input.state();
		let keyword_spur = state.rodeo.get_or_intern_static(keyword);

		match input.next() {
			Some(Token::Word(spur)) if spur == keyword_spur => Ok(AstWord::new(spur, input.span_since(before))),
			token => Err(Error::new(
				&[Token::Word(keyword_spur)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

pub fn number<'a>() -> impl Parser<'a, AstNumber> {
	custom(|input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Number(number)) => Ok(AstNumber::new(number, input.span_since(before))),
			token => Err(Error::new(
				&[Token::Number(0.0)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

pub fn string<'a>() -> impl Parser<'a, AstString> {
	custom(|input| {
		let before = input.offset();
		let state: &mut State<'a> = input.state();
		let empty_spur = state.rodeo.get_or_intern_static("");

		match input.next() {
			Some(Token::String(spur)) => Ok(AstString::new(spur, input.span_since(before))),
			token => Err(Error::new(
				&[Token::String(empty_spur)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

#[allow(unused)]
pub fn delim<'a, O: Clone>(
	delim: Delim,
	parser: impl Parser<'a, O>,
	fallback: impl Fn(Span) -> O + Clone,
) -> impl Parser<'a, O> {
	let open = custom(move |input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Open(d)) if d == delim => Ok(()),
			token => Err(Error::new(
				&[Token::Open(delim)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	});

	let close = custom(move |input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Close(d)) if d == delim => Ok(()),
			token => Err(Error::new(
				&[Token::Close(delim)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	});

	// todo: recover with nested_delimiters when chumsky supports it
	parser.delimited_by(open, close)
}

pub fn symbol<'a>(symbol: Symbol) -> impl Parser<'a, ()> {
	custom(move |input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Symbol(s)) if s == symbol => Ok(()),
			token => Err(Error::new(
				&[Token::Symbol(symbol)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}
