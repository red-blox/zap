use crate::config::{Config, EvCall, EvSource, TyDecl, YieldType};

use super::ConfigProvider;
use super::Output;

struct ClientOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl Output for ClientOutput<'_> {
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

impl ConfigProvider for ClientOutput<'_> {
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

			self.push_line(&format!("export declare const {name}: {{", name = ev.name));
			self.indent();

			self.push_indent();
			self.push(&format!("{fire}: ("));

			if !ev.data.is_empty() {
				self.push_parameters(&ev.data);
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
			self.push_line(&format!("export declare const {name}: {{", name = ev.name));
			self.indent();

			if ev.call == EvCall::Polling {
				let iter = self.config.casing.with("Iter", "iter", "iter");
				let index = self.config.casing.with("Index", "index", "index");
				let value = self.config.casing.with("Value", "value", "value");

				self.push_indent();
				self.push(&format!("{iter}: Iter<LuaTuple<[{index}: number"));

				for (index, parameter) in ev.data.iter().enumerate() {
					let name = match parameter.name {
						Some(name) => name.to_string(),
						None => {
							if index > 0 {
								format!("{value}{}", index + 1)
							} else {
								value.to_string()
							}
						}
					};

					self.push(&format!(", {}: ", name));
					self.push_ty(&parameter.ty);
				}

				self.push("]>>;\n");
			} else {
				let callback = self.config.casing.with("Callback", "callback", "callback");
				let set_callback = match ev.call {
					EvCall::SingleSync | EvCall::SingleAsync => {
						self.config.casing.with("SetCallback", "setCallback", "set_callback")
					}
					EvCall::ManySync | EvCall::ManyAsync => self.config.casing.with("On", "on", "on"),
					_ => unreachable!(),
				};

				self.push_indent();
				self.push(&format!("{set_callback}: ({callback}: ("));

				if !ev.data.is_empty() {
					self.push_parameters(&ev.data);
				}
				self.push(") => void) => () => void;\n");
			}

			self.dedent();
			self.push_line("};");
		}
	}

	fn push_return_functions(&mut self) {
		let call = self.config.casing.with("Call", "call", "call");

		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("export declare const {}: {{", fndecl.name));
			self.indent();

			self.push_indent();
			self.push(&format!("{call}: ("));

			if !fndecl.args.is_empty() {
				self.push_parameters(&fndecl.args);
			}

			self.push(") => ");

			if self.config.yield_type == YieldType::Promise {
				self.push("Promise<")
			}

			if let Some(types) = &fndecl.rets {
				if types.len() > 1 {
					self.push("LuaTuple<[");
				}

				for (i, ty) in types.iter().enumerate() {
					if i > 0 {
						self.push(", ");
					}

					self.push_ty(ty);
				}

				if types.len() > 1 {
					self.push("]>");
				}
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
			self.push_line("export {}");
			return self.buf;
		};

		if self.config.evdecls.iter().any(|ev| ev.call == EvCall::Polling) {
			self.push_iter_type()
		}

		self.push_event_loop();

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
