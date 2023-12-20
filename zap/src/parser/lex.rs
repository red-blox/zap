use logos::{Lexer, Logos, Skip};

fn block_comment<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Skip {
	let mut remainder = lexer.remainder().char_indices().peekable();

	while let Some((_, c)) = remainder.next() {
		lexer.bump(1);

		if c == ']' && matches!(remainder.peek(), Some((_, ']'))) {
			lexer.bump(1);
			break;
		}
	}

	Skip
}

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token<'a> {
	#[regex(r"[ \t\n\f]+", logos::skip)]
	#[regex(r"--.*", logos::skip)]
	#[regex(r"--\[\[", block_comment)]
	Skipped,

	// Values
	#[regex(r"[_a-zA-Z][_a-zA-Z0-9]*", |lex| lex.slice())]
	Word(&'a str),

	#[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice())]
	NumLit(&'a str),

	#[regex(r#""[^"]*""#, |lex| lex.slice())]
	StrLit(&'a str),

	// Symbols
	#[token("(")]
	OpenParen,

	#[token(")")]
	CloseParen,

	#[token("{")]
	OpenBrace,

	#[token("}")]
	CloseBrace,

	#[token("[")]
	OpenBracket,

	#[token("]")]
	CloseBracket,

	#[token("<")]
	OpenAngle,

	#[token(">")]
	CloseAngle,

	#[token(":")]
	Colon,

	#[token(";")]
	Semicolon,

	#[token(",")]
	Comma,

	#[token("=")]
	Equals,

	#[token("..")]
	DotDot,
}
