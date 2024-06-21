use std::collections::HashMap;

use lasso::Spur;

use crate::{
	ast::{
		decl::{AstConfig, AstConfigValue, AstDecl},
		primitive::AstWord,
	},
	hir::decl::{HirEvent, HirEventSource, HirRemote, HirRemoteBatching, HirRemoteId},
	meta::{Report, Span},
};

use super::{scope::ScopeId, HirBuilder};

impl<'a> HirBuilder<'a> {
	fn report_duplicate_decl(&mut self, decl_kind: &str, seen: &mut HashMap<Spur, Span>, name: &AstWord, span: Span) {
		if let Some(prev_span) = seen.insert(name.spur(), span) {
			self.report(Report::DuplicateDecl {
				decl_kind: decl_kind.to_string(),
				name: name.word(self.rodeo).to_string(),
				span,
				first_decl_span: prev_span,
			});
		}
	}

	pub fn decls(&mut self, scope: &ScopeId, decls: Vec<AstDecl>) {
		let mut seen_types = HashMap::new();
		let mut seen_scopes = HashMap::new();
		let mut seen_events = HashMap::new();
		let mut seen_remotes = HashMap::new();

		for decl in &decls {
			match decl {
				AstDecl::Ty { name, span, .. } => self.report_duplicate_decl("type", &mut seen_types, name, *span),
				AstDecl::Scope { name, span } => self.report_duplicate_decl("scope", &mut seen_scopes, name, *span),
				AstDecl::Event { name, span, .. } => self.report_duplicate_decl("event", &mut seen_events, name, *span),
				AstDecl::Remote { name, span, .. } => {
					self.report_duplicate_decl("remote", &mut seen_remotes, name, *span)
				}
			}
		}

		for decl in decls.into_iter() {
			self.decl(scope, decl);
		}
	}

	fn decl(&mut self, scope: &ScopeId, ast: AstDecl) {
		match ast {
			AstDecl::Ty { name, ty, span } => {
				let ty = self.ty(scope, ty);
				self.add_ty_decl(scope, name.spur(), span, ty);
			}

			AstDecl::Scope { name, span } => {
				// todo: find file and do stuff
			}

			AstDecl::Event { name, config, tys, .. } => {
				let event = self
					.event_config(scope, config)
					.add_tys(tys.into_iter().map(|ty| self.ty(scope, ty)).collect());

				self.add_event(scope, name.spur(), event);
			}

			AstDecl::Remote { name, config, span } => {
				let remote = self.remote_config(config);
				self.add_remote(scope, name.spur(), span, remote);
			}
		}
	}

	fn event_config(&mut self, scope: &ScopeId, ast: AstConfig) -> HirEvent {
		let mut seen = HashMap::new();

		let mut from = None;
		let mut over = None;

		for (field, value) in ast.into_fields() {
			let span = field.span().merge(value.span());

			if let Some(prev_span) = seen.insert(field.spur(), span) {
				self.report(Report::DuplicateField {
					span,
					first_field_span: prev_span,
					field: field.word(self.rodeo).to_string(),
				});

				continue;
			}

			match field.word(self.rodeo) {
				"from" => from = Some(self.event_config_from(value)),
				"over" => over = Some(self.event_config_over(scope, value)),
				_ => self.report(Report::UnexpectedField {
					span: field.span(),
					field: field.word(self.rodeo).to_string(),
					decl_kind: "events".to_string(),
				}),
			}
		}

		HirEvent::new(from.unwrap_or(HirEventSource::Server), over.unwrap_or(HirRemoteId(0)))
	}

	fn event_config_over(&mut self, scope: &ScopeId, ast: AstConfigValue) -> HirRemoteId {
		let span = ast.span();

		match ast {
			AstConfigValue::Boolean(_, span) => {
				// todo: report error
				HirRemoteId(0)
			}

			AstConfigValue::Number(number) => {
				// todo: report error
				HirRemoteId(0)
			}

			AstConfigValue::String(string) => {
				// todo: report error
				HirRemoteId(0)
			}

			AstConfigValue::Path(words) => self.get_remote_id(scope, &words, span),
		}
	}

	fn event_config_from(&mut self, ast: AstConfigValue) -> HirEventSource {
		match ast {
			AstConfigValue::String(string) => {
				match string.string(self.rodeo) {
					"server" => HirEventSource::Server,
					"client" => HirEventSource::Client,

					other => {
						// todo: report error
						HirEventSource::Server
					}
				}
			}

			AstConfigValue::Boolean(_, span) => {
				// todo: report error
				HirEventSource::Server
			}

			AstConfigValue::Number(number) => {
				// todo: report error
				HirEventSource::Server
			}

			AstConfigValue::Path(words) => {
				if words.len() == 1 {
					let word = words.first().unwrap().word(self.rodeo);

					if word == "server" || word == "Server" {
						// todo: report special error
					} else if word == "client" || word == "Client" {
						// todo: report special error
					}
				} else {
					// todo: report error
				}

				HirEventSource::Server
			}
		}
	}

	fn remote_config(&mut self, ast: AstConfig) -> HirRemote {
		let mut seen = HashMap::new();

		let mut reliable = None;
		let mut batching = None;

		for (field, value) in ast.into_fields() {
			let span = field.span().merge(value.span());

			if let Some(prev_span) = seen.insert(field.spur(), span) {
				self.report(Report::DuplicateField {
					span,
					first_field_span: prev_span,
					field: field.word(self.rodeo).to_string(),
				});

				continue;
			}

			match field.word(self.rodeo) {
				"reliable" => reliable = Some(self.remote_config_reliable(value)),
				"batching" => batching = Some(self.remote_config_batching(value)),
				_ => self.report(Report::UnexpectedField {
					span: field.span(),
					field: field.word(self.rodeo).to_string(),
					decl_kind: "remotes".to_string(),
				}),
			}
		}

		if batching.is_none() {
			// todo: report error
		}

		HirRemote::new(reliable.unwrap_or(true), batching.unwrap_or(HirRemoteBatching::None))
	}

	fn remote_config_batching(&mut self, ast: AstConfigValue) -> HirRemoteBatching {
		match ast {
			AstConfigValue::Boolean(value, span) => {
				if value {
					HirRemoteBatching::MaxTime(0.0)
				} else {
					HirRemoteBatching::None
				}
			}

			AstConfigValue::Number(number) => {
				if number.value().is_sign_negative() {
					// todo: report error
					HirRemoteBatching::None
				} else {
					HirRemoteBatching::MaxTime(number.value())
				}
			}

			AstConfigValue::String(string) => {
				// todo: report error
				HirRemoteBatching::None
			}

			AstConfigValue::Path(words) => {
				// todo: potential special error
				// todo: report error
				HirRemoteBatching::None
			}
		}
	}

	fn remote_config_reliable(&mut self, ast: AstConfigValue) -> bool {
		match ast {
			AstConfigValue::Boolean(value, _) => value,

			AstConfigValue::Number(number) => {
				// todo: report error
				true
			}

			AstConfigValue::String(string) => {
				// todo: report error
				true
			}

			AstConfigValue::Path(words) => {
				// todo: report error
				true
			}
		}
	}
}
