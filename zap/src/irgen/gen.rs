use super::{Expr, Stmt, Var};
use crate::{
	parser::Ty,
	util::{NumTy, Range},
};

fn local(stmts: &mut Vec<Stmt>, name: &'static str, expr: Option<Expr>) {
	// local <name> = <expr>
	stmts.push(Stmt::Local(name, expr));
}

fn assign(stmts: &mut Vec<Stmt>, var: Var, expr: Expr) {
	// <var> = <expr>
	stmts.push(Stmt::Assign(var, expr));
}

fn assert(stmts: &mut Vec<Stmt>, cond: Expr, msg: Option<String>) {
	// assert(<cond>, <msg>)
	stmts.push(Stmt::Assert(cond, msg));
}

fn buffer_writef32(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(4)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![4.0.into()])));

	// buffer.writef32(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writef32"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writef64(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(8)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![8.0.into()])));

	// buffer.writef64(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writef64"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writeu8(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(1)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![1.0.into()])));

	// buffer.writeu8(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writeu8"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writeu16(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(2)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![2.0.into()])));

	// buffer.writeu16(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writeu16"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writeu32(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(4)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![4.0.into()])));

	// buffer.writeu32(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writeu32"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writei8(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(1)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![1.0.into()])));

	// buffer.writei8(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writei8"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writei16(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(2)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![2.0.into()])));

	// buffer.writei16(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writei16"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writei32(stmts: &mut Vec<Stmt>, val: Expr) {
	// local pos = alloc(4)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![4.0.into()])));

	// buffer.writei32(outgoing_buff, pos, <val>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writei32"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val],
	));
}

fn buffer_writenumty(stmts: &mut Vec<Stmt>, val: Expr, numty: NumTy) {
	match numty {
		NumTy::F32 => buffer_writef32(stmts, val),
		NumTy::F64 => buffer_writef64(stmts, val),

		NumTy::U8 => buffer_writeu8(stmts, val),
		NumTy::U16 => buffer_writeu16(stmts, val),
		NumTy::U32 => buffer_writeu32(stmts, val),

		NumTy::I8 => buffer_writei8(stmts, val),
		NumTy::I16 => buffer_writei16(stmts, val),
		NumTy::I32 => buffer_writei32(stmts, val),
	}
}

fn buffer_writestring(stmts: &mut Vec<Stmt>, val: Expr, count: Expr) {
	// local pos = alloc(<count>)
	local(stmts, "pos", Some(Var::from("alloc").call(vec![count.clone()])));

	// buffer.writestring(outgoing_buff, pos, <val>, <count>)
	stmts.push(Stmt::Call(
		Var::from("buffer").nindex("writestring"),
		None,
		vec!["outgoing_buff".into(), "pos".into(), val, count],
	));
}

fn buffer_readf32() -> Expr {
	// buffer.readf32(incoming_buff, read(4))
	Var::from("buffer")
		.nindex("readf32")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![4.0.into()])])
}

fn buffer_readf64() -> Expr {
	// buffer.readf64(incoming_buff, read(8))
	Var::from("buffer")
		.nindex("readf64")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![8.0.into()])])
}

fn buffer_readu8() -> Expr {
	// buffer.readu8(incoming_buff, read(1))
	Var::from("buffer")
		.nindex("readu8")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![1.0.into()])])
}

fn buffer_readu16() -> Expr {
	// buffer.readu16(incoming_buff, read(2))
	Var::from("buffer")
		.nindex("readu16")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![2.0.into()])])
}

fn buffer_readu32() -> Expr {
	// buffer.readu32(incoming_buff, read(4))
	Var::from("buffer")
		.nindex("readu32")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![4.0.into()])])
}

fn buffer_readi8() -> Expr {
	// buffer.readi8(incoming_buff, read(1))
	Var::from("buffer")
		.nindex("readi8")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![1.0.into()])])
}

fn buffer_readi16() -> Expr {
	// buffer.readi16(incoming_buff, read(2))
	Var::from("buffer")
		.nindex("readi16")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![2.0.into()])])
}

fn buffer_readi32() -> Expr {
	// buffer.readi32(incoming_buff, read(4))
	Var::from("buffer")
		.nindex("readi32")
		.call(vec!["incoming_buff".into(), Var::from("read").call(vec![4.0.into()])])
}

fn buffer_readnumty(numty: NumTy) -> Expr {
	match numty {
		NumTy::F32 => buffer_readf32(),
		NumTy::F64 => buffer_readf64(),

		NumTy::U8 => buffer_readu8(),
		NumTy::U16 => buffer_readu16(),
		NumTy::U32 => buffer_readu32(),

		NumTy::I8 => buffer_readi8(),
		NumTy::I16 => buffer_readi16(),
		NumTy::I32 => buffer_readi32(),
	}
}

fn buffer_readstring(count: Expr) -> Expr {
	// buffer.readstring(incoming_buff, read(<count>), <count>)
	Var::from("buffer").nindex("readstring").call(vec![
		"incoming_buff".into(),
		Var::from("read").call(vec![count.clone()]),
		count,
	])
}

fn range_check(stmts: &mut Vec<Stmt>, val: Expr, range: Range<f64>) {
	if let Some(min) = range.min() {
		assert(stmts, val.clone().gte(min.into()), None);
	}

	if let Some(max) = range.max() {
		assert(
			stmts,
			if range.max_inclusive() {
				val.lte(max.into())
			} else {
				val.lt(max.into())
			},
			None,
		);
	}
}

pub fn gen_ser(ty: &Ty, from: Var, gen_checks: bool) -> Vec<Stmt> {
	let mut stmts = Vec::new();
	let from_expr = Expr::Var(Box::new(from.clone()));

	if gen_checks
		&& matches!(
			ty,
			Ty::F32(..) | Ty::F64(..) | Ty::U8(..) | Ty::U16(..) | Ty::U32(..) | Ty::I8(..) | Ty::I16(..) | Ty::I32(..)
		) {
		range_check(
			&mut stmts,
			from_expr.clone(),
			match ty {
				Ty::F32(range) => range.cast(),
				Ty::F64(range) => range.cast(),

				Ty::U8(range) => range.cast(),
				Ty::U16(range) => range.cast(),
				Ty::U32(range) => range.cast(),

				Ty::I8(range) => range.cast(),
				Ty::I16(range) => range.cast(),
				Ty::I32(range) => range.cast(),

				_ => unreachable!(),
			},
		);
	}

	match ty {
		Ty::Bool => buffer_writeu8(&mut stmts, from_expr.and(1.0.into()).or(0.0.into())),

		Ty::F32(..) => buffer_writef32(&mut stmts, from_expr),
		Ty::F64(..) => buffer_writef64(&mut stmts, from_expr),

		Ty::U8(..) => buffer_writeu8(&mut stmts, from_expr),
		Ty::U16(..) => buffer_writeu16(&mut stmts, from_expr),
		Ty::U32(..) => buffer_writeu32(&mut stmts, from_expr),

		Ty::I8(..) => buffer_writei8(&mut stmts, from_expr),
		Ty::I16(..) => buffer_writei16(&mut stmts, from_expr),
		Ty::I32(..) => buffer_writei32(&mut stmts, from_expr),

		Ty::Str { len } => {
			if let Some(len) = len.exact_f64() {
				if gen_checks {
					assert(&mut stmts, from_expr.clone().len().eq(len.into()), None);
				}

				buffer_writestring(&mut stmts, from_expr, len.into())
			} else {
				local(&mut stmts, "len", Some(from_expr.clone().len()));

				if gen_checks {
					range_check(&mut stmts, "len".into(), len.cast());
				}

				buffer_writeu16(&mut stmts, "len".into());
				buffer_writestring(&mut stmts, from_expr, "len".into())
			}
		}

		Ty::Arr { ty, len } => {
			if let Some(len) = len.exact_f64() {
				if gen_checks {
					assert(&mut stmts, from_expr.clone().len().eq(len.into()), None);
				}

				stmts.push(Stmt::NumFor {
					var: "i",
					from: 1.0.into(),
					to: len.into(),
				});

				stmts.extend(gen_ser(ty, from.eindex("i".into()), gen_checks));
				stmts.push(Stmt::End);
			} else {
				local(&mut stmts, "len", Some(from_expr.clone().len()));

				if gen_checks {
					range_check(&mut stmts, "len".into(), len.cast());
				}

				buffer_writeu16(&mut stmts, "len".into());

				stmts.push(Stmt::NumFor {
					var: "i",
					from: 1.0.into(),
					to: "len".into(),
				});

				stmts.extend(gen_ser(ty, from.eindex("i".into()), gen_checks));
				stmts.push(Stmt::End);
			}
		}

		Ty::Map { key, val } => {
			local(&mut stmts, "len_pos", Some(Var::from("alloc").call(vec![2.0.into()])));
			local(&mut stmts, "len", Some(0.0.into()));

			stmts.push(Stmt::GenFor {
				key: "k",
				val: "v",
				obj: from_expr,
			});

			assign(&mut stmts, "len".into(), Expr::from("len").add(1.0.into()));
			stmts.extend(gen_ser(key, "k".into(), gen_checks));
			stmts.extend(gen_ser(val, "v".into(), gen_checks));

			stmts.push(Stmt::End);

			stmts.push(Stmt::Call(
				Var::from("buffer").nindex("writeu16"),
				None,
				vec!["outgoing_buff".into(), "len_pos".into(), "len".into()],
			));
		}

		Ty::Struct { fields } => {
			for (name, ty) in fields {
				stmts.extend(gen_ser(ty, from.clone().nindex(name), gen_checks));
			}
		}

		Ty::Enum { variants } => {
			let numty = NumTy::from_f64(0.0, variants.len() as f64 - 1.0);

			for (i, name) in variants.iter().enumerate() {
				if i == 0 {
					stmts.push(Stmt::If(from_expr.clone().eq(name.clone().into())));
				} else {
					stmts.push(Stmt::ElseIf(from_expr.clone().eq(name.clone().into())));
				}

				buffer_writenumty(&mut stmts, (i as f64).into(), numty);
			}

			stmts.push(Stmt::Else);
			stmts.push(Stmt::Error("invalid enum variant!".to_string()));
			stmts.push(Stmt::End);
		}

		Ty::Instance(class) => {
			if gen_checks && class.is_some() {
				assert(
					&mut stmts,
					Expr::Call(Box::new(from), Some("IsA".into()), vec![class.clone().unwrap().into()]),
					None,
				);
			}

			buffer_writeu16(&mut stmts, Var::from("alloc_inst").call(vec![from_expr]))
		}

		Ty::Vector3 => {
			buffer_writef32(&mut stmts, from.clone().nindex("X").into());
			buffer_writef32(&mut stmts, from.clone().nindex("Y").into());
			buffer_writef32(&mut stmts, from.clone().nindex("Z").into());
		}

		Ty::Ref(name) => stmts.push(Stmt::Call(
			Var::from("types").nindex(format!("write_{name}")),
			None,
			vec![from_expr],
		)),

		Ty::Optional(ty) => {
			stmts.push(Stmt::If(from_expr.clone().eq(Expr::Nil)));

			buffer_writeu8(&mut stmts, 0.0.into());

			stmts.push(Stmt::Else);

			buffer_writeu8(&mut stmts, 1.0.into());
			stmts.extend(gen_ser(ty, from, gen_checks));

			stmts.push(Stmt::End);
		}
	}

	stmts
}

pub fn gen_des(ty: &Ty, to: Var, gen_checks: bool) -> Vec<Stmt> {
	let mut stmts = Vec::new();

	match ty {
		Ty::Bool => assign(&mut stmts, to.clone(), buffer_readu8().neq(0.0.into())),

		Ty::F32(..) => assign(&mut stmts, to.clone(), buffer_readf32()),
		Ty::F64(..) => assign(&mut stmts, to.clone(), buffer_readf64()),

		Ty::U8(..) => assign(&mut stmts, to.clone(), buffer_readu8()),
		Ty::U16(..) => assign(&mut stmts, to.clone(), buffer_readu16()),
		Ty::U32(..) => assign(&mut stmts, to.clone(), buffer_readu32()),

		Ty::I8(..) => assign(&mut stmts, to.clone(), buffer_readi8()),
		Ty::I16(..) => assign(&mut stmts, to.clone(), buffer_readi16()),
		Ty::I32(..) => assign(&mut stmts, to.clone(), buffer_readi32()),

		Ty::Str { len } => {
			if let Some(len) = len.exact_f64() {
				assign(&mut stmts, to.clone(), buffer_readstring(len.into()));
			} else {
				local(&mut stmts, "len", Some(buffer_readu16()));

				if gen_checks {
					range_check(&mut stmts, "len".into(), len.cast());
				}

				assign(&mut stmts, to.clone(), buffer_readstring("len".into()));
			}
		}

		Ty::Arr { len, ty } => {
			assign(&mut stmts, to.clone(), Expr::EmptyTab);

			if let Some(len) = len.exact_f64() {
				stmts.push(Stmt::NumFor {
					var: "i",
					from: 1.0.into(),
					to: len.into(),
				});

				stmts.extend(gen_des(ty, to.clone().eindex("i".into()), gen_checks));
				stmts.push(Stmt::End);
			} else {
				local(&mut stmts, "len", Some(buffer_readu16()));

				if gen_checks {
					range_check(&mut stmts, "len".into(), len.cast());
				}

				assign(&mut stmts, to.clone(), Var::from("alloc").call(vec!["len".into()]));

				stmts.push(Stmt::NumFor {
					var: "i",
					from: 1.0.into(),
					to: "len".into(),
				});

				stmts.extend(gen_des(ty, to.clone().eindex("i".into()), gen_checks));
				stmts.push(Stmt::End);
			}
		}

		Ty::Map { key, val } => {
			assign(&mut stmts, to.clone(), Expr::EmptyTab);

			stmts.push(Stmt::NumFor {
				var: "_",
				from: 0.0.into(),
				to: buffer_readu16(),
			});

			local(&mut stmts, "k", None);
			local(&mut stmts, "v", None);

			stmts.extend(gen_des(key, "k".into(), gen_checks));
			stmts.extend(gen_des(val, "v".into(), gen_checks));

			assign(&mut stmts, to.clone().eindex("k".into()), "v".into());

			stmts.push(Stmt::End);
		}

		Ty::Struct { fields } => {
			assign(&mut stmts, to.clone(), Expr::EmptyTab);

			for (name, ty) in fields {
				stmts.extend(gen_des(ty, to.clone().nindex(name), gen_checks));
			}
		}

		Ty::Enum { variants } => {
			let numty = NumTy::from_f64(0.0, variants.len() as f64 - 1.0);

			assign(&mut stmts, to.clone(), buffer_readnumty(numty));

			for (i, name) in variants.iter().enumerate() {
				if i == 0 {
					stmts.push(Stmt::If(Expr::from(to.clone()).eq((i as f64).into())));
				} else {
					stmts.push(Stmt::ElseIf(Expr::from(to.clone()).eq((i as f64).into())));
				}

				assign(&mut stmts, to.clone(), name.clone().into());
			}

			stmts.push(Stmt::Else);
			stmts.push(Stmt::Error("invalid enum variant!".to_string()));
			stmts.push(Stmt::End);
		}

		Ty::Instance(class) => {
			assign(
				&mut stmts,
				to.clone(),
				Var::from("incoming_inst").eindex(buffer_readu16()).into(),
			);

			// Assert that the instance is not nil even if we don't want checks
			// because roblox cannot ensure the instance's existance at the destination
			assert(&mut stmts, Expr::from(to.clone()).neq(Expr::Nil), None);

			if gen_checks && class.is_some() {
				assert(
					&mut stmts,
					Expr::Call(
						to.clone().into(),
						Some("IsA".into()),
						vec![class.clone().unwrap().into()],
					),
					None,
				);
			}
		}

		Ty::Vector3 => {
			local(&mut stmts, "X", Some(buffer_readf32()));
			local(&mut stmts, "Y", Some(buffer_readf32()));
			local(&mut stmts, "Z", Some(buffer_readf32()));

			assign(
				&mut stmts,
				to.clone(),
				Expr::Vector3(Box::new("X".into()), Box::new("Y".into()), Box::new("Z".into())),
			);
		}

		Ty::Ref(name) => assign(
			&mut stmts,
			to.clone(),
			Var::from("types").nindex(format!("read_{name}")).into(),
		),

		Ty::Optional(ty) => {
			stmts.push(Stmt::If(buffer_readu8().neq(0.0.into())));

			stmts.extend(gen_des(ty, to.clone(), gen_checks));

			stmts.push(Stmt::Else);
			assign(&mut stmts, to.clone(), Expr::Nil);
			stmts.push(Stmt::End);
		}
	}

	if gen_checks
		&& matches!(
			ty,
			Ty::F32(..) | Ty::F64(..) | Ty::U8(..) | Ty::U16(..) | Ty::U32(..) | Ty::I8(..) | Ty::I16(..) | Ty::I32(..)
		) {
		range_check(
			&mut stmts,
			to.into(),
			match ty {
				Ty::F32(range) => range.cast(),
				Ty::F64(range) => range.cast(),

				Ty::U8(range) => range.cast(),
				Ty::U16(range) => range.cast(),
				Ty::U32(range) => range.cast(),

				Ty::I8(range) => range.cast(),
				Ty::I16(range) => range.cast(),
				Ty::I32(range) => range.cast(),

				_ => unreachable!(),
			},
		);
	}

	stmts
}
