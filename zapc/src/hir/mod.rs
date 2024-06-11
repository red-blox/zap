use decl::HirRemote;
use scope::HirScope;
use ty::HirTy;

pub mod decl;
pub mod scope;
pub mod ty;

#[derive(Debug)]
pub struct Hir {
	init_scope: HirScope,
	ty_decls: Vec<HirTy>,
	remote_decls: Vec<HirRemote>,
}

impl Hir {
	pub fn new(init_scope: HirScope, ty_decls: Vec<HirTy>, remote_decls: Vec<HirRemote>) -> Self {
		Self {
			init_scope,
			ty_decls,
			remote_decls,
		}
	}
}
