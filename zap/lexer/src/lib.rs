// This file contains the lexer for zap.
// The lexer is implemented using `logos` lexer generator.
//
// Please note that zap doesn't have *any* keywords and instead
// relies on the parser to determine the meaning of `Word` tokens.

use meta::{FileId, Report, Span};
use token::Token;

#[allow(clippy::type_complexity)]
pub fn lex(file: FileId, input: &str) -> Result<(Vec<(Token, Span)>, Span), Vec<Report>> {
	let mut tokens = Vec::new();
	let mut reports = Vec::new();

	for (result, span) in Token::lexer(input) {
		let span = Span::new(file, span);

		match result {
			Ok(token) => tokens.push((token, span)),

			Err(_) => reports.push(Report::LexerUnexpectedCharacter {
				c: input[span.start()..span.end()].chars().next().unwrap(),
				span,
			}),
		}
	}

	if reports.is_empty() {
		Ok((tokens, Span::new(file, input.len()..input.len())))
	} else {
		Err(reports)
	}
}

#[cfg(test)]
mod test {
	use std::ops::Range;

	use super::*;

	const FILE: FileId = 0;

	#[allow(clippy::type_complexity)]
	fn lex(input: &str) -> Result<(Vec<(Token, Span)>, Span), Vec<Report>> {
		super::lex(FILE, input)
	}

	fn span(range: Range<usize>) -> Span {
		Span::new(FILE, range)
	}

	#[test]
	fn empty() {
		let input = "";

		assert_eq!(lex(input), Ok((vec![], span(0..0))));
	}

	#[test]
	fn whitespace() {
		let input = " \t\r\n";

		assert_eq!(lex(input), Ok((vec![], span(4..4))));
	}

	#[test]
	fn comment() {
		let input = "word-- line comment
		another_word
		--[[
			block comment
		]]";

		assert_eq!(
			lex(input),
			Ok((
				vec![
					(Token::Word("word"), span(3..7)),
					(Token::Word("another_word"), span(25..37))
				],
				span(66..66)
			))
		);
	}

	#[test]
	fn word() {
		let input = "word another_word";

		assert_eq!(
			lex(input),
			Ok((
				vec![
					(Token::Word("word"), span(0..4)),
					(Token::Word("another_word"), span(5..17))
				],
				span(17..17)
			))
		);
	}

	#[test]
	fn number() {
		let input = "123 456.789";

		assert_eq!(
			lex(input),
			Ok((
				vec![
					(Token::Number("123"), span(0..3)),
					(Token::Number("456.789"), span(4..11))
				],
				span(11..11)
			))
		);
	}

	#[test]
	fn string() {
		let input = "\"hello\" 'world'";

		assert_eq!(
			lex(input),
			Ok((
				vec![
					(Token::String("\"hello\""), span(0..7)),
					(Token::String("'world'"), span(8..15))
				],
				span(15..15)
			))
		);
	}

	#[test]
	fn symbols() {
		let input = "{[()]}:;=,.->..";

		assert_eq!(
			lex(input),
			Ok((
				vec![
					(Token::OpenBrace, span(0..1)),
					(Token::OpenBracket, span(1..2)),
					(Token::OpenParen, span(2..3)),
					(Token::CloseParen, span(3..4)),
					(Token::CloseBracket, span(4..5)),
					(Token::CloseBrace, span(5..6)),
					(Token::Colon, span(6..7)),
					(Token::Semicolon, span(7..8)),
					(Token::Equal, span(8..9)),
					(Token::Comma, span(9..10)),
					(Token::Dot, span(10..11)),
					(Token::Arrow, span(11..13)),
					(Token::DotDot, span(13..15))
				],
				span(15..15)
			))
		);
	}

	#[test]
	fn unexpected_character() {
		let input = "word $";

		assert_eq!(
			lex(input),
			Err(vec![Report::LexerUnexpectedCharacter {
				c: '$',
				span: span(5..6)
			}])
		);
	}

	#[test]
	fn multiple_errors() {
		let input = "word $ another_word $";

		assert_eq!(
			lex(input),
			Err(vec![
				Report::LexerUnexpectedCharacter {
					c: '$',
					span: span(5..6)
				},
				Report::LexerUnexpectedCharacter {
					c: '$',
					span: span(20..21)
				}
			])
		);
	}
}
