use std::fmt::Display;

use crate::{
	irgen::{gen_ser, Expr, Stmt, Var},
	parser::{Ty, TyDecl},
	util::NumTy,
};

mod client;
mod server;

impl Display for TyDecl {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let name = &self.name;
		let ty = &self.ty;

		writeln!(f, "export type {name} = {ty}")?;

		writeln!(f, "function types.write_{name}(value: {name})")?;

		for stmt in gen_ser(ty, "value", false) {
			writeln!(f, "\t{stmt}")?;
		}

		writeln!(f, "end;")?;

		writeln!(f, "function types.read_{name}()")?;
		writeln!(f, "\tlocal value;")?;

		for stmt in gen_ser(&ty, "value", true) {
			writeln!(f, "\t{stmt}")?;
		}

		writeln!(f, "\treturn value;")?;
		writeln!(f, "end;")?;

		Ok(())
	}
}

impl Display for Ty {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Ty::Bool => write!(f, "boolean"),

			Ty::F32(_) => write!(f, "number"),
			Ty::F64(_) => write!(f, "number"),

			Ty::U8(_) => write!(f, "number"),
			Ty::U16(_) => write!(f, "number"),
			Ty::U32(_) => write!(f, "number"),

			Ty::I8(_) => write!(f, "number"),
			Ty::I16(_) => write!(f, "number"),
			Ty::I32(_) => write!(f, "number"),

			Ty::Str { .. } => write!(f, "string"),
			Ty::Arr { ty, .. } => write!(f, "{{ {ty} }}"),
			Ty::Map { key, val } => write!(f, "{{ [{key}]: {val} }}"),

			Ty::Struct { fields } => {
				write!(f, "{{ ")?;

				for (name, ty) in fields.iter() {
					write!(f, "{name}: {ty}, ")?;
				}

				write!(f, "}}")
			}

			Ty::Enum { variants } => write!(
				f,
				"{}",
				variants
					.iter()
					.map(|v| format!("\"{}\"", v))
					.collect::<Vec<_>>()
					.join(" | ")
			),

			Ty::Ref(name) => write!(f, "{name}"),

			Ty::Optional(ty) => write!(f, "{ty}?"),
		}
	}
}

impl Display for Stmt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Stmt::Local { name } => write!(f, "local {name}"),
			Stmt::Assign { var, val } => write!(f, "{var} = {val}"),
			Stmt::Throw { msg } => write!(f, "error(\"{msg}\")"),
			Stmt::Assert { cond, msg } => write!(f, "assert({cond}, \"{msg}\")"),

			Stmt::BlockStart => write!(f, "do"),
			Stmt::NumFor { var, start, end } => write!(f, "for {var} = {start}, {end} do"),
			Stmt::GenFor { key, val, expr } => write!(f, "for {key}, {val} in {expr} do"),
			Stmt::If { cond } => write!(f, "if {cond} then"),
			Stmt::ElseIf { cond } => write!(f, "elseif {cond} then"),
			Stmt::Else => write!(f, "else"),

			Stmt::BlockEnd => write!(f, "end"),

			Stmt::Alloc { into, len } => write!(f, "{into} = alloc({len})"),

			Stmt::WriteNum { expr, ty, at } => match at {
				Some(at) => write!(f, "buffer.write{ty}(outgoing_buff, {expr}, {at})"),
				None => write!(f, "buffer.write{ty}(outgoing_buff, {expr}, alloc({}))", ty.size()),
			},

			Stmt::WriteStr { expr, len } => {
				write!(f, "buffer.writestring(outgoing_buff, alloc({len}), {expr}, {len})")
			}

			Stmt::WriteRef { expr, ref_name } => write!(f, "buffer.write_{ref_name}({expr})"),

			Stmt::ReadNum { into, ty } => write!(f, "{into} = buffer.read{ty}(incoming_buff, read({}))", ty.size()),
			Stmt::ReadStr { into, len } => write!(f, "{into} = buffer.readstring(incoming_buff, read({len}), {len})"),
			Stmt::ReadRef { into, ref_name } => write!(f, "{into} = buffer.read_{ref_name}()"),
		}
	}
}

impl Display for NumTy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			NumTy::F32 => write!(f, "f32"),
			NumTy::F64 => write!(f, "f64"),

			NumTy::U8 => write!(f, "u8"),
			NumTy::U16 => write!(f, "u16"),
			NumTy::U32 => write!(f, "u32"),

			NumTy::I8 => write!(f, "i8"),
			NumTy::I16 => write!(f, "i16"),
			NumTy::I32 => write!(f, "i32"),
		}
	}
}

impl Display for Var {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Var::Name(name) => write!(f, "{}", name),

			Var::NameIndex(prefix, suffix) => write!(f, "{}.{}", prefix, suffix),
			Var::ExprIndex(prefix, expr) => write!(f, "{}[{}]", prefix, expr),
		}
	}
}

impl Display for Expr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Expr::False => write!(f, "false"),
			Expr::True => write!(f, "true"),
			Expr::Nil => write!(f, "nil"),

			Expr::Num(num) => write!(f, "{num}"),
			Expr::Str(string) => write!(f, "\"{string}\""),
			Expr::Var(var) => write!(f, "{var}"),

			Expr::EmptyArr => write!(f, "{{}}"),
			Expr::EmptyObj => write!(f, "{{}}"),

			Expr::Len(expr) => write!(f, "#{expr}"),

			Expr::Lt(left, right) => write!(f, "{left} < {right}"),
			Expr::Gt(left, right) => write!(f, "{left} > {right}"),
			Expr::Le(left, right) => write!(f, "{left} <= {right}"),
			Expr::Ge(left, right) => write!(f, "{left} >= {right}"),
			Expr::Eq(left, right) => write!(f, "{left} == {right}"),
			Expr::Add(left, right) => write!(f, "{left} + {right}"),
		}
	}
}
