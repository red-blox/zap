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
			Ty::Vector3 => {
				self.push_local("x", Some(self.readnumty(NumTy::F32)));
				self.push_local("y", Some(self.readnumty(NumTy::F32)));
				self.push_local("z", Some(self.readnumty(NumTy::F32)));

				self.push_assign(
					into,
					Expr::Vector3(Box::new("x".into()), Box::new("y".into()), Box::new("z".into())),
				);
			}
		}
	}
}

pub fn gen(ty: &Ty, var: &str, checks: bool) -> Vec<Stmt> {
	Des { checks, buf: vec![] }.gen(var.into(), ty)
}
