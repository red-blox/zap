use std::{collections::HashSet, fmt::Display};

#[derive(Debug, Clone)]
pub struct Config<'src> {
	pub tydecls: Vec<TyDecl<'src>>,
	pub evdecls: Vec<EvDecl<'src>>,

	pub write_checks: bool,
	pub typescript: bool,

	pub server_output: Option<&'src str>,
	pub client_output: Option<&'src str>,

	pub casing: Casing,
}

impl<'src> Config<'src> {
	pub fn event_id_ty(&self) -> NumTy {
		NumTy::from_f64(1.0, self.evdecls.len() as f64)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Casing {
	Pascal,
	Camel,
	Snake,
}

impl Casing {
	pub fn with(&self, pascal: &'static str, camel: &'static str, snake: &'static str) -> &'static str {
		match self {
			Self::Pascal => pascal,
			Self::Camel => camel,
			Self::Snake => snake,
		}
	}
}

#[derive(Debug, Clone)]
pub struct EvDecl<'src> {
	pub name: &'src str,
	pub from: EvSource,
	pub evty: EvType,
	pub call: EvCall,
	pub data: Ty<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvSource {
	Server,
	Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvType {
	Reliable,
	Unreliable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvCall {
	SingleSync,
	SingleAsync,
	ManySync,
	ManyAsync,
}

#[derive(Debug, Clone)]
pub struct TyDecl<'src> {
	pub name: &'src str,
	pub ty: Ty<'src>,
}

#[derive(Debug, Clone)]
pub enum Ty<'src> {
	Num(NumTy, Range),
	Str(Range),
	Buf(Range),
	Arr(Box<Ty<'src>>, Range),
	Map(Box<Ty<'src>>, Box<Ty<'src>>),
	Opt(Box<Ty<'src>>),
	Ref(&'src str),

	Enum(Enum<'src>),
	Struct(Struct<'src>),
	Instance(Option<&'src str>),

	Vector3,
	Boolean,
	Unknown,
}

impl<'src> Ty<'src> {
	pub fn max_size(&self, config: &Config<'src>, recursed: &mut HashSet<&'src str>) -> Option<usize> {
		match self {
			Self::Num(numty, _) => Some(numty.size()),
			Self::Vector3 => Some(NumTy::F32.size() * 3),
			Self::Boolean => Some(1),
			Self::Opt(ty) => ty.max_size(config, recursed).map(|size| size + 1),
			Self::Str(len) => len.max().map(|len| len as usize),
			Self::Buf(len) => len.max().map(|len| len as usize),
			Self::Arr(ty, range) => range
				.max()
				.and_then(|len| ty.max_size(config, recursed).map(|size| size * len as usize)),
			Self::Map(..) => None,
			Self::Enum(enum_ty) => enum_ty.max_size(config, recursed),
			Self::Struct(struct_ty) => struct_ty.max_size(config, recursed),
			Self::Instance(_) => Some(2),
			Self::Unknown => None,
			Self::Ref(name) => {
				if recursed.contains(name) {
					None
				} else {
					recursed.insert(name);
					config
						.tydecls
						.iter()
						.find(|tydecl| tydecl.name == *name)
						.and_then(|tydecl| tydecl.ty.max_size(config, recursed))
				}
			}
		}
	}
}

#[derive(Debug, Clone)]
pub enum Enum<'src> {
	Unit(Vec<&'src str>),

	Tagged {
		tag: &'src str,
		variants: Vec<(&'src str, Struct<'src>)>,
	},
}

impl<'src> Enum<'src> {
	pub fn max_size(&self, config: &Config<'src>, recursed: &mut HashSet<&'src str>) -> Option<usize> {
		match self {
			Self::Unit(vec) => Some(NumTy::from_f64(0.0, vec.len() as f64).size()),

			Self::Tagged { variants, .. } => {
				let mut size = NumTy::from_f64(0.0, variants.len() as f64).size();

				for (_, ty) in variants {
					if let Some(ty_size) = ty.max_size(config, recursed) {
						size += ty_size;
					} else {
						return None;
					}
				}

				Some(size)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct Struct<'src> {
	pub fields: Vec<(&'src str, Ty<'src>)>,
}

impl<'src> Struct<'src> {
	pub fn max_size(&self, config: &Config<'src>, recursed: &mut HashSet<&'src str>) -> Option<usize> {
		let mut size = 0;

		for (_, ty) in &self.fields {
			if let Some(ty_size) = ty.max_size(config, recursed) {
				size += ty_size;
			} else {
				return None;
			}
		}

		Some(size)
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Range {
	min: Option<f64>,
	max: Option<f64>,
}

impl Range {
	pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
		Self { min, max }
	}

	pub fn min(&self) -> Option<f64> {
		self.min
	}

	pub fn max(&self) -> Option<f64> {
		self.max
	}

	pub fn exact(&self) -> Option<f64> {
		if self.min.is_some() && self.min == self.max {
			Some(self.min.unwrap())
		} else {
			None
		}
	}
}

impl Display for Range {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (self.min, self.max) {
			(Some(min), Some(max)) => write!(f, "{}..{}", min, max),
			(Some(min), None) => write!(f, "{}..", min),
			(None, Some(max)) => write!(f, "..{}", max),
			(None, None) => write!(f, ".."),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumTy {
	F32,
	F64,

	U8,
	U16,
	U32,

	I8,
	I16,
	I32,
}

impl NumTy {
	pub fn from_f64(min: f64, max: f64) -> NumTy {
		if min < 0.0 {
			if max < 0.0 {
				NumTy::I32
			} else if max <= u8::MAX as f64 {
				NumTy::I8
			} else if max <= u16::MAX as f64 {
				NumTy::I16
			} else {
				NumTy::I32
			}
		} else if max <= u8::MAX as f64 {
			NumTy::U8
		} else if max <= u16::MAX as f64 {
			NumTy::U16
		} else if max <= u32::MAX as f64 {
			NumTy::U32
		} else {
			NumTy::F64
		}
	}

	pub fn size(&self) -> usize {
		match self {
			NumTy::F32 => 4,
			NumTy::F64 => 8,

			NumTy::U8 => 1,
			NumTy::U16 => 2,
			NumTy::U32 => 4,

			NumTy::I8 => 1,
			NumTy::I16 => 2,
			NumTy::I32 => 4,
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
