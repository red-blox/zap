use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	UnknownToken(String),

	Word(String),

	// Literals
	NumLit(f64),
	StrLit(String),

	// Symbols
	ParenOpen,
	ParenClose,
	BraceOpen,
	BraceClose,
	BracketOpen,
	BracketClose,
	AngleOpen,
	AngleClose,
	Comma,
	Colon,
	Equals,
}

pub fn lex(input: &str) -> Vec<Token> {
	let mut tokens = Vec::new();
	let mut chars = input.chars().peekable();

	let mut unknown_buf = String::new();

	while let Some(c) = chars.next() {
		match c {
			c if c.is_whitespace() => continue,

			c if c == '-' && chars.peek() == Some(&'-') => {
				chars.next();

				if chars.next() == Some('[') && chars.peek() == Some(&'[') {
					chars.next();

					while let Some(c) = chars.next() {
						if c == ']' && chars.peek() == Some(&']') {
							chars.next();

							break;
						}
					}
				} else {
					for c in chars.by_ref() {
						if c == '\n' {
							break;
						}
					}
				}
			}

			'(' => tokens.push(Token::ParenOpen),
			')' => tokens.push(Token::ParenClose),
			'{' => tokens.push(Token::BraceOpen),
			'}' => tokens.push(Token::BraceClose),
			'[' => tokens.push(Token::BracketOpen),
			']' => tokens.push(Token::BracketClose),
			'<' => tokens.push(Token::AngleOpen),
			'>' => tokens.push(Token::AngleClose),
			',' => tokens.push(Token::Comma),
			':' => tokens.push(Token::Colon),
			'=' => tokens.push(Token::Equals),

			'0'..='9' => {
				let mut had_dot = false;
				let mut num = String::new();
				num.push(c);

				while let Some(&c) = chars.peek() {
					if c.is_numeric() {
						num.push(chars.next().unwrap());
					} else if c == '.' && !had_dot {
						had_dot = true;
						num.push(chars.next().unwrap());
					} else {
						break;
					}
				}

				tokens.push(Token::NumLit(num.parse().unwrap()));
			}

			'"' => {
				let mut string = String::new();

				while let Some(&c) = chars.peek() {
					if c == '"' {
						chars.next();
						break;
					} else {
						string.push(chars.next().unwrap());
					}
				}

				tokens.push(Token::StrLit(string));
			}

			'_' | 'a'..='z' | 'A'..='Z' => {
				let mut word = String::new();
				word.push(c);

				while let Some(&c) = chars.peek() {
					if c == '_' || c.is_alphanumeric() {
						word.push(chars.next().unwrap());
					} else {
						break;
					}
				}

				tokens.push(Token::Word(word));
			}

			_ => {
				unknown_buf.push(c);
				continue;
			}
		}

		if !unknown_buf.is_empty() {
			tokens.push(Token::UnknownToken(unknown_buf.clone()));
			unknown_buf.clear();
		}
	}

	tokens
}
