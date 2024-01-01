use std::collections::HashSet;

use crate::config::{Casing, Config, Enum, EvDecl, NumTy, Range, Struct, Ty, TyDecl};

use super::{
	reports::Report,
	syntax_tree::{
		Spanned, SyntaxBoolLit, SyntaxConfig, SyntaxDecl, SyntaxEnum, SyntaxEnumKind, SyntaxEvDecl, SyntaxIdentifier,
		SyntaxNumLit, SyntaxOptValueKind, SyntaxRange, SyntaxRangeKind, SyntaxStrLit, SyntaxStruct, SyntaxTy,
		SyntaxTyDecl, SyntaxTyKind,
	},
};

pub fn convert(syntax_config: SyntaxConfig<'_>) -> (Config<'_>, Vec<Report>) {
	let mut state = ConvertState::new(
		syntax_config
			.decls
			.iter()
			.filter_map(|decl| match decl {
				SyntaxDecl::Ty(tydecl) => Some(tydecl.name.name),

				_ => None,
			})
			.collect(),
	);

	let config = syntax_config.into_config(&mut state);

	(config, state.into_reports())
}

struct ConvertState<'src> {
	reports: Vec<Report<'src>>,
	tydecls: HashSet<&'src str>,
}

impl<'src> ConvertState<'src> {
	fn new(tydecls: HashSet<&'src str>) -> Self {
		Self {
			reports: Vec::new(),
			tydecls,
		}
	}

	fn push_report(&mut self, report: Report<'src>) {
		self.reports.push(report);
	}

	fn into_reports(self) -> Vec<Report<'src>> {
		self.reports
	}

	fn tydecl_exists(&self, name: &'src str) -> bool {
		self.tydecls.contains(name)
	}
}

impl<'src> SyntaxConfig<'src> {
	fn into_config(self, state: &mut ConvertState<'src>) -> Config<'src> {
		let mut write_checks = false;
		let mut typescript = false;

		let mut server_output = None;
		let mut client_output = None;

		let mut casing = Casing::Pascal;

		for opt in self.opts {
			match opt.name.into_config() {
				"write_checks" => match opt.value.kind {
					SyntaxOptValueKind::Bool(value) => write_checks = value.into_config(),

					_ => state.push_report(Report::AnalyzeInvalidOptValue {
						span: opt.value.span(),
						expected: "boolean",
					}),
				},

				"typescript" => match opt.value.kind {
					SyntaxOptValueKind::Bool(value) => typescript = value.into_config(),

					_ => state.push_report(Report::AnalyzeInvalidOptValue {
						span: opt.value.span(),
						expected: "boolean",
					}),
				},

				"server_output" => match opt.value.kind {
					SyntaxOptValueKind::Str(value) => server_output = Some(value.into_config()),

					_ => state.push_report(Report::AnalyzeInvalidOptValue {
						span: opt.value.span(),
						expected: "string",
					}),
				},

				"client_output" => match opt.value.kind {
					SyntaxOptValueKind::Str(value) => client_output = Some(value.into_config()),

					_ => state.push_report(Report::AnalyzeInvalidOptValue {
						span: opt.value.span(),
						expected: "string",
					}),
				},

				"casing" => match opt.value.kind {
					SyntaxOptValueKind::Str(value) => match value.into_config() {
						"PascalCase" => casing = Casing::Pascal,
						"camelCase" => casing = Casing::Camel,
						"snake_case" => casing = Casing::Snake,

						_ => state.push_report(Report::AnalyzeInvalidOptValue {
							span: opt.value.span(),
							expected: "`PascalCase`, `camelCase`, or `snake_case`",
						}),
					},

					_ => state.push_report(Report::AnalyzeInvalidOptValue {
						span: opt.value.span(),
						expected: "`PascalCase`, `camelCase`, or `snake_case`",
					}),
				},

				_ => state.push_report(Report::AnalyzeUnknownOptName { span: opt.name.span() }),
			}
		}

		let mut tydecls = Vec::new();
		let mut evdecls = Vec::new();

		for decl in self.decls {
			match decl {
				SyntaxDecl::Ty(tydecl) => tydecls.push(tydecl.into_config(state)),
				SyntaxDecl::Ev(evdecl) => evdecls.push(evdecl.into_config(state)),
			}
		}

		Config {
			tydecls,
			evdecls,

			write_checks,
			typescript,

			server_output,
			client_output,

			casing,
		}
	}
}

impl<'src> SyntaxEvDecl<'src> {
	fn into_config(self, state: &mut ConvertState<'src>) -> EvDecl<'src> {
		let name = self.name.into_config();
		let from = self.from;
		let evty = self.evty;
		let call = self.call;
		let data = self.data.into_config(state);

		EvDecl {
			name,
			from,
			evty,
			call,
			data,
		}
	}
}

impl<'src> SyntaxTyDecl<'src> {
	fn into_config(self, state: &mut ConvertState<'src>) -> TyDecl<'src> {
		let name = self.name.into_config();
		let ty = self.ty.into_config(state);

		TyDecl { name, ty }
	}
}

impl<'src> SyntaxTy<'src> {
	fn into_config(self, state: &mut ConvertState<'src>) -> Ty<'src> {
		match self.kind {
			SyntaxTyKind::Num(numty, range) => {
				let range = range.map(|r| match numty {
					NumTy::F32 => r.into_config_with_range(state, f32::MIN.into(), f32::MAX.into()),
					NumTy::F64 => r.into_config_with_range(state, f64::MIN, f64::MAX),

					NumTy::U8 => r.into_config_with_range(state, u8::MIN.into(), u8::MAX.into()),
					NumTy::U16 => r.into_config_with_range(state, u16::MIN.into(), u16::MAX.into()),
					NumTy::U32 => r.into_config_with_range(state, u32::MIN.into(), u32::MAX.into()),

					NumTy::I8 => r.into_config_with_range(state, i8::MIN.into(), i8::MAX.into()),
					NumTy::I16 => r.into_config_with_range(state, i16::MIN.into(), i16::MAX.into()),
					NumTy::I32 => r.into_config_with_range(state, i32::MIN.into(), i32::MAX.into()),
				});

				Ty::Num(numty, range.unwrap_or_default())
			}

			SyntaxTyKind::Str(range) => Ty::Str(
				range
					.map(|r| r.into_config_with_range(state, u16::MIN.into(), u16::MAX.into()))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Buf(range) => Ty::Buf(
				range
					.map(|r| r.into_config_with_range(state, u16::MIN.into(), u16::MAX.into()))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Arr(ty, range) => Ty::Arr(
				Box::new(ty.into_config(state)),
				range
					.map(|r| r.into_config_with_range(state, u16::MIN.into(), u16::MAX.into()))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Map(key, val) => Ty::Map(Box::new(key.into_config(state)), Box::new(val.into_config(state))),
			SyntaxTyKind::Opt(ty) => {
				let ty = ty.into_config(state);

				if let Ty::Opt(ty) = &ty {
					if let Ty::Unknown = **ty {
						state.push_report(Report::AnalyzeInvalidOptionalType {
							span: (self.end - 1)..self.end,
						});
					}
				}

				Ty::Opt(Box::new(ty))
			}

			SyntaxTyKind::Ref(name) => match name.into_config() {
				"boolean" => Ty::Boolean,
				"Vector3" => Ty::Vector3,
				"AlignedCFrame" => Ty::AlignedCFrame,
				"CFrame" => Ty::CFrame,
				"unknown" => Ty::Opt(Box::new(Ty::Unknown)),

				_ => {
					let name = name.into_config();

					if !state.tydecl_exists(name) {
						state.push_report(Report::AnalyzeUnknownTypeRef {
							span: self.span(),
							name,
						});
					}

					Ty::Ref(name)
				}
			},

			SyntaxTyKind::Enum(syntax_enum) => Ty::Enum(syntax_enum.into_config(state)),
			SyntaxTyKind::Struct(syntax_struct) => Ty::Struct(syntax_struct.into_config(state)),
			SyntaxTyKind::Instance(name) => Ty::Instance(name.map(|name| name.into_config())),
		}
	}
}

impl<'src> SyntaxEnum<'src> {
	fn into_config(self, state: &mut ConvertState<'src>) -> Enum<'src> {
		let span = self.span();

		match self.kind {
			SyntaxEnumKind::Unit(enumerators) => {
				if enumerators.is_empty() {
					state.push_report(Report::AnalyzeEmptyEnum { span })
				}

				Enum::Unit(enumerators.into_iter().map(|v| v.into_config()).collect())
			}

			SyntaxEnumKind::Tagged { tag, variants } => {
				if variants.is_empty() {
					state.push_report(Report::AnalyzeEmptyEnum { span })
				}

				let tag_name = tag.into_config();

				Enum::Tagged {
					tag: tag_name,
					variants: variants
						.into_iter()
						.map(|(name, value)| {
							if let Some(field) = value.fields.iter().find(|field| field.0.name == tag_name) {
								state.push_report(Report::AnalyzeEnumTagUsed {
									tag_span: tag.span(),
									used_span: field.0.span(),
									tag: tag_name,
								});
							}

							(name.into_config(), value.into_config(state))
						})
						.collect(),
				}
			}
		}
	}
}

impl<'src> SyntaxStruct<'src> {
	fn into_config(self, state: &mut ConvertState<'src>) -> Struct<'src> {
		let mut fields = Vec::new();

		for field in self.fields {
			fields.push((field.0.into_config(), field.1.into_config(state)));
		}

		Struct { fields }
	}
}

impl<'src> SyntaxRange<'src> {
	fn into_config(self, state: &mut ConvertState<'src>) -> Range {
		let range = match self.kind {
			SyntaxRangeKind::None => Range::new(None, None),
			SyntaxRangeKind::Exact(num) => Range::new(Some(num.into_config()), Some(num.into_config())),
			SyntaxRangeKind::WithMin(min) => Range::new(Some(min.into_config()), None),
			SyntaxRangeKind::WithMax(max) => Range::new(None, Some(max.into_config())),
			SyntaxRangeKind::WithMinMax(min, max) => Range::new(Some(min.into_config()), Some(max.into_config())),
		};

		if range.min().is_some() && range.max().is_some() && range.min().unwrap() > range.max().unwrap() {
			state.push_report(Report::AnalyzeInvalidRange { span: self.span() });
		}

		range
	}

	fn into_config_with_range(self, state: &mut ConvertState<'src>, min: f64, max: f64) -> Range {
		let range = match self.kind {
			SyntaxRangeKind::None => Range::new(None, None),
			SyntaxRangeKind::Exact(num) => Range::new(
				Some(num.into_config_with_range(state, min, max)),
				Some(num.into_config_with_range(state, min, max)),
			),
			SyntaxRangeKind::WithMin(min_) => Range::new(Some(min_.into_config_with_range(state, min, max)), None),
			SyntaxRangeKind::WithMax(max_) => Range::new(None, Some(max_.into_config_with_range(state, min, max))),
			SyntaxRangeKind::WithMinMax(min_, max_) => Range::new(
				Some(min_.into_config_with_range(state, min, max)),
				Some(max_.into_config_with_range(state, min, max)),
			),
		};

		if range.min().is_some() && range.max().is_some() && range.min().unwrap() > range.max().unwrap() {
			state.push_report(Report::AnalyzeInvalidRange { span: self.span() });
		}

		range
	}
}

impl<'src> SyntaxStrLit<'src> {
	fn into_config(self) -> &'src str {
		self.value.trim_matches('"')
	}
}

impl<'src> SyntaxNumLit<'src> {
	fn into_config(self) -> f64 {
		self.value.parse().unwrap()
	}

	fn into_config_with_range(self, state: &mut ConvertState, min: f64, max: f64) -> f64 {
		let value = self.into_config();

		if value < min || value > max {
			state.push_report(Report::AnalyzeNumOutsideRange {
				span: self.span(),
				min,
				max,
			});
		}

		value
	}
}

impl SyntaxBoolLit {
	fn into_config(self) -> bool {
		self.value
	}
}

impl<'src> SyntaxIdentifier<'src> {
	fn into_config(self) -> &'src str {
		self.name
	}
}
