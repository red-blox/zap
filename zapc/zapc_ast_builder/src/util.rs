use std::ops::Range;

use chumsky::{extra::Full, input::SpannedInput};
use lasso::{Resolver, Rodeo};
use zapc_meta::{FileId, Report};
use zapc_token::Token;

pub type Input<'a> = SpannedInput<Token, Span, &'a [(Token, Span)]>;

pub struct ParserState<'a> {
	pub rodeo: &'a mut Rodeo,
}

pub type Extra<'a> = Full<BuilderError, ParserState<'a>, ()>;

pub trait Parser<'a, O: Clone>: chumsky::Parser<'a, Input<'a>, O, Extra<'a>> + Clone {}
impl<'a, O: Clone, P> Parser<'a, O> for P where P: chumsky::Parser<'a, Input<'a>, O, Extra<'a>> + Clone {}

#[derive(Clone, Copy)]
pub struct Span(zapc_meta::Span);

impl Span {
	pub fn new(file: FileId, start: usize, end: usize) -> Self {
		Span(zapc_meta::Span::new(file, start, end))
	}

	pub fn from_range(file: FileId, range: Range<usize>) -> Self {
		Span(zapc_meta::Span::from_range(file, range))
	}

	pub fn file(&self) -> FileId {
		self.0.file()
	}

	pub fn start(&self) -> usize {
		self.0.start()
	}

	pub fn end(&self) -> usize {
		self.0.end()
	}

	pub fn range(&self) -> Range<usize> {
		self.start()..self.end()
	}
}

impl From<Span> for zapc_meta::Span {
	fn from(span: Span) -> Self {
		span.0
	}
}

impl From<zapc_meta::Span> for Span {
	fn from(span: zapc_meta::Span) -> Self {
		Span(span)
	}
}

impl chumsky::span::Span for Span {
	type Context = FileId;
	type Offset = usize;

	fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
		Span(zapc_meta::Span::new(context, range.start, range.end))
	}

	fn context(&self) -> Self::Context {
		self.0.file()
	}

	fn start(&self) -> Self::Offset {
		self.0.start()
	}

	fn end(&self) -> Self::Offset {
		self.0.end()
	}
}

pub struct BuilderError {
	expected: Vec<Token>,
	found: Token,
	label: Option<&'static str>,
	span: Span,
}

impl BuilderError {
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
			span: self.span.into(),
		}
	}
}

impl<'a> chumsky::error::Error<'a, Input<'a>> for BuilderError {
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

impl<'a> chumsky::label::LabelError<'a, Input<'a>, &'static str> for BuilderError {
	#[allow(unused)]
	fn in_context(&mut self, label: &'static str, span: Span) {
		panic!("chumsky doesn't document this method, so it's unclear what it should do")
	}

	fn label_with(&mut self, label: &'static str) {
		self.label = Some(label);
	}
}
