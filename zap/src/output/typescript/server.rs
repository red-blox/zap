use crate::{
	parser::{EvCall, EvDecl, EvSource, File, TyDecl},
	util::casing,
};

use super::Output;

struct ServerOutput<'a> {
	file: &'a File,
	buff: String,
	tabs: u32,
}

impl<'a> Output for ServerOutput<'a> {
	fn push(&mut self, s: &str) {
		self.buff.push_str(s);
	}

	fn indent(&mut self) {
		self.tabs += 1;
	}

	fn dedent(&mut self) {
		self.tabs -= 1;
	}

	fn push_indent(&mut self) {
		for _ in 0..self.tabs {
			self.push("\t");
		}
	}
}

impl<'a> ServerOutput<'a> {
	pub fn new(file: &'a File) -> Self {
		Self {
			file,
			buff: String::new(),
			tabs: 0,
		}
	}

	fn push_tydecl(&mut self, tydecl: &TyDecl) {
		let name = &tydecl.name;
		let ty = &tydecl.ty;

		self.push_indent();
		self.push(&format!("type {name} = "));
		self.push_ty(ty);
		self.push("\n");
	}

	fn push_tydecls(&mut self) {
		for tydecl in self.file.ty_decls.iter() {
			self.push_tydecl(tydecl);
		}

		if !self.file.ty_decls.is_empty() {
			self.push("\n")
		}
	}

	fn push_return_fire(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire = casing(self.file.casing, "Fire", "fire", "fire");
		let player = casing(self.file.casing, "Player", "player", "player");
		let value = casing(self.file.casing, "Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire}: ({player}: Player, {value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_fire_all(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_all = casing(self.file.casing, "FireAll", "fireAll", "fire_all");
		let value = casing(self.file.casing, "Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_all}: ({value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_fire_except(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_except = casing(self.file.casing, "FireExcept", "fireExcept", "fire_except");
		let except = casing(self.file.casing, "Except", "except", "except");
		let value = casing(self.file.casing, "Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_except}: ({except}: Player, {value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_fire_list(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_list = casing(self.file.casing, "FireList", "fireList", "fire_list");
		let list = casing(self.file.casing, "List", "list", "list");
		let value = casing(self.file.casing, "Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_list}: ({list}: Player[], {value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_outgoing(&mut self) {
		for (_i, ev) in self
			.file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
		{
			self.push_line(&format!("export const {name}: {{", name = ev.name));
			self.indent();

			self.push_return_fire(ev);
			self.push_return_fire_all(ev);
			self.push_return_fire_except(ev);
			self.push_return_fire_list(ev);

			self.dedent();
			self.push_line("};");
		}
	}

	fn push_return_setcallback(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let set_callback = match ev.call {
			EvCall::SingleSync | EvCall::SingleAsync => {
				casing(self.file.casing, "SetCallback", "setCallback", "set_callback")
			}
			EvCall::ManySync | EvCall::ManyAsync => casing(self.file.casing, "On", "on", "on"),
		};
		let callback = casing(self.file.casing, "Callback", "callback", "callback");
		let player = casing(self.file.casing, "Player", "player", "player");
		let value = casing(self.file.casing, "Value", "value", "value");

		self.push_indent();
		self.push(&format!("{set_callback}: ({callback}: ({player}: Player, {value}: "));
		self.push_ty(ty);
		self.push(") => void) => void\n");
	}

	pub fn push_return_listen(&mut self) {
		for (_i, ev) in self
			.file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			self.push_line(&format!("export const {name}: {{", name = ev.name));
			self.indent();

			self.push_return_setcallback(ev);

			self.dedent();
			self.push_line("};");
		}
	}

	pub fn push_return(&mut self) {
		self.push_return_outgoing();
		self.push_return_listen();
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Server");

		self.push_tydecls();

		self.push_return();

		self.buff
	}
}

pub fn code(file: &File) -> Option<String> {
	if !file.typescript {
		return None;
	}

	Some(ServerOutput::new(file).output())
}
