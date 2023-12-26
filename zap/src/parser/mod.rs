use std::collections::HashSet;

use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use lalrpop_util::{lalrpop_mod, ParseError};

use crate::config::{Config, EvType};

use self::syntax_tree::{Spanned, SyntaxEvDecl};

mod convert;
mod syntax_tree;

lalrpop_mod!(pub grammar);

pub fn parse(input: &str) -> (Option<Config<'_>>, Vec<Diagnostic<()>>) {
	let parse_result = grammar::ConfigParser::new().parse(input);

	if let Ok(syntax_config) = parse_result {
		let evdecls: Vec<SyntaxEvDecl<'_>> = syntax_config
			.decls
			.iter()
			.filter_map(|decl| match decl {
				syntax_tree::SyntaxDecl::Ev(evdecl) => Some(evdecl.clone()),
				_ => None,
			})
			.collect();

		let (config, mut diags) = convert::convert(syntax_config);

		if diags.iter().any(|diag| diag.severity == Severity::Error) {
			(None, diags)
		} else {
			let mut had_err = false;

			for ev in config.evdecls.iter().filter(|ev| ev.evty == EvType::Unreliable) {
				if !ev
					.data
					.max_size(&config, &mut HashSet::new())
					.is_some_and(|size| size <= 900 - config.event_id_ty().size())
				{
					let evdecl = evdecls.iter().find(|evdecl| evdecl.name.name == ev.name).unwrap();

					diags.push(
						Diagnostic::error()
							.with_code("E2:001")
							.with_message("unreliable event is too large")
							.with_labels(vec![
								Label::primary((), evdecl.span()).with_message("event is here"),
								Label::secondary((), evdecl.data.span()).with_message("event data is here"),
							]),
					);

					had_err = true;
				}
			}

			if had_err {
				(None, diags)
			} else {
				(Some(config), diags)
			}
		}
	} else {
		let diagnostic = match parse_result.unwrap_err() {
			ParseError::InvalidToken { location } => Diagnostic::error()
				.with_code("E0:001")
				.with_message("invalid token")
				.with_labels(vec![
					Label::primary((), location..location).with_message("invalid token")
				]),

			ParseError::UnrecognizedEof { location, expected } => Diagnostic::error()
				.with_code("E0:002")
				.with_message(format!("expected one of: {} but found EOF", expected.join(", ")))
				.with_labels(vec![Label::primary((), location..location)
					.with_message(format!("expected one of: {}", expected.join(", ")))]),

			ParseError::UnrecognizedToken {
				token: (start, token, end),
				expected,
			} => Diagnostic::error()
				.with_code("E0:003")
				.with_message(format!("expected one of: {} but found {}", expected.join(", "), token))
				.with_labels(vec![
					Label::primary((), start..end).with_message(format!("expected one of: {}", expected.join(", ")))
				]),

			ParseError::ExtraToken {
				token: (start, token, end),
			} => Diagnostic::error()
				.with_code("E0:004")
				.with_message(format!("unexpected token {}", token))
				.with_labels(vec![
					Label::primary((), start..end).with_message(format!("unexpected token {}", token))
				]),

			ParseError::User { .. } => unimplemented!("zap doesn't throw user errors, this is a bug!"),
		};

		(None, vec![diagnostic])
	}
}
