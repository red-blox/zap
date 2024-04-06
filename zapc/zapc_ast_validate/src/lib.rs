use std::collections::HashMap;

use lasso::RodeoReader;
use zapc_ast::{visit, Ast, AstDecl, AstRange, AstTy, AstTyTable, AstVisitor, AstWord, BUILTIN_TYPES};
use zapc_meta::Report;

pub fn validate(ast: &Ast, rodeo: &RodeoReader) -> Vec<Report> {
	let mut validator = Validator {
		reports: Vec::new(),
		rodeo,
	};

	visit(ast, &mut validator);
	validator.reports
}

struct Validator<'a> {
	reports: Vec<Report>,
	rodeo: &'a RodeoReader,
}

impl<'a> Validator<'a> {
	fn emit(&mut self, report: Report) {
		self.reports.push(report);
	}
}

impl<'a> Validator<'a> {
	fn tagged_enum_tag_as_field(&mut self, ty: &AstTy) {
		if let AstTy::TaggedEnum {
			tag,
			variants,
			catch_all,
			..
		} = ty
		{
			for (_, table) in variants {
				for (field_name, _) in table.fields() {
					if field_name.spur() == tag.spur() {
						self.emit(Report::TaggedEnumTagAsField {
							tag: tag.str(self.rodeo).to_string(),
							tag_span: tag.span(),
							field_span: field_name.span(),
						})
					}
				}
			}

			if let Some(table) = catch_all {
				for (field_name, _) in table.fields() {
					if field_name.spur() == tag.spur() {
						self.emit(Report::TaggedEnumTagAsField {
							tag: tag.str(self.rodeo).to_string(),
							tag_span: tag.span(),
							field_span: field_name.span(),
						})
					}
				}
			}
		}
	}

	fn duplicate_name(&mut self, names: impl IntoIterator<Item = AstWord>) {
		let mut seen = HashMap::new();

		for name in names {
			let spur = name.spur();

			if let Some(prev) = seen.get(&spur) {
				self.emit(Report::MultipleNameDefinition {
					name: name.str(self.rodeo).to_string(),
					first_span: *prev,
					second_span: name.span(),
				})
			} else {
				seen.insert(spur, name.span());
			}
		}
	}

	fn enum_duplicate_variant(&mut self, ty: &AstTy) {
		if let AstTy::TaggedEnum { variants, .. } = ty {
			self.duplicate_name(variants.iter().map(|(name, _)| *name));
		} else if let AstTy::UnitEnum { variants, .. } = ty {
			self.duplicate_name(variants.iter().copied());
		}
	}

	fn ty_table_duplicate_field(&mut self, table: &AstTyTable) {
		self.duplicate_name(table.fields().iter().map(|(name, _)| *name));
	}

	fn scope(&mut self, scope: &[AstDecl]) {
		let names = scope.iter().filter_map(|decl| match *decl {
			AstDecl::Ty { name, .. } => Some(name),
			AstDecl::Mod { name, .. } => Some(name),
			AstDecl::Event { name, .. } => Some(name),
			AstDecl::Funct { name, .. } => Some(name),

			AstDecl::Err(_) => None,
		});

		self.duplicate_name(names.clone());

		for name in names {
			let spur = name.spur();

			for builtin in BUILTIN_TYPES {
				if self.rodeo.get(builtin).is_some_and(|builtin_spur| spur == builtin_spur) {
					self.emit(Report::BuiltinNameConflict {
						builtin,
						span: name.span(),
					});

					break;
				}
			}
		}
	}

	fn invalid_or_exact_range(&mut self, range: &AstRange) {
		if let AstRange::WithMinMax(span, min, max) = range {
			if min.value() > max.value() {
				self.emit(Report::InvalidRange { span: *span })
			} else if min.value() == max.value() {
				self.emit(Report::UseExactRange {
					value: min.value(),
					span: *span,
				})
			}
		}
	}
}

impl<'a> AstVisitor for Validator<'a> {
	fn visit_ast(&mut self, ast: &Ast) {
		self.scope(ast.decls());
	}

	fn visit_decl(&mut self, decl: &AstDecl) {
		if let AstDecl::Mod { decls, .. } = decl {
			self.scope(decls);
		}
	}

	fn visit_ty(&mut self, ty: &AstTy) {
		self.tagged_enum_tag_as_field(ty);
		self.enum_duplicate_variant(ty);
	}

	fn visit_ty_table(&mut self, table: &AstTyTable) {
		self.ty_table_duplicate_field(table);
	}

	fn visit_range(&mut self, range: &AstRange) {
		self.invalid_or_exact_range(range);
	}
}
