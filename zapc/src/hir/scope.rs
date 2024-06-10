use std::collections::HashMap;

use lasso::Spur;

use super::decl::{HirEvent, HirRemoteId, HirTyDeclId};

#[derive(Debug, Clone, Default)]
pub struct HirScope {
	scopes: HashMap<Spur, HirScope>,
	ty_decls: HashMap<Spur, HirTyDeclId>,
	remote_decls: HashMap<Spur, HirRemoteId>,
	event_decls: HashMap<Spur, HirEvent>,
}

impl HirScope {
	pub fn get_scope(&mut self, name: Spur) -> &mut HirScope {
		self.scopes.entry(name).or_default()
	}

	pub fn add_ty_decl(&mut self, name: Spur, id: HirTyDeclId) {
		self.ty_decls.insert(name, id);
	}

	pub fn get_ty_decl(&mut self, name: Spur) -> Option<HirTyDeclId> {
		self.ty_decls.get(&name).copied()
	}

	pub fn add_remote_decl(&mut self, name: Spur, id: HirRemoteId) {
		self.remote_decls.insert(name, id);
	}

	pub fn get_remote_decl(&mut self, name: Spur) -> Option<HirRemoteId> {
		self.remote_decls.get(&name).copied()
	}

	pub fn add_event_decl(&mut self, name: Spur, event: HirEvent) {
		self.event_decls.insert(name, event);
	}
}
