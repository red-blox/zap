use std::collections::HashSet;

use codespan_reporting::diagnostic::Severity;
use lalrpop_util::{lalrpop_mod, ParseError};

use crate::config::{Config, EvType};

use self::{
	reports::Report,
	syntax_tree::{Spanned, SyntaxEvDecl},
};

mod convert;
mod reports;
mod syntax_tree;

lalrpop_mod!(pub grammar);

pub fn parse(input: &str) -> (Option<Config<'_>>, Vec<Report>) {
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

		let (config, mut reports) = convert::convert(syntax_config);
		let max_unreliable_size = 900 - config.event_id_ty().size();

		if config.evdecls.is_empty() {
			reports.push(Report::AnalyzeEmptyEvDecls);
		}

		for ev in config.evdecls.iter().filter(|ev| ev.evty == EvType::Unreliable) {
			let max_size = ev.data.max_size(&config, &mut HashSet::new());

			if let Some(max_size) = max_size {
				if max_size > max_unreliable_size {
					let evdecl = evdecls.iter().find(|evdecl| evdecl.name.name == ev.name).unwrap();

					reports.push(Report::AnalyzeOversizeUnreliable {
						ev_span: evdecl.span(),
						ty_span: evdecl.data.span(),
						max_size: max_unreliable_size,
						size: max_size,
					});
				}
			} else {
				let evdecl = evdecls.iter().find(|evdecl| evdecl.name.name == ev.name).unwrap();

				reports.push(Report::AnalyzePotentiallyOversizeUnreliable {
					ev_span: evdecl.span(),
					ty_span: evdecl.data.span(),
					max_size: max_unreliable_size,
				});
			}
		}

		if reports.iter().any(|report| report.severity() == Severity::Error) {
			(None, reports)
		} else {
			(Some(config), reports)
		}
	} else {
		let report = match parse_result.unwrap_err() {
			ParseError::InvalidToken { location } => Report::LexerInvalidToken {
				span: location..location,
			},

			ParseError::UnrecognizedEof { location, expected } => Report::ParserUnexpectedEOF {
				span: location..location,
				expected,
			},

			ParseError::UnrecognizedToken { token, expected } => Report::ParserUnexpectedToken {
				span: token.0..token.2,
				expected,
				token: token.1,
			},

			ParseError::ExtraToken { token } => Report::ParserExtraToken { span: token.0..token.2 },

			ParseError::User { error } => error,
		};

		(None, vec![report])
	}
}
