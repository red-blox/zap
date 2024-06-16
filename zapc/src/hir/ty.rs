use std::collections::HashMap;

use lasso::Spur;

use crate::{ty::NumberTy, ty::Range};

use super::decl::HirTyDeclId;

#[derive(Debug, Clone)]
pub enum HirTy {
	Reference(HirTyDeclId),

	Boolean,
	Number(NumberTy),
	Buffer(Range<u16>),

	Struct(HirStruct),
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
