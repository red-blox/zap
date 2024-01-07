use crate::config::{Enum, NumTy, Struct, Ty};

use super::{Expr, Gen, Stmt, Var};

struct Des {
	checks: bool,
	buf: Vec<Stmt>,
}

impl Gen for Des {
	fn push_stmt(&mut self, stmt: Stmt) {
		self.buf.push(stmt);
	}

	fn gen(mut self, var: Var, ty: &Ty) -> Vec<Stmt> {
		self.push_ty(ty, var);
		self.buf
	}
}

impl Des {
	fn push_struct(&mut self, struct_ty: &Struct, into: Var) {
		for (name, ty) in struct_ty.fields.iter() {
			self.push_ty(ty, into.clone().nindex(*name))
		}
	}

	fn push_enum(&mut self, enum_ty: &Enum, into: Var) {
		match enum_ty {
			Enum::Unit(enumerators) => {
				let numty = NumTy::from_f64(0.0, enumerators.len() as f64 - 1.0);

				self.push_local("enum_value", Some(self.readnumty(numty)));

				for (i, enumerator) in enumerators.iter().enumerate() {
					if i == 0 {
						self.push_stmt(Stmt::If(Expr::from("enum_value").eq((i as f64).into())));
					} else {
						self.push_stmt(Stmt::ElseIf(Expr::from("enum_value").eq((i as f64).into())));
					}

					self.push_assign(into.clone(), Expr::Str(enumerator.to_string()));
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid enumerator".into()));
				self.push_stmt(Stmt::End);
			}

			Enum::Tagged { tag, variants } => {
				let numty = NumTy::from_f64(0.0, variants.len() as f64 - 1.0);

				self.push_local("enum_value", Some(self.readnumty(numty)));

				for (i, (name, struct_ty)) in variants.iter().enumerate() {
					if i == 0 {
						self.push_stmt(Stmt::If(Expr::from("enum_value").eq((i as f64).into())));
					} else {
						self.push_stmt(Stmt::ElseIf(Expr::from("enum_value").eq((i as f64).into())));
					}

					self.push_assign(into.clone().nindex(*tag), Expr::Str(name.to_string()));
					self.push_struct(struct_ty, into.clone());
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid variant".into()));
				self.push_stmt(Stmt::End);
			}
		}
	}

	fn push_ty(&mut self, ty: &Ty, into: Var) {
		let into_expr = Expr::from(into.clone());

		match ty {
			Ty::Num(numty, range) => {
				self.push_assign(into, self.readnumty(*numty));

				if self.checks {
					self.push_range_check(into_expr, *range);
				}
			}

			Ty::Str(range) => {
				if let Some(len) = range.exact() {
					self.push_assign(into, self.readstring(len.into()));
				} else {
					self.push_local("len", Some(self.readnumty(NumTy::U16)));

					if self.checks {
						self.push_range_check(Expr::from("len"), *range);
					}

					self.push_assign(into, self.readstring(Expr::from("len")));
				}
			}

			Ty::Buf(range) => {
				if let Some(len) = range.exact() {
					self.push_read_copy(into, len.into());
				} else {
					self.push_local("len", Some(self.readnumty(NumTy::U16)));

					if self.checks {
						self.push_range_check(Expr::from("len"), *range);
					}

					self.push_read_copy(into, Expr::from("len"))
				}
			}

			Ty::Arr(ty, range) => {
				self.push_assign(into.clone(), Expr::EmptyTable);

				if let Some(len) = range.exact() {
					self.push_stmt(Stmt::NumFor {
						var: "i",
						from: 1.0.into(),
						to: len.into(),
					});

					self.push_ty(ty, into.clone().eindex("i".into()));
					self.push_stmt(Stmt::End);
				} else {
					self.push_local("len", Some(self.readnumty(NumTy::U16)));

					if self.checks {
						self.push_range_check(Expr::from("len"), *range);
					}

					self.push_stmt(Stmt::NumFor {
						var: "i",
						from: 1.0.into(),
						to: "len".into(),
					});

					self.push_ty(ty, into.clone().eindex("i".into()));
					self.push_stmt(Stmt::End);
				}
			}

			Ty::Map(key, val) => {
				self.push_assign(into.clone(), Expr::EmptyTable);

				self.push_stmt(Stmt::NumFor {
					var: "_",
					from: 1.0.into(),
					to: self.readu16(),
				});

				self.push_local("key", None);
				self.push_local("val", None);

				self.push_ty(key, "key".into());
				self.push_ty(val, "val".into());

				self.push_assign(into.clone().eindex("key".into()), "val".into());

				self.push_stmt(Stmt::End);
			}

			Ty::Opt(ty) => {
				self.push_stmt(Stmt::If(self.readu8().eq(1.0.into())));

				match **ty {
					Ty::Instance(class) => {
						self.push_assign(Var::from("incoming_ipos"), Expr::from("incoming_ipos").add(1.0.into()));
						self.push_assign(
							into.clone(),
							Var::from("incoming_inst")
								.eindex(Var::from("incoming_ipos").into())
								.into(),
						);

						if self.checks && class.is_some() {
							self.push_assert(
								into_expr.clone().eq(Expr::Nil).or(Expr::Call(
									Box::new(into.clone()),
									Some("IsA".into()),
									vec![Expr::Str(class.unwrap().into())],
								)),
								None,
							)
						}
					}

					Ty::Unknown => {
						self.push_assign(Var::from("incoming_ipos"), Expr::from("incoming_ipos").add(1.0.into()));
						self.push_assign(
							into.clone(),
							Var::from("incoming_inst")
								.eindex(Var::from("incoming_ipos").into())
								.into(),
						);
					}

					_ => self.push_ty(ty, into.clone()),
				}

				self.push_stmt(Stmt::Else);
				self.push_assign(into, Expr::Nil);
				self.push_stmt(Stmt::End);
			}

			Ty::Ref(name) => {
				self.push_assign(
					into,
					Expr::Call(
						Box::new(Var::from("types").nindex(format!("read_{name}"))),
						None,
						vec![],
					),
				);
			}

			Ty::Enum(enum_ty) => {
				self.push_assign(into.clone(), Expr::EmptyTable);
				self.push_enum(enum_ty, into)
			}

			Ty::Struct(struct_ty) => {
				self.push_assign(into.clone(), Expr::EmptyTable);
				self.push_struct(struct_ty, into)
			}

			Ty::Instance(class) => {
				self.push_assign(Var::from("incoming_ipos"), Expr::from("incoming_ipos").add(1.0.into()));
				self.push_assign(
					into.clone(),
					Var::from("incoming_inst")
						.eindex(Var::from("incoming_ipos").into())
						.into(),
				);

				// always assert non-optional instances as roblox
				// will sometimes vaporize them
				self.push_assert(into_expr.clone().neq(Expr::Nil), None);

				if self.checks && class.is_some() {
					self.push_assert(
						Expr::Call(
							Box::new(into),
							Some("IsA".into()),
							vec![Expr::Str(class.unwrap().into())],
						),
						None,
					)
				}
			}

			// unknown is always an opt
			Ty::Unknown => unreachable!(),

			Ty::Boolean => self.push_assign(into, self.readu8().eq(1.0.into())),
			Ty::Color3 => self.push_assign(
				into,
				Expr::Color3(
					Box::new(self.readu8()),
					Box::new(self.readu8()),
					Box::new(self.readu8()),
				),
			),
			Ty::Vector3 => self.push_assign(into, self.readvector3()),
			Ty::AlignedCFrame => {
				self.push_local("axis_alignment", Some(self.readu8()));

				self.push_local("pos", Some(self.readvector3()));

				self.push_assign(
					into,
					Expr::Mul(
						Box::new(Expr::Call(
							Box::new(Var::from("CFrame").nindex("new")),
							None,
							vec!["pos".into()],
						)),
						Box::new(Var::from("CFrameSpecialCases").eindex("axis_alignment".into()).into()),
					),
				);
			}
			Ty::CFrame => {
				self.push_local("pos", Some(self.readvector3()));
				self.push_local("axisangle", Some(self.readvector3()));
				self.push_local("angle", Some(Var::from("axisangle").nindex("Magnitude").into()));

				// We don't need to convert the axis back to a unit vector as the constructor does that for us
				// The angle is the magnitude of the axis vector
				// If the magnitude is 0, there is no rotation, so just make a cframe at the right position.
				// 	Trying to use fromAxisAngle in this situation gives NAN which is not ideal, so the branch is required.

				// if angle ~= 0 then
				//		value = CFrame.fromAxisAngle(axisangle, angle) + pos
				// else
				//		value = CFrame.new(pos)
				// end

				self.push_stmt(Stmt::If(Expr::Neq(Box::new("angle".into()), Box::new("0".into()))));
				self.push_assign(
					into.clone(),
					Expr::Add(
						Box::new(Expr::Call(
							Box::new(Var::from("CFrame").nindex("fromAxisAngle")),
							None,
							vec!["axisangle".into(), "angle".into()],
						)),
						Box::new("pos".into()),
					),
				);
				self.push_stmt(Stmt::Else);
				self.push_assign(
					into,
					Expr::Call(Box::new(Var::from("CFrame").nindex("new")), None, vec!["pos".into()]),
				);
				self.push_stmt(Stmt::End);
			}
		}
	}
}

pub fn gen(ty: &Ty, var: &str, checks: bool) -> Vec<Stmt> {
	Des { checks, buf: vec![] }.gen(var.into(), ty)
}
