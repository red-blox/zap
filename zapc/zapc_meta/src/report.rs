use ariadne::{Color, Fmt};

use crate::Span;

const ERROR: Color = Color::Red;
const WARNING: Color = Color::Yellow;
const INFO: Color = Color::Cyan;
const CORRECT: Color = Color::Green;

pub enum ReportKind {
	Error,
	Warning,
	Advice,
}

impl ReportKind {
	pub fn fatal(&self) -> bool {
		match self {
			Self::Error => true,
			Self::Warning => false,
			Self::Advice => false,
		}
	}
}

impl From<ReportKind> for ariadne::ReportKind<'_> {
	fn from(value: ReportKind) -> Self {
		match value {
			ReportKind::Error => ariadne::ReportKind::Error,
			ReportKind::Warning => ariadne::ReportKind::Warning,
			ReportKind::Advice => ariadne::ReportKind::Advice,
		}
	}
}

#[derive(Debug)]
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

	TaggedEnumTagAsField {
		tag: String,
		tag_span: Span,
		field_span: Span,
	},

	MultipleNameDefinition {
		name: String,
		first_span: Span,
		second_span: Span,
	},

	InvalidRange {
		span: Span,
	},

	UseExactRange {
		value: f64,
		span: Span,
	},

	BuiltinNameConflict {
		builtin: &'static str,
		span: Span,
	},
}

fn build<'a>(kind: ReportKind, span: Span) -> ariadne::ReportBuilder<'a, Span> {
	ariadne::Report::build(kind.into(), span.file(), span.start())
}

fn lb(span: Span) -> ariadne::Label<Span> {
	ariadne::Label::new(span)
}

impl Report {
	pub fn kind(&self) -> ReportKind {
		match self {
			Self::UnknownCharacter { .. } => ReportKind::Error,
			Self::ExpectedTokenFound { .. } => ReportKind::Error,
			Self::TaggedEnumTagAsField { .. } => ReportKind::Error,
			Self::MultipleNameDefinition { .. } => ReportKind::Error,
			Self::InvalidRange { .. } => ReportKind::Error,
			Self::UseExactRange { .. } => ReportKind::Advice,
			Self::BuiltinNameConflict { .. } => ReportKind::Error,
		}
	}

	pub fn into_ariadne<'a>(self) -> ariadne::Report<'a, Span> {
		let kind = self.kind();

		match self {
			Self::UnknownCharacter { span, char } => build(kind, span)
				.with_message(format!("unknown character {}", format!("`{char}`").fg(ERROR)))
				.with_label(
					lb(span)
						.with_color(ERROR)
						.with_message(format!("unknown character {}", format!("`{char}`").fg(ERROR))),
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
						"unexpected {}{}, expected {expected_text}",
						found.fg(ERROR),
						if let Some(l) = label {
							format!("while parsing {}", l.fg(INFO))
						} else {
							"".to_string()
						},
					))
					.with_label(
						lb(span)
							.with_color(ERROR)
							.with_message(format!("expected {expected_text}")),
					)
			}

			Self::TaggedEnumTagAsField {
				tag,
				tag_span,
				field_span,
			} => build(kind, field_span)
				.with_message(format!("tagged enum tag `{}` used as field", tag.fg(ERROR)))
				.with_label(lb(tag_span).with_color(INFO).with_message("tag declared"))
				.with_label(lb(field_span).with_color(ERROR).with_message("tag used as field")),

			Self::MultipleNameDefinition {
				ref name,
				first_span,
				second_span,
			} => build(kind, first_span)
				.with_message(format!("name `{}` defined multiple times", name.fg(INFO)))
				.with_label(
					lb(first_span)
						.with_color(INFO)
						.with_message(format!("`{}` first defined here", name.fg(INFO))),
				)
				.with_label(
					lb(second_span)
						.with_color(ERROR)
						.with_message(format!("`{}` redefined here", name.fg(ERROR))),
				),

			Self::InvalidRange { span } => build(kind, span)
				.with_message("invalid range")
				.with_label(lb(span).with_color(ERROR).with_message("invalid range here"))
				.with_note("the minimum value of a range must be less than or equal to the maximum value"),

			Self::UseExactRange { value, span } => build(kind, span).with_message("use an exact range").with_label(
				lb(span).with_color(WARNING).with_message(format!(
					"use `{}` instead of `{}`",
					format!("({value})").fg(CORRECT),
					format!("({value}..{value})").fg(ERROR),
				)),
			),

			Self::BuiltinNameConflict { builtin, span } => build(kind, span)
				.with_message(format!("conflict with builtin name `{}`", builtin.fg(ERROR)))
				.with_label(
					lb(span)
						.with_color(ERROR)
						.with_message(format!("builtin name `{}`", builtin.fg(ERROR))),
				)
				.with_note("builtin names cannot be redefined"),
		}
		.finish()
	}
}
