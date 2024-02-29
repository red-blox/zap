use ast::*;
use chumsky::{
	primitive::{choice, end},
	recovery::nested_delimiters,
	recursive::Recursive,
	select, Parser, Stream,
};
use meta::{Report, Span};
use token::{Token, Token::*};

const DELIMITERS: [(Token, Token); 3] = [
	(OpenParen, CloseParen),
	(OpenBracket, CloseBracket),
	(OpenBrace, CloseBrace),
];

pub fn build(eoi: Span, tokens: Vec<(Token<'_>, Span)>) -> Result<Ast<'_>, Vec<Report<'_>>> {
	let (ast, reports) = ast().parse_recovery(Stream::from_iter(eoi, tokens.into_iter()));

	if reports.is_empty() {
		Ok(ast.unwrap())
	} else {
		Err(reports)
	}
}

fn expected<'a>(mut report: Report<'a>, tok: Token<'a>) -> Report<'a> {
	if let Report::ParserExpectedToken { expected, .. } = &mut report {
		expected.insert(tok);
	}

	report
}

fn sm(symbol: Token<'_>) -> impl Parser<Token<'_>, (), Error = Report<'_>> + Clone {
	chumsky::primitive::just(symbol).ignored()
}

fn kw<'a>(keyword: &'static str) -> impl Parser<Token<'a>, AstIdent<'a>, Error = Report<'a>> + Clone {
	select! { Word(ident) if ident == keyword => ident }
		.map_with_span(AstIdent::new)
		.map_err(|e| expected(e, Word(keyword)))
}

fn ident<'a>() -> impl Parser<Token<'a>, AstIdent<'a>, Error = Report<'a>> + Clone {
	select! { Word(ident) => ident }
		.map_with_span(AstIdent::new)
		.map_err(|e| expected(e, Word("")))
		.labelled("identifier")
}

fn bool<'a>() -> impl Parser<Token<'a>, AstBool, Error = Report<'a>> + Clone {
	select! { Word("true") => true, Word("false") => false }
		.map_with_span(|value, span| AstBool::new(span, value))
		.map_err(|e| expected(expected(e, Word("true")), Word("false")))
		.labelled("boolean")
}

fn number<'a>() -> impl Parser<Token<'a>, AstNumber, Error = Report<'a>> + Clone {
	select! { Number(num) => num }
		.map_with_span(|num, span| AstNumber::new(span, num.parse().unwrap()))
		.map_err(|e| expected(e, Number("")))
		.labelled("number")
}

fn string<'a>() -> impl Parser<Token<'a>, AstString<'a>, Error = Report<'a>> + Clone {
	select! { Token::String(s) => s }
		.map_with_span(AstString::new)
		.map_err(|e| expected(e, String("")))
		.labelled("string")
}

fn range<'a>() -> impl Parser<Token<'a>, AstRange, Error = Report<'a>> + Clone {
	let with_min_max = number()
		.then_ignore(sm(DotDot))
		.then(number())
		.map_with_span(|(min, max), span| AstRange::WithMinMax(span, min, max));

	let with_max = sm(DotDot)
		.ignore_then(number())
		.map_with_span(|min, span| AstRange::WithMax(span, min));

	let with_min = number()
		.then_ignore(sm(DotDot))
		.map_with_span(|max, span| AstRange::WithMin(span, max));

	let exact = number().map_with_span(|num, span| AstRange::Exact(span, num));

	let none = sm(DotDot).map_with_span(|(), span| AstRange::None(span));

	choice((with_min_max, with_max, with_min, exact, none))
		.delimited_by(sm(OpenParen), sm(CloseParen))
		.recover_with(nested_delimiters(OpenParen, CloseParen, DELIMITERS, AstRange::None))
		.labelled("range")
		.boxed()
}

fn ty<'a>() -> impl Parser<Token<'a>, AstTy<'a>, Error = Report<'a>> + Clone {
	let mut ty = Recursive::declare();

	let ty_reference = ident()
		.separated_by(sm(Dot))
		.map_with_span(|name, span| AstTy::Reference { span, name });

	let ty_instance = kw("Instance")
		.then(
			string()
				.delimited_by(sm(OpenParen), sm(CloseParen))
				.or_not()
				.recover_with(nested_delimiters(OpenParen, CloseParen, DELIMITERS, |_| None)),
		)
		.map_with_span(|(_, class), span| AstTy::Instance { span, class });

	let ty_number = choice((
		kw("f32"),
		kw("f64"),
		kw("u8"),
		kw("u16"),
		kw("u32"),
		kw("i8"),
		kw("i16"),
		kw("i32"),
	))
	.then(range().or_not())
	.map_with_span(|(name, range), span| AstTy::Number { span, name, range });

	let ty_string = kw("string")
		.then(range().or_not())
		.map_with_span(|(_, range), span| AstTy::String { span, range });

	let ty_buffer = kw("buffer")
		.then(range().or_not())
		.map_with_span(|(_, range), span| AstTy::Buffer { span, range });

	let ty_array = ty
		.clone()
		.delimited_by(sm(OpenBracket), sm(CloseBracket))
		.recover_with(nested_delimiters(OpenBracket, CloseBracket, DELIMITERS, AstTy::Error))
		.then(range().or_not())
		.map_with_span(|(ty, range), span| AstTy::Array {
			span,
			ty: Box::new(ty),
			range,
		});

	let ty_map = ty
		.clone()
		.then_ignore(sm(Colon))
		.then(ty.clone())
		.delimited_by(sm(OpenBracket), sm(CloseBracket))
		.recover_with(nested_delimiters(OpenBracket, CloseBracket, DELIMITERS, |span| {
			(AstTy::Error(span), AstTy::Error(span))
		}))
		.then(range().or_not())
		.map_with_span(|((key_ty, val_ty), range), span| AstTy::Map {
			span,
			range,
			key_ty: Box::new(key_ty),
			val_ty: Box::new(val_ty),
		});

	let struct_body = ident()
		.then_ignore(sm(Colon))
		.then(ty.clone())
		.separated_by(sm(Comma))
		.allow_trailing()
		.delimited_by(sm(OpenBrace), sm(CloseBrace))
		.recover_with(nested_delimiters(OpenBrace, CloseBrace, DELIMITERS, |_| vec![]))
		.map_with_span(|fields, span| AstStruct::new(span, fields));

	let ty_struct = kw("struct")
		.ignore_then(struct_body.clone())
		.map_with_span(|s, span| AstTy::Struct(span, s));

	let enum_unit = ident()
		.separated_by(sm(Comma))
		.allow_trailing()
		.at_least(1)
		.delimited_by(sm(OpenBrace), sm(CloseBrace))
		.recover_with(nested_delimiters(OpenBrace, CloseBrace, DELIMITERS, |_| vec![]))
		.map_with_span(|variants, span| AstEnum::Unit { span, variants });

	let enum_tagged = string()
		.then(
			ident()
				.then(struct_body.clone())
				.separated_by(sm(Comma))
				.at_least(1)
				.then(
					sm(Comma)
						.ignore_then(sm(DotDot))
						.ignore_then(struct_body.clone())
						.then_ignore(sm(Comma).or_not())
						.map(Some)
						.or(sm(Comma).or_not().to(None)),
				)
				.delimited_by(sm(OpenBrace), sm(CloseBrace))
				.recover_with(nested_delimiters(OpenBrace, CloseBrace, DELIMITERS, |_| (vec![], None))),
		)
		.map_with_span(|(field, (variants, catch_all)), span| AstEnum::Tagged {
			span,
			field,
			variants,
			catch_all,
		});

	let ty_enum = kw("enum")
		.ignore_then(enum_unit.or(enum_tagged))
		.map_with_span(|e, span| AstTy::Enum(span, e));

	// reference is placed at the bottom because it's the most generic - will consume any Token::Word
	ty.define(
		choice((
			ty_instance,
			ty_number,
			ty_string,
			ty_buffer,
			ty_array,
			ty_map,
			ty_struct,
			ty_enum,
			ty_reference,
		))
		.then(sm(Question).or_not())
		.map_with_span(|(ty, optional), span| {
			if optional.is_some() {
				AstTy::Optional { span, ty: Box::new(ty) }
			} else {
				ty
			}
		}),
	);

	ty
}

fn config_struct<'a>() -> impl Parser<Token<'a>, AstConfigStruct<'a>, Error = Report<'a>> + Clone {
	let mut config_value = Recursive::declare();

	let config_struct = ident()
		.then_ignore(sm(Colon))
		.then(config_value.clone())
		.separated_by(sm(Comma))
		.allow_trailing()
		.delimited_by(sm(OpenBrace), sm(CloseBrace))
		.recover_with(nested_delimiters(OpenBrace, CloseBrace, DELIMITERS, |_| vec![]))
		.map_with_span(|fields, span| AstConfigStruct::new(span, fields));

	let config_bool = bool().map(AstConfigValue::Bool);
	let config_number = number().map(AstConfigValue::Number);
	let config_string = string().map(AstConfigValue::String);

	let config_enum = ident()
		.then(
			config_value
				.clone()
				.separated_by(sm(Comma))
				.allow_trailing()
				.delimited_by(sm(OpenParen), sm(CloseParen)),
		)
		.map_with_span(|(name, args), span| AstConfigValue::Enum(span, name, args));

	let config_reference = ident()
		.separated_by(sm(Dot))
		.at_least(1)
		.map_with_span(|name, span| AstConfigValue::Reference(span, name));

	config_value.define(choice((
		config_bool,
		config_number,
		config_string,
		config_enum,
		config_reference,
		config_struct.clone().map(AstConfigValue::Struct),
	)));

	config_struct
}

fn decl<'a>() -> impl Parser<Token<'a>, AstDecl<'a>, Error = Report<'a>> + Clone {
	let mut decl = Recursive::declare();

	let decl_ty = kw("type")
		.ignore_then(ident())
		.then_ignore(sm(Equal))
		.then(ty())
		.map_with_span(|(name, ty), span| AstDecl::Ty { span, name, ty });

	let ty_list = ty()
		.separated_by(sm(Comma))
		.delimited_by(sm(OpenParen), sm(CloseParen))
		.or(ty().map(|ty| vec![ty]));

	let decl_ev = kw("event")
		.ignore_then(ident())
		.then(config_struct())
		.then_ignore(sm(Equal))
		.then(ty_list.clone())
		.map_with_span(|((name, config), data), span| AstDecl::Ev {
			span,
			name,
			config,
			data,
		});

	let decl_fn = kw("funct")
		.ignore_then(ident())
		.then(config_struct())
		.then_ignore(sm(Equal))
		.then(ty_list.clone())
		.then_ignore(sm(Arrow))
		.then(ty_list.clone())
		.map_with_span(|(((name, config), args), rets), span| AstDecl::Fn {
			span,
			name,
			config,
			args,
			rets,
		});

	let decl_ch = kw("channel")
		.ignore_then(ident())
		.then(config_struct())
		.map_with_span(|(name, config), span| AstDecl::Ch { span, name, config });

	let decl_ns = kw("namespace")
		.ignore_then(ident())
		.then(
			decl.clone()
				.repeated()
				.delimited_by(sm(OpenBrace), sm(CloseBrace))
				.recover_with(nested_delimiters(OpenBrace, CloseBrace, DELIMITERS, |_| vec![])),
		)
		.map_with_span(|(name, body), span| AstDecl::Ns { span, name, body });

	decl.define(choice((decl_ty, decl_ev, decl_fn, decl_ch, decl_ns)).then_ignore(sm(Semicolon).or_not()));

	decl
}

fn opts<'a>() -> impl Parser<Token<'a>, Opts<'a>, Error = Report<'a>> + Clone {
	kw("opts")
		.ignore_then(string().or_not())
		.then(config_struct())
		.then_ignore(sm(Semicolon).or_not())
		.map_with_span(|(name, config), span| Opts::new(span, name, config))
}

fn ast<'a>() -> impl Parser<Token<'a>, Ast<'a>, Error = Report<'a>> + Clone {
	opts()
		.repeated()
		.then(decl().repeated())
		.then_ignore(end())
		.map(|(opts, decls)| Ast::new(opts, decls))
}
