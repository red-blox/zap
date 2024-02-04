use crate::config::{Enum, NumTy, Range, Ty};

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

	fn push_line_indent(&mut self, s: &str) {
		self.push_line(s);
		self.indent();
	}

	fn push_dedent_line(&mut self, s: &str) {
		self.dedent();
		self.push_line(s);
	}

	fn push_dedent_line_indent(&mut self, s: &str) {
		self.dedent();
		self.push_line(s);
		self.indent();
	}

	fn push_range_check(&mut self, range: Range, val: &str) {
		if let Some(exact) = range.exact() {
			self.push_line(&format!("assert({val} == {exact}, \"value not {exact}\")"));
		} else {
			if let Some(min) = range.min() {
				self.push_line(&format!("assert({val} >= {min}, \"value outside range\")"));
			}

			if let Some(max) = range.max() {
				self.push_line(&format!("assert({val} <= {max}, \"value outside range\")"));
			}
		}
	}

	fn push_ser(&mut self, from: &str, ty: &Ty<'_>, checks: bool) {
		self.push_line_indent("do");

		match ty {
			Ty::Num(numty, range) => {
				if checks {
					self.push_range_check(*range, from);
				}

				self.push_line(&format!("alloc({})", numty.size()));
				self.push_line(&format!("buffer.write{numty}(outgoing_buff, outgoing_apos, {from})"));
			}

			Ty::Str(len) => {
				if let Some(exact) = len.exact() {
					if checks {
						self.push_line(&format!("assert(#{from} == {exact}, \"length not {exact}\")"));
					}

					self.push_line(&format!("alloc({exact})"));
					self.push_line(&format!(
						"buffer.writestring(outgoing_buff, outgoing_apos, {from}, {exact})"
					));
				} else {
					self.push_line(&format!("local len = #{from}"));

					if checks {
						self.push_range_check(*len, "len");
					}

					self.push_line("alloc(len + 2)");
					self.push_line("buffer.writeu16(outgoing_buff, outgoing_apos, len)");
					self.push_line(&format!(
						"buffer.writestring(outgoing_buff, outgoing_apos + 2, {from}, len)"
					));
				}
			}

			Ty::Buf(len) => {
				if let Some(exact) = len.exact() {
					if checks {
						self.push_line(&format!(
							"assert(buffer.len({from}) == {exact}, \"length not {exact}\")"
						));
					}

					self.push_line(&format!("alloc({exact})"));
					self.push_line(&format!(
						"buffer.copy(outgoing_buff, outgoing_apos, {from}, 0, {exact})"
					));
				} else {
					self.push_line(&format!("local len = buffer.len({from})"));

					if checks {
						self.push_range_check(*len, "len");
					}

					self.push_line("alloc(len + 2)");
					self.push_line("buffer.writeu16(outgoing_buff, outgoing_apos, len)");
					self.push_line(&format!(
						"buffer.copy(outgoing_buff, outgoing_apos + 2, {from}, 0, len)"
					));
				}
			}

			Ty::Arr(ty, len) => {
				if let Some(exact) = len.exact() {
					if checks {
						self.push_line(&format!("assert(#{from} == {exact}, \"length not {exact}\")"));
					}

					self.push_line_indent(&format!("for i = 1, {exact} do"));
					self.push_line(&format!("local value = {from}[i]"));
					self.push_ser("value", ty, checks);
					self.push_dedent_line("end");
				} else {
					self.push_line(&format!("local len = #{from}"));

					if checks {
						self.push_range_check(*len, "len");
					}

					self.push_line("alloc(2)");
					self.push_line("buffer.writeu16(outgoing_buff, outgoing_apos, len)");

					self.push_line_indent("for i = 1, len do");
					self.push_line(&format!("local value = {from}[i]"));
					self.push_ser("value", ty, checks);
					self.push_dedent_line("end");
				}
			}

			Ty::Map(key, val) => {
				self.push_line("local len = 0");
				self.push_line("local len_pos = alloc(2)");

				self.push_line_indent(&format!("for k, v in {from} do"));
				self.push_line("len += 1");
				self.push_ser("k", key, checks);
				self.push_ser("v", val, checks);
				self.push_dedent_line("end");

				self.push_line("buffer.writeu16(outgoing_buff, len_pos, len)");
			}

			Ty::Opt(ty) => {
				self.push_line("alloc(1)");
				self.push_line_indent(&format!("if {from} == nil then"));
				self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, 0)");
				self.push_dedent_line_indent("else");
				self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, 1)");
				self.push_ser(from, ty, checks);
				self.push_dedent_line("end");
			}

			Ty::Ref(name) => self.push_line(&format!("types.write_{name}({from})")),

			Ty::Enum(enum_ty) => match enum_ty {
				Enum::Unit(enumerators) => {
					let numty = NumTy::from_f64(0.0, enumerators.len() as f64 - 1.0);

					self.push_line(&format!("alloc({})", numty.size()));

					for (i, enumerator) in enumerators.iter().enumerate() {
						if i == 0 {
							self.push_line_indent(&format!("if {from} == \"{enumerator}\" then"));
						} else {
							self.push_dedent_line_indent(&format!("elseif {from} == \"{enumerator}\" then"));
						}

						self.push_line(&format!("buffer.write{numty}(outgoing_buff, outgoing_apos, {i})"));
					}

					self.push_dedent_line_indent("else");
					self.push_line("error(\"invalid enumerator value\")");
					self.push_dedent_line("end");
				}

				Enum::Tagged {
					tag,
					variants,
					catch_all,
				} => {
					let numty = NumTy::from_f64(0.0, variants.len() as f64);

					for (i, (variant_name, variant_struct)) in variants.iter().enumerate() {
						if i == 0 {
							self.push_line_indent(&format!("if {from}.{tag} == \"{variant_name}\" then"));
						} else {
							self.push_dedent_line_indent(&format!("elseif {from}.{tag} == \"{variant_name}\" then"));
						}

						self.push_line(&format!("alloc({})", numty.size()));
						self.push_line(&format!("buffer.write{numty}(outgoing_buff, outgoing_apos, {})", i + 1));

						for (field_name, field_ty) in &variant_struct.fields {
							self.push_ser(&format!("{from}.{field_name}"), field_ty, checks);
						}
					}

					self.push_dedent_line_indent("else");

					if let Some(catch_all) = catch_all {
						self.push_line(&format!("alloc({})", numty.size()));
						self.push_line(&format!("buffer.write{numty}(outgoing_buff, outgoing_apos, 0)"));

						self.push_ser(&format!("{from}.{tag}"), &Ty::Str(Range::default()), checks);

						for (field_name, field_ty) in &catch_all.fields {
							self.push_ser(&format!("{from}.{field_name}"), field_ty, checks);
						}
					} else {
						self.push_line("error(\"invalid variant value\")");
					}

					self.push_dedent_line("end");
				}
			},

			Ty::Struct(struct_ty) => {
				for (field_name, field_ty) in &struct_ty.fields {
					self.push_ser(&format!("{from}.{field_name}"), field_ty, checks);
				}
			}

			Ty::Instance(class) => {
				if checks && class.is_some() {
					self.push_line(&format!(
						"assert({from}:IsA(\"{class}\"), \"instance is not a {class}\")",
						class = class.unwrap()
					));
				}

				self.push_line(&format!("table.insert(outgoing_inst, {from})"));
			}

			Ty::Color3 => {
				self.push_line("alloc(3)");
				self.push_line(&format!("buffer.writeu8(outgoing_buff, outgoing_apos, {from}.R * 255)"));
				self.push_line(&format!(
					"buffer.writeu8(outgoing_buff, outgoing_apos + 1, {from}.G * 255)"
				));
				self.push_line(&format!(
					"buffer.writeu8(outgoing_buff, outgoing_apos + 1, {from}.B * 255)"
				));
			}

			Ty::Vector3 => {
				self.push_line("alloc(12)");
				self.push_line(&format!("buffer.writef32(outgoing_buff, outgoing_apos, {from}.X)"));
				self.push_line(&format!("buffer.writef32(outgoing_buff, outgoing_apos + 4, {from}.Y)"));
				self.push_line(&format!("buffer.writef32(outgoing_buff, outgoing_apos + 8, {from}.Z)"));
			}

			Ty::AlignedCFrame => {
				self.push_line(&format!(
					"local axis_alignment = table.find(CFrameSpecialCases, {from}.Rotation)"
				));
				self.push_line("assert(axis_alignment, \"invalid axis alignment\")");
				self.push_line("alloc(1)");
				self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, axis_alignment)");

				self.push_ser(&format!("{from}.Position"), &Ty::Vector3, checks);
			}

			Ty::CFrame => {
				self.push_line(&format!("local axis, angle = {from}:ToAxisAngle()"));
				self.push_line("axis = axis * angle");

				self.push_ser(&format!("{from}.Position"), &Ty::Vector3, checks);
				self.push_ser("axis", &Ty::Vector3, checks);
			}

			Ty::Boolean => {
				self.push_line("alloc(1)");
				self.push_line(&format!(
					"buffer.writeu8(outgoing_buff, outgoing_apos, if {from} then 1 else 0)"
				))
			}

			Ty::Unknown => self.push_line(&format!("table.insert(outgoing_inst, {from})")),
		}

		self.push_dedent_line("end");
	}

	fn push_des(&mut self, into: &str, ty: &Ty<'_>, checks: bool) {
		self.push_line_indent("do");

		match ty {
			Ty::Num(numty, range) => {
				self.push_line(&format!(
					"{into} = buffer.read{numty}(incoming_buff, read({}))",
					numty.size()
				));

				if checks {
					self.push_range_check(*range, into);
				}
			}

			Ty::Str(len) => {
				if let Some(exact) = len.exact() {
					self.push_line(&format!(
						"{into} = buffer.readstring(incoming_buff, read({exact}), {exact})"
					));
				} else {
					self.push_line("local len = buffer.readu16(incoming_buff, read(2))");

					if checks {
						self.push_range_check(*len, "len");
					}

					self.push_line(&format!("{into} = buffer.readstring(incoming_buff, read(len), len)"));
				}
			}

			Ty::Buf(len) => {
				if let Some(exact) = len.exact() {
					self.push_line(&format!("{into} = buffer.create({exact})"));
					self.push_line(&format!("buffer.copy({into}, incoming_buff, read({exact}), {exact})"));
				} else {
					self.push_line("local len = buffer.readu16(incoming_buff, read(2))");

					if checks {
						self.push_range_check(*len, "len");
					}

					self.push_line(&format!("{into} = buffer.create(len)"));
					self.push_line(&format!("buffer.copy({into}, incoming_buff, read(len), len)"));
				}
			}

			Ty::Arr(ty, len) => {
				if let Some(exact) = len.exact() {
					self.push_line(&format!("{into} = table.create({exact})"));

					self.push_line_indent(&format!("for i = 1, {exact} do"));
					self.push_line("local v");
					self.push_des("v", ty, checks);
					self.push_line(&format!("{into}[i] = v"));
					self.push_dedent_line("end");
				} else {
					self.push_line("local len = buffer.readu16(incoming_buff, read(2))");

					if checks {
						self.push_range_check(*len, "len");
					}

					self.push_line(&format!("{into} = table.create(len)"));

					self.push_line_indent("for i = 1, len do");
					self.push_line("local v");
					self.push_des("v", ty, checks);
					self.push_line(&format!("{into}[i] = v"));
					self.push_dedent_line("end");
				}
			}

			Ty::Map(key, val) => {
				self.push_line(&format!("{into} = {{}}"));

				self.push_line_indent("for i = 1, buffer.readu16(incoming_buff, read(2)) do");

				self.push_line("local key, val");
				self.push_des("key", key, checks);
				self.push_des("val", val, checks);

				self.push_line(&format!("{into}[key] = val"));

				self.push_dedent_line("end");
			}

			Ty::Opt(ty) => {
				self.push_line_indent("if buffer.readu8(incoming_buff, read(1)) == 0 then");
				self.push_line(&format!("{into} = nil"));
				self.push_dedent_line_indent("else");

				if let Ty::Instance(class) = **ty {
					self.push_line("incoming_ipos += 1");
					self.push_line(&format!("{into} = incoming_inst[incoming_ipos]"));

					if checks && class.is_some() {
						self.push_line(&format!(
							"assert({into} == nil or {into}:IsA(\"{class}\"), \"Expected {into} to be nil or an instance of {class}\")",
							class = class.unwrap(),
						));
					}
				} else {
					self.push_des(into, ty, checks);
				}

				self.push_dedent_line("end");
			}

			Ty::Ref(name) => self.push_line(&format!("{into} = types.read_{name}()")),

			Ty::Enum(enum_ty) => match enum_ty {
				Enum::Unit(enumerators) => {
					let numty = NumTy::from_f64(0.0, enumerators.len() as f64 - 1.0);

					self.push_line(&format!(
						"local enum_index = buffer.read{numty}(incoming_buff, read({}))",
						numty.size()
					));

					for (i, enumerator) in enumerators.iter().enumerate() {
						if i == 0 {
							self.push_line_indent("if enum_index == 0 then");
						} else {
							self.push_dedent_line_indent(&format!("elseif enum_index == {i} then"));
						}

						self.push_line(&format!("{into} = \"{enumerator}\""));
					}

					self.push_dedent_line_indent("else");
					self.push_line("error(\"unknown enum index\")");
					self.push_dedent_line("end");
				}

				Enum::Tagged {
					tag,
					variants,
					catch_all,
				} => {
					let numty = NumTy::from_f64(0.0, variants.len() as f64);

					self.push_line(&format!("{into} = {{}}"));
					self.push_line(&format!(
						"local enum_index = buffer.read{numty}(incoming_buff, read({}))",
						numty.size()
					));

					for (i, (variant_name, variant_struct)) in variants.iter().enumerate() {
						if i == 0 {
							self.push_line_indent("if enum_index == 1 then");
						} else {
							self.push_dedent_line_indent(&format!("elseif enum_index == {} then", i + 1));
						}

						self.push_line(&format!("{into}.{tag} = \"{variant_name}\""));

						for (field_name, field_ty) in &variant_struct.fields {
							self.push_des(&format!("{into}.{field_name}"), field_ty, checks);
						}
					}

					self.push_dedent_line_indent("else");

					if let Some(catch_all) = catch_all {
						self.push_des(&format!("{into}.{tag}"), &Ty::Str(Range::default()), checks);

						for (field_name, field_ty) in &catch_all.fields {
							self.push_des(&format!("{into}.{field_name}"), field_ty, checks);
						}
					} else {
						self.push_line("error(\"unknown enum index\")");
					}

					self.push_dedent_line("end");
				}
			},

			Ty::Struct(struct_ty) => {
				self.push_line(&format!("{into} = {{}}"));

				for (field_name, field_ty) in &struct_ty.fields {
					self.push_des(&format!("{into}.{field_name}"), field_ty, checks);
				}
			}

			Ty::Instance(class) => {
				self.push_line("incoming_ipos += 1");
				self.push_line(&format!("{into} = incoming_inst[incoming_ipos]"));

				if checks && class.is_some() {
					self.push_line(&format!(
						"assert({into} and {into}:IsA(\"{class}\"), \"instance does not exist or is not a {class}\")",
						class = class.unwrap(),
					));
				} else {
					// we always assert that the instance is not nil
					// because roblox will sometimes just turn instances into nil
					// Ty::Opt covers the nil-able cases
					self.push_line(&format!("assert({into}, \"instance does not exist\")"));
				}
			}

			Ty::Color3 => {
				self.push_line_indent(&format!("{into} = Color3.fromRGB("));
				self.push_line("buffer.readu8(incoming_buff, read(1)),");
				self.push_line("buffer.readu8(incoming_buff, read(1)),");
				self.push_line("buffer.readu8(incoming_buff, read(1))");
				self.push_dedent_line(")");
			}

			Ty::Vector3 => {
				self.push_line_indent(&format!("{into} = Vector3.new("));
				self.push_line("buffer.readf32(incoming_buff, read(4)),");
				self.push_line("buffer.readf32(incoming_buff, read(4)),");
				self.push_line("buffer.readf32(incoming_buff, read(4))");
				self.push_dedent_line(")");
			}

			Ty::AlignedCFrame => {
				self.push_line("local axis_alignment = buffer.readu8(incoming_buff, read(1))");

				self.push_line("local pos");
				self.push_des("pos", &Ty::Vector3, checks);

				self.push_line(&format!(
					"{into} = CFrame.new(pos) * CFrameSpecialCases[axis_alignment]"
				));
			}

			Ty::CFrame => {
				self.push_line("local pos, axis_angle");
				self.push_des("pos", &Ty::Vector3, checks);
				self.push_des("axis_angle", &Ty::Vector3, checks);

				self.push_line("local angle = axis_angle.Magnitude");

				self.push_line_indent("if angle ~= 0 then");
				self.push_line(&format!("{into} = CFrame.fromAxisAngle(axis_angle, angle) + pos"));
				self.push_dedent_line_indent("else");
				self.push_line(&format!("{into} = CFrame.new(pos)"));
				self.push_dedent_line("end");
			}

			Ty::Boolean => self.push_line(&format!("{into} = buffer.readu8(incoming_buff, read(1)) == 1")),

			Ty::Unknown => {
				self.push_line("incoming_ipos += 1");
				self.push_line(&format!("{into} = incoming_inst[incoming_ipos]"))
			}
		}

		self.push_dedent_line("end");
	}

	fn push_ty(&mut self, ty: &Ty) {
		self.push("(");

		match ty {
			Ty::Num(..) => self.push("number"),
			Ty::Str(..) => self.push("string"),
			Ty::Buf(..) => self.push("buffer"),

			Ty::Arr(ty, ..) => {
				self.push("{ ");
				self.push_ty(ty);
				self.push(" }");
			}

			Ty::Map(key, val) => {
				self.push("{ [");
				self.push_ty(key);
				self.push("]: ");
				self.push_ty(val);
				self.push(" }");
			}

			Ty::Opt(ty) => {
				self.push_ty(ty);
				self.push("?");
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

				Enum::Tagged {
					tag,
					variants,
					catch_all,
				} => {
					let mut first = true;

					for (name, struct_ty) in variants.iter() {
						if !first {
							self.push(" | ");
						}
						first = false;

						self.push("{\n");
						self.indent();

						self.push_indent();

						self.push(&format!("{tag}: \"{name}\",\n"));

						for (name, ty) in struct_ty.fields.iter() {
							self.push_indent();
							self.push(&format!("{name}: "));
							self.push_ty(ty);
							self.push(",\n");
						}

						self.dedent();

						self.push_indent();
						self.push("}");
					}

					if let Some(catch_all) = catch_all {
						if !first {
							self.push(" | ");
						}

						self.push("{\n");
						self.indent();

						self.push_indent();

						self.push(&format!("{tag}: string,\n"));

						for (name, ty) in catch_all.fields.iter() {
							self.push_indent();
							self.push(&format!("{name}: "));
							self.push_ty(ty);
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
					self.push(&format!("{name}: "));
					self.push_ty(ty);
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

		self.push(")");
	}

	fn push_file_header(&mut self, scope: &str) {
		self.push_line("--!native");
		self.push_line("--!optimize 2");
		self.push_line("--!nocheck");
		self.push_line("--!nolint");
		self.push_line("--#selene: allow(unused_variable, shadowing, incorrect_standard_library_use)");

		self.push_line(&format!(
			"-- {scope} generated by Zap v{} (https://github.com/red-blox/zap)",
			env!("CARGO_PKG_VERSION")
		));
	}
}
