use chumsky::{
	error::Simple,
	primitive::{choice, empty, just},
	recursive::recursive,
	select, Error, Parser,
};

use std::cell::Cell;

use crate::util::{NumTy, Range};

use super::{lex::Token, Config, Enum, Struct, Ty};

fn ty<'a>() -> impl Parser<Token<'a>, Ty, Error = Simple<Token<'a>>> {
	recursive(|ty| {
		let ty = &ty;

		let bool = keyword("boolean").to(Ty::Boolean).boxed();

		let opt_paren_delim_num_range = || {
			(numrange().delimited_by(just(Token::OpenParen), just(Token::CloseParen)))
				.or_not()
				.map(|r| r.unwrap_or_default())
		};

		let opt_paren_delim_int_range = || {
			(intrange().delimited_by(just(Token::OpenParen), just(Token::CloseParen)))
				.or_not()
				.map(|r| r.unwrap_or_default())
		};

		let num = choice::<_, Simple<Token<'a>>>((
			keyword("f32")
				.then(opt_paren_delim_num_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::F32, range }),
			keyword("f64")
				.then(opt_paren_delim_num_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::F32, range }),
			keyword("u8")
				.then(opt_paren_delim_int_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::U8, range }),
			keyword("u16")
				.then(opt_paren_delim_int_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::U16, range }),
			keyword("u32")
				.then(opt_paren_delim_int_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::U32, range }),
			keyword("i8")
				.then(opt_paren_delim_int_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::I8, range }),
			keyword("i16")
				.then(opt_paren_delim_int_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::I16, range }),
			keyword("i32")
				.then(opt_paren_delim_int_range())
				.map(|(_, range)| Ty::Num { ty: NumTy::I32, range }),
		))
		.boxed();

		let str = keyword("string")
			.then(opt_paren_delim_int_range())
			.map(|(_, range)| Ty::Str { len: range })
			.boxed();

		let arr = ty
			.then(intrange().delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)))
			.map(|(ty, len)| Ty::Arr { ty: Box::new(ty), len })
			.boxed();

		let map = keyword("map")
			.then(ty.delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)))
			.then_ignore(just(Token::Colon))
			.then(ty)
			.map(|((_, key), val)| Ty::Map {
				key: Box::new(key),
				val: Box::new(val),
			});

		let struct_field = |invalid_name: &'a str| {
			word()
				.then_ignore(just(Token::Colon))
				.then(ty)
				.try_map(move |(name, ty), s| {
					if name == invalid_name {
						Err(Simple::custom(s, "field name cannot be the same as the struct name"))
					} else {
						Ok((name.to_string(), ty))
					}
				})
		};

		let r#struct = |invalid_name: &'a str| {
			keyword("struct")
				.then(
					struct_field(invalid_name)
						.separated_by(just(Token::Comma))
						.allow_trailing()
						.delimited_by(just(Token::OpenBrace), just(Token::CloseBrace)),
				)
				.map(|(_, fields)| Struct { fields })
		};

		let enum_unit = keyword("enum")
			.then(
				word()
					.map(|s| s.to_string())
					.separated_by(just(Token::Comma))
					.at_least(1)
					.allow_trailing(),
			)
			.map(|(_, variants)| Enum::Unit(variants));

		let enum_tagged = keyword("enum")
			.then(strlit().then_with(|tag| {
				word()
					.then(r#struct(tag))
					.map(|(name, s)| (name.to_string(), s))
					.separated_by(just(Token::Comma))
					.at_least(1)
					.allow_trailing()
					.map(move |variants| Enum::Tagged {
						tag: tag.to_string(),
						variants,
					})
			}))
			.map(|(_, e)| e);

		let r#enum = enum_unit.or(enum_tagged).boxed();

		let instance = keyword("Instance")
			.then(
				word()
					.delimited_by(just(Token::OpenParen), just(Token::CloseParen))
					.or_not(),
			)
			.map(|(_, class)| Ty::Instance {
				strict: false,
				class: class.map(|s| s.to_string()),
			})
			.boxed();

		let vector3 = keyword("Vector3").to(Ty::Vector3).boxed();

		choice((bool, num))
	})
}

fn intrange<'a>() -> impl Parser<Token<'a>, Range, Error = Simple<Token<'a>>> {
	choice((
		intlit()
			.then_ignore(just(Token::DotDot))
			.then(intlit())
			.map(|(a, b)| Range::new(Some(a), Some(b))),
		intlit()
			.then_ignore(just(Token::DotDot))
			.map(|a| Range::new(Some(a), None)),
		just(Token::DotDot)
			.then(intlit())
			.map(|(_, b)| Range::new(None, Some(b))),
		just(Token::DotDot).to(Range::new(None, None)),
		intlit().map(|n| Range::new(Some(n), Some(n))),
		empty().to(Range::new(None, None)),
	))
	.boxed()
}

fn numrange<'a>() -> impl Parser<Token<'a>, Range, Error = Simple<Token<'a>>> {
	choice((
		numlit()
			.then_ignore(just(Token::DotDot))
			.then(numlit())
			.map(|(a, b)| Range::new(Some(a), Some(b))),
		numlit()
			.then_ignore(just(Token::DotDot))
			.map(|a| Range::new(Some(a), None)),
		just(Token::DotDot)
			.then(numlit())
			.map(|(_, b)| Range::new(None, Some(b))),
		just(Token::DotDot).to(Range::new(None, None)),
		numlit().map(|n| Range::new(Some(n), Some(n))),
		empty().to(Range::new(None, None)),
	))
	.boxed()
}

fn keyword<'a>(s: &'static str) -> impl Parser<Token<'a>, (), Error = Simple<Token<'a>>> {
	just(Token::Word(s)).to(())
}

fn word<'a>() -> impl Parser<Token<'a>, &'a str, Error = Simple<Token<'a>>> {
	select! {
		Token::Word(s) => s,
	}
}

fn strlit<'a>() -> impl Parser<Token<'a>, &'a str, Error = Simple<Token<'a>>> {
	select! {
		Token::StrLit(s) => s,
	}
}

fn intlit<'a>() -> impl Parser<Token<'a>, f64, Error = Simple<Token<'a>>> {
	numlit().try_map(|n, s| {
		if n.fract() != 0.0 {
			Err(Simple::custom(s, "number could not be parsed as integer"))
		} else {
			Ok(n)
		}
	})
}

fn numlit<'a>() -> impl Parser<Token<'a>, f64, Error = Simple<Token<'a>>> {
	select! {
		Token::NumLit(s) => s.parse().unwrap(),
	}
}
