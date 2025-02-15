use crate::config::{Config, Enum, Parameter, Ty};

pub mod client;
pub mod server;
pub mod types;

pub trait ConfigProvider {
	fn get_config(&self) -> &Config;
}

pub trait Output: ConfigProvider {
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
						if exact > self.get_config().typescript_max_tuple_length {
							self.push_ty(ty);
							self.push("[]");
						} else {
							self.push("[");

							for i in 0..exact as usize {
								if i != 0 {
									self.push(", ");
								}

								self.push_ty(ty);
							}

							self.push("]");
						}
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
				self.push("Map<");
				self.push_ty(key);
				self.push(", ");
				self.push_ty(val);
				self.push(">");
			}

			Ty::Set(key) => {
				self.push("Set<");
				self.push_ty(key);
				self.push(">");
			}

			Ty::Opt(ty) => {
				self.push_ty(ty);

				if !matches!(**ty, Ty::Unknown) {
					self.push("| undefined");
				}
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

						if *name == "true" || *name == "false" {
							self.push(&format!("{tag}: {name},\n"));
						} else {
							self.push(&format!("{tag}: \"{name}\",\n"));
						}

						for (name, ty) in struct_ty.fields.iter() {
							self.push_indent();
							self.push(name);
							self.push_arg_ty(ty);
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
					self.push_arg_ty(ty);
					self.push(",\n");
				}

				self.dedent();
				self.push_indent();
				self.push("}");
			}

			Ty::Instance(name) => self.push(name.unwrap_or("Instance")),

			Ty::BrickColor => self.push("BrickColor"),
			Ty::DateTimeMillis => self.push("DateTime"),
			Ty::DateTime => self.push("DateTime"),
			Ty::Unknown => self.push("unknown"),
			Ty::Boolean => self.push("boolean"),
			Ty::Color3 => self.push("Color3"),
			Ty::Vector2 => self.push("Vector3"),
			Ty::Vector3 => self.push("Vector3"),
			Ty::Vector(..) => self.push("vector"),
			Ty::AlignedCFrame => self.push("CFrame"),
			Ty::CFrame => self.push("CFrame"),
		}
	}

	fn push_arg_ty(&mut self, ty: &Ty) {
		if let Ty::Opt(ty) = ty {
			if let Ty::Unknown = **ty {
				self.push(": ");
				self.push_ty(ty);
			} else {
				self.push("?: ");
				self.push_ty(ty);
				self.push(" | undefined");
			}
		} else {
			self.push(": ");
			self.push_ty(ty);
		}
	}

	fn push_parameters(&mut self, parameters: &[Parameter]) {
		let value = self.get_config().casing.with("Value", "value", "value");

		for (i, parameter) in parameters.iter().enumerate() {
			if i > 0 {
				self.push(", ");
			}

			if let Some(name) = parameter.name {
				self.push(name);
			} else {
				self.push(&format!(
					"{value}{}",
					if i > 0 { (i + 1).to_string() } else { "".to_string() },
				));
			}

			self.push_arg_ty(&parameter.ty);
		}
	}

	fn push_file_header(&mut self, scope: &str) {
		self.push_line(&format!(
			"// {scope} generated by Zap v{} (https://github.com/red-blox/zap)",
			env!("CARGO_PKG_VERSION")
		));
	}

	fn push_event_loop(&mut self) {
		let send_events = self.get_config().casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("export declare const {send_events}: () => void"))
	}
}
