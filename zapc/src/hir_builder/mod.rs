use lasso::Rodeo;
use scope::Resolved;

use crate::{
	ast::Ast,
	hir::{decl::HirRemote, scope::HirScope, ty::HirTy, Hir},
	meta::Report,
};

mod decl;
mod range;
mod scope;
mod ty;

pub struct HirBuilder<'a> {
	reports: Vec<Report>,
	rodeo: &'a mut Rodeo,

	init_scope: HirScope,
	ty_decls: Vec<Resolved<HirTy>>,
	remote_decls: Vec<Resolved<HirRemote>>,
}

impl<'a> HirBuilder<'a> {
	pub fn new(rodeo: &'a mut Rodeo) -> Self {
		Self {
			rodeo,
			reports: Vec::new(),
			init_scope: Default::default(),
			ty_decls: Vec::new(),
			remote_decls: Vec::new(),
		}
	}

	pub fn init_ast(mut self, ast: Ast) -> Result<Hir, Vec<Report>> {
		self.decls(&Self::INIT_SCOPEID, ast.into_decls());

		let mut ty_decls = Vec::new();

		for ty in self.ty_decls {
			if let Resolved::Resolved(_, ty) = ty {
				ty_decls.push(ty);
			} else {
				// todo: report error
			}
		}

		let mut remote_decls = Vec::new();

		for remote in self.remote_decls {
			if let Resolved::Resolved(_, remote) = remote {
				remote_decls.push(remote);
			} else {
				// todo: report error
			}
		}

		if self.reports.is_empty() {
			Ok(Hir::new(self.init_scope, ty_decls, remote_decls))
		} else {
			Err(self.reports)
		}
	}

	fn report(&mut self, report: Report) {
		self.reports.push(report);
	}
}
