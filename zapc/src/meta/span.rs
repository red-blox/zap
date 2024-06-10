use std::ops::Range;

use super::FileId;

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

	pub fn merge(&self, other: Self) -> Self {
		assert_eq!(self.file, other.file);
		Self::new(self.file, self.start.min(other.start), self.end.max(other.end))
	}
}

impl ariadne::Span for Span {
	type SourceId = FileId;

	fn start(&self) -> usize {
		self.start()
	}

	fn end(&self) -> usize {
		self.end()
	}

	fn source(&self) -> &Self::SourceId {
		&self.file
	}
}

impl chumsky::span::Span for Span {
	type Context = FileId;
	type Offset = usize;

	fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
		Self::from_range(context, range)
	}

	fn context(&self) -> Self::Context {
		self.file()
	}

	fn start(&self) -> Self::Offset {
		self.start()
	}

	fn end(&self) -> usize {
		self.end()
	}
}
