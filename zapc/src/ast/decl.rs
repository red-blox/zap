use crate::meta::Span;

use super::{
	primitive::{AstNumber, AstString, AstWord},
	ty::AstTy,
};

#[derive(Debug, Clone)]
pub enum AstConfigValue {
	Boolean(bool, Span),
	Number(AstNumber),
	String(AstString),
	Path(Vec<AstWord>),
}

impl AstConfigValue {
	pub fn span(&self) -> Span {
		match self {
			Self::Boolean(_, span) => *span,
			Self::Number(number) => number.span(),
			Self::String(string) => string.span(),
			Self::Path(words) => words.first().unwrap().span().merge(words.last().unwrap().span()),
		}
	}
}

#[derive(Debug, Clone)]
pub struct AstConfig {
	fields: Vec<(AstWord, AstConfigValue)>,
	span: Span,
}

impl AstConfig {
	pub fn new(fields: Vec<(AstWord, AstConfigValue)>, span: Span) -> Self {
		Self { fields, span }
	}

	pub fn fields(&self) -> &[(AstWord, AstConfigValue)] {
		&self.fields
	}

	pub fn into_fields(self) -> Vec<(AstWord, AstConfigValue)> {
		self.fields
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone)]
pub enum AstDecl {
	Ty {
		name: AstWord,
		ty: AstTy,
		span: Span,
	},

	Scope {
		name: AstWord,
		span: Span,
	},

	Event {
		name: AstWord,
		config: AstConfig,
		tys: Vec<AstTy>,
		span: Span,
	},

	Remote {
		name: AstWord,
		config: AstConfig,
		span: Span,
	},
}
