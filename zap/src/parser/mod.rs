use std::collections::HashSet;

use lalrpop_util::lalrpop_mod;

use crate::{
	util::{NumTy, Range},
	Error,
};

mod working_ast;

lalrpop_mod!(pub grammar);

#[derive(Debug)]
pub struct File {
	pub ty_decls: Vec<TyDecl>,
	pub ev_decls: Vec<EvDecl>,

	pub casing: Casing,
	pub write_checks: bool,
}

impl File {
	pub fn event_id_ty(&self) -> NumTy {
		NumTy::from_f64(0.0, self.ev_decls.len() as f64)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Casing {
	Pascal,
	Camel,
	Snake,
}

#[derive(Debug, Clone)]
pub struct EvDecl {
	pub name: String,
	pub from: EvSource,
	pub evty: EvType,
	pub call: EvCall,
	pub data: Ty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvSource {
	Server,
	Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvType {
	Reliable,
	Unreliable,
}

#[derive(Debug, Clone, Copy)]
pub enum EvCall {
	SingleSync,
	SingleAsync,
	ManySync,
	ManyAsync,
}

#[derive(Debug, Clone)]
pub struct TyDecl {
	pub name: String,
	pub ty: Ty,
}

#[derive(Debug, Clone)]
pub enum Ty {
	Bool,

	F32(Range<f32>),
	F64(Range<f64>),

	I8(Range<i8>),
	I16(Range<i16>),
	I32(Range<i32>),

	U8(Range<u8>),
	U16(Range<u16>),
	U32(Range<u32>),

	Str { len: Range<u16> },
	Arr { len: Range<u16>, ty: Box<Ty> },
	Map { key: Box<Ty>, val: Box<Ty> },

	Struct { fields: Vec<(String, Ty)> },
	Enum { variants: Vec<String> },

	Instance(Option<String>),
	Vector3,

	Ref(String),

	Optional(Box<Ty>),
}

impl Ty {
	pub fn exact_size(&self) -> Option<usize> {
		match self {
			Ty::Bool => Some(1),

			Ty::F32(_) => Some(4),
			Ty::F64(_) => Some(8),

			Ty::I8(_) => Some(1),
			Ty::I16(_) => Some(2),
			Ty::I32(_) => Some(4),

			Ty::U8(_) => Some(1),
			Ty::U16(_) => Some(2),
			Ty::U32(_) => Some(4),

			Ty::Str { len } => {
				if len.is_exact() {
					Some(len.min().unwrap() as usize)
				} else {
					None
				}
			}

			Ty::Arr { len, ty } => {
				if len.is_exact() && ty.exact_size().is_some() {
					Some(len.min().unwrap() as usize * ty.exact_size().unwrap())
				} else {
					None
				}
			}

			Ty::Map { .. } => None,

			Ty::Struct { fields } => {
				let mut size = 0;

				for (_, ty) in fields {
					if let Some(ty_size) = ty.exact_size() {
						size += ty_size;
					} else {
						return None;
					}
				}

				Some(size)
			}

			Ty::Enum { variants } => Some(NumTy::from_f64(0.0, variants.len() as f64).size()),

			Ty::Instance(_) => Some(2),
			Ty::Vector3 => Some(12),

			// At some point this should evaluate the size of the referenced type
			// for now the extra complexity isn't worth it
			Ty::Ref(_) => None,

			Ty::Optional(ty) => ty.exact_size().map(|size| size + 1),
		}
	}
}

pub fn parse(code: &str) -> Result<File, Error> {
	let mut ref_decl = HashSet::new();
	let mut ref_used = HashSet::new();

	let file = grammar::FileParser::new()
		.parse(&mut ref_decl, &mut ref_used, &mut HashSet::new(), code)
		.map_err(|e| Error::ParseError(e.to_string()))?;

	let unknown_refs = ref_used.difference(&ref_decl).collect::<Vec<_>>();

	// TODO: Better error reporting with error location
	if !unknown_refs.is_empty() {
		return Err(Error::UnknownTypeRef(unknown_refs[0].to_owned()));
	}

	Ok(file)
}
