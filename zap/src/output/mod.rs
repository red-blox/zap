use std::fmt::Display;

use crate::{parser::Ty, util::NumTy};

pub mod client;
pub mod server;

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

			Ty::Instance(name) => write!(f, "{}", if let Some(name) = name { name } else { "Instance" }),
			Ty::Vector3 => write!(f, "Vector3"),

			Ty::Ref(name) => write!(f, "{name}"),

			Ty::Optional(ty) => write!(f, "{ty}?"),
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
