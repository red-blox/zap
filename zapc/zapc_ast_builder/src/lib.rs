use chumsky::{
	primitive::{choice, custom},
	recursive::Recursive,
	IterParser, Parser as _,
};
use lasso::Rodeo;
use util::{BuilderError, Parser, ParserState, Span};
use zapc_ast::{
	Ast, AstBoolean, AstConfigTable, AstConfigValue, AstDecl, AstGenericTy, AstNumber, AstRange, AstString, AstTy,
	AstTyTable, AstTys, AstWord,
};
use zapc_meta::{FileId, Report};
use zapc_token::{Delim, Symbol, Token};

mod util;

fn into_span(span: Span) -> zapc_meta::Span {
	span.into()
}

fn word<'a>() -> impl Parser<'a, AstWord> {
	custom(|input| {
		let before = input.offset();
		let state: &mut ParserState<'a> = input.state();
		let empty_spur = state.rodeo.get_or_intern_static("");

		match input.next() {
			Some(Token::Word(spur)) => Ok(AstWord::new(spur, into_span(input.span_since(before)))),
			token => Err(BuilderError::new(
				&[Token::Word(empty_spur)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

fn keyword<'a>(keyword: &'static str) -> impl Parser<'a, AstWord> {
	custom(move |input| {
		let before = input.offset();
		let state: &mut ParserState<'a> = input.state();
		let keyword_spur = state.rodeo.get_or_intern_static(keyword);

		match input.next() {
			Some(Token::Word(spur)) if spur == keyword_spur => {
				Ok(AstWord::new(spur, into_span(input.span_since(before))))
			}

			token => Err(BuilderError::new(
				&[Token::Word(keyword_spur)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

fn number<'a>() -> impl Parser<'a, AstNumber> {
	custom(|input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Number(number)) => Ok(AstNumber::new(number, into_span(input.span_since(before)))),
			token => Err(BuilderError::new(
				&[Token::Number(0.0)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

fn string<'a>() -> impl Parser<'a, AstString> {
	custom(|input| {
		let before = input.offset();
		let state: &mut ParserState<'a> = input.state();
		let empty_spur = state.rodeo.get_or_intern_static("");

		match input.next() {
			Some(Token::String(spur)) => Ok(AstString::new(spur, into_span(input.span_since(before)))),
			token => Err(BuilderError::new(
				&[Token::String(empty_spur)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

#[allow(unused)]
fn delim<'a, O: Clone>(
	delim: Delim,
	parser: impl Parser<'a, O>,
	fallback: impl Fn(Span) -> O + Clone,
) -> impl Parser<'a, O> {
	let open = custom(move |input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Open(d)) if d == delim => Ok(()),
			token => Err(BuilderError::new(
				&[Token::Open(delim)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	});

	let close = custom(move |input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Close(d)) if d == delim => Ok(()),
			token => Err(BuilderError::new(
				&[Token::Close(delim)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	});

	// TODO: recover with nested_delimiters as soon as chumsky supports it
	open.ignore_then(parser).then_ignore(close)
}

fn symbol<'a>(symbol: Symbol) -> impl Parser<'a, ()> {
	custom(move |input| {
		let before = input.offset();

		match input.next() {
			Some(Token::Symbol(s)) if s == symbol => Ok(()),
			token => Err(BuilderError::new(
				&[Token::Symbol(symbol)],
				token.unwrap_or(Token::Eof),
				input.span_since(before),
			)),
		}
	})
}

fn range<'a>() -> impl Parser<'a, AstRange> {
	let with_min_max = number()
		.then_ignore(symbol(Symbol::DotDot))
		.then(number())
		.map_with(|(min, max), e| AstRange::WithMinMax(into_span(e.span()), min, max));

	let with_min = number()
		.then_ignore(symbol(Symbol::DotDot))
		.map_with(|min, e| AstRange::WithMin(into_span(e.span()), min));

	let with_max = symbol(Symbol::DotDot)
		.ignore_then(number())
		.map_with(|max, e| AstRange::WithMax(into_span(e.span()), max));

	let exact = number().map_with(|number, e| AstRange::Exact(into_span(e.span()), number));

	let none = symbol(Symbol::DotDot).map_with(|_, e| AstRange::None(into_span(e.span())));

	choice((with_min_max, with_min, with_max, exact, none)).labelled("Range")
}

fn ty<'a>() -> impl Parser<'a, AstTy> {
	let mut ty = Recursive::declare();

	let ty_table = delim(
		Delim::Brace,
		word()
			.then_ignore(symbol(Symbol::Colon))
			.then(ty.clone())
			.labelled("Field")
			.separated_by(symbol(Symbol::Comma))
			.allow_trailing()
			.collect()
			.map_with(|fields, e| AstTyTable::new(fields, into_span(e.span()))),
		|span| AstTyTable::new(Vec::new(), into_span(span)),
	);

	let generic_ty = choice((
		string().map(AstGenericTy::String),
		range().map(AstGenericTy::Range),
		ty.clone().map(AstGenericTy::Ty),
	));

	let ty_path = word()
		.separated_by(symbol(Symbol::Dot))
		.at_least(1)
		.collect()
		.then(
			delim(
				Delim::Angle,
				generic_ty
					.clone()
					.separated_by(symbol(Symbol::Comma))
					.allow_trailing()
					.at_least(1)
					.collect(),
				|_| Vec::new(),
			)
			.or_not()
			.map(|generics| generics.unwrap_or_default()),
		)
		.labelled("Path")
		.map_with(|(segments, generics), e| AstTy::Path {
			segments,
			generics,
			span: into_span(e.span()),
		});

	let ty_struct = keyword("struct")
		.ignore_then(ty_table.clone())
		.labelled("Struct")
		.map_with(|body, e| AstTy::Struct {
			body,
			span: into_span(e.span()),
		});

	let ty_unit_enum = keyword("enum")
		.ignore_then(delim(
			Delim::Brace,
			word().separated_by(symbol(Symbol::Comma)).collect(),
			|_| Vec::new(),
		))
		.labelled("Unit Enum")
		.map_with(|variants, e| AstTy::UnitEnum {
			variants,
			span: into_span(e.span()),
		});

	let ty_tagged_enum = keyword("enum")
		.ignore_then(string())
		.then(delim(
			Delim::Brace,
			word()
				.then(ty_table.clone())
				.separated_by(symbol(Symbol::Comma))
				.collect()
				.then(choice((
					symbol(Symbol::Comma)
						.ignore_then(symbol(Symbol::DotDot))
						.ignore_then(ty_table.clone())
						.map(Some),
					symbol(Symbol::Comma).or_not().to(None),
				))),
			|_| (Vec::new(), None),
		))
		.labelled("Tagged Enum")
		.map_with(|(tag, (variants, catch_all)), e| AstTy::TaggedEnum {
			tag,
			variants,
			catch_all,
			span: into_span(e.span()),
		});

	ty.define(
		choice((ty_struct, ty_unit_enum, ty_tagged_enum, ty_path))
			.then(symbol(Symbol::Question).or_not())
			.map_with(|(ty, opt), e| {
				if opt.is_some() {
					AstTy::Optional {
						ty: Box::new(ty),
						span: into_span(e.span()),
					}
				} else {
					ty
				}
			}),
	);

	ty
}

fn tys<'a>() -> impl Parser<'a, AstTys> {
	let single = ty().map(|ty| vec![ty]);

	let paren = delim(Delim::Paren, ty().separated_by(symbol(Symbol::Comma)).collect(), |_| {
		Vec::new()
	});

	choice((paren, single)).map_with(|tys, e| AstTys::new(tys, into_span(e.span())))
}

fn config<'a>() -> impl Parser<'a, AstConfigTable> {
	let mut config_value = Recursive::declare();

	let config_table = delim(
		Delim::Brace,
		word()
			.then_ignore(symbol(Symbol::Colon))
			.then(config_value.clone())
			.separated_by(symbol(Symbol::Comma))
			.allow_trailing()
			.collect()
			.map_with(|fields, e| AstConfigTable::new(fields, into_span(e.span()))),
		|span| AstConfigTable::new(Vec::new(), into_span(span)),
	);

	config_value.define(choice((
		number().map(AstConfigValue::Number),
		string().map(AstConfigValue::String),
		config_table.clone().map(AstConfigValue::Table),
		keyword("true")
			.to(true)
			.or(keyword("false").to(false))
			.map_with(|value, e| AstConfigValue::Boolean(AstBoolean::new(value, into_span(e.span())))),
		word()
			.separated_by(symbol(Symbol::Dot))
			.collect()
			.map_with(|segments, e| AstConfigValue::Path {
				segments,
				span: into_span(e.span()),
			}),
	)));

	config_table
}

fn decl<'a>() -> impl Parser<'a, AstDecl> {
	let mut decl = Recursive::declare();

	let ty_decl = keyword("type")
		.ignore_then(word())
		.then_ignore(symbol(Symbol::Equal))
		.then(ty())
		.labelled("Type Decl")
		.map_with(|(name, ty), e| AstDecl::Ty {
			name,
			ty,
			span: into_span(e.span()),
		});

	let mod_decl = keyword("mod")
		.ignore_then(word())
		.then(delim(Delim::Brace, decl.clone().repeated().collect(), |_| Vec::new()))
		.labelled("Mod Decl")
		.map_with(|(name, decls), e| AstDecl::Mod {
			name,
			decls,
			span: into_span(e.span()),
		});

	let event_decl = keyword("event")
		.ignore_then(word())
		.then(config())
		.then_ignore(symbol(Symbol::Equal))
		.then(tys())
		.labelled("Event Decl")
		.map_with(|((name, config), tys), e| AstDecl::Event {
			name,
			config,
			tys,
			span: into_span(e.span()),
		});

	let funct_decl = keyword("funct")
		.ignore_then(word())
		.then(config())
		.then_ignore(symbol(Symbol::Arrow))
		.then(tys())
		.then_ignore(symbol(Symbol::Arrow))
		.then(tys())
		.labelled("Funct Decl")
		.map_with(|(((name, config), args), rets), e| AstDecl::Funct {
			name,
			config,
			args,
			rets,
			span: into_span(e.span()),
		});

	decl.define(choice((ty_decl, mod_decl, event_decl, funct_decl)).then_ignore(symbol(Symbol::Semicolon).or_not()));

	decl
}

fn ast<'a>() -> impl Parser<'a, Ast> {
	decl().repeated().collect().map(Ast::new)
}

pub fn build(file: FileId, input: Vec<(Token, zapc_meta::Span)>, rodeo: &mut Rodeo) -> Result<Ast, Vec<Report>> {
	let mut input: Vec<(Token, Span)> = input.into_iter().map(|(t, span)| (t, span.into())).collect();

	let (_, eoi) = input.pop().unwrap_or((Token::Eof, Span::new(file, 0, 0)));
	let result = ast().parse_with_state(chumsky::input::Input::spanned(&input, eoi), &mut ParserState { rodeo });

	if result.has_errors() {
		Err(result.into_errors().into_iter().map(|e| e.into_report(rodeo)).collect())
	} else {
		Ok(result.into_output().unwrap())
	}
}
