use std::ops::Range;

use crate::FileId;

#[derive(Debug, Clone, Copy)]
pub struct Span {
	file: FileId,
	start: usize,
	end: usize,
}

impl Span {
	pub fn new(file: FileId, start: usize, end: usize) -> Self {
		Self { file, start, end }
	}

	pub fn from_range(file: FileId, range: Range<usize>) -> Self {
		Self::new(file, range.start, range.end)
	}

	pub fn file(&self) -> FileId {
		self.file
	}

	pub fn start(&self) -> usize {
		self.start
	}

	pub fn end(&self) -> usize {
		self.end
	}

	pub fn range(&self) -> Range<usize> {
		self.start..self.end
	}
}

impl ariadne::Span for Span {
	type SourceId = FileId;

	fn start(&self) -> usize {
		self.start
	}

	fn end(&self) -> usize {
		self.end
	}

	fn source(&self) -> &Self::SourceId {
		&self.file
	}
}
