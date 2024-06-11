use std::collections::HashMap;

use lasso::Spur;

use super::{decl::HirTyDeclId, range::HirRange};

#[derive(Debug, Clone)]
pub enum HirTy {
	Reference(HirTyDeclId),

	Boolean,
	Number(HirNumberTy),
	Buffer(HirRange<u16>),

	Struct(HirStruct),
}

#[derive(Debug, Clone)]
pub enum HirNumberTy {
	U8(HirRange<u8>),
	I8(HirRange<i8>),
	U16(HirRange<u16>),
	I16(HirRange<i16>),
	U32(HirRange<u32>),
	I32(HirRange<i32>),
	F32(HirRange<f32>),
	F64(HirRange<f64>),
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
