use chumsky::prelude::*;

use crate::util::{EvCall, EvSource, EvType, NumTy};

use super::{
	lexer::{lex, Token},
	syntax_tree::*,
};

fn word<'a>() -> impl Parser<Token<'a>, Token<'a>, Error = Simple<Token<'a>>> {
	filter(|tok: &Token<'a>| matches!(tok, Token::Word(_)))
}

fn keyword<'a>(kw: &'static str) -> impl Parser<Token<'a>, (), Error = Simple<Token<'a>>> {
	filter(|tok: &Token<'a>| *tok == Token::Word(kw)).to(())
}

fn numlit<'a>() -> impl Parser<Token<'a>, Token<'a>, Error = Simple<Token<'a>>> {
	filter(|tok: &Token<'a>| matches!(tok, Token::NumLit(..)))
}

fn intlit<'a>() -> impl Parser<Token<'a>, Token<'a>, Error = Simple<Token<'a>>> {
	filter(|tok: &Token<'a>| match tok {
		Token::NumLit(s) => s.parse::<i64>().is_ok(),
		_ => false,
	})
}

fn strlit<'a>() -> impl Parser<Token<'a>, Token<'a>, Error = Simple<Token<'a>>> {
	filter(|tok: &Token<'a>| matches!(tok, Token::StrLit(..)))
}

fn numrange<'a>() -> impl Parser<Token<'a>, SyntaxRange<'a>, Error = Simple<Token<'a>>> {
	choice((
		numlit()
			.then_ignore(just(Token::DotDot))
			.then(numlit())
			.map(|(min, max)| SyntaxRange::WithMinMax(min, max)),
		numlit().then_ignore(just(Token::DotDot)).map(SyntaxRange::WithMin),
		numlit().map(SyntaxRange::Exact),
		just(Token::DotDot)
			.then(numlit())
			.map(|(_, max)| SyntaxRange::WithMax(max)),
		just(Token::DotDot).to(SyntaxRange::None),
		empty().to(SyntaxRange::None),
	))
	.boxed()
}

fn intrange<'a>() -> impl Parser<Token<'a>, SyntaxRange<'a>, Error = Simple<Token<'a>>> {
	choice((
		intlit()
			.then_ignore(just(Token::DotDot))
			.then(intlit())
			.map(|(min, max)| SyntaxRange::WithMinMax(min, max)),
		intlit().then_ignore(just(Token::DotDot)).map(SyntaxRange::WithMin),
		intlit().map(SyntaxRange::Exact),
		just(Token::DotDot)
			.then(intlit())
			.map(|(_, max)| SyntaxRange::WithMax(max)),
		just(Token::DotDot).to(SyntaxRange::None),
		empty().to(SyntaxRange::None),
	))
	.boxed()
}

fn ty<'a>() -> impl Parser<Token<'a>, SyntaxTy<'a>, Error = Simple<Token<'a>>> {
	recursive(|ty| {
		let fnum = choice((keyword("f32").to(NumTy::F32), keyword("f64").to(NumTy::F64)))
			.then(
				numrange()
					.delimited_by(just(Token::OpenParen), just(Token::CloseParen))
					.or_not(),
			)
			.map(|(ty, range)| SyntaxTy::Num(ty, range));

		let num = choice((
			keyword("u8").to(NumTy::U8),
			keyword("u16").to(NumTy::U16),
			keyword("u32").to(NumTy::U32),
			keyword("i8").to(NumTy::I8),
			keyword("i16").to(NumTy::I16),
			keyword("i32").to(NumTy::I32),
		))
		.then(
			intrange()
				.delimited_by(just(Token::OpenParen), just(Token::CloseParen))
				.or_not(),
		)
		.map(|(ty, range)| SyntaxTy::Num(ty, range));

		let ty_num = fnum.or(num).boxed();

		let ty_str = keyword("string")
			.then(
				intrange()
					.delimited_by(just(Token::OpenParen), just(Token::CloseParen))
					.or_not(),
			)
			.map(|(_, range)| SyntaxTy::Str(range))
			.boxed();

		let ty_arr = ty
			.clone()
			.then(intrange().delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)))
			.map(|(ty, range)| SyntaxTy::Arr(Box::new(ty), range))
			.boxed();

		let ty_map = keyword("map")
			.then(
				ty.clone()
					.delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)),
			)
			.then_ignore(just(Token::Colon))
			.then(ty.clone())
			.map(|((_, key), val)| SyntaxTy::Map(Box::new(key), Box::new(val)))
			.boxed();

		let struct_field = word().then_ignore(just(Token::Colon)).then(ty);
		let r#struct = {
			keyword("struct")
				.then(
					struct_field
						.separated_by(just(Token::Comma))
						.allow_trailing()
						.delimited_by(just(Token::OpenBrace), just(Token::CloseBrace)),
				)
				.map(|(_, fields)| SyntaxStruct(fields))
				.boxed()
		};

		let ty_struct = r#struct.clone().map(SyntaxTy::Struct).boxed();

		let enum_unit = keyword("enum")
			.then(word().separated_by(just(Token::Comma)).at_least(1).allow_trailing())
			.map(|(_, variants)| SyntaxEnum::Unit(variants));

		// This value gets moved into the closure below, so it needs to be cloned into it's own var.
		let enum_tagged_struct = r#struct.clone();

		let enum_tagged = keyword("enum")
			.then(strlit().then_with(move |tag| {
				word()
					.then(enum_tagged_struct.clone())
					.separated_by(just(Token::Comma))
					.at_least(1)
					.allow_trailing()
					.map(move |variants| SyntaxEnum::Tagged { tag, variants })
			}))
			.map(|(_, e)| e);

		let ty_enum = enum_unit.or(enum_tagged).map(SyntaxTy::Enum).boxed();

		let ty_instance = keyword("Instance")
			.then(
				word()
					.delimited_by(just(Token::OpenParen), just(Token::CloseParen))
					.or_not(),
			)
			.map(|(_, class)| SyntaxTy::Instance { strict: false, class })
			.boxed();

		let ty_ref = word().map(SyntaxTy::Ref);

		choice((ty_num, ty_str, ty_arr, ty_map, ty_struct, ty_enum, ty_instance, ty_ref))
	})
	.boxed()
}

fn tydecl<'a>() -> impl Parser<Token<'a>, SyntaxTyDecl<'a>, Error = Simple<Token<'a>>> {
	keyword("type")
		.then(word())
		.then_ignore(just(Token::Equals))
		.then(ty())
		.then_ignore(just(Token::Semicolon).or_not())
		.map(|((_, name), ty)| SyntaxTyDecl { name, ty })
		.boxed()
}

fn evdecl_field<'a, T>(
	name: &'static str,
	value: impl Parser<Token<'a>, T, Error = Simple<Token<'a>>>,
) -> impl Parser<Token<'a>, T, Error = Simple<Token<'a>>> {
	keyword(name)
		.then_ignore(just(Token::Colon))
		.then(value)
		.map(|(_, value)| value)
}

fn evdecl<'a>() -> impl Parser<Token<'a>, SyntaxEvDecl<'a>, Error = Simple<Token<'a>>> {
	keyword("event")
		.then(word())
		.then_ignore(just(Token::Equals))
		.then(
			evdecl_field(
				"from",
				choice((
					keyword("Server").to(EvSource::Server),
					keyword("Client").to(EvSource::Client),
				)),
			)
			.then_ignore(just(Token::Comma))
			.then(evdecl_field(
				"type",
				choice((
					keyword("Reliable").to(EvType::Reliable),
					keyword("Unreliable").to(EvType::Unreliable),
				)),
			))
			.then_ignore(just(Token::Comma))
			.then(evdecl_field(
				"call",
				choice((
					keyword("SingleSync").to(EvCall::SingleSync),
					keyword("SingleAsync").to(EvCall::SingleAsync),
					keyword("ManySync").to(EvCall::ManySync),
					keyword("ManyAsync").to(EvCall::ManyAsync),
				)),
			))
			.then_ignore(just(Token::Comma))
			.then(evdecl_field("data", ty()))
			.then_ignore(just(Token::Comma).or_not())
			.delimited_by(just(Token::OpenBrace), just(Token::CloseBrace)),
		)
		.then_ignore(just(Token::Semicolon).or_not())
		.map(|((_, name), (((from, evty), call), data))| SyntaxEvDecl {
			name,
			from,
			evty,
			call,
			data,
		})
		.boxed()
}

fn opt<'a>() -> impl Parser<Token<'a>, SyntaxOpt<'a>, Error = Simple<Token<'a>>> {
	keyword("opt")
		.then(word())
		.then_ignore(just(Token::Equals))
		.then(word().or(strlit()))
		.then_ignore(just(Token::Semicolon).or_not())
		.map(|((_, name), value)| SyntaxOpt { name, value })
}

pub fn parser<'a>() -> impl Parser<Token<'a>, SyntaxConfig<'a>, Error = Simple<Token<'a>>> {
	opt()
		.repeated()
		.then(choice((evdecl().map(SyntaxDecl::Ev), tydecl().map(SyntaxDecl::Ty))).repeated())
		.map(|(opts, decls)| SyntaxConfig { opts, decls })
}
