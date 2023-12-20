use std::fmt::Display;

use crate::parser::Casing;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

pub fn casing(casing: Casing, pascal: &'static str, camel: &'static str, snake: &'static str) -> &'static str {
	match casing {
		Casing::Pascal => pascal,
		Casing::Camel => camel,
		Casing::Snake => snake,
	}
}
