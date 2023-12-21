use chumsky::prelude::*;

use crate::util::NumTy;

use super::{lexer::Token, syntax_tree::*};

fn word<'a>() -> impl Parser<Token<'a>, Token<'a>, Error = Simple<Token<'a>>> {
	filter(|tok: &Token<'a>| matches!(tok, Token::Word(_)))
}

fn keyword<'a>(kw: &'static str) -> impl Parser<Token<'a>, Token<'a>, Error = Simple<Token<'a>>> {
	filter(|tok: &Token<'a>| *tok == Token::Word(kw))
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

fn parser<'a>() -> impl Parser<Token<'a>, SyntaxConfig<'a>, Error = Simple<Token<'a>>> {
	let numrange = || {
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
	};

	let intrange = || {
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
	};

	let ty = recursive(|ty| {
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
			.map(|(ty, range)| SyntaxTy::Str(ty, range))
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
	});

	empty().to(SyntaxConfig {
		tydecls: todo!(),
		evdecls: todo!(),
		server_output: todo!(),
		client_output: todo!(),
		write_checks: todo!(),
		typescript: todo!(),
	})
}
