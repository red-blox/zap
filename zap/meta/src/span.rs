use std::ops::Range;

use crate::files::FileId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
	file_id: FileId,
	start: usize,
	end: usize,
}

impl Span {
	pub fn new(file_id: FileId, range: Range<usize>) -> Self {
		Self {
			file_id,
			start: range.start,
			end: range.end,
		}
	}

	pub fn file(&self) -> FileId {
		self.file_id
	}

	pub fn start(&self) -> usize {
		self.start
	}

	pub fn end(&self) -> usize {
		self.end
	}

	pub fn merge(&self, other: &Self) -> Self {
		assert_eq!(self.file_id, other.file_id);

		Self {
			file_id: self.file_id,
			start: self.start.min(other.start),
			end: self.end.max(other.end),
		}
	}

	pub fn overlaps(&self, other: &Self) -> bool {
		self.file_id == other.file_id
			&& (self.start <= other.start && other.start < self.end
				|| other.start <= self.start && self.start < other.end)
	}
}

impl From<&Span> for Range<usize> {
	fn from(value: &Span) -> Self {
		value.start..value.end
	}
}

impl chumsky::Span for Span {
	type Context = FileId;
	type Offset = usize;

	fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
		Span::new(context, range)
	}

	fn context(&self) -> Self::Context {
		self.file_id
	}

	fn start(&self) -> Self::Offset {
		self.start
	}

	fn end(&self) -> Self::Offset {
		self.end
	}
}
