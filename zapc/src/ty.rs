#[derive(Debug, Clone)]
pub struct Range<T: Clone + Copy + PartialEq> {
	start: Option<T>,
	end: Option<T>,
}

impl<T: Clone + Copy + PartialEq> Default for Range<T> {
	fn default() -> Self {
		Self { start: None, end: None }
	}
}

impl<T: Clone + Copy + PartialEq> Range<T> {
	pub fn new(start: Option<T>, end: Option<T>) -> Self {
		Self { start, end }
	}

	pub fn start(&self) -> Option<T> {
		self.start
	}

	pub fn end(&self) -> Option<T> {
		self.end
	}

	pub fn exact(&self) -> Option<T> {
		if self.start == self.end {
			self.start
		} else {
			None
		}
	}
}

#[derive(Debug, Clone)]
pub enum NumberTy {
	U8(Range<u8>),
	I8(Range<i8>),
	U16(Range<u16>),
	I16(Range<i16>),
	U32(Range<u32>),
	I32(Range<i32>),
	F32(Range<f32>),
	F64(Range<f64>),
}
