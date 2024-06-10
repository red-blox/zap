use crate::meta::Span;

use super::{primitive::AstWord, range::AstRange};

#[derive(Debug, Clone)]
pub enum AstTy {
	Path {
		segments: Vec<AstWord>,
		generics: Vec<AstGeneric>,
		span: Span,
	},

	// UnitEnum {
	// 	variants: Vec<AstString>,
	// 	span: Span,
	// },

	// TaggedEnum {
	// 	tag: AstString,
	// 	variants: Vec<(AstString, AstStruct)>,
	// 	catch_all: Option<AstStruct>,
	// 	span: Span,
	// },
	Struct {
		strukt: AstStruct,
		span: Span,
	},
}

impl AstTy {
	pub fn span(&self) -> Span {
		match self {
			Self::Path { span, .. } => *span,
			// Self::UnitEnum { span, .. } => *span,
			// Self::TaggedEnum { span, .. } => *span,
			Self::Struct { span, .. } => *span,
		}
	}
}

#[derive(Debug, Clone)]
pub enum AstGeneric {
	// Ty(AstTy),
	Range(AstRange),
	// String(AstString),
}

impl AstGeneric {
	pub fn span(&self) -> Span {
		match self {
			// Self::Ty(ty) => ty.span(),
			Self::Range(range) => range.span(),
			// Self::String(string) => string.span(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct AstStruct {
	fields: Vec<(AstWord, AstTy)>,
	span: Span,
}

impl AstStruct {
	pub fn new(fields: Vec<(AstWord, AstTy)>, span: Span) -> Self {
		Self { fields, span }
	}

	pub fn fields(&self) -> &[(AstWord, AstTy)] {
		&self.fields
	}

	pub fn into_fields(self) -> Vec<(AstWord, AstTy)> {
		self.fields
	}

	pub fn span(&self) -> Span {
		self.span
	}
}
