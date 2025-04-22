use crate::config::NamespaceEntry;
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
		let ty = &tydecl.ty;

		self.push_indent();
		self.push(&format!("type {tydecl} = "));
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

	pub fn push_return(&mut self) {
		let iter = self.config.casing.with("Iter", "iter", "iter");
		let index = self.config.casing.with("Index", "index", "index");
		let fire = self.config.casing.with("Fire", "fire", "fire");
		let value = self.config.casing.with("Value", "value", "value");
		let callback = self.config.casing.with("Callback", "callback", "callback");
		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let on = self.config.casing.with("On", "on", "on");
		let call = self.config.casing.with("Call", "call", "call");

		self.config.traverse_namespaces(
			self,
			|this, diff| {
				for _ in 0..diff {
					this.dedent();
					this.push_line("}");
				}
			},
			|this, name, entry, depth| {
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
					NamespaceEntry::EvDecl(evdecl) if evdecl.from == EvSource::Client => {
						this.push_indent();
						this.push(&format!("{fire}: ("));

						if !evdecl.data.is_empty() {
							this.push_parameters(&evdecl.data);
						}

						this.push(") => void;\n");

						this.dedent();
						this.push_line("};");
					}
					NamespaceEntry::EvDecl(evdecl) => {
						if evdecl.call == EvCall::Polling {
							this.push_indent();
							this.push(&format!("{iter}: Iter<LuaTuple<[{index}: number"));

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
								EvCall::Polling => unreachable!(),
							};

							this.push_indent();
							this.push(&format!("{set_callback}: ({callback}: ("));

							if !evdecl.data.is_empty() {
								this.push_parameters(&evdecl.data);
							}
							this.push(") => void) => () => void;\n");
						}

						this.dedent();
						this.push_line("};");
					}
					NamespaceEntry::FnDecl(fndecl) => {
						this.push_indent();
						this.push(&format!("{call}: ("));

						if !fndecl.args.is_empty() {
							this.push_parameters(&fndecl.args);
						}

						this.push(") => ");

						if this.config.yield_type == YieldType::Promise {
							this.push("Promise<")
						}

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

						if this.config.yield_type == YieldType::Promise {
							this.push(">")
						}

						this.push(";\n");

						this.dedent();
						this.push_line("};");
					}
					NamespaceEntry::Ns(..) => {}
				}
			},
		);
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Client");

		if self.config.namespaces.is_empty() {
			self.push_line("export {}");
			return self.buf;
		};

		if self.config.evdecls().iter().any(|ev| ev.call == EvCall::Polling) {
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
