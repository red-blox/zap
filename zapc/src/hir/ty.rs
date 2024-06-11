use std::collections::HashMap;

use lasso::Spur;

use crate::range::Range;

use super::decl::HirTyDeclId;

#[derive(Debug, Clone)]
pub enum HirTy {
	Reference(HirTyDeclId),

	Boolean,
	Number(HirNumberTy),
	Buffer(Range<u16>),

	Struct(HirStruct),
}

#[derive(Debug, Clone)]
pub enum HirNumberTy {
	U8(Range<u8>),
	I8(Range<i8>),
	U16(Range<u16>),
	I16(Range<i16>),
	U32(Range<u32>),
	I32(Range<i32>),
	F32(Range<f32>),
	F64(Range<f64>),
}

#[derive(Debug, Clone)]
pub struct HirStruct {
	fields: HashMap<Spur, HirTy>,
}

impl HirStruct {
	pub fn new(fields: HashMap<Spur, HirTy>) -> Self {
		Self { fields }
	}

	pub fn fields(&self) -> &HashMap<Spur, HirTy> {
		&self.fields
	}

	pub fn into_fields(self) -> HashMap<Spur, HirTy> {
		self.fields
	}
}
