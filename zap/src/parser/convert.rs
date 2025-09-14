use std::{
	borrow::Cow,
	cell::RefCell,
	cmp::Ordering,
	collections::{BTreeMap, HashMap, HashSet, VecDeque},
	rc::Rc,
};

use crate::config::{
	Casing, Config, Enum, EvCall, EvDecl, EvSource, EvType, FnDecl, NamespaceEntry, NonPrimitiveTy, NumTy, Parameter,
	PrimitiveTy, Range, Struct, Ty, TyDecl, TypeScriptEnumType, UNRELIABLE_ORDER_NUMTY, YieldType,
};

use super::{
	reports::{Report, Span},
	syntax_tree::*,
};

// We subtract two for the `inst` array.
pub const MAX_UNRELIABLE_SIZE: usize = 998;

struct Converter<'src> {
	config: SyntaxConfig<'src>,
	tydecls: HashMap<String, SyntaxTyDecl<'src>>,
	resolved_tys: HashMap<String, Rc<RefCell<Ty<'src>>>>,
	all_tydecls: HashMap<String, TyDecl<'src>>,
	current_tydecl: Option<SyntaxTyDecl<'src>>,
	path: Vec<&'src str>,

	reports: Vec<Report<'src>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TyRecursionKind<'src> {
	Unbounded(SyntaxIdentifier<'src>),
	// recursive, although bounded
	Recursive,
	None,
}

impl<'src> Converter<'src> {
	fn new(config: SyntaxConfig<'src>) -> Self {
		Self {
			config,
			tydecls: HashMap::new(),
			all_tydecls: HashMap::new(),
			current_tydecl: None,
			path: Vec::new(),
			resolved_tys: Default::default(),

			reports: Vec::new(),
		}
	}

	fn convert(mut self) -> (Config<'src>, Vec<Report<'src>>) {
		let config = self.config.clone();

		self.check_duplicate_decls(&config.decls);

		let mut namespaces = BTreeMap::new();

		let mut server_reliable_id = 0;
		let mut server_unreliable_id = 0;
		let mut client_reliable_id = 0;
		let mut client_unreliable_id = 0;

		let mut nsdecls = Vec::new();
		let mut queue = config
			.decls
			.iter()
			.filter_map(|decl| match decl {
				SyntaxDecl::Ns(nsdecl) => Some((nsdecl, vec![nsdecl.name.name])),
				_ => None,
			})
			.collect::<VecDeque<_>>();

		while let Some((nsdecl, path)) = queue.pop_front() {
			nsdecls.push((&nsdecl.decls, path.clone()));

			for decl in &nsdecl.decls {
				if let SyntaxDecl::Ns(nsdecl) = decl {
					queue.push_back((
						nsdecl,
						path.iter().copied().chain(std::iter::once(nsdecl.name.name)).collect(),
					));
				}
			}
		}

		for (decls, path) in nsdecls
			.into_iter()
			// reverse so namespaces higher can use types from namespaces lower
			.rev()
			.chain(std::iter::once((&config.decls, vec![])))
		{
			self.path = path;

			let current_tydecls = decls.iter().filter_map(|decl| match decl {
				SyntaxDecl::Ty(tydecl) => Some(tydecl),
				_ => None,
			});

			for tydecl in current_tydecls.clone() {
				self.tydecls.insert(
					self.path
						.iter()
						.copied()
						.chain(std::iter::once(tydecl.name.name))
						.collect::<Vec<_>>()
						.join("."),
					tydecl.clone(),
				);
			}

			for tydecl in current_tydecls {
				let tydecl = self.tydecl(tydecl);
				self.all_tydecls.insert(
					self.path
						.iter()
						.copied()
						.chain(std::iter::once(tydecl.name))
						.collect::<Vec<_>>()
						.join("."),
					tydecl,
				);
			}

			let mut push_ns_entry = |this: &mut Self, data: NamespaceEntry<'src>| {
				if this.path.is_empty() {
					namespaces.insert(data.name(), data);
					return;
				}

				let mut path = this.path.iter().copied();

				let entry = namespaces
					.entry(path.next().unwrap())
					.or_insert_with(|| NamespaceEntry::Ns(BTreeMap::new()));
				let NamespaceEntry::Ns(entries) = entry else {
					unreachable!()
				};

				let mut prev_entry: &mut BTreeMap<&str, NamespaceEntry<'src>> = entries;

				for part in path {
					let new_entry = prev_entry
						.entry(part)
						.or_insert_with(|| NamespaceEntry::Ns(BTreeMap::new()));
					let NamespaceEntry::Ns(new_entry) = new_entry else {
						unreachable!();
					};
					prev_entry = new_entry;
				}

				prev_entry.insert(data.name(), data);
			};

			for evdecl in decls.iter().filter_map(|decl| match decl {
				SyntaxDecl::Ev(evdecl) => Some(evdecl),
				_ => None,
			}) {
				let id = match evdecl.from {
					EvSource::Server => match evdecl.evty {
						EvType::Reliable => {
							let current_id = client_reliable_id;
							client_reliable_id += 1;
							current_id
						}
						EvType::Unreliable(_) => {
							let current_id = client_unreliable_id;
							client_unreliable_id += 1;
							current_id
						}
					},
					EvSource::Client => match evdecl.evty {
						EvType::Reliable => {
							let current_id = server_reliable_id;
							server_reliable_id += 1;
							current_id
						}
						EvType::Unreliable(_) => {
							let current_id = server_unreliable_id;
							server_unreliable_id += 1;
							current_id
						}
					},
				};

				let evdecl = self.evdecl(evdecl, id);
				push_ns_entry(&mut self, NamespaceEntry::EvDecl(evdecl));
			}

			for fndecl in decls.iter().filter_map(|decl| match decl {
				SyntaxDecl::Fn(fndecl) => Some(fndecl),
				_ => None,
			}) {
				let fndecl = self.fndecl(fndecl, client_reliable_id, server_reliable_id);
				push_ns_entry(&mut self, NamespaceEntry::FnDecl(fndecl));
				client_reliable_id += 1;
				server_reliable_id += 1;
			}
		}

		let (typescript, ..) = self.boolean_opt("typescript", false, &config.opts);
		let (typescript_max_tuple_length, ..) = self.num_opt("typescript_max_tuple_length", 10.0, &config.opts);
		let typescript_enum = self.typescript_enum_opt(&config.opts);

		let (tooling, ..) = self.boolean_opt("tooling", false, &config.opts);
		let (tooling_show_internal_data, ..) = self.boolean_opt("tooling_show_internal_data", false, &config.opts);

		let (write_checks, ..) = self.boolean_opt("write_checks", true, &config.opts);
		let (manual_event_loop, ..) = self.boolean_opt("manual_event_loop", false, &config.opts);

		let (remote_scope, ..) = self.str_opt("remote_scope", "ZAP", &config.opts);
		let (remote_folder, ..) = self.str_opt("remote_folder", "ZAP", &config.opts);

		let (server_output, ..) = self.str_opt("server_output", "network/server.lua", &config.opts);
		let (client_output, ..) = self.str_opt("client_output", "network/client.lua", &config.opts);
		let types_output = self.types_output_opt(&config.opts);
		let (tooling_output, ..) = self.str_opt("tooling_output", "network/tooling.lua", &config.opts);

		let casing = self.casing_opt(&config.opts);
		let call_default = self.call_default_opt(&config.opts);
		let yield_type = self.yield_type_opt(typescript, &config.opts);
		let async_lib = self.async_lib(yield_type, &config.opts, typescript);
		let (disable_fire_all, ..) = self.boolean_opt("disable_fire_all", false, &config.opts);

		let config = Config {
			tydecls: self.all_tydecls.drain().map(|(_, tydecl)| tydecl).collect(),
			namespaces,

			typescript,
			typescript_max_tuple_length,
			typescript_enum,

			tooling,
			tooling_show_internal_data,

			write_checks,
			manual_event_loop,

			remote_scope,
			remote_folder,

			server_output,
			client_output,
			types_output,
			tooling_output,

			casing,
			call_default,
			yield_type,
			async_lib,
			disable_fire_all,
		};

		self.check_empty_file(&config);
		self.check_conflicting_exports(casing);

		(config, self.reports)
	}

	fn async_lib(&mut self, yield_type: YieldType, opts: &[SyntaxOpt<'src>], typescript: bool) -> &'src str {
		let (async_lib, async_lib_span) = self.str_opt("async_lib", "", opts);

		if let Some(span) = async_lib_span {
			if !async_lib.starts_with("require") {
				self.report(Report::AnalyzeInvalidOptValue {
					span,
					expected: "that `async_lib` path must be a `require` statement",
				});
			} else if yield_type == YieldType::Yield {
				self.report(Report::AnalyzeInvalidOptValue {
					span,
					expected: "that `async_lib` cannot be defined when using a `yield_type` of `yield`",
				});
			}
		} else if async_lib.is_empty() && yield_type != YieldType::Yield && !typescript {
			self.report(Report::AnalyzeMissingOptValue {
				expected: "`async_lib`",
				required_when: "`yield_type` is set to `promise` or `future`.",
			});
		}

		async_lib
	}

	fn yield_type_opt(&mut self, typescript: bool, opts: &[SyntaxOpt<'src>]) -> YieldType {
		match self.str_opt("yield_type", "yield", opts) {
			("yield", ..) => YieldType::Yield,
			("promise", ..) => YieldType::Promise,
			("future", Some(span)) => {
				if typescript {
					self.report(Report::AnalyzeInvalidOptValue {
						span,
						expected: "`yield` or `promise`",
					});
				}

				YieldType::Future
			}

			(_, Some(span)) => {
				self.report(Report::AnalyzeInvalidOptValue {
					span,
					expected: "`yield`, `future`, or `promise`",
				});

				YieldType::Yield
			}

			_ => unreachable!(),
		}
	}

	fn casing_opt(&mut self, opts: &[SyntaxOpt<'src>]) -> Casing {
		match self.str_opt("casing", "PascalCase", opts) {
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
		}
	}

	fn typescript_enum_opt(&mut self, opts: &[SyntaxOpt<'src>]) -> TypeScriptEnumType {
		match self.str_opt("typescript_enum", "StringLiteral", opts) {
			("StringLiteral", ..) => TypeScriptEnumType::StringLiteral,
			("StringConstEnum", ..) => TypeScriptEnumType::ConstString,
			("ConstEnum", ..) => TypeScriptEnumType::ConstNumber,

			(_, Some(span)) => {
				self.report(Report::AnalyzeInvalidOptValue {
					span,
					expected: "`StringLiteral`, `ConstEnum`, or `StringConstEnum`",
				});

				TypeScriptEnumType::StringLiteral
			}

			_ => unreachable!(),
		}
	}

	fn types_output_opt(&mut self, opts: &[SyntaxOpt<'src>]) -> Option<&'src str> {
		let opt = opts.iter().find(|opt| opt.name.name == "types_output")?;

		if let SyntaxOptValueKind::Str(opt_value) = &opt.value.kind {
			Some(self.str(opt_value))
		} else {
			self.report(Report::AnalyzeInvalidOptValue {
				span: opt.value.span(),
				expected: "Types output path expected.",
			});

			None
		}
	}

	fn call_default_opt(&mut self, opts: &[SyntaxOpt<'src>]) -> Option<EvCall> {
		let opt = opts.iter().find(|opt| opt.name.name == "call_default")?;

		let (value, span) = if let SyntaxOptValueKind::Str(opt_value) = &opt.value.kind {
			(Some(self.str(opt_value)), Some(opt_value.span()))
		} else {
			self.report(Report::AnalyzeInvalidOptValue {
				span: opt.value.span(),
				expected: "`\"SingleSync\", \"ManySync\", \"SingleAsync\", \"ManyAsync\", or \"Polling\".",
			});

			(None, None)
		};

		match (value, span) {
			(Some("SingleSync"), ..) => Some(EvCall::SingleSync),
			(Some("ManySync"), ..) => Some(EvCall::ManySync),
			(Some("SingleAsync"), ..) => Some(EvCall::SingleAsync),
			(Some("ManyAsync"), ..) => Some(EvCall::ManyAsync),
			(Some("Polling"), ..) => Some(EvCall::Polling),
			(_, Some(span)) => {
				self.report(Report::AnalyzeInvalidOptValue {
					span,
					expected: "`\"SingleSync\", \"ManySync\", \"SingleAsync\", \"ManyAsync\", or \"Polling\".",
				});

				None
			}
			_ => None,
		}
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

	fn check_duplicate_decls(&mut self, decls: &[SyntaxDecl<'src>]) {
		let mut tydecls = HashMap::new();
		let mut ntdecls = HashMap::new();

		for decl in decls.iter() {
			let (name, span) = match decl {
				SyntaxDecl::Ns(ns) => {
					self.check_duplicate_decls(&ns.decls);

					(&ns.name.name, ns.span())
				}
				SyntaxDecl::Ev(ev) => (&ev.name.name, ev.span()),
				SyntaxDecl::Fn(fn_) => (&fn_.name.name, fn_.span()),
				SyntaxDecl::Ty(ty) => {
					if let Some(prev_span) = tydecls.insert(ty.name.name, ty.span()) {
						self.report(Report::AnalyzeDuplicateDecl {
							prev_span,
							dup_span: ty.span(),
							name: ty.name.name,
						});
					}

					continue;
				}
			};

			if let Some(prev_span) = ntdecls.insert(name, span.clone()) {
				self.report(Report::AnalyzeDuplicateDecl {
					prev_span,
					dup_span: span,
					name,
				});
			}
		}
	}

	fn check_duplicate_parameters(&mut self, syntax_parameters: &SyntaxParameters<'src>) {
		let mut seen: HashMap<_, std::ops::Range<usize>> = HashMap::new();
		for (identifier, _) in &syntax_parameters.parameters {
			if let Some(identifier) = identifier {
				if let Some(first_span) = seen.get(identifier.name) {
					self.report(Report::AnalyzeDuplicateParameter {
						prev_span: first_span.clone(),
						dup_span: identifier.span(),
						name: identifier.name,
					});
				} else {
					seen.insert(identifier.name, identifier.span());
				}
			}
		}
	}

	fn check_empty_file(&mut self, config: &Config) {
		let mut has_evdecls = false;

		config.visit_ns_entries(|_, entry| {
			has_evdecls = has_evdecls || matches!(entry, NamespaceEntry::EvDecl(_) | NamespaceEntry::FnDecl(_));
		});

		if !has_evdecls {
			self.report(Report::AnalyzeEmptyEvDecls);
		}
	}

	fn check_conflicting_exports(&mut self, casing: Casing) {
		let send_events = casing.with("SendEvents", "sendEvents", "send_events");

		// we can do self.config.decls instead of traverse_namespaces as we're only interested in checking at the top level!
		let report = self.config.decls.iter().find_map(|decl| match decl {
			SyntaxDecl::Ev(evdecl) => {
				if evdecl.name.name == send_events {
					return Some(Report::AnalyzeConflictingExport {
						span: evdecl.span(),
						name: evdecl.name.name,
					});
				}
				None
			}
			SyntaxDecl::Fn(fndecl) => {
				if fndecl.name.name == send_events {
					return Some(Report::AnalyzeConflictingExport {
						span: fndecl.span(),
						name: fndecl.name.name,
					});
				}
				None
			}
			SyntaxDecl::Ns(nsdecl) => {
				if nsdecl.name.name == send_events {
					return Some(Report::AnalyzeConflictingExport {
						span: nsdecl.span(),
						name: nsdecl.name.name,
					});
				}
				None
			}
			SyntaxDecl::Ty(_) => None,
		});

		if let Some(report) = report {
			self.report(report);
		}
	}

	fn evdecl(&mut self, evdecl: &SyntaxEvDecl<'src>, id: usize) -> EvDecl<'src> {
		if let Some(syntax_parameters) = &evdecl.data {
			self.check_duplicate_parameters(syntax_parameters);
		}

		let name = evdecl.name.name;
		let from = evdecl.from;
		let evty = evdecl.evty;
		let call = if let Some(call) = evdecl.call {
			call
		} else if let Some(default) = self.call_default_opt(&self.config.opts.clone()) {
			default
		} else {
			self.report(Report::AnalyzeMissingEvDeclCall { span: evdecl.span() });

			// This value is not a default, it's only to allow execution to continue until error reporting.
			EvCall::ManySync
		};
		let data = evdecl.data.as_ref().map(|parameters| {
			parameters
				.parameters
				.iter()
				.map(|(identifier, ty)| {
					let name = identifier.map(|identifier| identifier.name);

					Parameter { name, ty: self.ty(ty) }
				})
				.collect::<Vec<_>>()
		});

		if let Some(parameters) = &data
			&& matches!(evty, EvType::Unreliable(_))
		{
			let start_size = match evty {
				EvType::Unreliable(true) => UNRELIABLE_ORDER_NUMTY.size(),
				_ => 0,
			};
			let mut min = start_size;
			let mut max = Some(start_size);

			for parameter in parameters {
				let (ty_min, ty_max) = parameter.ty.size(&mut HashSet::new());

				min += ty_min;

				if let (Some(ty_max), Some(max)) = (ty_max, max.as_mut()) {
					*max += ty_max;
				} else {
					max = None;
				}
			}

			if min > MAX_UNRELIABLE_SIZE {
				self.report(Report::AnalyzeOversizeUnreliable {
					ev_span: evdecl.span(),
					ty_span: evdecl.data.as_ref().unwrap().span(),
					size: min,
				});
			} else if max.is_none_or(|max| max >= MAX_UNRELIABLE_SIZE) {
				self.report(Report::AnalyzePotentiallyOversizeUnreliable {
					ev_span: evdecl.span(),
					ty_span: evdecl.data.as_ref().unwrap().span(),
				});
			}
		}

		EvDecl {
			name,
			from,
			evty,
			call,
			data: data.unwrap_or_default(),
			id,
			path: self.path.clone(),
		}
	}

	fn fndecl(&mut self, fndecl: &SyntaxFnDecl<'src>, client_id: usize, server_id: usize) -> FnDecl<'src> {
		if let Some(syntax_parameters) = &fndecl.args {
			self.check_duplicate_parameters(syntax_parameters);
		}

		if let Some(syntax_parameters) = &fndecl.rets {
			for parameter in &syntax_parameters.parameters {
				if let Some(identifier) = parameter.0 {
					self.report(Report::AnalyzeNamedReturn {
						name_span: identifier.span(),
					});
				}
			}
		}

		let name = fndecl.name.name;
		let call = fndecl.call;
		let args = fndecl.args.as_ref().map(|parameters| {
			parameters
				.parameters
				.iter()
				.map(|(identifier, ty)| {
					let name = identifier.map(|identifier| identifier.name);

					Parameter { name, ty: self.ty(ty) }
				})
				.collect::<Vec<_>>()
		});

		let rets = fndecl.rets.as_ref().map(|parameters| {
			parameters
				.parameters
				.iter()
				.map(|(_, ty)| self.ty(ty))
				.collect::<Vec<_>>()
		});

		FnDecl {
			name,
			args: args.unwrap_or_default(),
			call,
			rets,
			client_id,
			server_id,
			path: self.path.clone(),
		}
	}

	fn tydecl(&mut self, tydecl: &SyntaxTyDecl<'src>) -> TyDecl<'src> {
		let key = self
			.path
			.iter()
			.copied()
			.chain(std::iter::once(tydecl.name.name))
			.collect::<Vec<_>>()
			.join(".");

		let recursion_type = self.ty_recursion_kind(&key, &tydecl.ty, &mut HashSet::new());

		let ty = if let Some(ty) = self.resolved_tys.get(&*key) {
			ty.clone()
		} else {
			if let TyRecursionKind::Unbounded(ref_ty) = recursion_type {
				self.report(Report::AnalyzeUnboundedRecursiveType {
					decl_span: tydecl.span(),
					use_span: ref_ty.span(),
				});
			}

			let cache_ty = Rc::new(RefCell::new(Ty::Opt(Box::new(Ty::Unknown))));
			self.resolved_tys.insert(key, cache_ty.clone());
			self.current_tydecl = Some(tydecl.clone());
			let ty = self.ty(&tydecl.ty);
			self.current_tydecl = None;
			cache_ty.replace(ty);
			cache_ty
		};

		TyDecl {
			name: tydecl.name.name,
			ty,
			path: self.path.clone(),
			inline: recursion_type == TyRecursionKind::None,
		}
	}

	fn ty(&mut self, ty: &SyntaxTy<'src>) -> Ty<'src> {
		match &ty.kind {
			SyntaxTyKind::Num(numty, range) => Ty::Num(
				*numty,
				range
					.map(|range| self.checked_range_within(&range, numty.min(), numty.max()))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Str(kind, len) => {
				if kind.is_none() {
					self.report(Report::DeprecationNoStringDataKind { span: ty.span() });
				}

				let utf8 = kind.unwrap_or(false);

				Ty::Str(
					utf8,
					len.map(|range| self.checked_range_within(&range, 0.0, u16::MAX as f64))
						.unwrap_or_default()
						.mul_max(if utf8 { 4.0 } else { 1.0 }),
				)
			}

			SyntaxTyKind::Buf(len) => Ty::Buf(
				len.map(|range| self.checked_range_within(&range, 0.0, u16::MAX as f64))
					.unwrap_or_default(),
			),

			SyntaxTyKind::Vector(x_ty, y_ty, z_ty) => {
				match self.ty(x_ty) {
					Ty::Num(numty, _) => {
						if let NumTy::F64 = numty {
							self.report(Report::AnalyzeOversizeVectorComponent {
								span: Span {
									start: x_ty.start,
									end: x_ty.end,
								},
							});
						}
					}
					_ => self.report(Report::AnalyzeOversizeVectorComponent {
						span: Span {
							start: x_ty.start,
							end: x_ty.end,
						},
					}),
				};
				match self.ty(y_ty) {
					Ty::Num(numty, _) => {
						if let NumTy::F64 = numty {
							self.report(Report::AnalyzeOversizeVectorComponent {
								span: Span {
									start: y_ty.start,
									end: y_ty.end,
								},
							})
						}
					}
					_ => self.report(Report::AnalyzeInvalidVectorType {
						span: Span {
							start: y_ty.start,
							end: y_ty.end,
						},
					}),
				};
				if let Some(z_ty) = z_ty {
					match self.ty(z_ty) {
						Ty::Num(numty, _) => {
							if let NumTy::F64 = numty {
								self.report(Report::AnalyzeOversizeVectorComponent {
									span: Span {
										start: z_ty.start,
										end: z_ty.end,
									},
								})
							}
						}
						_ => self.report(Report::AnalyzeInvalidVectorType {
							span: Span {
								start: z_ty.start,
								end: z_ty.end,
							},
						}),
					};
				}

				Ty::Vector(
					Box::new(self.ty(x_ty)),
					Box::new(self.ty(y_ty)),
					z_ty.as_ref().map(|z_ty| Box::new(self.ty(z_ty))),
				)
			}

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

				Ty::Map(Box::new(key_ty), Box::new(val_ty))
			}

			SyntaxTyKind::Set(key) => {
				let key_ty = self.ty(key);

				Ty::Set(Box::new(key_ty))
			}

			SyntaxTyKind::Opt(ty) => {
				if let SyntaxTyKind::Or(tys) = &ty.kind {
					return self.or_ty(tys, true);
				}

				let parsed_ty = self.ty(ty);

				if let Ty::Opt(_) = parsed_ty {
					self.report(Report::AnalyzeInvalidOptionalType {
						span: (ty.span().end - 1)..ty.span().end,
					});
				}

				Ty::Opt(Box::new(parsed_ty))
			}

			SyntaxTyKind::Ref(ref_ty) => match ref_ty.name {
				"BrickColor" => Ty::BrickColor,
				"DateTimeMillis" => Ty::DateTimeMillis,
				"DateTime" => Ty::DateTime,
				"boolean" => Ty::Boolean,
				"Color3" => Ty::Color3,
				"Vector2" => Ty::Vector2,
				"Vector3" => Ty::Vector3,
				"AlignedCFrame" => Ty::AlignedCFrame,
				"CFrame" => Ty::CFrame,
				"unknown" => Ty::Opt(Box::new(Ty::Unknown)),

				_ => {
					let path = self
						.path
						.iter()
						.copied()
						.chain(std::iter::once(ref_ty.name))
						.collect::<Vec<_>>()
						.join(".");

					let Some(tydecl) = self.tydecls.get(&path).cloned() else {
						self.report(Report::AnalyzeUnknownTypeRef {
							span: ref_ty.span(),
							name: Cow::Borrowed(ref_ty.name),
						});

						return Ty::Opt(Box::new(Ty::Unknown));
					};

					let tydecl = self.tydecl(&tydecl);
					if tydecl.inline {
						(*tydecl.ty.borrow()).clone()
					} else {
						Ty::Ref(tydecl)
					}
				}
			},

			SyntaxTyKind::Path(raw_path) => {
				let path = self
					.path
					.iter()
					.copied()
					.chain(raw_path.iter().map(|i| i.name))
					.collect::<Vec<_>>()
					.join(".");

				let Some(tydecl) = self.all_tydecls.get(&path).cloned() else {
					self.report(Report::AnalyzeUnknownTypeRef {
						span: ty.span(),
						name: Cow::Owned(raw_path.iter().map(|i| i.name).collect::<Vec<_>>().join(".")),
					});

					return Ty::Opt(Box::new(Ty::Unknown));
				};

				if tydecl.inline {
					(*tydecl.ty.borrow()).clone()
				} else {
					Ty::Ref(tydecl)
				}
			}

			SyntaxTyKind::Enum(enum_ty) => Ty::Enum(self.enum_ty(enum_ty)),

			SyntaxTyKind::Struct(struct_ty) => Ty::Struct(self.struct_ty(struct_ty)),

			SyntaxTyKind::Instance(old, instance_ty) => {
				if *old {
					self.report(Report::DeprecationOldInstanceClassSpecifier {
						span: ty.span(),
						// old is true only if it's present
						name: instance_ty.unwrap().name,
					});
				}

				Ty::Instance(instance_ty.as_ref().map(|ty| ty.name))
			}

			SyntaxTyKind::Or(or_tys) => self.or_ty(or_tys, false),
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
				let tag_name = self.str(tag);

				if variants.is_empty() {
					self.report(Report::AnalyzeEmptyEnum { span });
				}

				let variants = variants
					.iter()
					.map(|(variant_name, variant_struct)| {
						if variant_struct.fields.iter().any(|(field, _)| field.name == tag_name) {
							self.report(Report::AnalyzeEnumTagUsed {
								tag_span: tag.span(),
								used_span: variant_name.span(),
								tag: tag_name,
							});
						}

						(variant_name.name, self.struct_ty(variant_struct))
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

		for (field, ty) in ty.fields.iter() {
			fields.push((field.name, self.ty(ty)));
		}

		Struct { fields }
	}

	fn or_ty(&mut self, or_tys: &Vec<SyntaxTy<'src>>, optional: bool) -> Ty<'src> {
		let mut tys = vec![];

		let mut used_tys = HashMap::new();
		let mut used_instances = HashMap::new();
		let mut used_variants = HashMap::new();
		let mut used_tags_values = HashMap::new();
		let mut prev_unknown_span = None;

		let mut or_tys = or_tys.iter().collect::<VecDeque<_>>();

		while let Some(syntax_ty) = or_tys.pop_front() {
			if let SyntaxTyKind::Or(tys) = &syntax_ty.kind {
				or_tys.extend(tys);
				continue;
			}

			let ty = self.ty(syntax_ty);
			if let Some(curr_tydecl) = &self.current_tydecl
				&& let Ty::Ref(tydecl) = &ty
				&& tydecl.path == self.path
				&& tydecl.name == curr_tydecl.name.name
			{
				self.report(Report::AnalyzeRecursiveOr {
					decl_span: curr_tydecl.name.span(),
					usage_span: syntax_ty.span(),
				});
				continue;
			}

			let prev_spans: Vec<_> = match ty.primitive_ty() {
				PrimitiveTy::Name(primitive) => used_tys.insert(primitive, syntax_ty.span()).into_iter().collect(),
				PrimitiveTy::Instance(class) => used_instances.insert(class, syntax_ty.span()).into_iter().collect(),
				PrimitiveTy::Enum(Enum::Unit(variants)) => variants
					.into_iter()
					.filter_map(|variant| used_variants.insert(variant, syntax_ty.span()))
					.collect(),
				PrimitiveTy::Enum(Enum::Tagged { tag, variants }) => variants
					.into_iter()
					.filter_map(|(variant, _)| used_tags_values.insert((tag, variant), syntax_ty.span()))
					.collect(),
				PrimitiveTy::Unknown => prev_unknown_span.replace(syntax_ty.span()).into_iter().collect(),
				PrimitiveTy::None(NonPrimitiveTy::Opt) => {
					self.report(Report::AnalyzeOrNestedOptional { span: syntax_ty.span() });
					continue;
				}
				// handled by the nested OR resolution above
				PrimitiveTy::None(NonPrimitiveTy::Or) => unreachable!(),
			};

			for prev_span in prev_spans {
				self.report(Report::AnalyzeOrDuplicateType {
					prev_span,
					dup_span: syntax_ty.span(),
				});
			}

			tys.push(ty);
		}

		// reorder the types into:
		// unit enums
		// tagged enums
		// ..
		// instance (no class)
		// unknown
		tys.sort_by(|ty_a, ty_b| match (ty_a.primitive_ty(), ty_b.primitive_ty()) {
			(PrimitiveTy::Unknown, _) => Ordering::Greater,
			(_, PrimitiveTy::Unknown) => Ordering::Less,
			(PrimitiveTy::Instance(None), _) => Ordering::Greater,
			(_, PrimitiveTy::Instance(None)) => Ordering::Less,
			(PrimitiveTy::Enum(Enum::Unit(..)), _) => Ordering::Less,
			(_, PrimitiveTy::Enum(Enum::Unit(..))) => Ordering::Greater,
			(PrimitiveTy::Enum(Enum::Tagged { .. }), _) => Ordering::Less,
			(_, PrimitiveTy::Enum(Enum::Tagged { .. })) => Ordering::Greater,
			_ => Ordering::Equal,
		});

		let optional = prev_unknown_span.is_some() || optional;
		let ty = Ty::Or(tys, optional);

		if optional { Ty::Opt(Box::new(ty)) } else { ty }
	}

	fn ty_recursion_kind(
		&self,
		target_path: &str,
		ty: &SyntaxTy<'src>,
		searched: &mut HashSet<String>,
	) -> TyRecursionKind<'src> {
		match &ty.kind {
			SyntaxTyKind::Arr(ty, len) => {
				let len = len.map(|len| self.range(&len)).unwrap_or_default();

				// if array does not have a min size of 0, it is unbounded
				if len.min().is_some_and(|min| min != 0.0) {
					self.ty_recursion_kind(target_path, ty, searched)
				} else {
					TyRecursionKind::None
				}
			}

			SyntaxTyKind::Ref(ref_ty) => {
				let key = self
					.path
					.iter()
					.copied()
					.chain(std::iter::once(ref_ty.name))
					.collect::<Vec<_>>()
					.join(".");

				if key == target_path {
					TyRecursionKind::Unbounded(*ref_ty)
				} else if searched.contains(&key) {
					TyRecursionKind::None
				} else if let Some(tydecl) = self.tydecls.get(&key) {
					searched.insert(key);
					self.ty_recursion_kind(target_path, &tydecl.ty, searched)
				} else {
					TyRecursionKind::None
				}
			}

			SyntaxTyKind::Path(path) => {
				let path = self
					.path
					.iter()
					.copied()
					.chain(path.iter().map(|i| i.name))
					.collect::<Vec<_>>()
					.join(".");

				if searched.contains(&path) {
					TyRecursionKind::None
				} else if let Some(tydecl) = self.tydecls.get(&path) {
					searched.insert(path);
					self.ty_recursion_kind(target_path, &tydecl.ty, searched)
				} else {
					TyRecursionKind::None
				}
			}

			SyntaxTyKind::Enum(enum_ty) => self.enum_recursion_kind(target_path, enum_ty, searched),
			SyntaxTyKind::Struct(struct_ty) => self.struct_recursion_kind(target_path, struct_ty, searched),
			SyntaxTyKind::Set(key_ty) => self.ty_recursion_kind(target_path, key_ty, searched),
			SyntaxTyKind::Or(tys) => tys
				.iter()
				.find_map(|ty| match self.ty_recursion_kind(target_path, ty, searched) {
					TyRecursionKind::None => None,
					kind => Some(kind),
				})
				.unwrap_or(TyRecursionKind::None),

			SyntaxTyKind::Opt(ty) => match self.ty_recursion_kind(target_path, ty, searched) {
				TyRecursionKind::None => TyRecursionKind::None,
				// it is bounded because it's optional
				_ => TyRecursionKind::Recursive,
			},

			_ => TyRecursionKind::None,
		}
	}

	fn enum_recursion_kind(
		&self,
		target_path: &str,
		ty: &SyntaxEnum<'src>,
		searched: &mut HashSet<String>,
	) -> TyRecursionKind<'src> {
		match &ty.kind {
			SyntaxEnumKind::Unit { .. } => TyRecursionKind::None,

			SyntaxEnumKind::Tagged { variants, .. } => {
				let mut kind = TyRecursionKind::None;

				for variant in variants.iter() {
					match self.struct_recursion_kind(target_path, &variant.1, searched) {
						TyRecursionKind::Unbounded(ident) => return TyRecursionKind::Unbounded(ident),
						TyRecursionKind::Recursive => kind = TyRecursionKind::Recursive,
						TyRecursionKind::None => {}
					};
				}

				kind
			}
		}
	}

	fn struct_recursion_kind(
		&self,
		target_path: &str,
		ty: &SyntaxStruct<'src>,
		searched: &mut HashSet<String>,
	) -> TyRecursionKind<'src> {
		let mut kind = TyRecursionKind::None;

		for field in ty.fields.iter() {
			match self.ty_recursion_kind(target_path, &field.1, searched) {
				TyRecursionKind::Unbounded(ident) => return TyRecursionKind::Unbounded(ident),
				TyRecursionKind::Recursive => kind = TyRecursionKind::Recursive,
				TyRecursionKind::None => {}
			};
		}

		kind
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
