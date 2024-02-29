use std::collections::HashMap;

use ast::{visit, Ast, AstEnum, AstStruct, AstVisitor};
use meta::Report;

pub fn analyze<'a>(ast: &'a Ast<'a>) -> Vec<Report<'a>> {
	let mut analyzer = Analyzer { reports: Vec::new() };
	visit(ast, &mut analyzer);
	analyzer.reports
}

struct Analyzer<'a> {
	reports: Vec<Report<'a>>,
}

impl<'a> Analyzer<'a> {
	fn report(&mut self, report: Report<'a>) {
		self.reports.push(report);
	}
}

impl<'a> Analyzer<'a> {
	fn struct_duplicate_fields(&mut self, st: &'a AstStruct<'a>) {
		let mut fields = HashMap::new();

		for (field, _) in st.fields() {
			if let Some(first_span) = fields.insert(field.value(), field.span()) {
				self.report(Report::AnalysisDuplicateStructField {
					first_span,
					second_span: field.span(),
					field: field.value(),
				})
			}
		}
	}

	fn enum_duplicate_variants(&mut self, en: &'a AstEnum<'a>) {
		let mut variants_map = HashMap::new();

		match en {
			AstEnum::Unit { variants, .. } => {
				for variant in variants {
					if let Some(first_span) = variants_map.insert(variant.value(), variant.span()) {
						self.report(Report::AnalysisDuplicateEnumVariant {
							first_span,
							second_span: variant.span(),
							variant: variant.value(),
						})
					}
				}
			}

			AstEnum::Tagged { variants, .. } => {
				for (variant, _) in variants {
					if let Some(first_span) = variants_map.insert(variant.value(), variant.span()) {
						self.report(Report::AnalysisDuplicateEnumVariant {
							first_span,
							second_span: variant.span(),
							variant: variant.value(),
						})
					}
				}
			}
		}
	}

	fn tagged_enum_tag_as_field(&mut self, en: &'a AstEnum<'a>) {
		if let AstEnum::Tagged {
			field,
			variants,
			catch_all,
			..
		} = en
		{
			let tag = field.value_without_quotes();

			for (_, st) in variants {
				if let Some((name, _)) = st.field(tag) {
					self.report(Report::AnalysisTaggedEnumTagAsField {
						tag,
						tag_span: field.span(),
						field_span: name.span(),
					})
				}
			}

			if let Some(st) = catch_all {
				if let Some((name, _)) = st.field(tag) {
					self.report(Report::AnalysisTaggedEnumTagAsField {
						tag,
						tag_span: field.span(),
						field_span: name.span(),
					})
				}
			}
		} else {
			unreachable!("tagged_enum_tag_as_field called with non-tagged enum");
		}
	}
}

impl<'a> AstVisitor<'a> for Analyzer<'a> {
	fn visit_struct(&mut self, st: &'a AstStruct<'a>) {
		self.struct_duplicate_fields(st);
	}

	fn visit_enum(&mut self, en: &'a AstEnum<'a>) {
		self.enum_duplicate_variants(en);

		match en {
			AstEnum::Unit { .. } => {}

			AstEnum::Tagged { .. } => {
				self.tagged_enum_tag_as_field(en);
			}
		}
	}
}
