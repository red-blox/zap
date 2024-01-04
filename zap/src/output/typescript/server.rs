use crate::config::{Config, EvCall, EvDecl, EvSource, TyDecl};

use super::Output;

struct ServerOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl<'a> Output for ServerOutput<'a> {
	fn push(&mut self, s: &str) {
		self.buf.push_str(s);
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
	pub fn new(config: &'a Config) -> Self {
		Self {
			config,
			tabs: 0,
			buf: String::new(),
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
		for tydecl in self.config.tydecls.iter() {
			self.push_tydecl(tydecl);
		}

		if !self.config.tydecls.is_empty() {
			self.push("\n")
		}
	}

	fn push_return_fire(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire = self.config.casing.with("Fire", "fire", "fire");
		let player = self.config.casing.with("Player", "player", "player");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire}: ({player}: Player, {value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_fire_all(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_all = self.config.casing.with("FireAll", "fireAll", "fire_all");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_all}: ({value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_fire_except(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_except = self.config.casing.with("FireExcept", "fireExcept", "fire_except");
		let except = self.config.casing.with("Except", "except", "except");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_except}: ({except}: Player, {value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_fire_list(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_list = self.config.casing.with("FireList", "fireList", "fire_list");
		let list = self.config.casing.with("List", "list", "list");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_list}: ({list}: Player[], {value}: "));
		self.push_ty(ty);
		self.push(") => void\n");
	}

	fn push_return_outgoing(&mut self) {
		for (_i, ev) in self
			.config
			.evdecls
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

	pub fn push_return_listen(&mut self) {
		for (_i, ev) in self
			.config
			.evdecls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			self.push_line(&format!("export const {name}: {{", name = ev.name));
			self.indent();

			let set_callback = match ev.call {
				EvCall::SingleSync | EvCall::SingleAsync => {
					self.config.casing.with("SetCallback", "setCallback", "set_callback")
				}
				EvCall::ManySync | EvCall::ManyAsync => self.config.casing.with("On", "on", "on"),
			};
			let callback = self.config.casing.with("Callback", "callback", "callback");
			let player = self.config.casing.with("Player", "player", "player");
			let value = self.config.casing.with("Value", "value", "value");

			self.push_indent();
			self.push(&format!("{set_callback}: ({callback}: ({player}: Player, {value}: "));
			self.push_ty(&ev.data);
			self.push(") => void) => void\n");

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

		if self.config.evdecls.is_empty() {
			return self.buf;
		};

		if self.config.manual_event_loop {
			self.push_manual_event_loop(self.config);
		}

		self.push_tydecls();

		self.push_return();

		self.buf
	}
}

pub fn code(config: &Config) -> Option<String> {
	if !config.typescript {
		return None;
	}

	Some(ServerOutput::new(config).output())
}
