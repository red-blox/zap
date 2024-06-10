use chumsky::{primitive::choice, Parser as _};

use crate::{ast::range::AstRange, lexer::Symbol};

use super::{
	primitive::{number, symbol},
	Parser,
};

pub fn range<'a>() -> impl Parser<'a, AstRange> {
	let with_min_max = number()
		.then_ignore(symbol(Symbol::DotDot))
		.then(number())
		.map_with(|(min, max), e| AstRange::WithMinMax(e.span(), min, max));

	let with_min = number()
		.then_ignore(symbol(Symbol::DotDot))
		.map_with(|min, e| AstRange::WithMin(e.span(), min));

	let with_max = symbol(Symbol::DotDot)
		.ignore_then(number())
		.map_with(|max, e| AstRange::WithMax(e.span(), max));

	let exact = number().map_with(|exact, e| AstRange::Exact(e.span(), exact));

	let none = symbol(Symbol::DotDot).map_with(|(), e| AstRange::None(e.span()));

	choice((with_min_max, with_min, with_max, exact, none)).labelled("Range")
}
