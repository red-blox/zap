use std::collections::HashMap;

use crate::{
	ast::{
		primitive::AstWord,
		range::AstRange,
		ty::{AstGeneric, AstStruct, AstTy},
	},
	hir::ty::{HirStruct, HirTy},
	meta::Report,
	ty::NumberTy,
};

use super::{scope::ScopeId, HirBuilder};

impl<'a> HirBuilder<'a> {
	pub fn ty(&mut self, scope: &ScopeId, ast: AstTy) -> HirTy {
		match ast {
			AstTy::Path {
				segments,
				generics,
				span,
			} => 'path: {
				if segments.len() == 1 {
					if let Some(ty) = self.single_segment_std_path(segments.first().unwrap().clone(), &generics) {
						break 'path ty;
					}
				}

				if !generics.is_empty() {
					let last_segment = segments.last().unwrap();

					self.report(Report::IncorrectGenericCount {
						type_span: last_segment.span(),
						type_name: last_segment.word(self.rodeo).to_string(),
						generic_spans: generics.iter().map(|s| s.span()).collect::<Vec<_>>(),
						generics_optional: false,
						expected_count: 0,
						count: generics.len(),
					})
				}

				HirTy::Reference(self.get_ty_id(scope, &segments, span))
			}

			AstTy::Struct { strukt, .. } => HirTy::Struct(self.strukt(scope, strukt)),
		}
	}

	fn strukt(&mut self, scope: &ScopeId, ast: AstStruct) -> HirStruct {
		let mut fields = HashMap::new();
		let mut seen = HashMap::new();

		for (field, ty) in ast.into_fields() {
			let span = field.span().merge(ty.span());

			if let Some(prev_span) = seen.insert(field.spur(), span) {
				self.report(Report::DuplicateField {
					span,
					first_field_span: prev_span,
					field: field.word(self.rodeo).to_string(),
				});
			} else {
				fields.insert(field.spur(), self.ty(scope, ty));
			}
		}

		HirStruct::new(fields)
	}

	fn single_segment_std_path(&mut self, segment: AstWord, generics: &[AstGeneric]) -> Option<HirTy> {
		match segment.word(self.rodeo) {
			"boolean" => {
				if !generics.is_empty() {
					self.report(Report::IncorrectGenericCount {
						type_span: segment.span(),
						type_name: segment.word(self.rodeo).to_string(),
						generic_spans: generics.iter().map(|s| s.span()).collect::<Vec<_>>(),
						generics_optional: false,
						expected_count: 0,
						count: generics.len(),
					})
				}

				Some(HirTy::Boolean)
			}

			"u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "f32" | "f64" => Some(self.std_number_ty(segment, generics)),

			"buffer" => Some(HirTy::Buffer(
				self.generics_one_range(&segment, generics)
					.map(|r| self.range_u16(r))
					.unwrap_or_default(),
			)),

			_ => None,
		}
	}

	fn std_number_ty(&mut self, segment: AstWord, generics: &[AstGeneric]) -> HirTy {
		let range = self.generics_one_range(&segment, generics);

		HirTy::Number(match segment.word(self.rodeo) {
			"u8" => NumberTy::U8(range.map(|r| self.range_u8(r)).unwrap_or_default()),
			"i8" => NumberTy::I8(range.map(|r| self.range_i8(r)).unwrap_or_default()),
			"u16" => NumberTy::U16(range.map(|r| self.range_u16(r)).unwrap_or_default()),
			"i16" => NumberTy::I16(range.map(|r| self.range_i16(r)).unwrap_or_default()),
			"u32" => NumberTy::U32(range.map(|r| self.range_u32(r)).unwrap_or_default()),
			"i32" => NumberTy::I32(range.map(|r| self.range_i32(r)).unwrap_or_default()),
			"f32" => NumberTy::F32(range.map(|r| self.range_f32(r)).unwrap_or_default()),
			"f64" => NumberTy::F64(range.map(|r| self.range_f64(r)).unwrap_or_default()),

			_ => unreachable!(),
		})
	}

	fn generics_one_range(&mut self, segment: &AstWord, generics: &[AstGeneric]) -> Option<AstRange> {
		if generics.len() > 1 {
			self.report(Report::IncorrectGenericCount {
				type_span: segment.span(),
				type_name: segment.word(self.rodeo).to_string(),
				generic_spans: generics.iter().map(|s| s.span()).collect::<Vec<_>>(),
				generics_optional: true,
				expected_count: 1,
				count: generics.len(),
			})
		}

		if generics.is_empty() {
			None
		} else {
			match generics.first().unwrap() {
				AstGeneric::Range(range) => Some(range.clone()),

				_ => {
					// todo: report unexpected generic
					None
				}
			}
		}
	}
}
