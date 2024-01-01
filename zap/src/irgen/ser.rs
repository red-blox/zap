use crate::config::{Enum, NumTy, Struct, Ty};

use super::{Expr, Gen, Stmt, Var};

struct Ser {
	checks: bool,
	buf: Vec<Stmt>,
}

impl Gen for Ser {
	fn push_stmt(&mut self, stmt: Stmt) {
		self.buf.push(stmt);
	}

	fn gen(mut self, var: Var, ty: &Ty) -> Vec<Stmt> {
		self.push_ty(ty, var);
		self.buf
	}
}

impl Ser {
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
						self.push_stmt(Stmt::If(from_expr.clone().eq(Expr::Str(enumerator.to_string()))));
					} else {
						self.push_stmt(Stmt::ElseIf(from_expr.clone().eq(Expr::Str(enumerator.to_string()))));
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
						self.push_stmt(Stmt::If(tag_expr.clone().eq(Expr::Str(variant.0.to_string()))));
					} else {
						self.push_stmt(Stmt::ElseIf(tag_expr.clone().eq(Expr::Str(variant.0.to_string()))));
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
					self.push_local("len", Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check("len".into(), *range);
					}

					self.push_writeu16("len".into());
					self.push_writestring(from_expr, "len".into());
				}
			}

			Ty::Buf(range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_write_copy(from_expr, len.into());
				} else {
					self.push_local(
						"len",
						Some(Var::from("buffer").nindex("len").call(vec![from_expr.clone()])),
					);

					if self.checks {
						self.push_range_check("len".into(), *range);
					}

					self.push_writeu16("len".into());
					self.push_write_copy(from_expr, "len".into())
				}
			}

			Ty::Arr(ty, range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_stmt(Stmt::NumFor {
						var: "i",
						from: 1.0.into(),
						to: len.into(),
					});

					self.push_ty(ty, from.clone().eindex("i".into()));
					self.push_stmt(Stmt::End);
				} else {
					self.push_local("len", Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check("len".into(), *range);
					}

					self.push_writeu16("len".into());

					self.push_stmt(Stmt::NumFor {
						var: "i",
						from: 1.0.into(),
						to: "len".into(),
					});

					self.push_ty(ty, from.clone().eindex("i".into()));
					self.push_stmt(Stmt::End);
				}
			}

			Ty::Map(key, val) => {
				self.push_local("len_pos", Some(Var::from("alloc").call(vec![2.0.into()])));
				self.push_local("len", Some(0.0.into()));

				self.push_stmt(Stmt::GenFor {
					key: "k",
					val: "v",
					obj: from_expr,
				});

				self.push_assign("len".into(), Expr::from("len").add(1.0.into()));
				self.push_ty(key, "k".into());
				self.push_ty(val, "v".into());

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::Call(
					Var::from("buffer").nindex("writeu16"),
					None,
					vec!["outgoing_buff".into(), "len_pos".into(), "len".into()],
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

			Ty::Vector3 => {
				self.push_writef32(from.clone().nindex("X").into());
				self.push_writef32(from.clone().nindex("Y").into());
				self.push_writef32(from.clone().nindex("Z").into());
			}

			Ty::CFrame => {
				// with thanks to https://github.com/MaximumADHD/Roblox-File-Format
				self.push_local("orient_id", None);

				self.push_stmt(Stmt::GenFor {
					key: "_",
					val: "test",
					obj: Expr::Call(
						Box::new(Var::from("table").nindex("pack")),
						None,
						vec![
							Expr::Call(
								Box::new(from.clone().nindex("XVector").nindex("Dot")),
								None,
								vec![Var::from("Vector3").nindex("xAxis").into()],
							),
							Expr::Call(
								Box::new(from.clone().nindex("YVector").nindex("Dot")),
								None,
								vec![Var::from("Vector3").nindex("yAxis").into()],
							),
							Expr::Call(
								Box::new(from.clone().nindex("ZVector").nindex("Dot")),
								None,
								vec![Var::from("Vector3").nindex("zAxis").into()],
							),
						],
					),
				});

				self.push_local(
					"dot",
					Some(Expr::Call(
						Box::new(Var::from("math").nindex("abs")),
						None,
						vec![Var::from("test").into()],
					)),
				);

				self.push_stmt(Stmt::If(Expr::Or(
					Box::new(Expr::Eq(Box::new(Var::from("dot").into()), Box::new(Expr::Num(0.0)))),
					Box::new(Expr::Eq(Box::new(Var::from("dot").into()), Box::new(Expr::Num(1.0)))),
				)));

				self.push_stmt(Stmt::Continue);

				self.push_stmt(Stmt::End);

				self.push_assign("orient_id".into(), Expr::Num(0.0));

				self.push_stmt(Stmt::Break);

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::If(Expr::Neq(
					Box::new(Var::from("orient_id").into()),
					Box::new(Expr::Num(0.0)),
				)));

				self.push_stmt(Stmt::LocalTuple(
					vec!["_x", "_y", "_z", "R00", "R01", "R02", "R10", "R11", "R12"],
					Some(Expr::Call(Box::new(from.clone()), Some("GetComponents".into()), vec![])),
				));

				self.push_local("cols", Some(Expr::EmptyTable));

				self.push_stmt(Stmt::GenFor {
					key: "_",
					val: "col",
					obj: Expr::Call(
						Box::new(Var::from("table").nindex("pack")),
						None,
						vec![
							Expr::Call(
								Box::new(Var::from("Vector3").nindex("new")),
								None,
								vec![
									Var::from("R00").into(),
									Var::from("R01").into(),
									Var::from("R02").into(),
								],
							),
							Expr::Call(
								Box::new(Var::from("Vector3").nindex("new")),
								None,
								vec![
									Var::from("R10").into(),
									Var::from("R11").into(),
									Var::from("R12").into(),
								],
							),
						],
					),
				});

				self.push_local("result", Some(Expr::Num(-1.0)));

				self.push_stmt(Stmt::NumFor {
					var: "i",
					from: Expr::Num(1.0),
					to: Expr::Num(6.0),
				});

				self.push_local(
					"coords",
					Some(Expr::Call(
						Box::new(Var::from("table").nindex("pack")),
						None,
						vec![Expr::Num(0.0), Expr::Num(0.0), Expr::Num(0.0)],
					)),
				);

				self.push_stmt(Stmt::If(Expr::Gt(Box::new("i".into()), Box::new(Expr::Num(3.0)))));

				self.push_assign(
					Var::from("coords").eindex(Expr::Mod(Box::new("i".into()), Box::new(Expr::Num(3.0)))),
					Expr::Num(-1.0),
				);

				self.push_stmt(Stmt::Else);

				self.push_assign(
					Var::from("coords").eindex(Expr::Mod(Box::new("i".into()), Box::new(Expr::Num(3.0)))),
					Expr::Num(1.0),
				);

				self.push_stmt(Stmt::End);

				self.push_local(
					"vector",
					Some(Expr::Call(
						Box::new(Var::from("Vector3").nindex("new")),
						None,
						vec![Expr::Call(Box::new(Var::from("unpack")), None, vec!["coords".into()])],
					)),
				);

				self.push_stmt(Stmt::If(Expr::Eq(
					Box::new(Expr::Call(
						Box::new(Var::from("vector").nindex("Dot")),
						None,
						vec!["col".into()],
					)),
					Box::new(Expr::Num(1.0)),
				)));

				self.push_assign("result".into(), "i".into());
				self.push_stmt(Stmt::Break);

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::Call(
					Var::from("table").nindex("insert"),
					None,
					vec!["cols".into(), "result".into()],
				));

				self.push_stmt(Stmt::End);

				self.push_local(
					"unvalidated_orient_id",
					Some(Expr::Add(
						Box::new(Expr::Paren(Box::new(Expr::Mutiply(
							Box::new(Expr::Num(6.0)),
							Box::new(Var::from("cols").eindex(Expr::Num(0.0)).into()),
						)))),
						Box::new(Var::from("cols").eindex(Expr::Num(1.0)).into()),
					)),
				);

				self.push_stmt(Stmt::If(Expr::Eq(
					Box::new(Expr::Paren(Box::new(Expr::Mod(
						Box::new(Expr::Paren(Box::new(Expr::Div(
							Box::new("unvalidated_orient_id".into()),
							Box::new(Expr::Num(6.0)),
						)))),
						Box::new(Expr::Num(3.0)),
					)))),
					Box::new(Expr::Paren(Box::new(Expr::Mod(
						Box::new("unvalidated_orient_id".into()),
						Box::new(Expr::Num(3.0)),
					)))),
				)));

				self.push_assign("orient_id".into(), "unvalidated_orient_id".into());

				self.push_stmt(Stmt::Else);

				self.push_assign("orient_id".into(), Expr::Num(0.0));

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::End);

				self.push_writeu8("orient_id".into());

				self.push_stmt(Stmt::If(Expr::Eq(
					Box::new("orient_id".into()),
					Box::new(Expr::Num(0.0)),
				)));

				self.push_stmt(Stmt::GenFor {
					key: "_",
					val: "component",
					obj: Expr::Call(
						Box::new(Var::from("table").nindex("pack")),
						None,
						vec![Expr::Call(Box::new(from.clone()), Some("GetComponents".into()), vec![])],
					),
				});

				self.push_writef32("component".into());

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::Else);

				self.push_local("position", Some(from.clone().nindex("Position").into()));

				self.push_ty(&Ty::Vector3, "position".into());

				self.push_stmt(Stmt::End);
			}

			Ty::Boolean => self.push_writeu8(from_expr.and(1.0.into()).or(0.0.into())),
		}
	}
}

pub fn gen(ty: &Ty, var: &str, checks: bool) -> Vec<Stmt> {
	Ser { checks, buf: vec![] }.gen(var.into(), ty)
}
