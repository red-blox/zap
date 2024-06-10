#[derive(Debug, Clone)]
pub struct HirRange<T: Clone + Copy + PartialEq> {
	start: Option<T>,
	end: Option<T>,
}

impl<T: Clone + Copy + PartialEq> Default for HirRange<T> {
	fn default() -> Self {
		Self { start: None, end: None }
	}
}

impl<T: Clone + Copy + PartialEq> HirRange<T> {
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
