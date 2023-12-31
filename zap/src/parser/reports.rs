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

	AnalyzeEmptyEvDecls,

	AnalyzeOversizeUnreliable {
		ev_span: Span,
		ty_span: Span,
		max_size: usize,
		size: usize,
	},

	AnalyzePotentiallyOversizeUnreliable {
		ev_span: Span,
		ty_span: Span,
		max_size: usize,
	},

	AnalyzeInvalidRange {
		span: Span,
	},

	AnalyzeEmptyEnum {
		span: Span,
	},

	AnalyzeEnumTagUsed {
		tag_span: Span,
		used_span: Span,
		tag: &'src str,
	},

	AnalyzeInvalidOptValue {
		span: Span,
		expected: &'static str,
	},

	AnalyzeUnknownOptName {
		span: Span,
	},

	AnalyzeUnknownTypeRef {
		span: Span,
		name: &'src str,
	},

	AnalyzeNumOutsideRange {
		span: Span,
		min: f64,
		max: f64,
	},

	AnalyzeInvalidOptionalType {
		span: Span,
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

			Self::AnalyzeEmptyEvDecls => Severity::Warning,
			Self::AnalyzeOversizeUnreliable { .. } => Severity::Error,
			Self::AnalyzePotentiallyOversizeUnreliable { .. } => Severity::Warning,
			Self::AnalyzeInvalidRange { .. } => Severity::Error,
			Self::AnalyzeEmptyEnum { .. } => Severity::Error,
			Self::AnalyzeEnumTagUsed { .. } => Severity::Error,
			Self::AnalyzeInvalidOptValue { .. } => Severity::Error,
			Self::AnalyzeUnknownOptName { .. } => Severity::Warning,
			Self::AnalyzeUnknownTypeRef { .. } => Severity::Error,
			Self::AnalyzeNumOutsideRange { .. } => Severity::Error,
			Self::AnalyzeInvalidOptionalType { .. } => Severity::Error,
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

			Self::AnalyzeEmptyEvDecls => "no event declarations".to_string(),
			Self::AnalyzeOversizeUnreliable { .. } => "oversize unreliable".to_string(),
			Self::AnalyzePotentiallyOversizeUnreliable { .. } => "potentially oversize unreliable".to_string(),
			Self::AnalyzeInvalidRange { .. } => "invalid range".to_string(),
			Self::AnalyzeEmptyEnum { .. } => "empty enum".to_string(),
			Self::AnalyzeEnumTagUsed { .. } => "enum tag used in variant".to_string(),
			Self::AnalyzeInvalidOptValue { expected, .. } => format!("invalid opt value, expected {}", expected),
			Self::AnalyzeUnknownOptName { .. } => "unknown opt name".to_string(),
			Self::AnalyzeUnknownTypeRef { name, .. } => format!("unknown type reference '{}'", name),
			Self::AnalyzeNumOutsideRange { .. } => "number outside range".to_string(),
			Self::AnalyzeInvalidOptionalType { .. } => "type must not be optional".to_string(),
		}
	}

	fn code(&self) -> &str {
		match self {
			Self::LexerInvalidToken { .. } => "1001",

			Self::ParserUnexpectedEOF { .. } => "2001",
			Self::ParserUnexpectedToken { .. } => "2002",
			Self::ParserExtraToken { .. } => "2003",
			Self::ParserExpectedInt { .. } => "2004",

			Self::AnalyzeEmptyEvDecls { .. } => "3001",
			Self::AnalyzeOversizeUnreliable { .. } => "3002",
			Self::AnalyzePotentiallyOversizeUnreliable { .. } => "3003",
			Self::AnalyzeInvalidRange { .. } => "3004",
			Self::AnalyzeEmptyEnum { .. } => "3005",
			Self::AnalyzeEnumTagUsed { .. } => "3006",
			Self::AnalyzeInvalidOptValue { .. } => "3007",
			Self::AnalyzeUnknownOptName { .. } => "3008",
			Self::AnalyzeUnknownTypeRef { .. } => "3009",
			Self::AnalyzeNumOutsideRange { .. } => "3010",
			Self::AnalyzeInvalidOptionalType { .. } => "3011",
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

			Self::AnalyzeEmptyEvDecls => vec![],

			Self::AnalyzeOversizeUnreliable { ev_span, ty_span, .. } => {
				vec![
					Label::primary((), ev_span.clone()).with_message("event is unreliable"),
					Label::secondary((), ty_span.clone()).with_message("type is too large"),
				]
			}

			Self::AnalyzePotentiallyOversizeUnreliable { ev_span, ty_span, .. } => {
				vec![
					Label::primary((), ev_span.clone()).with_message("event is unreliable"),
					Label::secondary((), ty_span.clone()).with_message("type may be too large"),
				]
			}

			Self::AnalyzeInvalidRange { span } => {
				vec![Label::primary((), span.clone()).with_message("invalid range")]
			}

			Self::AnalyzeEmptyEnum { span } => {
				vec![Label::primary((), span.clone()).with_message("empty enum")]
			}

			Self::AnalyzeEnumTagUsed {
				tag_span, used_span, ..
			} => {
				vec![
					Label::primary((), tag_span.clone()).with_message("enum tag"),
					Label::secondary((), used_span.clone()).with_message("used in variant"),
				]
			}

			Self::AnalyzeInvalidOptValue { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("invalid opt value")]
			}

			Self::AnalyzeUnknownOptName { span } => {
				vec![Label::primary((), span.clone()).with_message("unknown opt name")]
			}

			Self::AnalyzeUnknownTypeRef { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("unknown type reference")]
			}

			Self::AnalyzeNumOutsideRange { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("number outside range")]
			}

			Self::AnalyzeInvalidOptionalType { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("must be removed")]
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

			Self::AnalyzeEmptyEvDecls => Some(vec!["add an event declaration to allow zap to output code".to_string()]),
			Self::AnalyzeOversizeUnreliable { max_size, .. } => Some(vec![
				format!("all unreliable events must be under {max_size} bytes in size"),
				"consider adding a upper limit to any arrays or strings".to_string(),
				"upper limits can be added for arrays by doing `[..10]`".to_string(),
				"upper limits can be added for strings by doing `[..10]`".to_string(),
			]),
			Self::AnalyzePotentiallyOversizeUnreliable { max_size, .. } => Some(vec![
				format!("all unreliable events must be under {max_size} bytes in size"),
				"consider adding a upper limit to any arrays or strings".to_string(),
				"upper limits can be added for arrays by doing `[..10]`".to_string(),
				"upper limits can be added for strings by doing `(..10)`".to_string(),
			]),
			Self::AnalyzeInvalidRange { .. } => Some(vec![
				"ranges must be in the form `min..max`".to_string(),
				"ranges can be invalid if `min` is greater than `max`".to_string(),
			]),
			Self::AnalyzeEmptyEnum { .. } => Some(vec![
				"enums cannot be empty".to_string(),
				"if you're looking to create an empty type, use a struct with no fields".to_string(),
				"a struct with no fields can be created by doing `struct {}`".to_string(),
			]),
			Self::AnalyzeEnumTagUsed { .. } => Some(vec![
				"tagged enums use the tag field in passed structs to determine what variant to use".to_string(),
				"you cannot override this tag field in a variant as that would break this behavior".to_string(),
			]),
			Self::AnalyzeInvalidOptValue { .. } => None,
			Self::AnalyzeUnknownOptName { .. } => None,
			Self::AnalyzeUnknownTypeRef { .. } => None,
			Self::AnalyzeNumOutsideRange { min, max, .. } => Some(vec![
				format!("(inclusive) min: {}", min),
				format!("(inclusive) max: {}", max),
			]),
			Self::AnalyzeInvalidOptionalType { .. } => Some(vec![
				"this type can only be used when it is not marked as optional".to_string(),
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
