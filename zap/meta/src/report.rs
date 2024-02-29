use std::collections::HashSet;

use codespan_reporting::{
	diagnostic::{Diagnostic, Label},
	term,
};
use token::Token;

use crate::{FileDatabase, FileId, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum Report<'a> {
	LexerUnexpectedCharacter {
		span: Span,
		c: char,
	},

	ParserUnexpectedEof {
		span: Span,
		expected: Vec<Token<'a>>,
		label: Option<&'static str>,
	},

	ParserExpectedToken {
		span: Span,
		expected: HashSet<Token<'a>>,
		found: Token<'a>,
		label: Option<&'static str>,
	},

	ParserUnclosedDelimiter {
		unclosed_span: Span,
		unclosed: Token<'a>,
		before_span: Span,
		before: Option<Token<'a>>,
		expected: Token<'a>,
		label: Option<&'static str>,
	},

	AnalysisDuplicateStructField {
		first_span: Span,
		second_span: Span,
		field: &'a str,
	},

	AnalysisDuplicateEnumVariant {
		first_span: Span,
		second_span: Span,
		variant: &'a str,
	},

	AnalysisTaggedEnumTagAsField {
		tag: &'a str,
		tag_span: Span,
		field_span: Span,
	},
}

impl<'a> From<&Report<'a>> for Diagnostic<FileId> {
	fn from(value: &Report) -> Self {
		match value {
			Report::LexerUnexpectedCharacter { span, c } => Diagnostic::error()
				.with_message(format!("Unexpected character '{c}' while tokenizing"))
				.with_labels(vec![
					Label::primary(span.file(), span).with_message("Unexpected character")
				]),

			Report::ParserUnexpectedEof { span, expected, label } => {
				let msg = format!(
					"Unexpected end of file{}, expected {}",
					label.map(|l| format!(" while parsing {l}")).unwrap_or("".to_string()),
					if expected.len() == 1 {
						format!("{}", expected[0])
					} else {
						format!(
							"one of {}",
							expected.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", ")
						)
					}
				);

				Diagnostic::error().with_message(msg).with_labels(vec![
					Label::primary(span.file(), span).with_message("Unexpected end of file")
				])
			}

			Report::ParserExpectedToken {
				span,
				expected,
				found,
				label,
			} => {
				let msg = format!(
					"Unexpected token '{}'{}, expected {}",
					found,
					label.map(|l| format!(" while parsing {l}")).unwrap_or("".to_string()),
					if expected.len() == 1 {
						format!("{}", expected.iter().next().unwrap())
					} else {
						format!(
							"one of {}",
							expected.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", ")
						)
					}
				);

				Diagnostic::error()
					.with_message(msg)
					.with_labels(vec![Label::primary(span.file(), span).with_message("Unexpected token")])
			}

			Report::ParserUnclosedDelimiter {
				unclosed_span: span,
				unclosed,
				before_span,
				before,
				expected,
				label,
			} => {
				let msg = format!(
					"Unclosed delimiter '{}'{}, expected '{}'",
					unclosed,
					label.map(|l| format!(" while parsing {l}")).unwrap_or("".to_string()),
					expected
				);

				Diagnostic::error().with_message(msg).with_labels(vec![
					Label::primary(span.file(), span).with_message(format!("Delimiter '{unclosed}' is not closed")),
					Label::secondary(before_span.file(), before_span).with_message(format!(
						"Must be closed with '{expected}'{}",
						before.map(|b| format!(" before '{b}'")).unwrap_or("".to_string())
					)),
				])
			}

			Report::AnalysisDuplicateStructField {
				first_span,
				second_span,
				field,
			} => Diagnostic::error()
				.with_message(format!("Duplicate field '{field}' in struct"))
				.with_labels(vec![
					Label::primary(second_span.file(), second_span).with_message("Field used as duplicate"),
					Label::secondary(first_span.file(), first_span).with_message("Field used previously in struct"),
				]),

			Report::AnalysisDuplicateEnumVariant {
				first_span,
				second_span,
				variant,
			} => Diagnostic::error()
				.with_message(format!("Duplicate variant '{variant}' in enum"))
				.with_labels(vec![
					Label::primary(second_span.file(), second_span).with_message("Variant used as duplicate"),
					Label::secondary(first_span.file(), first_span).with_message("Variant used previously in enum"),
				]),

			Report::AnalysisTaggedEnumTagAsField {
				tag,
				tag_span,
				field_span,
			} => Diagnostic::error()
				.with_message(format!("Tagged enum tag '{tag}' used as field within variant struct"))
				.with_labels(vec![
					Label::primary(field_span.file(), field_span).with_message("Tag used as field name"),
					Label::secondary(tag_span.file(), tag_span).with_message(format!("Tag '{tag}' declared")),
				])
				.with_notes(vec![[
					"Tagged enums use the tag as a field to determine which variant the value represents",
					"Consider renaming the field to something else or renaming the tag to something else",
				]
				.join("\n")]),
		}
	}
}

impl<'a> chumsky::Error<Token<'a>> for Report<'a> {
	type Span = Span;
	type Label = &'static str;

	fn expected_input_found<Iter: IntoIterator<Item = Option<Token<'a>>>>(
		span: Self::Span,
		expected: Iter,
		found: Option<Token<'a>>,
	) -> Self {
		if let Some(found) = found {
			Report::ParserExpectedToken {
				span,
				expected: HashSet::from_iter(expected.into_iter().flatten()),
				found,
				label: None,
			}
		} else {
			Report::ParserUnexpectedEof {
				span,
				expected: expected.into_iter().flatten().collect(),
				label: None,
			}
		}
	}

	fn with_label(self, label: Self::Label) -> Self {
		match self {
			Self::ParserUnexpectedEof { span, expected, .. } => Self::ParserUnexpectedEof {
				span,
				expected,
				label: Some(label),
			},

			Self::ParserExpectedToken {
				span, expected, found, ..
			} => Self::ParserExpectedToken {
				span,
				expected,
				found,
				label: Some(label),
			},

			Self::ParserUnclosedDelimiter {
				unclosed_span: span,
				unclosed,
				before_span,
				before,
				expected,
				..
			} => Self::ParserUnclosedDelimiter {
				unclosed_span: span,
				unclosed,
				before_span,
				before,
				expected,
				label: Some(label),
			},

			_ => unreachable!("Chumsky tried to add a label to a non-chumsky error"),
		}
	}

	fn unclosed_delimiter(
		unclosed_span: Self::Span,
		unclosed: Token<'a>,
		before_span: Self::Span,
		expected: Token<'a>,
		before: Option<Token<'a>>,
	) -> Self {
		Report::ParserUnclosedDelimiter {
			unclosed_span,
			unclosed,
			before_span,
			before,
			expected,
			label: None,
		}
	}

	fn merge(mut self, mut other: Self) -> Self {
		if let (
			Self::ParserExpectedToken { expected, .. },
			Self::ParserExpectedToken {
				expected: expected_other,
				..
			},
		) = (&mut self, &mut other)
		{
			for e in expected_other.drain() {
				expected.insert(e);
			}
		}

		self
	}
}

pub fn write_stdout(file_database: &FileDatabase, reports: &[Report]) {
	let writer = term::termcolor::StandardStream::stdout(term::termcolor::ColorChoice::Auto);
	let config = term::Config::default();

	for report in reports {
		term::emit(&mut writer.lock(), &config, file_database, &report.into())
			.expect("failed to write report to stdout");
	}
}

pub fn write_string(file_database: &FileDatabase, reports: &[Report]) -> String {
	let mut writer = term::termcolor::NoColor::new(Vec::new());
	let config = term::Config::default();

	for report in reports {
		term::emit(&mut writer, &config, file_database, &report.into()).expect("failed to write report to string");
	}

	String::from_utf8(writer.into_inner()).expect("failed to convert writer into string while writing reports")
}
