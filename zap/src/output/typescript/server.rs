use crate::config::{Config, EvCall, EvDecl, EvSource, TyDecl};

use super::ConfigProvider;
use super::Output;

struct ServerOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl Output for ServerOutput<'_> {
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

impl ConfigProvider for ServerOutput<'_> {
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

		self.push_indent();
		self.push(&format!("{fire}: ({player}: Player"));

		if !ev.data.is_empty() {
			self.push(", ");
			self.push_parameters(&ev.data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_all(&mut self, ev: &EvDecl) {
		let fire_all = self.config.casing.with("FireAll", "fireAll", "fire_all");

		self.push_indent();
		self.push(&format!("{fire_all}: ("));

		if !ev.data.is_empty() {
			self.push_parameters(&ev.data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_except(&mut self, ev: &EvDecl) {
		let fire_except = self.config.casing.with("FireExcept", "fireExcept", "fire_except");
		let except = self.config.casing.with("Except", "except", "except");

		self.push_indent();
		self.push(&format!("{fire_except}: ({except}: Player"));

		if !ev.data.is_empty() {
			self.push(", ");
			self.push_parameters(&ev.data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_list(&mut self, ev: &EvDecl) {
		let fire_list = self.config.casing.with("FireList", "fireList", "fire_list");
		let list = self.config.casing.with("List", "list", "list");

		self.push_indent();
		self.push(&format!(
			"{fire_list}: ({list}: Player[] | Record<string | number | symbol, Player> | Map<unknown, Player>"
		));

		if !ev.data.is_empty() {
			self.push(", ");
			self.push_parameters(&ev.data);
		}

		self.push(") => void;\n");
	}

	fn push_return_fire_set(&mut self, ev: &EvDecl) {
		let fire_set = self.config.casing.with("FireSet", "fireSet", "fire_set");
		let set = self.config.casing.with("Set", "set", "set");

		self.push_indent();
		self.push(&format!("{fire_set}: ({set}: Set<Player> | Map<Player, unknown>"));

		if !ev.data.is_empty() {
			self.push(", ");
			self.push_parameters(&ev.data);
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

			if !self.config.disable_fire_all {
				self.push_return_fire_all(ev);
			}

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

			if ev.call == EvCall::Polling {
				let index = self.config.casing.with("Index", "index", "index");
				let iter = self.config.casing.with("Iter", "iter", "iter");
				let player = self.config.casing.with("Player", "player", "player");
				let value = self.config.casing.with("Value", "value", "value");

				self.push_indent();
				self.push(&format!("{iter}: Iter<LuaTuple<[{index}: number, {player}: Player"));

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
				let set_callback = match ev.call {
					EvCall::SingleSync | EvCall::SingleAsync => {
						self.config.casing.with("SetCallback", "setCallback", "set_callback")
					}
					EvCall::ManySync | EvCall::ManyAsync => self.config.casing.with("On", "on", "on"),
					_ => unreachable!(),
				};

				let callback = self.config.casing.with("Callback", "callback", "callback");
				let player = self.config.casing.with("Player", "player", "player");

				self.push_indent();
				self.push(&format!("{set_callback}: ({callback}: ({player}: Player"));

				if !ev.data.is_empty() {
					self.push(", ");
					self.push_parameters(&ev.data);
				}

				self.push(") => void) => () => void;\n");
			}

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

			self.push_indent();
			self.push(&format!("{set_callback}: ({callback}: ({player}: Player"));

			if !fndecl.args.is_empty() {
				self.push(", ");
				self.push_parameters(&fndecl.args);
			}

			self.push(") => ");

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

	Some(ServerOutput::new(config).output())
}
