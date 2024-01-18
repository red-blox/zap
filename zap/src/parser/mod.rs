use codespan_reporting::diagnostic::Severity;
use lalrpop_util::{lalrpop_mod, ParseError};

use crate::config::Config;

use self::reports::Report;

mod convert;
mod reports;
mod syntax_tree;

lalrpop_mod!(pub grammar);

pub fn parse<'src>(input: &'src str, mode: Option<&'static str>) -> (Option<Config<'src>>, Vec<Report<'src>>) {
	let parse_result = grammar::ConfigParser::new().parse(input);

	if let Ok(syntax_config) = parse_result {
		let (config, reports) = convert::convert(syntax_config, mode);

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
