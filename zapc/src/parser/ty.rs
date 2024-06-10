use chumsky::{primitive::choice, recursive::Recursive, IterParser, Parser as _};

use crate::{
	ast::ty::{AstGeneric, AstStruct, AstTy},
	lexer::{Delim, Symbol},
};

use super::{
	primitive::{delim, keyword, symbol, word},
	range::range,
	Parser,
};

pub fn ty<'a>() -> impl Parser<'a, AstTy> {
	let mut ty = Recursive::declare();
	let mut generic = Recursive::declare();

	let strukt = delim(
		Delim::Brace,
		word()
			.then_ignore(symbol(Symbol::Colon))
			.then(ty.clone())
			.separated_by(symbol(Symbol::Comma))
			.allow_trailing()
			.collect::<Vec<_>>(),
		|_| Vec::new(),
	)
	.map_with(|fields, e| AstStruct::new(fields, e.span()));

	// let ty_unit_enum = keyword("enum")
	// 	.ignore_then(delim(
	// 		Delim::Brace,
	// 		string()
	// 			.separated_by(symbol(Symbol::Comma))
	// 			.allow_trailing()
	// 			.collect::<Vec<_>>(),
	// 		|_| Vec::new(),
	// 	))
	// 	.map_with(|variants, e| AstTy::UnitEnum {
	// 		variants,
	// 		span: e.span(),
	// 	});

	// let ty_tagged_enum = keyword("enum")
	// 	.ignore_then(string())
	// 	.then(delim(
	// 		Delim::Brace,
	// 		string()
	// 			.then(strukt.clone())
	// 			.separated_by(symbol(Symbol::Comma))
	// 			.collect::<Vec<_>>()
	// 			.then(
	// 				symbol(Symbol::Comma)
	// 					.ignore_then(symbol(Symbol::DotDot))
	// 					.ignore_then(strukt.clone())
	// 					.or_not(),
	// 			)
	// 			.then_ignore(symbol(Symbol::Comma).or_not()),
	// 		|_| (Vec::new(), None),
	// 	))
	// 	.map_with(|(tag, (variants, catch_all)), e| AstTy::TaggedEnum {
	// 		tag,
	// 		variants,
	// 		catch_all,
	// 		span: e.span(),
	// 	});

	let ty_struct = keyword("struct")
		.ignore_then(strukt.clone())
		.map_with(|strukt, e| AstTy::Struct { strukt, span: e.span() });

	let ty_path = word()
		.separated_by(symbol(Symbol::Dot))
		.at_least(1)
		.collect::<Vec<_>>()
		.then(
			delim(
				Delim::Angle,
				generic
					.clone()
					.separated_by(symbol(Symbol::Comma))
					.allow_trailing()
					.collect::<Vec<_>>(),
				|_| Vec::new(),
			)
			.or_not(),
		)
		.map_with(|(segments, generics), e| AstTy::Path {
			segments,
			generics: generics.unwrap_or(Vec::new()),
			span: e.span(),
		});

	ty.define(choice((ty_struct, /* ty_unit_enum, ty_tagged_enum, */ ty_path)));

	generic.define(choice((
		// ty.clone().map(AstGeneric::Ty),
		range().map(AstGeneric::Range),
		// string().map(AstGeneric::String),
	)));

	ty.labelled("Type")
}

pub fn ty_pack<'a>() -> impl Parser<'a, Vec<AstTy>> {
	ty().map(|ty| vec![ty]).or(delim(
		Delim::Paren,
		ty().separated_by(symbol(Symbol::Comma))
			.allow_trailing()
			.collect::<Vec<_>>(),
		|_| Vec::new(),
	))
}
