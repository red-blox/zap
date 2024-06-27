use lasso::Spur;

use crate::{
	ast::primitive::AstWord,
	hir::{
		decl::{HirEvent, HirRemote, HirRemoteId, HirTyDeclId},
		scope::HirScope,
		ty::HirTy,
	},
	meta::Span,
};

use super::HirBuilder;

#[derive(Debug, Clone)]
pub enum Resolved<T> {
	Resolved(Span, T),
	Unresolved(Vec<Span>),
}

#[derive(Debug, Clone)]
pub struct ScopeId(Vec<Spur>);

impl ScopeId {
	pub fn index(&self, name: Spur) -> Self {
		Self(self.0.iter().copied().chain(std::iter::once(name)).collect())
	}

	pub fn index_all(&self, names: &[Spur]) -> Self {
		let mut scope_id = self.clone();

		for name in names {
			scope_id = scope_id.index(*name);
		}

		scope_id
	}
}

impl<'a> HirBuilder<'a> {
	pub const INIT_SCOPEID: ScopeId = ScopeId(vec![]);

	fn get_scope(&mut self, scope_id: &ScopeId) -> &mut HirScope {
		let mut scope = &mut self.init_scope;

		for name in scope_id.0.iter() {
			scope = scope.get_scope(*name);
		}

		scope
	}

	fn get_or_create_ty_decl_id(&mut self, from: &ScopeId, name: Spur, span: Span) -> HirTyDeclId {
		let next_id = self.ty_decls.len();
		let scope = self.get_scope(from);

		match scope.get_ty_decl(name) {
			Some(id) => id,

			None => {
				let id = HirTyDeclId(next_id);
				scope.add_ty_decl(name, id);
				self.ty_decls.push(Resolved::Unresolved(vec![span]));

				id
			}
		}
	}

	pub fn add_ty_decl(&mut self, from: &ScopeId, name: Spur, span: Span, ty: HirTy) -> HirTyDeclId {
		let id = self.get_or_create_ty_decl_id(from, name, span);

		if let Resolved::Resolved(..) = self.ty_decls[id.0] {
			// Duplicate error is reported in `decl.rs`.
		} else {
			self.ty_decls[id.0] = Resolved::Resolved(span, ty);
		}

		id
	}

	pub fn get_ty_id(&mut self, from: &ScopeId, path: &[AstWord], span: Span) -> HirTyDeclId {
		let mut spurs = path.iter().map(|w| w.spur()).collect::<Vec<_>>();

		match spurs.first() {
			Some(init) if self.rodeo.get_or_intern_static("init") == *init => {
				if spurs.len() == 1 {
					// todo: report error

					HirTyDeclId(0)
				} else {
					let last_spur = spurs.pop().unwrap();
					let scope_id = ScopeId(spurs);

					self.get_or_create_ty_decl_id(&scope_id, last_spur, span)
				}
			}

			Some(..) => {
				let last_spur = spurs.pop().unwrap();
				let scope_id = from.index_all(&spurs);

				self.get_or_create_ty_decl_id(&scope_id, last_spur, span)
			}

			None => unreachable!(),
		}
	}

	fn get_or_create_remote_id(&mut self, from: &ScopeId, name: Spur, span: Span) -> HirRemoteId {
		let next_id = self.remote_decls.len();
		let scope = self.get_scope(from);

		match scope.get_remote_decl(name) {
			Some(id) => id,

			None => {
				let id = HirRemoteId(next_id);
				scope.add_remote_decl(name, id);
				self.remote_decls.push(Resolved::Unresolved(vec![span]));

				id
			}
		}
	}

	pub fn add_remote(&mut self, from: &ScopeId, name: Spur, span: Span, remote: HirRemote) -> HirRemoteId {
		let id = self.get_or_create_remote_id(from, name, span);

		if let Resolved::Resolved(..) = self.remote_decls[id.0] {
			// Duplicate error is reported in `decl.rs`.
		} else {
			self.remote_decls[id.0] = Resolved::Resolved(span, remote);
		}

		id
	}

	pub fn get_remote_id(&mut self, from: &ScopeId, path: &[AstWord], span: Span) -> HirRemoteId {
		let mut spurs = path.iter().map(|w| w.spur()).collect::<Vec<_>>();

		match spurs.first() {
			Some(init) if self.rodeo.get_or_intern_static("init") == *init => {
				if spurs.len() == 1 {
					// todo: report error

					HirRemoteId(0)
				} else {
					let last_spur = spurs.pop().unwrap();
					let scope_id = ScopeId(spurs);

					self.get_or_create_remote_id(&scope_id, last_spur, span)
				}
			}

			Some(..) => {
				let last_spur = spurs.pop().unwrap();
				let scope_id = from.index_all(&spurs);

				self.get_or_create_remote_id(&scope_id, last_spur, span)
			}

			None => unreachable!(),
		}
	}

	pub fn add_event(&mut self, from: &ScopeId, name: Spur, event: HirEvent) {
		self.get_scope(from).add_event_decl(name, event);
	}
}
