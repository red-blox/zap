use std::path::PathBuf;

use crate::util::{NumTy, Range};

mod ast;
mod lex;

#[derive(Debug, Clone)]
pub struct Config {
	pub tydecls: Vec<TyDecl>,
	pub evdecls: Vec<EvDecl>,

	pub server_output: PathBuf,
	pub client_output: PathBuf,

	pub casing: Casing,
	pub typescript: bool,
	pub write_checks: bool,
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

#[derive(Debug, Clone, Copy)]
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
pub struct TyDecl(pub String, pub Ty);

#[derive(Debug, Clone)]
pub enum Ty {
	Boolean,

	Num { ty: NumTy, range: Range },
	Str { len: Range },
	Arr { ty: Box<Ty>, len: Range },
	Map { key: Box<Ty>, val: Box<Ty> },

	Struct(Struct),
	Enum(Enum),

	Instance { strict: bool, class: Option<String> },
	Vector3,

	Ref { ty: Box<Ty>, name: String },
	Opt { ty: Box<Ty> },
}

#[derive(Debug, Clone)]
pub enum Enum {
	Unit(Vec<String>),

	Tagged {
		tag: String,
		variants: Vec<(String, Struct)>,
	},
}

#[derive(Debug, Clone)]
pub struct Struct {
	fields: Vec<(String, Ty)>,
}
