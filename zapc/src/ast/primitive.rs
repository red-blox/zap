use lasso::{Resolver, Spur};

use crate::meta::Span;

#[derive(Debug, Clone)]
pub struct AstWord {
	spur: Spur,
	span: Span,
}

impl AstWord {
	pub fn new(spur: Spur, span: Span) -> Self {
		Self { spur, span }
	}

	pub fn spur(&self) -> Spur {
		self.spur
	}

	pub fn word<'a>(&'a self, rodeo: &'a impl Resolver) -> &str {
		rodeo.resolve(&self.spur)
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone)]
pub struct AstString {
	spur: Spur,
	span: Span,
}

impl AstString {
	pub fn new(spur: Spur, span: Span) -> Self {
		Self { spur, span }
	}

	pub fn spur(&self) -> Spur {
		self.spur
	}

	pub fn string<'a>(&'a self, rodeo: &'a impl Resolver) -> &str {
		rodeo.resolve(&self.spur)
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone)]
pub struct AstNumber {
	value: f64,
	span: Span,
}

impl AstNumber {
	pub fn new(value: f64, span: Span) -> Self {
		Self { value, span }
	}

	pub fn value(&self) -> f64 {
		self.value
	}

	pub fn span(&self) -> Span {
		self.span
	}
}
