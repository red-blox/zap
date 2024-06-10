use logos::Logos;

#[derive(Logos, Clone, Copy)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"--[^\n]*")]
#[logos(skip r"--\[\[[^(\]\])]*\]\]")]
pub enum Atom {
	#[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
	Word,

	#[regex(r"-?[0-9]+(\.[0-9]+)?")]
	Number,

	#[regex(r#""[^"]*""#)]
	#[regex(r"'[^']*'")]
	String,

	#[token("<")]
	OpenAngle,

	#[token(">")]
	CloseAngle,

	#[token("{")]
	OpenBrace,

	#[token("}")]
	CloseBrace,

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

pub fn lex(code: &str) -> impl Iterator<Item = (Result<Atom, ()>, logos::Span)> + '_ {
	Atom::lexer(code).spanned()
}
