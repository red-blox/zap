use std::collections::HashSet;

use codespan_reporting::diagnostic::{Diagnostic, Label};

use crate::config::{Casing, Config, Enum, EvDecl, Range, Struct, Ty, TyDecl};

use super::syntax_tree::{
	Spanned, SyntaxBoolLit, SyntaxConfig, SyntaxDecl, SyntaxEnum, SyntaxEnumKind, SyntaxEvDecl, SyntaxIdentifier,
	SyntaxNumLit, SyntaxOptValueKind, SyntaxRange, SyntaxRangeKind, SyntaxStrLit, SyntaxStruct, SyntaxTy, SyntaxTyDecl,
	SyntaxTyKind,
};

pub fn convert(syntax_config: SyntaxConfig<'_>) -> (Config<'_>, Vec<Diagnostic<()>>) {
	let mut state = ConvertState::new(
		syntax_config
			.decls
			.iter()
			.filter(|decl| matches!(decl, SyntaxDecl::Ty(_)))
			.map(|decl| match decl {
				SyntaxDecl::Ty(tydecl) => tydecl.name.name,
				_ => unreachable!(),
			})
			.collect(),
	);

	let config = syntax_config.into_config(&mut state);

	(config, state.into_daigs())
}

struct ConvertState<'src> {
	daigs: Vec<Diagnostic<()>>,
	tydecls: HashSet<&'src str>,
}

impl<'src> ConvertState<'src> {
	fn new(tydecls: HashSet<&'src str>) -> Self {
		Self {
			daigs: Vec::new(),
			tydecls,
		}
	}

	fn push_diag(&mut self, diag: Diagnostic<()>) {
		self.daigs.push(diag);
	}

	fn into_daigs(self) -> Vec<Diagnostic<()>> {
		self.daigs
	}

	fn tydecl_exists(&self, name: &'src str) -> bool {
		self.tydecls.contains(name)
	}
}

trait IntoConfig<T> {
	fn into_config(self, state: &mut ConvertState) -> T;
}

impl<'src> IntoConfig<Config<'src>> for SyntaxConfig<'src> {
	fn into_config(self, state: &mut ConvertState) -> Config<'src> {
		let mut write_checks = false;
		let mut typescript = false;

		let mut server_output = None;
		let mut client_output = None;

		let mut casing = Casing::Pascal;

		for opt in self.opts {
			match opt.name.into_config(state) {
				"write_checks" => match opt.value.kind {
					SyntaxOptValueKind::Bool(value) => write_checks = value.into_config(state),

					_ => state.push_diag(
						Diagnostic::warning()
							.with_code("W0:001")
							.with_message("invalid value for option, expected boolean")
							.with_labels(vec![Label::primary((), opt.value.span())
								.with_message("invalid value here, expected boolean")]),
					),
				},

				"typescript" => match opt.value.kind {
					SyntaxOptValueKind::Bool(value) => typescript = value.into_config(state),

					_ => state.push_diag(
						Diagnostic::warning()
							.with_code("W0:001")
							.with_message("invalid value for option, expected boolean")
							.with_labels(vec![Label::primary((), opt.value.span())
								.with_message("invalid value here, expected boolean")]),
					),
				},

				"server_output" => match opt.value.kind {
					SyntaxOptValueKind::Str(value) => server_output = Some(value.into_config(state)),

					_ => state.push_diag(
						Diagnostic::warning()
							.with_code("W0:001")
							.with_message("invalid value for option, expected string")
							.with_labels(vec![Label::primary((), opt.value.span())
								.with_message("invalid value here, expected string")]),
					),
				},

				"client_output" => match opt.value.kind {
					SyntaxOptValueKind::Str(value) => client_output = Some(value.into_config(state)),

					_ => state.push_diag(
						Diagnostic::warning()
							.with_code("W0:001")
							.with_message("invalid value for option, expected string")
							.with_labels(vec![Label::primary((), opt.value.span())
								.with_message("invalid value here, expected string")]),
					),
				},

				"casing" => match opt.value.kind {
					SyntaxOptValueKind::Str(value) => match value.into_config(state) {
						"pascal" => casing = Casing::Pascal,
						"camel" => casing = Casing::Camel,
						"snake" => casing = Casing::Snake,

						_ => state.push_diag(
							Diagnostic::warning()
								.with_code("W0:001")
								.with_message("invalid value for option, expected \"pascal\", \"camel\", or \"snake\"")
								.with_labels(vec![Label::primary((), opt.value.span()).with_message(
									"invalid value here, expected \"pascal\", \"camel\", or \"snake\"",
								)]),
						),
					},

					_ => state.push_diag(
						Diagnostic::warning()
							.with_code("W0:001")
							.with_message("invalid value for option, expected  \"pascal\", \"camel\", or \"snake\"")
							.with_labels(vec![Label::primary((), opt.value.span()).with_message(
								"invalid value here, expected  \"pascal\", \"camel\", or \"snake\"",
							)]),
					),
				},

				_ => state.push_diag(
					Diagnostic::warning()
						.with_code("W0:001")
						.with_message("unknown option")
						.with_labels(vec![
							Label::primary((), opt.name.span()).with_message("unknown option name here")
						]),
				),
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

impl<'src> IntoConfig<EvDecl<'src>> for SyntaxEvDecl<'src> {
	fn into_config(self, state: &mut ConvertState) -> EvDecl<'src> {
		let name = self.name.into_config(state);
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

impl<'src> IntoConfig<TyDecl<'src>> for SyntaxTyDecl<'src> {
	fn into_config(self, state: &mut ConvertState) -> TyDecl<'src> {
		let name = self.name.into_config(state);
		let ty = self.ty.into_config(state);

		TyDecl { name, ty }
	}
}

impl<'src> IntoConfig<Ty<'src>> for SyntaxTy<'src> {
	fn into_config(self, state: &mut ConvertState) -> Ty<'src> {
		match self.kind {
			SyntaxTyKind::Num(numty, range) => Ty::Num(numty, range.map(|r| r.into_config(state)).unwrap_or_default()),
			SyntaxTyKind::Str(range) => Ty::Str(range.map(|r| r.into_config(state)).unwrap_or_default()),

			SyntaxTyKind::Arr(ty, range) => Ty::Arr(
				Box::new(ty.into_config(state)),
				range.map(|r| r.into_config(state)).unwrap_or_default(),
			),

			SyntaxTyKind::Map(key, val) => Ty::Map(Box::new(key.into_config(state)), Box::new(val.into_config(state))),
			SyntaxTyKind::Opt(ty) => Ty::Opt(Box::new(ty.into_config(state))),

			SyntaxTyKind::Ref(name) => match name.into_config(state) {
				"boolean" => Ty::Boolean,
				"Vector3" => Ty::Vector3,

				_ => {
					let name = name.into_config(state);

					if !state.tydecl_exists(name) {
						state.push_diag(
							Diagnostic::error()
								.with_code("E1:002")
								.with_message("unknown type reference")
								.with_labels(vec![
									Label::primary((), self.span()).with_message("type declaration does not exist")
								]),
						);
					}

					Ty::Ref(name)
				}
			},

			SyntaxTyKind::Enum(syntax_enum) => Ty::Enum(syntax_enum.into_config(state)),
			SyntaxTyKind::Struct(syntax_struct) => Ty::Struct(syntax_struct.into_config(state)),
			SyntaxTyKind::Instance(name) => Ty::Instance(name.map(|name| name.into_config(state))),
		}
	}
}

impl<'src> IntoConfig<Enum<'src>> for SyntaxEnum<'src> {
	fn into_config(self, state: &mut ConvertState) -> Enum<'src> {
		match self.kind {
			SyntaxEnumKind::Unit(variants) => Enum::Unit(variants.into_iter().map(|v| v.into_config(state)).collect()),

			SyntaxEnumKind::Tagged { tag, variants } => {
				let tag_name = tag.into_config(state);

				Enum::Tagged {
					tag: tag_name,
					variants: variants
						.into_iter()
						.map(|(name, value)| {
							if let Some(field) = value.fields.iter().find(|field| field.0.name == tag_name) {
								state.push_diag(
									Diagnostic::error()
										.with_code("E1:001")
										.with_message("tag name cannot be used as field name")
										.with_labels(vec![
											Label::secondary((), tag.span()).with_message("tag name here"),
											Label::primary((), field.0.span())
												.with_message("tag used as field name here"),
										]),
								);
							}

							(name.into_config(state), value.into_config(state))
						})
						.collect(),
				}
			}
		}
	}
}

impl<'src> IntoConfig<Struct<'src>> for SyntaxStruct<'src> {
	fn into_config(self, state: &mut ConvertState) -> Struct<'src> {
		let mut fields = Vec::new();

		for field in self.fields {
			fields.push((field.0.into_config(state), field.1.into_config(state)));
		}

		Struct { fields }
	}
}

impl<'src> IntoConfig<Range> for SyntaxRange<'src> {
	fn into_config(self, state: &mut ConvertState) -> Range {
		match self.kind {
			SyntaxRangeKind::None => Range::new(None, None),
			SyntaxRangeKind::Exact(num) => Range::new(Some(num.into_config(state)), Some(num.into_config(state))),
			SyntaxRangeKind::WithMin(min) => Range::new(Some(min.into_config(state)), None),
			SyntaxRangeKind::WithMax(max) => Range::new(None, Some(max.into_config(state))),
			SyntaxRangeKind::WithMinMax(min, max) => {
				Range::new(Some(min.into_config(state)), Some(max.into_config(state)))
			}
		}
	}
}

impl<'src> IntoConfig<&'src str> for SyntaxStrLit<'src> {
	fn into_config(self, _: &mut ConvertState) -> &'src str {
		self.value.trim_matches('"')
	}
}

impl<'src> IntoConfig<f64> for SyntaxNumLit<'src> {
	fn into_config(self, _: &mut ConvertState) -> f64 {
		self.value.parse().unwrap()
	}
}

impl IntoConfig<bool> for SyntaxBoolLit {
	fn into_config(self, _: &mut ConvertState) -> bool {
		self.value
	}
}

impl<'src> IntoConfig<&'src str> for SyntaxIdentifier<'src> {
	fn into_config(self, _: &mut ConvertState) -> &'src str {
		self.name
	}
}
