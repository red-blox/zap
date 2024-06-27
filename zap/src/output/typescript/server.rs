use crate::config::{Config, EvCall, EvDecl, EvSource, TyDecl};

use super::ConfigProvider;
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

impl<'a> ConfigProvider for ServerOutput<'a> {
	fn get_config(&self) -> &Config {
		self.config
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
		self.push(";\n");
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
		let fire = self.config.casing.with("Fire", "fire", "fire");
		let player = self.config.casing.with("Player", "player", "player");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire}: ({player}: Player"));

		if let Some(data) = &ev.data {
			self.push(&format!(", {value}"));
			self.push_arg_ty(data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_all(&mut self, ev: &EvDecl) {
		let fire_all = self.config.casing.with("FireAll", "fireAll", "fire_all");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_all}: ("));

		if let Some(data) = &ev.data {
			self.push(value);
			self.push_arg_ty(data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_except(&mut self, ev: &EvDecl) {
		let fire_except = self.config.casing.with("FireExcept", "fireExcept", "fire_except");
		let except = self.config.casing.with("Except", "except", "except");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_except}: ({except}: Player"));

		if let Some(data) = &ev.data {
			self.push(&format!(", {value}"));
			self.push_arg_ty(data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_list(&mut self, ev: &EvDecl) {
		let fire_list = self.config.casing.with("FireList", "fireList", "fire_list");
		let list = self.config.casing.with("List", "list", "list");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_list}: ({list}: Player[]"));

		if let Some(data) = &ev.data {
			self.push(&format!(", {value}"));
			self.push_arg_ty(data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_set(&mut self, ev: &EvDecl) {
		let fire_set = self.config.casing.with("FireSet", "fireSet", "fire_set");
		let set = self.config.casing.with("Set", "set", "set");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_set}: ({set}: Set<Player>"));

		if let Some(data) = &ev.data {
			self.push(&format!(", {value}"));
			self.push_arg_ty(data);
		}

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
			self.push_line(&format!("export declare const {name}: {{", name = ev.name));
			self.indent();

			self.push_return_fire(ev);
			self.push_return_fire_all(ev);
			self.push_return_fire_except(ev);
			self.push_return_fire_list(ev);
			self.push_return_fire_set(ev);

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
			self.push_line(&format!("export declare const {name}: {{", name = ev.name));
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
			self.push(&format!("{set_callback}: ({callback}: ({player}: Player"));

			if let Some(data) = &ev.data {
				self.push(&format!(", {value}"));
				self.push_arg_ty(data);
			}

			self.push(") => void) => () => void;\n");

			self.dedent();
			self.push_line("};");
		}
	}

	pub fn push_return_functions(&mut self) {
		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("export declare const {name}: {{", name = fndecl.name));
			self.indent();

			let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
			let callback = self.config.casing.with("Callback", "callback", "callback");
			let player = self.config.casing.with("Player", "player", "player");
			let value = self.config.casing.with("Value", "value", "value");

			self.push_indent();
			self.push(&format!("{set_callback}: ({callback}: ({player}: Player"));

			if let Some(data) = &fndecl.args {
				self.push(&format!(", {value}"));
				self.push_arg_ty(data);
			}

			self.push(") => ");

			if let Some(data) = &fndecl.rets {
				self.push_ty(data);
			} else {
				self.push("void");
			}

			self.push(") => () => void;\n");

			self.dedent();
			self.push_line("};");
		}
	}

	pub fn push_return(&mut self) {
		self.push_return_outgoing();
		self.push_return_listen();
		self.push_return_functions();
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Server");

		if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
			return self.buf;
		};

		self.push_manual_event_loop();

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
