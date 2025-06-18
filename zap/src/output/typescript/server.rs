use crate::config::NamespaceEntry;
use crate::config::{Config, EvCall, EvDecl, EvSource, TyDecl};

use super::ConfigProvider;
use super::Output;

struct ServerOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl<'src> Output<'src> for ServerOutput<'src> {
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

impl<'src> ConfigProvider<'src> for ServerOutput<'src> {
	fn get_config(&self) -> &'src Config<'src> {
		self.config
	}
}

impl<'src> ServerOutput<'src> {
	pub fn new(config: &'src Config<'src>) -> Self {
		Self {
			config,
			tabs: 0,
			buf: String::new(),
		}
	}

	fn push_tydecl(&mut self, tydecl: &TyDecl) {
		let ty = &*tydecl.ty.borrow();

		let mut depth = 0usize;
		for name in &tydecl.path {
			depth += 1;
			self.push_indent();
			self.push("export ");
			if depth == 1 {
				self.push("declare ");
			}
			self.push(&format!("namespace {name} {{\n"));
			self.indent();
		}
		self.push_indent();
		self.push(&format!("export type {} = ", tydecl.name));
		self.push_ty(ty);
		self.push(";\n");
		for _ in 0..depth {
			self.dedent();
			self.push_line("}");
		}
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

	fn push_return(&mut self) {
		let iter = self.config.casing.with("Iter", "iter", "iter");
		let index = self.config.casing.with("Index", "index", "index");
		let value = self.config.casing.with("Value", "value", "value");
		let callback = self.config.casing.with("Callback", "callback", "callback");
		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let on = self.config.casing.with("On", "on", "on");
		let player = self.config.casing.with("Player", "player", "player");

		self.config.traverse_namespaces(
			self,
			|this, diff| {
				for _ in 0..diff {
					this.dedent();
					this.push_line("}");
				}
			},
			|this, path, entry| {
				let depth = path.len() - 1;
				let name = path.last().unwrap();
				this.push_line(&format!(
					"export {}{} {name}{} {{",
					if depth == 0 { "declare " } else { "" },
					if matches!(entry, NamespaceEntry::Ns(..)) {
						"namespace"
					} else {
						"const"
					},
					if matches!(entry, NamespaceEntry::Ns(..)) {
						""
					} else {
						":"
					}
				));
				this.indent();

				match entry {
					NamespaceEntry::EvDecl(evdecl) if evdecl.from == EvSource::Server => {
						this.push_return_fire(evdecl);

						if !this.config.disable_fire_all {
							this.push_return_fire_all(evdecl);
						}

						this.push_return_fire_except(evdecl);
						this.push_return_fire_list(evdecl);
						this.push_return_fire_set(evdecl);

						this.dedent();
						this.push_line("};");
					}
					NamespaceEntry::EvDecl(evdecl) => {
						if evdecl.call == EvCall::Polling {
							this.push_indent();
							this.push(&format!(
								"{iter}: () => IterableFunction<LuaTuple<[{index}: number, {player}: Player"
							));

							for (index, parameter) in evdecl.data.iter().enumerate() {
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

								this.push(&format!(", {}: ", name));
								this.push_ty(&parameter.ty);
							}

							this.push("]>>;\n");
						} else {
							let set_callback = match evdecl.call {
								EvCall::SingleSync | EvCall::SingleAsync => set_callback,
								EvCall::ManySync | EvCall::ManyAsync => on,
								_ => unreachable!(),
							};

							this.push_indent();
							this.push(&format!("{set_callback}: ({callback}: ({player}: Player"));

							if !evdecl.data.is_empty() {
								this.push(", ");
								this.push_parameters(&evdecl.data);
							}

							this.push(") => void) => () => void;\n");
						}

						this.dedent();
						this.push_line("};");
					}
					NamespaceEntry::FnDecl(fndecl) => {
						this.push_indent();
						this.push(&format!("{set_callback}: ({callback}: ({player}: Player"));

						if !fndecl.args.is_empty() {
							this.push(", ");
							this.push_parameters(&fndecl.args);
						}

						this.push(") => ");

						if let Some(types) = &fndecl.rets {
							if types.len() > 1 {
								this.push("LuaTuple<[");
							}

							for (i, ty) in types.iter().enumerate() {
								if i > 0 {
									this.push(", ");
								}

								this.push_ty(ty);
							}

							if types.len() > 1 {
								this.push("]>");
							}
						} else {
							this.push("void");
						}

						this.push(") => () => void;\n");

						this.dedent();
						this.push_line("};");
					}
					NamespaceEntry::Ns(..) => {}
				}
			},
		)
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Server");

		if self.config.namespaces.is_empty() {
			self.push_line("export {}");
			return self.buf;
		};

		self.push_event_loop();

		self.push_tydecls();

		self.push_return();

		self.buf
	}
}

pub fn code<'src>(config: &'src Config<'src>) -> Option<String> {
	if !config.typescript {
		return None;
	}

	Some(ServerOutput::new(config).output())
}
