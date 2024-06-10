use chumsky::{input::SpannedInput, primitive::custom, IterParser, Parser as _};
use lasso::Rodeo;

use crate::{
	ast::Ast,
	lexer::Token,
	meta::{Report, Span},
};

use self::{decl::decl, error::Error};

mod decl;
mod error;
mod primitive;
mod range;
mod ty;

type Input<'a> = SpannedInput<Token, Span, &'a [(Token, Span)]>;

struct State<'a> {
	pub rodeo: &'a mut Rodeo,
}

type Extra<'a> = chumsky::extra::Full<Error, State<'a>, ()>;

trait Parser<'a, O>: chumsky::Parser<'a, Input<'a>, O, Extra<'a>> + Clone {}
impl<'a, O: Clone, P> Parser<'a, O> for P where P: chumsky::Parser<'a, Input<'a>, O, Extra<'a>> + Clone {}

fn ast<'a>() -> impl Parser<'a, Ast> {
	decl()
		.repeated()
		.collect::<Vec<_>>()
		.then_ignore(custom(|input| {
			let before = input.offset();

			match input.next() {
				Some(Token::Eof) => Ok(()),
				token => Err(Error::new(&[Token::Eof], token.unwrap(), input.span_since(before))),
			}
		}))
		.map(Ast::new)
}

pub fn parse(rodeo: &mut Rodeo, tokens: Vec<(Token, Span)>) -> Result<Ast, Vec<Report>> {
	let result = ast().parse_with_state(
		chumsky::input::Input::spanned(&tokens, tokens.last().unwrap().1),
		&mut State { rodeo },
	);

	if result.has_errors() {
		Err(result.into_errors().into_iter().map(|e| e.into_report(rodeo)).collect())
	} else {
		Ok(result.into_output().unwrap())
	}
}
