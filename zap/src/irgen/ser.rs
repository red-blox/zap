use crate::{
	config::{Enum, NumTy, PrimitiveTy, Struct, Ty, TypeScriptEnumType},
	irgen::{BitpackMask, OutputBuffer, Scope, VariantStorageKind},
};
use std::collections::HashMap;

use super::{Expr, Gen, Stmt, Var};

struct Ser<'src> {
	checks: bool,
	buf: OutputBuffer,
	var_occurrences: &'src mut HashMap<String, usize>,
	scopes: Vec<Scope>,
	typescript_enum_type: TypeScriptEnumType,
}

impl Gen for Ser<'_> {
	fn push_stmt(&mut self, stmt: Stmt) {
		self.buf.push(stmt);
	}

	fn generate<'a, 'src: 'a, I>(mut self, names: &[String], types: I) -> Vec<Stmt>
	where
		I: Iterator<Item = &'a Ty<'src>>,
	{
		self.new_scope();

		for (ty, name) in types.zip(names) {
			self.push_ty(ty, Var::Name(name.to_string()));
		}

		self.end_scope();

		self.buf.output()
	}

	fn get_var_occurrences(&mut self) -> &mut HashMap<String, usize> {
		self.var_occurrences
	}

	fn new_scope(&mut self) {
		let scope_buf = OutputBuffer::new();
		self.buf.push(scope_buf.clone());
		self.scopes.push(Scope {
			bitpack_budget: vec![],
			buf: scope_buf,
		});
	}

	fn current_scope(&mut self) -> &mut Scope {
		self.scopes.last_mut().unwrap()
	}

	fn end_scope(&mut self) {
		let scope = self.scopes.pop().unwrap();

		for (shift, name) in scope.bitpack_budget {
			let (pos_name, pos_expr) = self.add_occurrence(&format!("{name}_pos"));
			let numty = NumTy::from_f64(0.0, ((1u64 << (shift + 1)) - 1) as f64);

			scope.buf.push(Stmt::Local(name.clone(), Some(Expr::Num(0.0))));
			scope.buf.push(Stmt::Local(
				pos_name,
				Some(Expr::Call(
					Var::Name("alloc".into()).into(),
					None,
					vec![Expr::Num(numty.size() as f64)],
				)),
			));
			self.buf.push(Stmt::Call(
				Var::NameIndex(Var::Name("buffer".into()).into(), format!("write{numty}")),
				None,
				vec![
					Expr::Var(Var::Name("outgoing_buff".into()).into()),
					pos_expr,
					Expr::Var(Var::Name(name).into()),
				],
			));
		}
	}
}

impl Ser<'_> {
	fn push_struct(&mut self, struct_ty: &Struct, from: Var) {
		for (name, ty) in struct_ty.fields.iter() {
			self.push_ty(ty, from.clone().eindex(Expr::Str((*name).into())));
		}
	}

	fn push_variant_storage(&mut self, storage: &VariantStorageKind, i: usize) {
		match storage {
			VariantStorageKind::Full(numty, _) => {
				self.push_writenumty((i as f64).into(), *numty);
			}
			VariantStorageKind::Bitpack(variants) => {
				let (bits, var) = &variants[i];
				self.set_bitfield(*bits, var.clone());
			}
			VariantStorageKind::Bit((bits, var)) => {
				if i != 0 {
					self.set_bitfield(*bits, var.clone());
				} else {
					// make selene happy
					self.push_stmt(Stmt::Local("_".into(), None));
				}
			}
			VariantStorageKind::None => {}
		}
	}

	fn push_enum(&mut self, enum_ty: &Enum, from: Var) {
		match enum_ty {
			Enum::Unit(enumerators) => {
				let from_expr = Expr::from(from.clone());
				let storage = self.variant_storage(enumerators.len());

				for (i, enumerator) in enumerators.iter().enumerate() {
					let condition = match self.typescript_enum_type {
						TypeScriptEnumType::ConstNumber => Expr::Num(i as f64),
						_ => Expr::StrOrBool(enumerator.to_string()),
					};

					if i == 0 {
						self.push_stmt(Stmt::If(from_expr.clone().eq(condition)));
					} else {
						self.push_stmt(Stmt::ElseIf(from_expr.clone().eq(condition)));
					}

					self.push_variant_storage(&storage, i);
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid enumerator".into()));
				self.push_stmt(Stmt::End);
			}

			Enum::Tagged { tag, variants } => {
				let tag_expr = Expr::from(from.clone().eindex(Expr::Str((*tag).into())));
				let storage = self.variant_storage(variants.len());

				for (i, variant) in variants.iter().enumerate() {
					if i == 0 {
						self.push_stmt(Stmt::If(tag_expr.clone().eq(Expr::StrOrBool(variant.0.to_string()))));
					} else {
						self.push_stmt(Stmt::ElseIf(
							tag_expr.clone().eq(Expr::StrOrBool(variant.0.to_string())),
						));
					}

					self.push_variant_storage(&storage, i);
					self.push_struct(&variant.1, from.clone());
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid variant".into()));
				self.push_stmt(Stmt::End);
			}
		}
	}

	fn push_or(&mut self, from: Var, tys: &Vec<Ty<'_>>, optional: bool) {
		let (from_ty_name, from_ty_expr) = self.add_occurrence("ty_name");

		self.push_local(
			from_ty_name,
			Some(Expr::Call(
				Box::new(Var::from("typeof")),
				None,
				vec![Expr::from(from.clone())],
			)),
		);

		let storage = self.variant_storage(
			tys.iter()
				.map(|ty| match ty.primitive_ty() {
					PrimitiveTy::Enum(Enum::Unit(variants)) => variants.len(),
					PrimitiveTy::Enum(Enum::Tagged { variants, .. }) => variants.len(),
					_ => 1,
				})
				.sum::<usize>()
				+ optional as usize,
		);

		let mut unknown_i = None;
		let mut initial_if = true;
		let mut i_offset = 0usize;

		for ty in tys {
			let i = i_offset;

			match ty.primitive_ty() {
				PrimitiveTy::Name(name) => {
					i_offset += 1;

					let condition = from_ty_expr.clone().eq(Expr::Str(name.to_string()));

					if initial_if {
						self.push_stmt(Stmt::If(condition));
						initial_if = false;
					} else {
						self.push_stmt(Stmt::ElseIf(condition));
					}

					self.push_variant_storage(&storage, i);
					self.push_ty(ty, from.clone());
				}
				PrimitiveTy::Instance(class) => {
					i_offset += 1;

					let mut condition = from_ty_expr.clone().eq(Expr::Str("Instance".to_string()));
					if let Some(class) = class {
						condition = condition.and(Expr::Call(
							Box::new(from.clone()),
							Some("IsA".to_string()),
							vec![Expr::Str(class.to_string())],
						))
					}

					if initial_if {
						self.push_stmt(Stmt::If(condition));
						initial_if = false;
					} else {
						self.push_stmt(Stmt::ElseIf(condition));
					}

					self.push_variant_storage(&storage, i);
					self.push_ty(ty, from.clone());
				}
				PrimitiveTy::Enum(Enum::Unit(variants)) => {
					i_offset += variants.len();

					for (offset, variant) in variants.into_iter().enumerate() {
						let condition = Expr::from(from.clone()).eq(match self.typescript_enum_type {
							TypeScriptEnumType::ConstNumber => Expr::Num(offset as f64),
							_ => Expr::StrOrBool(variant.to_string()),
						});

						if initial_if {
							self.push_stmt(Stmt::If(condition));
							initial_if = false;
						} else {
							self.push_stmt(Stmt::ElseIf(condition));
						}

						self.push_variant_storage(&storage, i + offset);
					}
				}
				PrimitiveTy::Enum(Enum::Tagged { tag, variants }) => {
					i_offset += variants.len();

					for (offset, (variant, data)) in variants.into_iter().enumerate() {
						let condition = from_ty_expr.clone().eq(Expr::Str("table".to_string())).and(
							Expr::Var(Box::new(from.clone().eindex(Expr::Str(tag.to_string()))))
								.eq(Expr::StrOrBool(variant.to_string())),
						);

						if initial_if {
							self.push_stmt(Stmt::If(condition));
							initial_if = false;
						} else {
							self.push_stmt(Stmt::ElseIf(condition));
						}

						self.push_variant_storage(&storage, i + offset);
						self.push_struct(&data, from.clone());
					}
				}
				PrimitiveTy::Unknown => {
					unknown_i = Some(i);
					i_offset += 1;
				}
				PrimitiveTy::None(..) => unreachable!(),
			};
		}

		if optional {
			self.push_stmt(Stmt::ElseIf(Expr::from(from.clone()).eq(Expr::Nil)));
			self.push_variant_storage(&storage, i_offset);
		}

		self.push_stmt(Stmt::Else);
		if let Some(unknown_i) = unknown_i {
			self.push_variant_storage(&storage, unknown_i);
			self.push_ty(&Ty::Unknown, from.clone());
		} else {
			self.push_stmt(Stmt::Error("Invalid type".into()));
		}
		self.push_stmt(Stmt::End);
	}

	fn set_bitfield(&mut self, bits: BitpackMask, var: Var) {
		self.push_assign(
			var.clone(),
			Expr::Call(
				Var::NameIndex(Var::Name("bit32".into()).into(), "bor".into()).into(),
				None,
				vec![Expr::Var(var.into()), Expr::BinaryNum(bits)],
			),
		);
	}

	fn push_ty(&mut self, ty: &Ty, from: Var) {
		let from_expr = Expr::from(from.clone());

		match ty {
			Ty::Num(numty, range) => {
				if self.checks {
					self.push_range_check(from_expr.clone(), *range);
				}

				self.push_writenumty(from_expr, *numty)
			}

			Ty::Str(utf8, range) => {
				if !utf8 && let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(
							from_expr.clone().len().eq(len.into()),
							format!("length is not equal to {len}!"),
						);
					}

					self.push_writestring(from_expr, len.into());
				} else {
					let (len_name, len_expr) = self.add_occurrence("len");
					let (len_numty, len_offset) = range.numty();

					self.push_local(len_name.clone(), Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check(len_expr.clone(), *range);
						if *utf8 {
							self.push_utf8_check(from_expr.clone());
						}
					}

					let mut offset_len_expr = len_expr.clone();
					if len_offset != 0.0 {
						offset_len_expr = offset_len_expr.sub(Expr::Num(len_offset))
					}

					self.push_writenumty(offset_len_expr, len_numty);
					self.push_writestring(from_expr, len_expr.clone());
				}
			}

			Ty::Buf(range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(
							Var::from("buffer")
								.nindex("len")
								.call(vec![from_expr.clone()])
								.eq(len.into()),
							format!("length is not equal to {len}!"),
						);
					}

					self.push_write_copy(from_expr, len.into());
				} else {
					let (len_name, len_expr) = self.add_occurrence("len");
					let (len_numty, len_offset) = range.numty();

					self.push_local(
						len_name.clone(),
						Some(Var::from("buffer").nindex("len").call(vec![from_expr.clone()])),
					);

					if self.checks {
						self.push_range_check(len_expr.clone(), *range);
					}

					let mut offset_len_expr = len_expr.clone();
					if len_offset != 0.0 {
						offset_len_expr = offset_len_expr.sub(Expr::Num(len_offset))
					}

					self.push_writenumty(offset_len_expr, len_numty);
					self.push_write_copy(from_expr, len_name.as_str().into())
				}
			}

			Ty::Arr(ty, range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(
							from_expr.clone().len().eq(len.into()),
							format!("length is not equal to {len}!"),
						);
					}

					for i in 1..=(len as usize) {
						self.push_ty(ty, from.clone().eindex((i as f64).into()));
					}
				} else {
					let (var_name, var_expr) = self.add_occurrence("i");
					let (len_name, len_expr) = self.add_occurrence("len");
					let (len_numty, len_offset) = range.numty();

					self.push_local(len_name.clone(), Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check(len_expr.clone(), *range);
					}

					let mut offset_len_expr = len_expr.clone();
					if len_offset != 0.0 {
						offset_len_expr = offset_len_expr.sub(Expr::Num(len_offset))
					}

					self.push_writenumty(offset_len_expr, len_numty);

					self.push_stmt(Stmt::NumFor {
						var: var_name.clone(),
						from: 1.0.into(),
						to: len_expr.clone(),
					});

					self.new_scope();

					let (inner_var_name, _) = self.add_occurrence("val");

					self.push_stmt(Stmt::Local(
						inner_var_name.clone(),
						Some(from.clone().eindex(var_expr.clone()).into()),
					));

					self.push_ty(ty, Var::Name(inner_var_name));

					self.end_scope();

					self.push_stmt(Stmt::End);
				}
			}

			Ty::Map(key, val) => {
				let (len_name, len_expr) = self.add_occurrence("len");
				let (len_pos_name, len_pos_expr) = self.add_occurrence("len_pos");
				let (empty_bits, empty_var) = self.get_bitpack();

				let length_numty = key.variants().map(|(numty, ..)| numty).unwrap_or(NumTy::U16);

				self.push_local(len_pos_name.clone(), None);
				self.push_local(len_name.clone(), Some(0.0.into()));

				let (key_name, _) = self.add_occurrence("k");
				let (val_name, _) = self.add_occurrence("v");

				self.push_stmt(Stmt::GenFor {
					key: key_name.clone(),
					val: val_name.clone(),
					obj: from_expr,
				});

				self.push_stmt(Stmt::If(len_expr.clone().eq(0.0.into())));

				self.push_assign(
					Var::Name(len_pos_name.clone()),
					Var::from("alloc").call(vec![(length_numty.size() as f64).into()]),
				);

				self.push_stmt(Stmt::End);

				self.new_scope();

				self.push_assign(Var::Name(len_name.clone()), len_expr.clone().add(1.0.into()));

				self.push_ty(key, key_name.as_str().into());
				self.push_ty(val, val_name.as_str().into());

				self.end_scope();

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::If(len_pos_expr.clone()));

				self.push_stmt(Stmt::Call(
					Var::from("buffer").nindex(format!("write{length_numty}")),
					None,
					vec![
						"outgoing_buff".into(),
						len_pos_expr.clone(),
						len_expr.clone().sub(1.0.into()),
					],
				));

				self.push_stmt(Stmt::Else);

				self.set_bitfield(empty_bits, empty_var);

				self.push_stmt(Stmt::End);
			}

			Ty::Set(key) => {
				let (len_name, len_expr) = self.add_occurrence("len");
				let (len_pos_name, len_pos_expr) = self.add_occurrence("len_pos");
				let (empty_bits, empty_var) = self.get_bitpack();

				let length_numty = key.variants().map(|(numty, ..)| numty).unwrap_or(NumTy::U16);

				self.push_local(len_pos_name.clone(), None);
				self.push_local(len_name.clone(), Some(0.0.into()));

				let (key_name, _) = self.add_occurrence("k");
				let (val_name, _) = self.add_occurrence("_");

				self.push_stmt(Stmt::GenFor {
					key: key_name.clone(),
					val: val_name.clone(),
					obj: from_expr,
				});

				self.push_stmt(Stmt::If(len_expr.clone().eq(0.0.into())));

				self.push_assign(
					Var::Name(len_pos_name.clone()),
					Var::from("alloc").call(vec![(length_numty.size() as f64).into()]),
				);

				self.push_stmt(Stmt::End);

				self.new_scope();

				self.push_assign(Var::Name(len_name.clone()), len_expr.clone().add(1.0.into()));
				self.push_ty(key, key_name.as_str().into());

				self.end_scope();

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::If(len_pos_expr.clone()));

				self.push_stmt(Stmt::Call(
					Var::from("buffer").nindex(format!("write{length_numty}")),
					None,
					vec![
						"outgoing_buff".into(),
						len_pos_expr.clone(),
						len_expr.clone().sub(1.0.into()),
					],
				));

				self.push_stmt(Stmt::Else);

				self.set_bitfield(empty_bits, empty_var);

				self.push_stmt(Stmt::End);
			}

			Ty::Opt(ty) => {
				if let Ty::Or(tys, _) = &**ty {
					return self.push_or(from, tys, true);
				}

				self.push_stmt(Stmt::If(from_expr.clone().neq(Expr::Nil)));
				let (bits, var) = self.get_bitpack();
				self.set_bitfield(bits, var);
				self.push_ty(ty, from);
				self.push_stmt(Stmt::End);
			}

			Ty::Ref(name, ..) => self.push_stmt(Stmt::Call(
				Var::from("types").nindex(format!("write_{name}")),
				None,
				vec![from_expr],
			)),

			Ty::Enum(enum_ty) => self.push_enum(enum_ty, from),
			Ty::Struct(struct_ty) => self.push_struct(struct_ty, from),

			Ty::Or(tys, _) => self.push_or(from, tys, false),

			Ty::Instance(class) => {
				if self.checks && class.is_some() {
					self.push_assert(
						Expr::Call(
							Box::new(from),
							Some("IsA".into()),
							vec![Expr::Str(class.unwrap().into())],
						),
						format!("received instance is not of the {} class!", class.unwrap()),
					);
				}

				self.push_stmt(Stmt::Call(
					Var::from("table").nindex("insert"),
					None,
					vec!["outgoing_inst".into(), from_expr],
				))
			}

			Ty::Unknown => self.push_stmt(Stmt::Call(
				Var::from("table").nindex("insert"),
				None,
				vec!["outgoing_inst".into(), from_expr],
			)),

			Ty::Color3 => {
				self.push_writeu8(Expr::Mul(
					Box::new(from.clone().nindex("R").into()),
					Box::new(Expr::Num(255.0)),
				));
				self.push_writeu8(Expr::Mul(
					Box::new(from.clone().nindex("G").into()),
					Box::new(Expr::Num(255.0)),
				));
				self.push_writeu8(Expr::Mul(
					Box::new(from.clone().nindex("B").into()),
					Box::new(Expr::Num(255.0)),
				));
			}

			Ty::BrickColor => self.push_writeu16(from.clone().nindex("Number").into()),

			Ty::DateTimeMillis => self.push_writef64(from.clone().nindex("UnixTimestampMillis").into()),
			Ty::DateTime => self.push_writef64(from.clone().nindex("UnixTimestamp").into()),

			Ty::Vector2 => {
				self.push_writef32(from.clone().nindex("X").into());
				self.push_writef32(from.clone().nindex("Y").into());
			}
			Ty::Vector3 => {
				self.push_writef32(from.clone().nindex("X").into());
				self.push_writef32(from.clone().nindex("Y").into());
				self.push_writef32(from.clone().nindex("Z").into());
			}
			Ty::Vector(x_ty, y_ty, z_ty) => {
				match **x_ty {
					Ty::Num(numty, range) => {
						if self.checks {
							self.push_range_check(from_expr.clone(), range);
						}

						self.push_writenumty(from.clone().nindex("x").into(), numty)
					}
					_ => unreachable!(),
				};

				match **y_ty {
					Ty::Num(numty, range) => {
						if self.checks {
							self.push_range_check(from_expr.clone(), range);
						}

						self.push_writenumty(from.clone().nindex("y").into(), numty)
					}
					_ => unreachable!(),
				};

				if let Some(z_ty) = z_ty {
					match **z_ty {
						Ty::Num(numty, range) => {
							if self.checks {
								self.push_range_check(from_expr.clone(), range);
							}

							self.push_writenumty(from.clone().nindex("z").into(), numty)
						}
						_ => unreachable!(),
					};
				}
			}

			Ty::AlignedCFrame => {
				let (axis_alignment_name, axis_alignment_expr) = self.add_occurrence("axis_alignment");

				self.push_local(
					axis_alignment_name.clone(),
					Some(Expr::Call(
						Box::new(Var::from("table").nindex("find")),
						None,
						vec!["CFrameSpecialCases".into(), from.clone().nindex("Rotation").into()],
					)),
				);

				self.push_assert(axis_alignment_expr.clone(), "CFrame not aligned to an axis!".into());

				self.push_writeu8(axis_alignment_expr.clone());

				self.push_ty(&Ty::Vector3, from.clone().nindex("Position"));
			}

			Ty::CFrame => {
				// local axis, angle = Value:ToAxisAngle()
				let (axis_name, axis_expr) = self.add_occurrence("axis");
				let (angle_name, angle_expr) = self.add_occurrence("angle");
				self.push_stmt(Stmt::LocalTuple(
					vec![axis_name.clone(), angle_name.clone()],
					Some(Expr::Call(from.clone().into(), Some("ToAxisAngle".into()), vec![])),
				));

				// axis = axis * angle
				// store the angle into the axis, as it is a unit vector, so the magnitude can be used to encode a number
				self.push_stmt(Stmt::Assign(
					Var::Name(axis_name.clone()),
					Expr::Mul(Box::new(axis_expr), Box::new(angle_expr)),
				));

				self.push_ty(&Ty::Vector3, from.clone().nindex("Position"));
				self.push_ty(&Ty::Vector3, axis_name.as_str().into());
			}

			Ty::Boolean => {
				self.push_stmt(Stmt::If(from_expr));
				let (bits, var) = self.get_bitpack();
				self.set_bitfield(bits, var);
				self.push_stmt(Stmt::End);
			}
		}
	}
}

pub fn generate<'a, 'src: 'a, I>(
	types: I,
	names: &[String],
	checks: bool,
	var_occurrences: &mut HashMap<String, usize>,
	typescript_enum_type: TypeScriptEnumType,
) -> Vec<Stmt>
where
	I: IntoIterator<Item = &'a Ty<'src>>,
{
	Ser {
		checks,
		buf: OutputBuffer::new(),
		var_occurrences,
		scopes: vec![],
		typescript_enum_type,
	}
	.generate(names, types.into_iter())
}
