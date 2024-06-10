use crate::meta::Span;

use crate::ast::primitive::AstNumber;

#[derive(Debug, Clone)]
pub enum AstRange {
	WithMinMax(Span, AstNumber, AstNumber),
	WithMin(Span, AstNumber),
	WithMax(Span, AstNumber),
	Exact(Span, AstNumber),
	None(Span),
}

impl AstRange {
	pub fn span(&self) -> Span {
		match self {
			AstRange::WithMinMax(span, _, _) => *span,
			AstRange::WithMin(span, _) => *span,
			AstRange::WithMax(span, _) => *span,
			AstRange::Exact(span, _) => *span,
			AstRange::None(span) => *span,
		}
	}
}
