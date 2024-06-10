use lasso::Resolver;

use crate::{
	lexer::Token,
	meta::{Report, Span},
};

use super::Input;

pub struct Error {
	expected: Vec<Token>,
	found: Token,
	label: Option<&'static str>,
	span: Span,
}

impl Error {
	pub fn new(expected: &'_ [Token], found: Token, span: Span) -> Self {
		Self {
			expected: expected.to_vec(),
			found,
			label: None,
			span,
		}
	}

	pub fn into_report(self, rodeo: &impl Resolver) -> Report {
		Report::ExpectedTokenFound {
			expected: self.expected.into_iter().map(|t| t.error_text(rodeo)).collect(),
			found: self.found.error_text(rodeo),
			label: self.label,
			span: self.span,
		}
	}
}

impl<'a> chumsky::error::Error<'a, Input<'a>> for Error {
	fn expected_found<E: IntoIterator<Item = Option<chumsky::util::MaybeRef<'a, Token>>>>(
		expected: E,
		found: Option<chumsky::util::MaybeRef<'a, Token>>,
		span: Span,
	) -> Self {
		Self {
			expected: expected
				.into_iter()
				.map(|e| e.map_or(Token::Eof, |t| t.into_inner()))
				.collect(),
			found: found.map_or(Token::Eof, |t| t.into_inner()),
			label: None,
			span,
		}
	}

	fn merge(mut self, other: Self) -> Self {
		for t in other.expected {
			if !self.expected.contains(&t) {
				self.expected.push(t);
			}
		}

		self
	}
}

impl<'a> chumsky::label::LabelError<'a, Input<'a>, &'static str> for Error {
	#[allow(unused)]
	fn in_context(&mut self, label: &'static str, span: Span) {
		panic!("chumsky doesn't document this method, so it's unclear what it should do")
	}

	fn label_with(&mut self, label: &'static str) {
		self.label = Some(label);
	}
}
