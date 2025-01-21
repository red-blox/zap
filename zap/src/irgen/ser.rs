use crate::config::{Enum, NumTy, Struct, Ty};
use std::collections::HashMap;

use super::{Expr, Gen, Stmt, Var};

struct Ser<'src> {
	checks: bool,
	buf: Vec<Stmt>,
	var_occurrences: &'src mut HashMap<String, usize>,
}

impl<'src> Gen for Ser<'src> {
	fn push_stmt(&mut self, stmt: Stmt) {
		self.buf.push(stmt);
	}

	fn gen<'a, I>(mut self, names: &[String], types: I) -> Vec<Stmt>
	where
		I: Iterator<Item = &'a Ty<'a>>,
	{
		for (ty, name) in types.zip(names) {
			self.push_ty(ty, Var::Name(name.to_string()));
		}

		self.buf
	}

	fn get_var_occurrences(&mut self) -> &mut HashMap<String, usize> {
		&mut self.var_occurrences
	}
}

impl<'src> Ser<'src> {
	fn push_struct(&mut self, struct_ty: &Struct, from: Var) {
		for (name, ty) in struct_ty.fields.iter() {
			self.push_ty(ty, from.clone().nindex(*name));
		}
	}

	fn push_enum(&mut self, enum_ty: &Enum, from: Var) {
		match enum_ty {
			Enum::Unit(enumerators) => {
				let from_expr = Expr::from(from.clone());
				let numty = NumTy::from_f64(0.0, enumerators.len() as f64 - 1.0);

				for (i, enumerator) in enumerators.iter().enumerate() {
					if i == 0 {
						self.push_stmt(Stmt::If(from_expr.clone().eq(Expr::StrOrBool(enumerator.to_string()))));
					} else {
						self.push_stmt(Stmt::ElseIf(
							from_expr.clone().eq(Expr::StrOrBool(enumerator.to_string())),
						));
					}

					self.push_writenumty((i as f64).into(), numty);
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid enumerator".into()));
				self.push_stmt(Stmt::End);
			}

			Enum::Tagged { tag, variants } => {
				let tag_expr = Expr::from(from.clone().nindex(*tag));

				for (i, variant) in variants.iter().enumerate() {
					if i == 0 {
						self.push_stmt(Stmt::If(tag_expr.clone().eq(Expr::StrOrBool(variant.0.to_string()))));
					} else {
						self.push_stmt(Stmt::ElseIf(
							tag_expr.clone().eq(Expr::StrOrBool(variant.0.to_string())),
						));
					}

					self.push_writeu8((i as f64).into());
					self.push_struct(&variant.1, from.clone());
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid variant".into()));
				self.push_stmt(Stmt::End);
			}
		}
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

			Ty::Str(range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_writestring(from_expr, len.into());
				} else {
					let (len_name, len_expr) = self.add_occurrence("len");
					self.push_local(len_name.clone(), Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check(len_expr.clone(), *range);
					}

					self.push_writeu16(len_expr.clone());
					self.push_writestring(from_expr, len_expr.clone());
				}
			}

			Ty::Buf(range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_write_copy(from_expr, len.into());
				} else {
					let (len_name, len_expr) = self.add_occurrence("len");
					self.push_local(
						len_name.clone(),
						Some(Var::from("buffer").nindex("len").call(vec![from_expr.clone()])),
					);

					if self.checks {
						self.push_range_check(len_expr.clone(), *range);
					}

					self.push_writeu16(len_expr.clone());
					self.push_write_copy(from_expr, len_name.as_str().into())
				}
			}

			Ty::Arr(ty, range) => {
				let (var_name, var_expr) = self.add_occurrence("i");

				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_stmt(Stmt::NumFor {
						var: var_name.clone(),
						from: 1.0.into(),
						to: len.into(),
					});

					self.push_ty(ty, from.clone().eindex(var_expr.clone()));
					self.push_stmt(Stmt::End);
				} else {
					let (len_name, len_expr) = self.add_occurrence("len");
					self.push_local(len_name.clone(), Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check(len_expr.clone(), *range);
					}

					self.push_writeu16(len_expr.clone());

					self.push_stmt(Stmt::NumFor {
						var: var_name.clone(),
						from: 1.0.into(),
						to: len_expr.clone(),
					});

					let (inner_var_name, _) = self.add_occurrence("val");

					self.push_stmt(Stmt::Local(
						inner_var_name.clone(),
						Some(from.clone().eindex(var_expr.clone()).into()),
					));

					self.push_ty(ty, Var::Name(inner_var_name));
					self.push_stmt(Stmt::End);
				}
			}

			Ty::Map(key, val) => {
				let (len_name, len_expr) = self.add_occurrence("len");
				let (len_pos_name, len_pos_expr) = self.add_occurrence("len_pos");

				self.push_local(len_pos_name.clone(), Some(Var::from("alloc").call(vec![2.0.into()])));
				self.push_local(len_name.clone(), Some(0.0.into()));

				let (key_name, _) = self.add_occurrence("k");
				let (val_name, _) = self.add_occurrence("v");

				self.push_stmt(Stmt::GenFor {
					key: key_name.clone(),
					val: val_name.clone(),
					obj: from_expr,
				});

				self.push_assign(Var::Name(len_name.clone()), len_expr.clone().add(1.0.into()));
				self.push_ty(key, key_name.as_str().into());
				self.push_ty(val, val_name.as_str().into());

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::Call(
					Var::from("buffer").nindex("writeu16"),
					None,
					vec!["outgoing_buff".into(), len_pos_expr.clone(), len_expr.clone()],
				));
			}

			Ty::Set(key) => {
				let (len_name, len_expr) = self.add_occurrence("len");
				let (len_pos_name, len_pos_expr) = self.add_occurrence("len_pos");

				self.push_local(len_pos_name.clone(), Some(Var::from("alloc").call(vec![2.0.into()])));
				self.push_local(len_name.clone(), Some(0.0.into()));

				let (key_name, _) = self.add_occurrence("k");
				let (val_name, _) = self.add_occurrence("_");

				self.push_stmt(Stmt::GenFor {
					key: key_name.clone(),
					val: val_name.clone(),
					obj: from_expr,
				});

				self.push_assign(Var::Name(len_name.clone()), len_expr.clone().add(1.0.into()));
				self.push_ty(key, key_name.as_str().into());

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::Call(
					Var::from("buffer").nindex("writeu16"),
					None,
					vec!["outgoing_buff".into(), len_pos_expr.clone(), len_expr.clone()],
				));
			}

			Ty::Opt(ty) => {
				self.push_stmt(Stmt::If(from_expr.clone().eq(Expr::Nil)));

				self.push_writeu8(0.0.into());

				self.push_stmt(Stmt::Else);

				self.push_writeu8(1.0.into());
				self.push_ty(ty, from);

				self.push_stmt(Stmt::End);
			}

			Ty::Ref(name) => self.push_stmt(Stmt::Call(
				Var::from("types").nindex(format!("write_{name}")),
				None,
				vec![from_expr],
			)),

			Ty::Enum(enum_ty) => self.push_enum(enum_ty, from),
			Ty::Struct(struct_ty) => self.push_struct(struct_ty, from),

			Ty::Instance(class) => {
				if self.checks && class.is_some() {
					self.push_assert(
						Expr::Call(
							Box::new(from),
							Some("IsA".into()),
							vec![Expr::Str(class.unwrap().into())],
						),
						None,
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

				self.push_assert(
					axis_alignment_expr.clone(),
					Some("CFrame not aligned to an axis!".to_string()),
				);

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

			Ty::Boolean => self.push_writeu8(from_expr.and(1.0.into()).or(0.0.into())),
		}
	}
}

pub fn gen<'a, I>(types: I, names: &[String], checks: bool, var_occurrences: &mut HashMap<String, usize>) -> Vec<Stmt>
where
	I: IntoIterator<Item = &'a Ty<'a>>,
{
	Ser {
		checks,
		buf: vec![],
		var_occurrences,
	}
	.gen(names, types.into_iter())
}
