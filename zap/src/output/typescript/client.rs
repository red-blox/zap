use crate::config::{Config, EvCall, EvSource, TyDecl, YieldType};

use super::ConfigProvider;
use super::Output;

struct ClientOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl<'a> Output for ClientOutput<'a> {
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

impl<'a> ConfigProvider for ClientOutput<'a> {
	fn get_config(&self) -> &Config {
		self.config
	}
}

impl<'src> ClientOutput<'src> {
	pub fn new(config: &'src Config<'src>) -> Self {
		Self {
			config,
			buf: String::new(),
			tabs: 0,
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
		for tydecl in &self.config.tydecls {
			self.push_tydecl(tydecl);
		}

		if !self.config.tydecls.is_empty() {
			self.push("\n")
		}
	}

	fn push_return_outgoing(&mut self) {
		for (_i, ev) in self
			.config
			.evdecls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			let fire = self.config.casing.with("Fire", "fire", "fire");
			let value = self.config.casing.with("Value", "value", "value");

			self.push_line(&format!("export declare const {name}: {{", name = ev.name));
			self.indent();

			self.push_indent();
			self.push(&format!("{fire}: ("));

			if let Some(data) = &ev.data {
				self.push(value);
				self.push_arg_ty(data);
			}

			self.push(") => void;\n");

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
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
		{
			let set_callback = match ev.call {
				EvCall::SingleSync | EvCall::SingleAsync => {
					self.config.casing.with("SetCallback", "setCallback", "set_callback")
				}
				EvCall::ManySync | EvCall::ManyAsync => self.config.casing.with("On", "on", "on"),
			};
			let callback = self.config.casing.with("Callback", "callback", "callback");
			let value = self.config.casing.with("Value", "value", "value");

			self.push_line(&format!("export declare const {name}: {{", name = ev.name));
			self.indent();

			self.push_indent();
			self.push(&format!("{set_callback}: ({callback}: ("));

			if let Some(data) = &ev.data {
				self.push(value);
				self.push_arg_ty(data);
			}

			self.push(") => void) => () => void;\n");

			self.dedent();
			self.push_line("};");
		}
	}

	fn push_return_functions(&mut self) {
		let call = self.config.casing.with("Call", "call", "call");
		let value = self.config.casing.with("Value", "value", "value");

		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("export declare const {}: {{", fndecl.name));
			self.indent();

			self.push_indent();
			self.push(&format!("{call}: ("));

			if let Some(data) = &fndecl.args {
				self.push(value);
				self.push_arg_ty(data);
			}

			self.push(") => ");

			if self.config.yield_type == YieldType::Promise {
				self.push("Promise<")
			}

			if let Some(data) = &fndecl.rets {
				self.push_ty(data);
			} else {
				self.push("void");
			}

			if self.config.yield_type == YieldType::Promise {
				self.push(">")
			}

			self.push(";\n");
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
		self.push_file_header("Client");

		if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
			return self.buf;
		};

		if self.config.manual_event_loop {
			self.push_manual_event_loop();
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

	Some(ClientOutput::new(config).output())
}
