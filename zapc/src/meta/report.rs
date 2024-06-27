use std::fmt::Display;

use ariadne::{Color, Fmt};

use super::Span;

const ERROR: Color = Color::Red;
const WARNING: Color = Color::Yellow;
const INFO: Color = Color::Cyan;
const CORRECT: Color = Color::Green;

pub enum ReportKind {
	Error,
	Warning,
}

impl ReportKind {
	fn is_fatal(&self) -> bool {
		match self {
			Self::Error => true,
			Self::Warning => false,
		}
	}
}

impl<'a> From<ReportKind> for ariadne::ReportKind<'a> {
	fn from(value: ReportKind) -> Self {
		match value {
			ReportKind::Error => Self::Custom("error", ERROR),
			ReportKind::Warning => Self::Custom("warning", ERROR),
		}
	}
}

pub enum Report {
	UnknownCharacter {
		span: Span,
		char: char,
	},

	ExpectedTokenFound {
		expected: Vec<String>,
		found: String,
		label: Option<&'static str>,
		span: Span,
	},

	InvalidRange {
		span: Span,
		min_span: Span,
	},

	ExpectedIntegerFoundNumber {
		span: Span,
		value: f64,
	},

	NumberAboveRange {
		span: Span,
		max: Box<dyn Display>,
	},

	NumberBelowRange {
		span: Span,
		min: Box<dyn Display>,
	},

	DuplicateDecl {
		decl_kind: String,
		name: String,
		span: Span,
		first_decl_span: Span,
	},

	DuplicateField {
		span: Span,
		first_field_span: Span,
		field: String,
	},

	UnexpectedField {
		span: Span,
		field: String,
		decl_kind: String,
	},

	ExpectedField {
		span: Span,
		field: String,
		decl_name: String,
		decl_kind: String,
	},

	ExpectedValue {
		value_span: Span,
		found: String,
		expected_values: String,
	},

	ExpectedValueWrongType {
		value_span: Span,
		found_type: String,
		expected_values: String,
	},

	ExpectedType {
		value_span: Span,
		expected_type: String,
		found_type: String,
	},

	IncorrectGenericCount {
		type_span: Span,
		type_name: String,
		generic_spans: Vec<Span>,
		generics_optional: bool,
		expected_count: usize,
		count: usize,
	},
}

fn build<'a>(kind: ReportKind, span: Span) -> ariadne::ReportBuilder<'a, Span> {
	ariadne::Report::build(kind.into(), span.file(), span.start())
}

fn label(span: Span) -> ariadne::Label<Span> {
	ariadne::Label::new(span)
}

fn ticks(s: impl Display) -> String {
	format!("`{s}`")
}

impl Report {
	pub fn kind(&self) -> ReportKind {
		match self {
			Self::UnknownCharacter { .. } => ReportKind::Error,
			Self::ExpectedTokenFound { .. } => ReportKind::Error,
			Self::InvalidRange { .. } => ReportKind::Error,
			Self::ExpectedIntegerFoundNumber { .. } => ReportKind::Error,
			Self::NumberAboveRange { .. } => ReportKind::Error,
			Self::NumberBelowRange { .. } => ReportKind::Error,
			Self::DuplicateDecl { .. } => ReportKind::Error,
			Self::DuplicateField { .. } => ReportKind::Error,
			Self::UnexpectedField { .. } => ReportKind::Error,
			Self::ExpectedField { .. } => ReportKind::Error,
			Self::ExpectedValue { .. } => ReportKind::Error,
			Self::ExpectedValueWrongType { .. } => ReportKind::Error,
			Self::ExpectedType { .. } => ReportKind::Error,
			Self::IncorrectGenericCount { .. } => ReportKind::Error,
		}
	}

	pub fn into_ariadne<'a>(self) -> ariadne::Report<'a, Span> {
		let kind = self.kind();

		match self {
			Self::UnknownCharacter { span, char } => build(kind, span)
				.with_message(format!("unknown character {}", ticks(char).fg(ERROR)))
				.with_label(
					label(span)
						.with_color(ERROR)
						.with_message(format!("unknown character {}", ticks(char).fg(ERROR))),
				),

			Self::ExpectedTokenFound {
				expected,
				found,
				label,
				span,
			} => {
				let expected_text = format!(
					"{}{}",
					if expected.len() == 1 { "" } else { "one of " },
					expected
						.iter()
						.map(|t| t.fg(CORRECT).to_string())
						.collect::<Vec<_>>()
						.join(", ")
				);

				build(kind, span)
					.with_message(format!(
						"unexpected token {}{}, expected {expected_text}",
						found.fg(ERROR),
						label.map_or("".to_string(), |l| format!(" while parsing {}", l.fg(INFO)))
					))
					.with_label(
						self::label(span)
							.with_color(ERROR)
							.with_message(format!("expected {expected_text}")),
					)
			}

			Self::InvalidRange { span, min_span } => build(kind, span).with_message("invalid range").with_label(
				label(min_span)
					.with_color(ERROR)
					.with_message("minimum value here is larger than maximum value"),
			),

			Self::ExpectedIntegerFoundNumber { span, value } => build(kind, span)
				.with_message(format!("expected integer number, found {}", ticks(value).fg(ERROR)))
				.with_label(label(span).with_color(ERROR).with_message("expected integer number"))
				.with_help(format!("did you mean to use {}", ticks(value.trunc()).fg(CORRECT))),

			Self::NumberAboveRange { span, max } => build(kind, span)
				.with_message(format!("number is above maximum value of {}", ticks(&max).fg(INFO)))
				.with_label(label(span).with_color(ERROR).with_message("number is above maximum"))
				.with_help(format!("try lowering your value to {}", ticks(&max).fg(INFO))),

			Self::NumberBelowRange { span, min } => build(kind, span)
				.with_message(format!("number is below minimum value of {}", ticks(&min).fg(INFO)))
				.with_label(label(span).with_color(ERROR).with_message("number is below minimum"))
				.with_help(format!("try raising your value to {}", ticks(&min).fg(INFO))),

			Self::DuplicateDecl {
				decl_kind,
				name,
				span,
				first_decl_span,
			} => build(kind, span)
				.with_message(format!(
					"the {decl_kind} {} is defined multiple times in the same scope",
					ticks(&name).fg(ERROR)
				))
				.with_labels([
					label(first_decl_span)
						.with_message(format!("{decl_kind} {} is first defined here", ticks(&name).fg(INFO)))
						.with_color(INFO),
					label(span)
						.with_color(ERROR)
						.with_message(format!("{decl_kind} {} later redefined here", ticks(&name).fg(ERROR))),
				]),

			Self::DuplicateField {
				span,
				first_field_span,
				field,
			} => build(kind, span)
				.with_message(format!("field {} is already declared", ticks(&field).fg(ERROR)))
				.with_labels([
					label(first_field_span)
						.with_message(format!("{} first declared here", ticks(&field).fg(INFO)))
						.with_color(INFO),
					label(span)
						.with_message(format!("{} is already declared", ticks(&field).fg(ERROR)))
						.with_color(ERROR),
				]),

			Self::UnexpectedField { span, field, decl_kind } => build(kind, span)
				.with_message(format!("{} have no field named {}", decl_kind, ticks(field).fg(ERROR)))
				.with_label(
					label(span)
						.with_message(format!("{} do not have this field", decl_kind))
						.with_color(ERROR),
				),

			Self::ExpectedField {
				span,
				field,
				decl_name,
				decl_kind,
			} => build(kind, span)
				.with_message(format!(
					"missing field {} in {} {}",
					ticks(&field).fg(ERROR),
					decl_kind,
					ticks(decl_name).fg(ERROR)
				))
				.with_label(
					label(span)
						.with_message(format!("missing {}", ticks(field).fg(ERROR)))
						.with_color(ERROR),
				),

			Self::ExpectedValue {
				value_span,
				found,
				expected_values,
			} => build(kind, value_span).with_message("invalid value").with_label(
				label(value_span)
					.with_message(format!("expected {}, found {}", expected_values, found.fg(ERROR)))
					.with_color(ERROR),
			),

			Self::ExpectedValueWrongType {
				value_span,
				found_type,
				expected_values,
			} => build(kind, value_span).with_message("invalid value").with_label(
				label(value_span)
					.with_message(format!(
						"expected {}, found {}",
						expected_values,
						ticks(found_type).fg(ERROR)
					))
					.with_color(ERROR),
			),

			Self::ExpectedType {
				value_span,
				expected_type,
				found_type,
			} => build(kind, value_span).with_message("mismatched types").with_label(
				label(value_span)
					.with_message(format!(
						"expected {}, found {}",
						expected_type,
						ticks(found_type).fg(ERROR)
					))
					.with_color(ERROR),
			),

			Self::IncorrectGenericCount {
				type_span,
				type_name,
				generic_spans,
				generics_optional,
				expected_count,
				count,
			} => {
				let mut labels = vec![label(type_span).with_color(ERROR).with_message(format!(
					"expected {}{expected_count} generic {}",
					if generics_optional { "at most " } else { "" },
					if expected_count == 1 { "argument" } else { "arguments" }
				))];

				if count > expected_count {
					let start = generic_spans[expected_count];
					let end = generic_spans.last().unwrap();

					let span = start.merge(*end);

					labels.push(
						label(span)
							.with_color(INFO)
							.with_message(if count - expected_count == 1 {
								"remove this generic argument"
							} else {
								"remove these generic arguments"
							}),
					);
				}

				build(kind, type_span)
					.with_message(format!(
						"type {} takes {}{expected_count} generic {} but {count} generic {} supplied",
						ticks(type_name).fg(ERROR),
						if generics_optional { "at most " } else { "" },
						if expected_count == 1 { "argument" } else { "arguments" },
						if count == 1 { "argument was" } else { "arguments were" },
					))
					.with_labels(labels)
			}
		}
		.finish()
	}
}
