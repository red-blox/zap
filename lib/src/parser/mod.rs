use lalrpop_util::lalrpop_mod;

use crate::util::Range;

mod working_ast;

lalrpop_mod!(pub grammar);

#[derive(Debug)]
pub struct File {
	pub ty_decls: Vec<TyDecl>,
	pub ev_decls: Vec<EvDecl>,

	pub lang: Lang,
	pub casing: Casing,

	pub write_checks: bool,
}

#[derive(Debug, Clone)]
pub enum Lang {
	Luau,
	Typescript,
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

#[derive(Debug, Clone, Copy)]
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

	Ref(String),

	Optional(Box<Ty>),
}

pub fn parse(code: &str) -> Result<File, String> {
	match grammar::FileParser::new().parse(code) {
		Ok(file) => Ok(file),
		Err(e) => Err(e.to_string()),
	}
}
