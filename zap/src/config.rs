use std::{cell::RefCell, collections::HashSet, fmt::Display, rc::Rc};

pub const UNRELIABLE_ORDER_NUMTY: NumTy = NumTy::U16;

#[derive(Debug, Clone)]
pub struct Config<'src> {
	pub tydecls: Vec<TyDecl<'src>>,
	pub evdecls: Vec<EvDecl<'src>>,
	pub fndecls: Vec<FnDecl<'src>>,

	pub typescript: bool,
	pub typescript_max_tuple_length: f64,

	pub tooling: bool,
	pub tooling_show_internal_data: bool,

	pub write_checks: bool,
	pub manual_event_loop: bool,

	pub remote_scope: &'src str,
	pub remote_folder: &'src str,

	pub server_output: &'src str,
	pub client_output: &'src str,
	pub types_output: Option<&'src str>,
	pub tooling_output: &'src str,

	pub casing: Casing,
	pub call_default: Option<EvCall>,
	pub yield_type: YieldType,
	pub async_lib: &'src str,
	pub disable_fire_all: bool,
}

impl Config<'_> {
	pub fn server_reliable_count(&self) -> usize {
		let reliable_count = self
			.evdecls
			.iter()
			.filter(|evdecl| evdecl.from == EvSource::Client && evdecl.evty == EvType::Reliable)
			.count();

		reliable_count + self.fndecls.len()
	}

	pub fn server_unreliable_count(&self) -> usize {
		self.evdecls
			.iter()
			.filter(|evdecl| evdecl.from == EvSource::Client && matches!(evdecl.evty, EvType::Unreliable(_)))
			.count()
	}

	pub fn client_reliable_count(&self) -> usize {
		let reliable_count = self
			.evdecls
			.iter()
			.filter(|evdecl| evdecl.from == EvSource::Server && evdecl.evty == EvType::Reliable)
			.count();

		reliable_count + self.fndecls.len()
	}

	pub fn client_unreliable_count(&self) -> usize {
		self.evdecls
			.iter()
			.filter(|evdecl| evdecl.from == EvSource::Server && matches!(evdecl.evty, EvType::Unreliable(_)))
			.count()
	}

	pub fn server_reliable_ty(&self) -> NumTy {
		NumTy::from_f64(0.0, self.server_reliable_count() as f64 - 1.0)
	}

	pub fn client_reliable_ty(&self) -> NumTy {
		NumTy::from_f64(0.0, self.client_reliable_count() as f64 - 1.0)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldType {
	Yield,
	Future,
	Promise,
}

impl std::fmt::Display for YieldType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			// do nothing, as yield will never import
			YieldType::Yield => Ok(()),
			YieldType::Future => write!(f, "Future"),
			YieldType::Promise => write!(f, "Promise"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct FnDecl<'src> {
	pub name: &'src str,
	pub call: FnCall,
	pub args: Vec<Parameter<'src>>,
	pub rets: Option<Vec<Ty<'src>>>,
	pub client_id: usize,
	pub server_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnCall {
	Async,
	Sync,
}

#[derive(Debug, Clone)]
pub struct EvDecl<'src> {
	pub name: &'src str,
	pub from: EvSource,
	pub evty: EvType,
	pub call: EvCall,
	pub data: Vec<Parameter<'src>>,
	pub id: usize,
}

#[derive(Debug, Clone)]
pub struct Parameter<'src> {
	pub name: Option<&'src str>,
	pub ty: Ty<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvSource {
	Server,
	Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvType {
	Reliable,
	Unreliable(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvCall {
	SingleSync,
	SingleAsync,
	ManySync,
	ManyAsync,
	Polling,
}

#[derive(Debug, Clone)]
pub struct TyDecl<'src> {
	pub name: &'src str,
	pub ty: Rc<RefCell<Ty<'src>>>,
}

#[derive(Debug, Clone)]
pub enum Ty<'src> {
	Num(NumTy, Range),
	Str(Range),
	Buf(Range),
	Vector(Box<Ty<'src>>, Box<Ty<'src>>, Option<Box<Ty<'src>>>),
	Arr(Box<Ty<'src>>, Range),
	Map(Box<Ty<'src>>, Box<Ty<'src>>),
	Set(Box<Ty<'src>>),
	Opt(Box<Ty<'src>>),
	Ref(&'src str, Rc<RefCell<Ty<'src>>>),

	Enum(Enum<'src>),
	Struct(Struct<'src>),
	Or(Vec<Ty<'src>>, NumTy),
	Instance(Option<&'src str>),

	BrickColor,
	DateTimeMillis,
	DateTime,
	Color3,
	Vector2,
	Vector3,
	AlignedCFrame,
	CFrame,
	Boolean,
	Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum NonPrimitiveTy {
	Opt,
	Or,
}

#[derive(Debug, Clone)]
pub enum PrimitiveTy<'src> {
	Name(&'static str),
	Instance(Option<&'src str>),
	Enum(Enum<'src>),
	Unknown,
	None(NonPrimitiveTy),
}

impl<'src> Ty<'src> {
	/// Returns the amount of data used by this type in bytes.
	///
	/// Note that this is not the same as the size of the type in the buffer.
	/// For example, an `Instance` will always send 4 bytes of data, but the
	/// size of the type in the buffer will be 0 bytes.
	pub fn size(&self, recursed: &mut HashSet<&'src str>) -> (usize, Option<usize>) {
		match self {
			Self::Num(numty, ..) => (numty.size(), Some(numty.size())),

			Self::Str(len) => {
				if let Some(exact) = len.exact() {
					(exact as usize, Some(exact as usize))
				} else {
					let (len_numty, ..) = len.numty();

					(
						len.min().map(|min| min as usize).unwrap_or(0) + len_numty.size(),
						len.max().map(|max| max as usize + len_numty.size()),
					)
				}
			}

			Self::Buf(len) => {
				if let Some(exact) = len.exact() {
					(exact as usize, Some(exact as usize))
				} else {
					let (len_numty, ..) = len.numty();

					(
						len.min().map(|min| min as usize).unwrap_or(0) + len_numty.size(),
						len.max().map(|max| max as usize + len_numty.size()),
					)
				}
			}

			Self::Arr(ty, len) => {
				let (ty_min, ty_max) = ty.size(recursed);
				let len_min = len.min().map(|min| min as usize).unwrap_or(0);
				let (len_numty, ..) = len.numty();

				if let Some(exact) = len.exact() {
					(ty_min * (exact as usize), ty_max.map(|max| max * exact as usize))
				} else {
					(
						ty_min * len_min + len_numty.size(),
						ty_max
							.zip(len.max())
							.map(|(ty_max, max)| ty_max * max as usize + len_numty.size()),
					)
				}
			}

			Self::Map(k, v) => {
				if let Some((variants_numty, variants)) = k.variants() {
					(
						variants_numty.size(),
						v.size(recursed).1.map(|size| variants * size + variants_numty.size()),
					)
				} else {
					(2, None)
				}
			}

			Self::Set(k) => {
				if let Some((variants_numty, variants)) = k.variants() {
					(
						variants_numty.size(),
						k.size(recursed).1.map(|size| variants * size + variants_numty.size()),
					)
				} else {
					(2, None)
				}
			}

			Self::Opt(ty) => {
				let (_, ty_max) = ty.size(recursed);

				(1, ty_max.map(|ty_max| ty_max + 1))
			}

			Self::Ref(name, tydecl) => {
				if recursed.contains(name) {
					// 0 is returned here because all valid recursive types are
					// bounded and all bounded types have their own min size
					(0, None)
				} else {
					recursed.insert(name);

					tydecl.borrow().size(recursed)
				}
			}

			Self::Enum(enum_ty) => enum_ty.size(recursed),
			Self::Struct(struct_ty) => struct_ty.size(recursed),
			Self::Or(or_tys, discriminant_numty) => {
				let mut min = 0;
				let mut max = Some(0usize);

				for ty in or_tys {
					let (ty_min, ty_max) = ty.size(recursed);

					if ty_min < min {
						min = ty_min;
					}

					if let Some(ty_max) = ty_max {
						if let Some(current_max) = max {
							if ty_max > current_max {
								max = Some(ty_max);
							}
						}
					} else {
						max = None;
					}
				}

				(
					min + discriminant_numty.size(),
					max.map(|max| max + discriminant_numty.size()),
				)
			}

			Self::Instance(_) => (4, Some(4)),

			Self::BrickColor => (2, Some(2)),
			Self::DateTimeMillis => (8, Some(8)),
			Self::DateTime => (8, Some(8)),
			Self::Boolean => (1, Some(1)),
			Self::Color3 => (12, Some(12)),
			Self::Vector2 => (8, Some(8)),
			Self::Vector3 => (12, Some(12)),
			Self::Vector(x_ty, y_ty, z_ty) => {
				let x_size = match **x_ty {
					Ty::Num(numty, _) => numty.size(),
					_ => 0,
				};
				let y_size = match **y_ty {
					Ty::Num(numty, _) => numty.size(),
					_ => 0,
				};
				let z_size = if let Some(z_ty) = z_ty {
					match **z_ty {
						Ty::Num(numty, _) => numty.size(),
						_ => 0,
					}
				} else {
					0
				};

				let total = x_size + y_size + z_size;
				(total, Some(total))
			}
			Self::AlignedCFrame => (13, Some(13)),
			Self::CFrame => (24, Some(24)),
			Self::Unknown => (0, None),
		}
	}

	pub fn variants(&self) -> Option<(NumTy, usize)> {
		match self {
			Ty::Enum(Enum::Unit(variants)) => Some(variants.len() - 1),
			Ty::Enum(Enum::Tagged { variants, .. }) => Some(variants.len() - 1),
			Ty::Num(num, ..) if matches!(num, NumTy::U8 | NumTy::U16 | NumTy::I8 | NumTy::I16) => {
				Some((num.min().abs() + num.max()) as usize)
			}
			// prevent an increase from u16 on previous versions to u32
			Ty::Num(num, ..) => return Some((NumTy::U16, (num.min().abs() + num.max()) as usize)),
			Ty::Ref(.., ty) => return ty.borrow().variants(),
			_ => None,
		}
		// add one to account for things like empty maps, where the 0 must be stored regardless.
		.map(|variants| (NumTy::from_f64(0.0, (variants + 1) as f64), variants))
	}

	pub fn primitive_ty(&self) -> PrimitiveTy<'src> {
		match self {
			Ty::Arr(..) => PrimitiveTy::Name("table"),
			Ty::Set(..) => PrimitiveTy::Name("table"),
			Ty::Map(..) => PrimitiveTy::Name("table"),
			Ty::Struct(..) => PrimitiveTy::Name("table"),
			Ty::Num(..) => PrimitiveTy::Name("number"),
			Ty::Str(..) => PrimitiveTy::Name("string"),
			Ty::Buf(..) => PrimitiveTy::Name("buffer"),
			Ty::BrickColor => PrimitiveTy::Name("BrickColor"),
			Ty::DateTimeMillis => PrimitiveTy::Name("DateTime"),
			Ty::DateTime => PrimitiveTy::Name("DateTime"),
			Ty::Boolean => PrimitiveTy::Name("boolean"),
			Ty::Color3 => PrimitiveTy::Name("Color3"),
			Ty::Vector2 => PrimitiveTy::Name("Vector2"),
			Ty::Vector3 => PrimitiveTy::Name("Vector3"),
			Ty::Vector(..) => PrimitiveTy::Name("Vector3"),
			Ty::AlignedCFrame => PrimitiveTy::Name("CFrame"),
			Ty::CFrame => PrimitiveTy::Name("CFrame"),
			Ty::Instance(class) => PrimitiveTy::Instance(*class),
			Ty::Enum(r#enum) => PrimitiveTy::Enum(r#enum.clone()),
			Ty::Ref(.., ty) => ty.borrow().primitive_ty(),
			Ty::Opt(ty) if matches!(**ty, Ty::Unknown) => PrimitiveTy::Unknown,
			Ty::Unknown => PrimitiveTy::Unknown,
			Ty::Opt(..) => PrimitiveTy::None(NonPrimitiveTy::Opt),
			Ty::Or(..) => PrimitiveTy::None(NonPrimitiveTy::Or),
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
	pub fn size(&self, recursed: &mut HashSet<&'src str>) -> (usize, Option<usize>) {
		match self {
			Self::Unit(enumerators) => {
				let numty = NumTy::from_f64(0.0, enumerators.len() as f64 - 1.0);

				(numty.size(), Some(numty.size()))
			}

			Self::Tagged { variants, .. } => {
				let mut min = 0;
				let mut max = Some(0);

				for (_, ty) in variants.iter() {
					let (ty_min, ty_max) = ty.size(recursed);

					if ty_min < min {
						min = ty_min;
					}

					if let Some(ty_max) = ty_max {
						if let Some(current_max) = max {
							if ty_max > current_max {
								max = Some(ty_max);
							}
						}
					} else {
						max = None;
					}
				}

				(min, max)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct Struct<'src> {
	pub fields: Vec<(&'src str, Ty<'src>)>,
}

impl<'src> Struct<'src> {
	pub fn size(&self, recursed: &mut HashSet<&'src str>) -> (usize, Option<usize>) {
		let mut min = 0;
		let mut max = Some(0);

		for (_, ty) in self.fields.iter() {
			let (ty_min, ty_max) = ty.size(recursed);

			if ty_min < min {
				min = ty_min;
			}

			if let Some(ty_max) = ty_max {
				if let Some(current_max) = max {
					max = Some(ty_max + current_max);
				}
			} else {
				max = None;
			}
		}

		(min, max)
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

	pub fn numty(&self) -> (NumTy, f64) {
		let min = self.min.unwrap_or(0.0);
		let Some(max) = self.max else { return (NumTy::U16, 0.0) };

		let (min, max, offset) = if min > 0.0 {
			(0.0, max - min, min)
		} else {
			(min, max, 0.0)
		};

		(NumTy::from_f64(min, max), offset)
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

	pub fn min(&self) -> f64 {
		match self {
			NumTy::F32 => f32::MIN.into(),
			NumTy::F64 => f64::MIN,

			NumTy::U8 => u8::MIN.into(),
			NumTy::U16 => u16::MIN.into(),
			NumTy::U32 => u32::MIN.into(),

			NumTy::I8 => i8::MIN.into(),
			NumTy::I16 => i16::MIN.into(),
			NumTy::I32 => i32::MIN.into(),
		}
	}

	pub fn max(&self) -> f64 {
		match self {
			NumTy::F32 => f32::MAX.into(),
			NumTy::F64 => f64::MAX,

			NumTy::U8 => u8::MAX.into(),
			NumTy::U16 => u16::MAX.into(),
			NumTy::U32 => u32::MAX.into(),

			NumTy::I8 => i8::MAX.into(),
			NumTy::I16 => i16::MAX.into(),
			NumTy::I32 => i32::MAX.into(),
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
