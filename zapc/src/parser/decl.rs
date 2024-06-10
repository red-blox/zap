use chumsky::{primitive::choice, recursive::Recursive, IterParser, Parser as _};

use crate::{
	ast::decl::{AstConfig, AstConfigValue, AstDecl},
	lexer::{Delim, Symbol},
};

use super::{
	primitive::{delim, keyword, number, string, symbol, word},
	ty::{ty, ty_pack},
	Parser,
};

pub fn config<'a>() -> impl Parser<'a, AstConfig> {
	let config_value = choice((
		keyword("true")
			.to(true)
			.or(keyword("false").to(false))
			.map_with(|v, e| AstConfigValue::Boolean(v, e.span())),
		number().map(AstConfigValue::Number),
		string().map(AstConfigValue::String),
		word()
			.separated_by(symbol(Symbol::Dot))
			.at_least(1)
			.collect::<Vec<_>>()
			.map(AstConfigValue::Path),
	));

	delim(
		Delim::Brace,
		word()
			.then_ignore(symbol(Symbol::Colon))
			.then(config_value)
			.separated_by(symbol(Symbol::Comma))
			.allow_trailing()
			.collect::<Vec<_>>(),
		|_| Vec::new(),
	)
	.map_with(|v, e| AstConfig::new(v, e.span()))
}

pub fn decl<'a>() -> impl Parser<'a, AstDecl> {
	let mut decl = Recursive::declare();

	let ty_decl = keyword("type")
		.ignore_then(word())
		.then_ignore(symbol(Symbol::Equal))
		.then(ty())
		.map_with(|(name, ty), e| AstDecl::Ty {
			name,
			ty,
			span: e.span(),
		});

	let event_decl = keyword("event")
		.ignore_then(word())
		.then(config())
		.then_ignore(symbol(Symbol::Equal))
		.then(ty_pack())
		.map_with(|((name, config), tys), e| AstDecl::Event {
			name,
			config,
			tys,
			span: e.span(),
		});

	let remote_decl = keyword("remote")
		.ignore_then(word())
		.then(config())
		.map_with(|(name, config), e| AstDecl::Remote {
			name,
			config,
			span: e.span(),
		});

	decl.define(choice((ty_decl, event_decl, remote_decl)).then_ignore(symbol(Symbol::Semicolon).or_not()));

	decl
}
