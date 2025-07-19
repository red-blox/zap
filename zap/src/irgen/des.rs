use crate::{
	config::{Enum, NumTy, PrimitiveTy, Struct, Ty, TypeScriptEnumType},
	irgen::{
		AllocScope, BitpackMask, DynamicScope, GenNumExt, OutputBuffer, Scopes, VariantStorageKind, read, readstring,
	},
};
use std::collections::HashMap;

use super::{Expr, Gen, Stmt, Var};

struct Des<'src> {
	checks: bool,
	buf: OutputBuffer,
	var_occurrences: &'src mut HashMap<String, usize>,
	scopes: Scopes,
	typescript_enum_type: TypeScriptEnumType,
}

impl Gen for Des<'_> {
	fn buf(&self) -> &OutputBuffer {
		&self.buf
	}

	fn generate<'a, 'src: 'a, I>(mut self, names: &[String], types: I) -> Vec<Stmt>
	where
		I: Iterator<Item = &'a Ty<'src>>,
	{
		self.new_alloc_scope();
		self.new_dyn_scope();

		for (ty, name) in types.zip(names) {
			self.push_ty(ty, Var::Name(name.to_string()));
		}

		self.end_dyn_scope();
		self.end_alloc_scope();

		self.buf.output()
	}

	fn get_var_occurrences(&mut self) -> &mut HashMap<String, usize> {
		self.var_occurrences
	}

	fn scopes(&mut self) -> &mut Scopes {
		&mut self.scopes
	}
}

impl Des<'_> {
	fn new_alloc_scope(&mut self) {
		let scope_buf = OutputBuffer::new();
		self.buf.push(scope_buf.clone());
		let cursor_name = self.add_occurrence("cursor").0;
		self.scopes.alloc.push(AllocScope::new(scope_buf, cursor_name));
	}

	fn end_alloc_scope(&mut self) {
		self.end_alloc_scope_complex(|e| e, 0);
	}

	fn end_alloc_scope_complex(&mut self, cb: impl FnOnce(Expr) -> Expr, offset: usize) {
		let scope = self.scopes.alloc.pop().unwrap();
		scope.offset_by(offset);
		let size_expr = Expr::Num(scope.size as f64);
		scope.buf.push_local(scope.cursor_var, Some(read(cb(size_expr))));
	}

	fn new_dyn_scope(&mut self) {
		let scope_buf = OutputBuffer::new();
		self.buf.push(scope_buf.clone());
		self.scopes.dynamic.push(DynamicScope {
			bitpack_budget: vec![],
			buf: scope_buf,
		});
	}

	fn end_dyn_scope(&mut self) {
		let scope = self.scopes.dynamic.pop().unwrap();

		for (shift, name) in scope.bitpack_budget {
			let numty = NumTy::from_f64(0.0, ((1u64 << (shift + 1)) - 1) as f64);
			let value = self.readnumty_raw(numty, true);

			scope.buf.push(move || Stmt::Local(name.clone(), Some(value())));
			self.scopes.alloc().offset_by(numty.size());
		}
	}

	fn push_struct(&mut self, struct_ty: &Struct, into: Var) {
		for (name, ty) in struct_ty.fields.iter() {
			self.push_ty(ty, into.clone().eindex(Expr::Str((*name).into())))
		}
	}

	fn read_variant_storage(&mut self, storage: VariantStorageKind, mut cb: impl FnMut(&mut Self, usize)) {
		match storage {
			VariantStorageKind::Full(numty, amount) => {
				let (variant_i, variant_expr) = self.add_occurrence("variant");
				let expr = self.readnumty(numty);
				self.buf.push(move || Stmt::Local(variant_i, Some(expr())));
				for i in 0..amount {
					let cond = variant_expr.clone().eq((i as f64).into());

					self.buf.push(if i == 0 { Stmt::If(cond) } else { Stmt::ElseIf(cond) });
					cb(self, i);
				}
				self.buf.push(Stmt::End);
			}
			VariantStorageKind::Bitpack(variants) => {
				for (i, (bits, var)) in variants.into_iter().enumerate() {
					let cond = self.check_bitfield(bits, var);

					self.buf.push(if i == 0 { Stmt::If(cond) } else { Stmt::ElseIf(cond) });
					cb(self, i);
				}
				self.buf.push(Stmt::End);
			}
			VariantStorageKind::Bit((bits, var)) => {
				let cond = self.check_bitfield(bits, var);
				self.buf.push(Stmt::If(cond));
				cb(self, 1);
				self.buf.push(Stmt::Else);
				cb(self, 0);
				self.buf.push(Stmt::End);
			}
			VariantStorageKind::None => {
				cb(self, 0);
			}
		}
	}

	fn push_enum(&mut self, enum_ty: &Enum, into: Var) {
		match enum_ty {
			Enum::Unit(enumerators) => {
				let storage = self.variant_storage(enumerators.len());

				self.read_variant_storage(storage, |this, i| {
					let condition = match this.typescript_enum_type {
						TypeScriptEnumType::ConstNumber => Expr::Num(i as f64),
						_ => Expr::StrOrBool(enumerators[i].to_string()),
					};

					this.buf.push_assign(into.clone(), condition);
				});
			}

			Enum::Tagged { tag, variants } => {
				let storage = self.variant_storage(variants.len());

				self.read_variant_storage(storage, |this, i| {
					let (name, struct_ty) = &variants[i];
					this.buf.push_assign(
						into.clone(),
						Expr::Table(Box::new(vec![(
							Expr::Str(tag.to_string()),
							Expr::StrOrBool(name.to_string()),
						)])),
					);
					this.push_struct(struct_ty, into.clone());
				});
			}
		}
	}

	fn push_or(&mut self, into: Var, tys: &Vec<Ty<'_>>, optional: bool) {
		#[allow(clippy::type_complexity)]
		let mut ty_functions: Vec<Box<dyn FnMut(&mut Self)>> = Vec::new();

		for ty in tys {
			match ty.primitive_ty() {
				PrimitiveTy::Enum(Enum::Unit(variants)) => {
					for (i, variant) in variants.into_iter().enumerate() {
						let into = into.clone();
						ty_functions.push(Box::new(move |this| {
							let condition = match this.typescript_enum_type {
								TypeScriptEnumType::ConstNumber => Expr::Num(i as f64),
								_ => Expr::StrOrBool(variant.to_string()),
							};

							this.buf.push_assign(into.clone(), condition);
						}));
					}
				}
				PrimitiveTy::Enum(Enum::Tagged { tag, variants }) => {
					for (variant, data) in variants {
						let into = into.clone();
						ty_functions.push(Box::new(move |this| {
							this.buf.push_assign(
								into.clone(),
								Expr::Table(Box::new(vec![(
									Expr::Str(tag.to_string()),
									Expr::StrOrBool(variant.to_string()),
								)])),
							);
							this.new_alloc_scope();
							this.push_struct(&data, into.clone());
							this.end_alloc_scope();
						}));
					}
				}
				PrimitiveTy::Unknown => {
					ty_functions.push(Box::new(|this| {
						this.push_ty(&Ty::Unknown, into.clone());
					}));
				}
				PrimitiveTy::None(..) => unreachable!(),
				_ => {
					ty_functions.push(Box::new(|this| {
						this.new_alloc_scope();
						this.push_ty(ty, into.clone());
						this.end_alloc_scope();
					}));
				}
			};
		}

		if optional {
			ty_functions.push(Box::new(|this| {
				this.buf.push_assign(into.clone(), Expr::Nil);
			}));
		}

		let storage = self.variant_storage(ty_functions.len());
		self.read_variant_storage(storage, |this, i| {
			ty_functions[i](this);
		});
	}

	fn check_bitfield(&mut self, bits: BitpackMask, var: Var) -> Expr {
		Expr::Call(
			Var::NameIndex(Var::Name("bit32".into()).into(), "btest".into()).into(),
			None,
			vec![Expr::Var(var.into()), Expr::BinaryNum(bits)],
		)
	}

	fn push_ty(&mut self, ty: &Ty, into: Var) {
		let into_expr = Expr::from(into.clone());

		match ty {
			Ty::Num(numty, range) => {
				let expr = self.readnumty(*numty);
				self.buf.push(move || Stmt::Assign(into, expr()));

				if self.checks {
					self.buf.push_range_check(into_expr, *range);
				}
			}

			Ty::Str(utf8, range) => {
				if !utf8 && let Some(len) = range.exact() {
					self.buf.push_assign(into, readstring(len.into()));
				} else {
					let (len_name, len_expr) = self.add_occurrence("len");
					let (len_numty, len_offset) = range.numty();

					let offset_len_expr = self.readnumty(len_numty);

					self.buf.push(move || {
						let mut offset_len_expr = offset_len_expr();
						if len_offset != 0.0 {
							offset_len_expr = offset_len_expr.add(Expr::Num(len_offset))
						}

						Stmt::Local(len_name.clone(), Some(offset_len_expr))
					});

					if self.checks {
						self.buf.push_range_check(len_expr.clone(), *range);
					}

					self.buf.push_assign(into.clone(), readstring(len_expr.clone()));

					if *utf8 {
						self.buf.push_utf8_check(Expr::Var(into.into()));
					}
				}
			}

			Ty::Buf(range) => {
				if let Some(len) = range.exact() {
					self.buf.push_read_copy(into, len.into());
				} else {
					let (len_name, len_expr) = self.add_occurrence("len");
					let (len_numty, len_offset) = range.numty();

					let offset_len_expr = self.readnumty(len_numty);

					self.buf.push(move || {
						let mut offset_len_expr = offset_len_expr();
						if len_offset != 0.0 {
							offset_len_expr = offset_len_expr.add(Expr::Num(len_offset))
						}

						Stmt::Local(len_name.clone(), Some(offset_len_expr))
					});

					if self.checks {
						self.buf.push_range_check(len_expr.clone(), *range);
					}

					self.buf.push_read_copy(into, len_expr.clone())
				}
			}

			Ty::Arr(ty, range) => {
				if let Some(len) = range.exact() {
					self.buf.push_assign(
						into.clone(),
						Expr::Call(
							Var::NameIndex(Var::Name("table".into()).into(), "create".into()).into(),
							None,
							vec![Expr::Num(len)],
						),
					);

					for i in 1..=(len as usize) {
						self.push_ty(ty, into.clone().eindex((i as f64).into()));
					}
				} else {
					let (var_name, var_expr) = self.add_occurrence("i");
					let (len_name, len_expr) = self.add_occurrence("len");
					let (len_numty, len_offset) = range.numty();

					let offset_len_expr = self.readnumty(len_numty);

					self.buf.push(move || {
						let mut offset_len_expr = offset_len_expr();
						if len_offset != 0.0 {
							offset_len_expr = offset_len_expr.add(Expr::Num(len_offset))
						}

						Stmt::Local(len_name.clone(), Some(offset_len_expr))
					});

					if self.checks {
						self.buf.push_range_check(len_expr.clone(), *range);
					}

					self.buf.push_assign(
						into.clone(),
						Expr::Call(
							Var::NameIndex(Var::Name("table".into()).into(), "create".into()).into(),
							None,
							vec![len_expr.clone()],
						),
					);

					self.new_alloc_scope();

					self.buf.push(Stmt::NumFor {
						var: var_name.clone(),
						from: 1.0.into(),
						to: len_expr.clone(),
					});

					self.new_dyn_scope();

					let (inner_var_name, _) = self.add_occurrence("val");

					self.buf.push_local(inner_var_name.clone(), None);

					self.push_ty(ty, Var::Name(inner_var_name.clone()));

					self.buf.push(Stmt::Assign(
						into.clone().eindex(var_expr.clone()),
						Var::Name(inner_var_name.clone()).into(),
					));

					self.end_dyn_scope();

					self.buf.push(Stmt::End);

					let scope = self.scopes.alloc();
					scope.offset_expr(var_expr.clone().sub(1.0.into()).mul((scope.size as f64).into()));
					self.end_alloc_scope_complex(|expr| expr.mul(len_expr), 0);
				}
			}

			Ty::Map(key, val) => {
				let (empty_bits, empty_var) = self.get_bitpack();
				let length_numty = key.variants().map(|(numty, ..)| numty).unwrap_or(NumTy::U16);

				self.buf.push_assign(into.clone(), Expr::Table(Default::default()));

				let empty_expr = self.check_bitfield(empty_bits, empty_var);
				self.buf.push(Stmt::If(empty_expr.not()));

				let offset_len_expr = self.readnumty(length_numty);

				self.buf.push(move || Stmt::NumFor {
					var: "_".into(),
					from: 1.0.into(),
					to: offset_len_expr().add(1.0.into()),
				});

				self.new_alloc_scope();
				self.new_dyn_scope();

				let (key_name, key_expr) = self.add_occurrence("key");
				self.buf.push_local(key_name.clone(), None);
				let (val_name, val_expr) = self.add_occurrence("val");
				self.buf.push_local(val_name.clone(), None);

				self.push_ty(key, Var::Name(key_name.clone()));
				self.push_ty(val, Var::Name(val_name.clone()));

				self.buf
					.push_assign(into.clone().eindex(key_expr.clone()), val_expr.clone());

				self.end_dyn_scope();
				self.end_alloc_scope();

				self.buf.push(Stmt::End);

				self.buf.push(Stmt::End);
			}

			Ty::Set(key) => {
				let (empty_bits, empty_var) = self.get_bitpack();
				let length_numty = key.variants().map(|(numty, ..)| numty).unwrap_or(NumTy::U16);

				self.buf.push_assign(into.clone(), Expr::Table(Default::default()));

				let empty_expr = self.check_bitfield(empty_bits, empty_var);
				self.buf.push(Stmt::If(empty_expr.not()));

				let offset_len_expr = self.readnumty(length_numty);

				self.buf.push(move || Stmt::NumFor {
					var: "_".into(),
					from: 1.0.into(),
					to: offset_len_expr().add(1.0.into()),
				});

				self.new_alloc_scope();
				self.new_dyn_scope();

				let (key_name, key_expr) = self.add_occurrence("key");
				self.buf.push_local(key_name.clone(), None);

				self.push_ty(key, Var::Name(key_name.clone()));

				self.buf.push_assign(into.clone().eindex(key_expr.clone()), Expr::True);

				self.end_dyn_scope();
				self.end_alloc_scope();

				self.buf.push(Stmt::End);

				self.buf.push(Stmt::End);
			}

			Ty::Opt(ty) => {
				if let Ty::Or(tys, _) = &**ty {
					return self.push_or(into, tys, true);
				}

				let (bits, var) = self.get_bitpack();
				let expr = self.check_bitfield(bits, var);

				self.buf.push(Stmt::If(expr));

				self.new_alloc_scope();

				if let Ty::Instance(class) = **ty {
					self.buf
						.push_assign(Var::from("incoming_ipos"), Expr::from("incoming_ipos").add(1.0.into()));
					self.buf.push_assign(
						into.clone(),
						Var::from("incoming_inst")
							.eindex(Var::from("incoming_ipos").into())
							.into(),
					);

					if self.checks && class.is_some() {
						self.buf.push_assert(
							into_expr.clone().eq(Expr::Nil).or(Expr::Call(
								Box::new(into.clone()),
								Some("IsA".into()),
								vec![Expr::Str(class.unwrap().into())],
							)),
							format!("received instance is not of the {} class!", class.unwrap()),
						)
					}
				} else {
					self.push_ty(ty, into.clone())
				}

				self.end_alloc_scope();

				self.buf.push(Stmt::Else);
				self.buf.push_assign(into, Expr::Nil);
				self.buf.push(Stmt::End);
			}

			Ty::Ref(name, ..) => {
				self.buf.push_assign(
					into,
					Expr::Call(
						Box::new(Var::from("types").nindex(format!("read_{name}"))),
						None,
						vec![],
					),
				);
			}

			Ty::Enum(enum_ty) => self.push_enum(enum_ty, into),

			Ty::Struct(struct_ty) => {
				self.buf.push_assign(into.clone(), Expr::Table(Default::default()));
				self.push_struct(struct_ty, into)
			}

			Ty::Or(tys, _) => self.push_or(into, tys, false),

			Ty::Instance(class) => {
				self.buf
					.push_assign(Var::from("incoming_ipos"), Expr::from("incoming_ipos").add(1.0.into()));
				self.buf.push_assign(
					into.clone(),
					Var::from("incoming_inst")
						.eindex(Var::from("incoming_ipos").into())
						.into(),
				);

				// always assert non-optional instances as roblox
				// will sometimes vaporize them
				self.buf
					.push_assert(into_expr.clone().neq(Expr::Nil), "received instance is nil!".into());

				if self.checks && class.is_some() {
					self.buf.push_assert(
						Expr::Call(
							Box::new(into),
							Some("IsA".into()),
							vec![Expr::Str(class.unwrap().into())],
						),
						format!("received instance is not of the {} class!", class.unwrap()),
					)
				}
			}

			Ty::Unknown => {
				self.buf
					.push_assign(Var::from("incoming_ipos"), Expr::from("incoming_ipos").add(1.0.into()));
				self.buf.push_assign(
					into.clone(),
					Var::from("incoming_inst")
						.eindex(Var::from("incoming_ipos").into())
						.into(),
				);
			}

			Ty::BrickColor => {
				let expr = self.readu16();

				self.buf.push(move || {
					Stmt::Assign(
						into,
						Expr::Call(Box::new(Var::from("BrickColor").nindex("new")), None, vec![expr()]),
					)
				})
			}

			Ty::DateTimeMillis => {
				let expr = self.readf64();
				self.buf.push(move || {
					Stmt::Assign(
						into,
						Expr::Call(
							Box::new(Var::from("DateTime").nindex("fromUnixTimestampMillis")),
							None,
							vec![expr()],
						),
					)
				})
			}

			Ty::DateTime => {
				let expr = self.readf64();
				self.buf.push(move || {
					Stmt::Assign(
						into,
						Expr::Call(
							Box::new(Var::from("DateTime").nindex("fromUnixTimestamp")),
							None,
							vec![expr()],
						),
					)
				})
			}

			Ty::Boolean => {
				let (bits, var) = self.get_bitpack();
				let expr = self.check_bitfield(bits, var);

				self.buf.push_assign(into, expr);
			}

			Ty::Color3 => {
				let r = self.readu8();
				let g = self.readu8();
				let b = self.readu8();

				self.buf
					.push(move || Stmt::Assign(into, Expr::Color3(Box::new(r()), Box::new(g()), Box::new(b()))))
			}

			Ty::Vector2 => {
				let x = self.readf32();
				let y = self.readf32();

				self.buf.push(move || {
					Stmt::Assign(
						into,
						Expr::Call(
							Box::new(Var::from("Vector3").nindex("new")),
							None,
							vec![x(), y(), "0".into()],
						),
					)
				})
			}
			Ty::Vector3 => {
				let expr = self.readvector3();
				self.buf.push(move || Stmt::Assign(into, expr()))
			}
			Ty::Vector(x_ty, y_ty, z_ty) => {
				let x_numty = match **x_ty {
					Ty::Num(numty, range) => {
						if self.checks {
							self.buf.push_range_check(into_expr.clone(), range);
						}

						numty
					}
					_ => unreachable!(),
				};
				let y_numty = match **y_ty {
					Ty::Num(numty, range) => {
						if self.checks {
							self.buf.push_range_check(into_expr.clone(), range);
						}

						numty
					}
					_ => unreachable!(),
				};
				let z_numty = if let Some(z_ty) = z_ty {
					match **z_ty {
						Ty::Num(numty, range) => {
							if self.checks {
								self.buf.push_range_check(into_expr.clone(), range);
							}

							Some(numty)
						}
						_ => unreachable!(),
					}
				} else {
					None
				};

				let expr = self.readvector(x_numty, y_numty, z_numty);
				self.buf.push(move || Stmt::Assign(into, expr()));
			}

			Ty::AlignedCFrame => {
				let (axis_alignment_name, axis_alignment_expr) = self.add_occurrence("axis_alignment");
				let align_expr = self.readu8();
				self.buf
					.push(move || Stmt::Local(axis_alignment_name, Some(align_expr())));
				let (pos_name, pos_expr) = self.add_occurrence("pos");

				let pos_vec = self.readvector3();
				self.buf.push(move || Stmt::Local(pos_name.clone(), Some(pos_vec())));

				self.buf.push_assign(
					into,
					Expr::Mul(
						Box::new(Expr::Call(
							Box::new(Var::from("CFrame").nindex("new")),
							None,
							vec![pos_expr.clone()],
						)),
						Box::new(
							Var::from("CFrameSpecialCases")
								.eindex(axis_alignment_expr.clone())
								.into(),
						),
					),
				);
			}
			Ty::CFrame => {
				let (pos_name, pos_expr) = self.add_occurrence("pos");
				let pos_vec = self.readvector3();
				self.buf.push(move || Stmt::Local(pos_name.clone(), Some(pos_vec())));
				let (axisangle_name, axisangle_expr) = self.add_occurrence("axisangle");
				let axis_expr = self.readvector3();
				let axis_name = axisangle_name.clone();
				self.buf.push(move || Stmt::Local(axis_name, Some(axis_expr())));
				let (angle_name, angle_expr) = self.add_occurrence("angle");
				self.buf.push_local(
					angle_name,
					Some(Var::from(axisangle_name.as_str()).nindex("Magnitude").into()),
				);

				// We don't need to convert the axis back to a unit vector as the constructor does that for us
				// The angle is the magnitude of the axis vector
				// If the magnitude is 0, there is no rotation, so just make a cframe at the right position.
				// 	Trying to use fromAxisAngle in this situation gives NAN which is not ideal, so the branch is required.

				// if angle ~= 0 then
				//		value = CFrame.fromAxisAngle(axisangle, angle) + pos
				// else
				//		value = CFrame.new(pos)
				// end

				self.buf
					.push(Stmt::If(Expr::Neq(Box::new(angle_expr.clone()), Box::new("0".into()))));
				self.buf.push_assign(
					into.clone(),
					Expr::Add(
						Box::new(Expr::Call(
							Box::new(Var::from("CFrame").nindex("fromAxisAngle")),
							None,
							vec![axisangle_expr.clone(), angle_expr.clone()],
						)),
						Box::new(pos_expr.clone()),
					),
				);
				self.buf.push(Stmt::Else);
				self.buf.push_assign(
					into,
					Expr::Call(
						Box::new(Var::from("CFrame").nindex("new")),
						None,
						vec![pos_expr.clone()],
					),
				);
				self.buf.push(Stmt::End);
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
	Des {
		checks,
		buf: OutputBuffer::new(),
		var_occurrences,
		scopes: Default::default(),
		typescript_enum_type,
	}
	.generate(names, types.into_iter())
}
