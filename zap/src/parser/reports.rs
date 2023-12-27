use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use lalrpop_util::lexer::Token;

pub type Span = core::ops::Range<usize>;

#[derive(Debug, Clone)]
pub enum Report<'src> {
	LexerInvalidToken {
		span: Span,
	},

	ParserUnexpectedEOF {
		span: Span,
		expected: Vec<String>,
	},

	ParserUnexpectedToken {
		span: Span,
		expected: Vec<String>,
		token: Token<'src>,
	},

	ParserExtraToken {
		span: Span,
	},

	ParserExpectedInt {
		span: Span,
	},

	SemanticOversizeUnreliable {
		ev_span: Span,
		ty_span: Span,
		max_size: usize,
		size: usize,
	},

	SemanticPotentiallyOversizeUnreliable {
		ev_span: Span,
		ty_span: Span,
		max_size: usize,
	},

	SemanticInvalidRange {
		span: Span,
	},

	SemanticEmptyEnum {
		span: Span,
	},

	SemanticEnumTagUsed {
		tag_span: Span,
		used_span: Span,
		tag: &'src str,
	},

	SemanticInvalidOptValue {
		span: Span,
		expected: &'static str,
	},

	SemanticUnknownOptName {
		span: Span,
	},

	SemanticUnknownTypeRef {
		span: Span,
		name: &'src str,
	},

	SemanticNumOutsideRange {
		span: Span,
		min: f64,
		max: f64,
	},
}

impl<'src> Report<'src> {
	pub fn severity(&self) -> Severity {
		match self {
			Self::LexerInvalidToken { .. } => Severity::Error,

			Self::ParserUnexpectedEOF { .. } => Severity::Error,
			Self::ParserUnexpectedToken { .. } => Severity::Error,
			Self::ParserExtraToken { .. } => Severity::Error,
			Self::ParserExpectedInt { .. } => Severity::Error,

			Self::SemanticOversizeUnreliable { .. } => Severity::Error,
			Self::SemanticPotentiallyOversizeUnreliable { .. } => Severity::Warning,
			Self::SemanticInvalidRange { .. } => Severity::Error,
			Self::SemanticEmptyEnum { .. } => Severity::Error,
			Self::SemanticEnumTagUsed { .. } => Severity::Error,
			Self::SemanticInvalidOptValue { .. } => Severity::Error,
			Self::SemanticUnknownOptName { .. } => Severity::Warning,
			Self::SemanticUnknownTypeRef { .. } => Severity::Error,
			Self::SemanticNumOutsideRange { .. } => Severity::Error,
		}
	}

	fn message(&self) -> String {
		match self {
			Self::LexerInvalidToken { .. } => "invalid token".to_string(),

			Self::ParserUnexpectedEOF { expected, .. } => {
				format!("expected {}, found end of file", expected.join(", "))
			}

			Self::ParserUnexpectedToken { expected, .. } => format!("expected {}", expected.join(", ")),
			Self::ParserExtraToken { .. } => "extra token".to_string(),
			Self::ParserExpectedInt { .. } => "expected integer".to_string(),

			Self::SemanticOversizeUnreliable { .. } => "oversize unreliable".to_string(),
			Self::SemanticPotentiallyOversizeUnreliable { .. } => "potentially oversize unreliable".to_string(),
			Self::SemanticInvalidRange { .. } => "invalid range".to_string(),
			Self::SemanticEmptyEnum { .. } => "empty enum".to_string(),
			Self::SemanticEnumTagUsed { .. } => "enum tag used in variant".to_string(),
			Self::SemanticInvalidOptValue { expected, .. } => format!("invalid opt value, expected {}", expected),
			Self::SemanticUnknownOptName { .. } => "unknown opt name".to_string(),
			Self::SemanticUnknownTypeRef { name, .. } => format!("unknown type reference '{}'", name),
			Self::SemanticNumOutsideRange { .. } => "number outside range".to_string(),
		}
	}

	fn code(&self) -> &str {
		match self {
			Self::LexerInvalidToken { .. } => "lex:001",

			Self::ParserUnexpectedEOF { .. } => "parse:002",
			Self::ParserUnexpectedToken { .. } => "parse:003",
			Self::ParserExtraToken { .. } => "parse:004",
			Self::ParserExpectedInt { .. } => "parse:005",

			Self::SemanticOversizeUnreliable { .. } => "analyze:006",
			Self::SemanticPotentiallyOversizeUnreliable { .. } => "analyze:007",
			Self::SemanticInvalidRange { .. } => "analyze:008",
			Self::SemanticEmptyEnum { .. } => "analyze:009",
			Self::SemanticEnumTagUsed { .. } => "analyze:010",
			Self::SemanticInvalidOptValue { .. } => "analyze:011",
			Self::SemanticUnknownOptName { .. } => "analyze:012",
			Self::SemanticUnknownTypeRef { .. } => "analyze:013",
			Self::SemanticNumOutsideRange { .. } => "analyze:014",
		}
	}

	fn labels(&self) -> Vec<Label<()>> {
		match self {
			Self::LexerInvalidToken { span } => vec![Label::primary((), span.clone()).with_message("invalid token")],

			Self::ParserUnexpectedEOF { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("unexpected end of file")]
			}

			Self::ParserUnexpectedToken { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("unexpected token")]
			}

			Self::ParserExtraToken { span } => {
				vec![Label::primary((), span.clone()).with_message("extra token")]
			}

			Self::ParserExpectedInt { span } => {
				vec![Label::primary((), span.clone()).with_message("expected integer")]
			}

			Self::SemanticOversizeUnreliable { ev_span, ty_span, .. } => {
				vec![
					Label::primary((), ev_span.clone()).with_message("event is unreliable"),
					Label::secondary((), ty_span.clone()).with_message("type is too large"),
				]
			}

			Self::SemanticPotentiallyOversizeUnreliable { ev_span, ty_span, .. } => {
				vec![
					Label::primary((), ev_span.clone()).with_message("event is unreliable"),
					Label::secondary((), ty_span.clone()).with_message("type may be too large"),
				]
			}

			Self::SemanticInvalidRange { span } => {
				vec![Label::primary((), span.clone()).with_message("invalid range")]
			}

			Self::SemanticEmptyEnum { span } => {
				vec![Label::primary((), span.clone()).with_message("empty enum")]
			}

			Self::SemanticEnumTagUsed {
				tag_span, used_span, ..
			} => {
				vec![
					Label::primary((), tag_span.clone()).with_message("enum tag"),
					Label::secondary((), used_span.clone()).with_message("used in variant"),
				]
			}

			Self::SemanticInvalidOptValue { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("invalid opt value")]
			}

			Self::SemanticUnknownOptName { span } => {
				vec![Label::primary((), span.clone()).with_message("unknown opt name")]
			}

			Self::SemanticUnknownTypeRef { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("unknown type reference")]
			}

			Self::SemanticNumOutsideRange { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("number outside range")]
			}
		}
	}

	fn notes(&self) -> Option<Vec<String>> {
		match self {
			Self::LexerInvalidToken { .. } => None,

			Self::ParserUnexpectedEOF { .. } => None,
			Self::ParserUnexpectedToken { .. } => None,
			Self::ParserExtraToken { .. } => None,
			Self::ParserExpectedInt { .. } => None,

			Self::SemanticOversizeUnreliable { max_size, .. } => Some(vec![
				format!("all unreliable events must be under {max_size} bytes in size"),
				"consider adding a upper limit to any arrays or strings".to_string(),
				"upper limits can be added for arrays by doing `[..10]`".to_string(),
				"upper limits can be added for strings by doing `[..10]`".to_string(),
			]),
			Self::SemanticPotentiallyOversizeUnreliable { max_size, .. } => Some(vec![
				format!("all unreliable events must be under {max_size} bytes in size"),
				"consider adding a upper limit to any arrays or strings".to_string(),
				"upper limits can be added for arrays by doing `[..10]`".to_string(),
				"upper limits can be added for strings by doing `(..10)`".to_string(),
			]),
			Self::SemanticInvalidRange { .. } => Some(vec![
				"ranges must be in the form `min..max`".to_string(),
				"ranges can be invalid if `min` is greater than `max`".to_string(),
			]),
			Self::SemanticEmptyEnum { .. } => Some(vec![
				"enums cannot be empty".to_string(),
				"if you're looking to create an empty type, use a struct with no fields".to_string(),
				"a struct with no fields can be created by doing `struct {}`".to_string(),
			]),
			Self::SemanticEnumTagUsed { .. } => Some(vec![
				"tagged enums use the tag field in passed structs to determine what variant to use".to_string(),
				"you cannot override this tag field in a variant as that would break this behavior".to_string(),
			]),
			Self::SemanticInvalidOptValue { .. } => None,
			Self::SemanticUnknownOptName { .. } => None,
			Self::SemanticUnknownTypeRef { .. } => None,
			Self::SemanticNumOutsideRange { min, max, .. } => Some(vec![
				format!("(inclusive) min: {}", min),
				format!("(inclusive) max: {}", max),
			]),
		}
	}
}

impl<'src> From<Report<'src>> for Diagnostic<()> {
	fn from(val: Report<'src>) -> Self {
		let diagnostic = Diagnostic::new(val.severity())
			.with_code(val.code())
			.with_message(val.message())
			.with_labels(val.labels());

		if let Some(notes) = val.notes() {
			diagnostic.with_notes(notes)
		} else {
			diagnostic
		}
	}
}
