use crate::config::{Config, Enum, Ty};

pub mod client;
pub mod server;

pub trait Output {
	fn push(&mut self, s: &str);
	fn indent(&mut self);
	fn dedent(&mut self);
	fn push_indent(&mut self);

	fn push_line(&mut self, s: &str) {
		self.push_indent();
		self.push(s);
		self.push("\n");
	}

	fn push_ty(&mut self, ty: &Ty) {
		match ty {
			Ty::Num(..) => self.push("number"),
			Ty::Str { .. } => self.push("string"),
			Ty::Buf { .. } => self.push("buffer"),

			Ty::Arr(ty, range) => match (range.min(), range.max()) {
				(Some(min), Some(max)) => {
					if let Some(exact) = range.exact() {
						self.push("[");

						for i in 0..exact as usize {
							if i != 0 {
								self.push(", ");
							}

							self.push_ty(ty);
						}

						self.push("]");
					} else {
						if min as usize != 0 {
							self.push("[");

							for i in 0..min as usize {
								if i != 0 {
									self.push(", ");
								}

								self.push_ty(ty);
							}

							self.push("] & ");
						}

						self.push("Partial<[");

						for i in 0..max as usize {
							if i != 0 {
								self.push(", ");
							}

							self.push_ty(ty);
						}

						self.push("]>");
					}
				}
				(Some(min), None) => {
					self.push("[");

					if min as usize != 0 {
						for i in 0..min as usize {
							if i != 0 {
								self.push(", ");
							}

							self.push_ty(ty);
						}

						self.push(", ");
					}

					self.push("...Array<");
					self.push_ty(ty);
					self.push(" | undefined>]");
				}
				(None, Some(max)) => {
					self.push("Partial<[");

					for i in 0..max as usize {
						if i != 0 {
							self.push(", ");
						}

						self.push_ty(ty);
					}

					self.push("]>");
				}
				_ => {
					self.push("(");
					self.push_ty(ty);
					self.push(")[]");
				}
			},

			Ty::Map(key, val) => {
				self.push("{ [index: ");
				self.push_ty(key);
				self.push("]: ");
				self.push_ty(val);
				self.push(" }");
			}

			Ty::Opt(ty) => {
				self.push_ty(ty);
				self.push(" | undefined");
			}

			Ty::Ref(name) => self.push(name),

			Ty::Enum(enum_ty) => match enum_ty {
				Enum::Unit(enumerators) => self.push(
					&enumerators
						.iter()
						.map(|v| format!("\"{}\"", v))
						.collect::<Vec<_>>()
						.join(" | ")
						.to_string(),
				),

				Enum::Tagged { tag, variants } => {
					for (i, (name, struct_ty)) in variants.iter().enumerate() {
						if i != 0 {
							self.push(" | ");
						}

						self.push("{\n");
						self.indent();

						self.push_indent();

						self.push(&format!("{tag}: \"{name}\",\n"));

						for (name, ty) in struct_ty.fields.iter() {
							self.push_indent();
							self.push(name);

							if let Ty::Opt(ty) = ty {
								self.push("?: ");
								self.push_ty(ty);
							} else {
								self.push(": ");
								self.push_ty(ty);
							}

							self.push(",\n");
						}

						self.dedent();

						self.push_indent();
						self.push("}");
					}
				}
			},

			Ty::Struct(struct_ty) => {
				self.push("{\n");
				self.indent();

				for (name, ty) in struct_ty.fields.iter() {
					self.push_indent();
					self.push(name);

					if let Ty::Opt(ty) = ty {
						self.push("?: ");
						self.push_ty(ty);
					} else {
						self.push(": ");
						self.push_ty(ty);
					}

					self.push(",\n");
				}

				self.dedent();
				self.push_indent();
				self.push("}");
			}

			Ty::Instance(name) => self.push(name.unwrap_or("Instance")),

			Ty::Unknown => self.push("unknown"),
			Ty::Boolean => self.push("boolean"),
			Ty::Color3 => self.push("Color3"),
			Ty::Vector3 => self.push("Vector3"),
			Ty::AlignedCFrame => self.push("CFrame"),
			Ty::CFrame => self.push("CFrame"),
		}
	}

	fn push_file_header(&mut self, scope: &str) {
		self.push_line(&format!(
			"// {scope} generated by Zap v{} (https://github.com/red-blox/zap)",
			env!("CARGO_PKG_VERSION")
		));
	}

	fn push_manual_event_loop(&mut self, config: &Config) {
		let send_events = config.casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("export const {send_events}: () => void"))
	}
}
