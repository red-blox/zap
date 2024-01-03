use std::collections::{HashMap, HashSet};

use crate::config::{Casing, Config, Enum, EvDecl, EvType, NumTy, Range, Struct, Ty, TyDecl};

use super::{
	reports::{Report, Span},
	syntax_tree::*,
};

struct Converter<'src> {
	config: SyntaxConfig<'src>,

	tydecls: HashMap<&'src str, SyntaxTyDecl<'src>>,
	evdecls: HashMap<&'src str, SyntaxEvDecl<'src>>,

	reports: Vec<Report<'src>>,
}

impl<'src> Converter<'src> {
	fn new(config: SyntaxConfig<'src>) -> Self {
		let mut tydecls = HashMap::new();
		let mut evdecls = HashMap::new();

		for decl in config.decls.iter() {
			match decl {
				SyntaxDecl::Ty(tydecl) => {
					tydecls.insert(tydecl.name.name, tydecl.clone());
				}

				SyntaxDecl::Ev(evdecl) => {
					evdecls.insert(evdecl.name.name, evdecl.clone());
				}
			}
		}

		Self {
			config,

			tydecls,
			evdecls,

			reports: Vec::new(),
		}
	}

	fn convert(mut self) -> (Config<'src>, Vec<Report<'src>>) {
		let config = self.config.clone();
		let mut tydecls = HashMap::new();

		for tydecl in config.decls.iter().filter_map(|decl| match decl {
			SyntaxDecl::Ty(tydecl) => Some(tydecl),
			_ => None,
		}) {
			tydecls.insert(tydecl.name.name, self.tydecl(tydecl));
		}

		let mut evdecls = Vec::new();

		for evdecl in config.decls.iter().filter_map(|decl| match decl {
			SyntaxDecl::Ev(evdecl) => Some(evdecl),
			_ => None,
		}) {
			evdecls.push(self.evdecl(evdecl, &tydecls));
		}

		if evdecls.is_empty() {
			self.report(Report::AnalyzeEmptyEvDecls);
		}

		let write_checks = self.boolean_opt("write_checks", true, &config.opts).0;
		let typescript = self.boolean_opt("typescript", false, &config.opts).0;

		let server_output = self.str_opt("server_output", "network/server.lua", &config.opts).0;
		let client_output = self.str_opt("client_output", "network/client.lua", &config.opts).0;

		let casing = match self.str_opt("casing", "PascalCase", &config.opts) {
			("snake_case", ..) => Casing::Snake,
			("camelCase", ..) => Casing::Camel,
			("PascalCase", ..) => Casing::Pascal,

			(_, Some(span)) => {
				self.report(Report::AnalyzeInvalidOptValue {
					span,
					expected: "`snake_case`, `camelCase`, or `PascalCase`",
				});

				Casing::Pascal
			}

			_ => unreachable!(),
		};

		let config = Config {
			tydecls: tydecls.into_values().collect(),
			evdecls,

			write_checks,
			typescript,

			server_output,
			client_output,

			casing,
		};

		(config, self.reports)
	}

	fn boolean_opt(&mut self, name: &'static str, default: bool, opts: &[SyntaxOpt<'src>]) -> (bool, Option<Span>) {
		let mut value = default;
		let mut span = None;

		for opt in opts.iter().filter(|opt| opt.name.name == name) {
			if let SyntaxOptValueKind::Bool(opt_value) = &opt.value.kind {
				value = opt_value.value;
				span = Some(opt_value.span());
			} else {
				self.report(Report::AnalyzeInvalidOptValue {
					span: opt.value.span(),
					expected: "boolean",
				});
			}
		}

		(value, span)
	}

	fn str_opt(
		&mut self,
		name: &'static str,
		default: &'static str,
		opts: &[SyntaxOpt<'src>],
	) -> (&'src str, Option<Span>) {
		let mut value = default;
		let mut span = None;

		for opt in opts.iter().filter(|opt| opt.name.name == name) {
			if let SyntaxOptValueKind::Str(opt_value) = &opt.value.kind {
				value = self.str(opt_value);
				span = Some(opt_value.span());
			} else {
				self.report(Report::AnalyzeInvalidOptValue {
					span: opt.value.span(),
					expected: "string",
				});
			}
		}

		(value, span)
	}

	#[allow(dead_code)]
	fn num_opt(&mut self, name: &'static str, default: f64, opts: &[SyntaxOpt<'src>]) -> (f64, Option<Span>) {
		let mut value = default;
		let mut span = None;

		for opt in opts.iter().filter(|opt| opt.name.name == name) {
			if let SyntaxOptValueKind::Num(opt_value) = &opt.value.kind {
				value = self.num(opt_value);
				span = Some(opt_value.span());
			} else {
				self.report(Report::AnalyzeInvalidOptValue {
					span: opt.value.span(),
					expected: "number",
				});
			}
		}

		(value, span)
	}

	fn evdecl(&mut self, evdecl: &SyntaxEvDecl<'src>, tydecls: &HashMap<&'src str, TyDecl<'src>>) -> EvDecl<'src> {
		let name = evdecl.name.name;
		let from = evdecl.from;
		let evty = evdecl.evty;
		let call = evdecl.call;
		let data = self.ty(&evdecl.data);

		if let EvType::Unreliable = evty {
			let (min, max) = data.size(tydecls, &mut HashSet::new());
			let event_id_size = NumTy::from_f64(1.0, self.evdecls.len() as f64).size();

			if (min + event_id_size) > 900 {
				self.report(Report::AnalyzeOversizeUnreliable {
					ev_span: evdecl.span(),
					ty_span: evdecl.data.span(),
					max_size: 900 - event_id_size,
					size: min,
				});
			} else if !max.is_some_and(|max| max < 900 - event_id_size) {
				self.report(Report::AnalyzePotentiallyOversizeUnreliable {
					ev_span: evdecl.span(),
					ty_span: evdecl.data.span(),
					max_size: 900 - event_id_size,
				});
			}
		}

		EvDecl {
			name,
			from,
			evty,
			call,
			data,
		}
	}

	fn tydecl(&mut self, tydecl: &SyntaxTyDecl<'src>) -> TyDecl<'src> {
		let name = tydecl.name.name;
		let ty = self.ty(&tydecl.ty);

		if let Some(ref_ty) = self.ty_has_unbounded_ref(name, &tydecl.ty, &mut HashSet::new()) {
			self.report(Report::AnalyzeUnboundedRecursiveType {
				decl_span: tydecl.span(),
				use_span: ref_ty.span(),
			});
		}

		TyDecl { name, ty }
	}

	fn ty(&mut self, ty: &SyntaxTy<'src>) -> Ty<'src> {
		match &ty.kind {
			SyntaxTyKind::Num(numty, range) => Ty::Num(
				*numty,
				range
					.map(|range| self.checked_range_within(&range, numty.min(), numty.max()))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Str(len) => Ty::Str(
				len.map(|range| self.checked_range_within(&range, 0.0, u16::MAX as f64))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Buf(len) => Ty::Buf(
				len.map(|range| self.checked_range_within(&range, 0.0, u16::MAX as f64))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Arr(ty, len) => Ty::Arr(
				Box::new(self.ty(ty)),
				len.map(|len| self.checked_range_within(&len, 0.0, u16::MAX as f64))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Map(key, val) => {
				let key_ty = self.ty(key);
				let val_ty = self.ty(val);

				if let Ty::Opt(_) = key_ty {
					self.report(Report::AnalyzeInvalidOptionalType {
						span: (key.span().end - 1)..key.span().end,
					});
				}

				if let Ty::Opt(_) = val_ty {
					self.report(Report::AnalyzeInvalidOptionalType {
						span: (val.span().end - 1)..val.span().end,
					});
				}

				Ty::Map(Box::new(key_ty), Box::new(val_ty))
			}

			SyntaxTyKind::Opt(ty) => {
				let parsed_ty = self.ty(ty);

				if let Ty::Opt(_) = parsed_ty {
					self.report(Report::AnalyzeInvalidOptionalType {
						span: (ty.span().end - 1)..ty.span().end,
					});
				}

				Ty::Opt(Box::new(parsed_ty))
			}

			SyntaxTyKind::Ref(ref_ty) => {
				let name = ref_ty.name;

				match name {
					"boolean" => Ty::Boolean,
					"Color3" => Ty::Color3,
					"Vector3" => Ty::Vector3,
					"AlignedCFrame" => Ty::AlignedCFrame,
					"CFrame" => Ty::CFrame,
					"unknown" => Ty::Opt(Box::new(Ty::Unknown)),

					_ => {
						if self.tydecls.get(name).is_none() {
							self.report(Report::AnalyzeUnknownTypeRef {
								span: ref_ty.span(),
								name,
							});
						}

						Ty::Ref(name)
					}
				}
			}

			SyntaxTyKind::Enum(enum_ty) => Ty::Enum(self.enum_ty(enum_ty)),

			SyntaxTyKind::Struct(struct_ty) => Ty::Struct(self.struct_ty(struct_ty)),

			SyntaxTyKind::Instance(instance_ty) => Ty::Instance(instance_ty.as_ref().map(|ty| ty.name)),
		}
	}

	fn enum_ty(&mut self, ty: &SyntaxEnum<'src>) -> Enum<'src> {
		let span = ty.span();

		match &ty.kind {
			SyntaxEnumKind::Unit(enumerators) => {
				if enumerators.is_empty() {
					self.report(Report::AnalyzeEmptyEnum { span });
				}

				Enum::Unit(enumerators.iter().map(|e| e.name).collect())
			}

			SyntaxEnumKind::Tagged { tag, variants } => {
				let tag_name = tag.value;

				let variants = variants
					.iter()
					.map(|variant| {
						if variant.1.fields.iter().any(|field| field.0.name == tag_name) {
							self.report(Report::AnalyzeEnumTagUsed {
								tag_span: tag.span(),
								used_span: variant.0.span(),
								tag: tag_name,
							});
						}

						(variant.0.name, self.struct_ty(&variant.1))
					})
					.collect();

				Enum::Tagged {
					tag: tag_name,
					variants,
				}
			}
		}
	}

	fn struct_ty(&mut self, ty: &SyntaxStruct<'src>) -> Struct<'src> {
		let mut fields = Vec::new();

		for field in ty.fields.iter() {
			fields.push((field.0.name, self.ty(&field.1)));
		}

		Struct { fields }
	}

	fn ty_has_unbounded_ref(
		&self,
		name: &'src str,
		ty: &SyntaxTy<'src>,
		searched: &mut HashSet<&'src str>,
	) -> Option<SyntaxIdentifier<'src>> {
		match &ty.kind {
			SyntaxTyKind::Arr(ty, len) => {
				let len = len.map(|len| self.range(&len)).unwrap_or_default();

				// if array does not have a min size of 0, it is unbounded
				if len.min().is_some_and(|min| min != 0.0) {
					self.ty_has_unbounded_ref(name, ty, searched)
				} else {
					None
				}
			}

			SyntaxTyKind::Ref(ref_ty) => {
				let ref_name = ref_ty.name;

				match ref_name {
					ref_name if ref_name == name => Some(*ref_ty),

					"boolean" | "Color3" | "Vector3" | "AlignedCFrame" | "CFrame" | "unknown" => None,

					_ => {
						if searched.contains(ref_name) {
							None
						} else if let Some(tydecl) = self.tydecls.get(ref_name) {
							searched.insert(ref_name);
							self.ty_has_unbounded_ref(name, &tydecl.ty, searched)
						} else {
							None
						}
					}
				}
			}

			SyntaxTyKind::Enum(enum_ty) => self.enum_has_unbounded_ref(name, enum_ty, searched),
			SyntaxTyKind::Struct(struct_ty) => self.struct_has_unbounded_ref(name, struct_ty, searched),

			_ => None,
		}
	}

	fn enum_has_unbounded_ref(
		&self,
		name: &'src str,
		ty: &SyntaxEnum<'src>,
		searched: &mut HashSet<&'src str>,
	) -> Option<SyntaxIdentifier<'src>> {
		match &ty.kind {
			SyntaxEnumKind::Unit { .. } => None,

			SyntaxEnumKind::Tagged { variants, .. } => {
				for variant in variants.iter() {
					if let Some(ty) = self.struct_has_unbounded_ref(name, &variant.1, searched) {
						return Some(ty);
					}
				}

				None
			}
		}
	}

	fn struct_has_unbounded_ref(
		&self,
		name: &'src str,
		ty: &SyntaxStruct<'src>,
		searched: &mut HashSet<&'src str>,
	) -> Option<SyntaxIdentifier<'src>> {
		for field in ty.fields.iter() {
			if let Some(ty) = self.ty_has_unbounded_ref(name, &field.1, searched) {
				return Some(ty);
			}
		}

		None
	}

	fn report(&mut self, report: Report<'src>) {
		self.reports.push(report);
	}

	fn checked_range_within(&mut self, range: &SyntaxRange<'src>, min: f64, max: f64) -> Range {
		let value = self.range_within(range, min, max);

		if value.min().is_some() && value.max().is_some() && value.min().unwrap() > value.max().unwrap() {
			self.report(Report::AnalyzeInvalidRange { span: range.span() });
		}

		value
	}

	fn range_within(&mut self, range: &SyntaxRange<'src>, min: f64, max: f64) -> Range {
		match range.kind {
			SyntaxRangeKind::None => Range::new(None, None),

			SyntaxRangeKind::Exact(num) => {
				let value = self.num_within(&num, min, max);
				Range::new(Some(value), Some(value))
			}

			SyntaxRangeKind::WithMin(min_num) => {
				let value = self.num_within(&min_num, min, max);
				Range::new(Some(value), None)
			}

			SyntaxRangeKind::WithMax(max_num) => {
				let value = self.num_within(&max_num, min, max);
				Range::new(None, Some(value))
			}

			SyntaxRangeKind::WithMinMax(min_num, max_num) => {
				let min_value = self.num_within(&min_num, min, max);
				let max_value = self.num_within(&max_num, min, max);
				Range::new(Some(min_value), Some(max_value))
			}
		}
	}

	#[allow(dead_code)]
	fn checked_range(&mut self, range: &SyntaxRange<'src>) -> Range {
		let value = self.range(range);

		if value.min().is_some() && value.max().is_some() && value.min().unwrap() > value.max().unwrap() {
			self.report(Report::AnalyzeInvalidRange { span: range.span() });
		}

		value
	}

	fn range(&self, range: &SyntaxRange<'src>) -> Range {
		match range.kind {
			SyntaxRangeKind::None => Range::new(None, None),
			SyntaxRangeKind::Exact(num) => Range::new(Some(self.num(&num)), Some(self.num(&num))),
			SyntaxRangeKind::WithMin(min) => Range::new(Some(self.num(&min)), None),
			SyntaxRangeKind::WithMax(max) => Range::new(None, Some(self.num(&max))),
			SyntaxRangeKind::WithMinMax(min, max) => Range::new(Some(self.num(&min)), Some(self.num(&max))),
		}
	}

	fn num_within(&mut self, num: &SyntaxNumLit<'src>, min: f64, max: f64) -> f64 {
		let value = self.num(num);

		if value < min || value > max {
			self.report(Report::AnalyzeNumOutsideRange {
				span: num.span(),
				min,
				max,
			});
		}

		value
	}

	fn str(&self, str: &SyntaxStrLit<'src>) -> &'src str {
		// unwrapping here is safe because the parser already validated the string earlier
		str.value[1..str.value.len() - 1].as_ref()
	}

	fn num(&self, num: &SyntaxNumLit<'src>) -> f64 {
		// unwrapping here is safe because the parser already validated the number earlier
		num.value.parse().unwrap()
	}
}

pub fn convert(config: SyntaxConfig<'_>) -> (Config<'_>, Vec<Report<'_>>) {
	Converter::new(config).convert()
}
