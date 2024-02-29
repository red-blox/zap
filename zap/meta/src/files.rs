use std::ops::Range;

use codespan_reporting::files::{Error, Files};

pub type FileId = usize;

pub struct File {
	name: String,
	content: String,
	line_starts: Vec<usize>,
}

impl File {
	pub fn new(name: String, content: String) -> Self {
		let line_starts = std::iter::once(0)
			.chain(content.match_indices('\n').map(|(i, _)| i + 1))
			.collect();

		Self {
			name,
			content,
			line_starts,
		}
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn content(&self) -> &str {
		&self.content
	}

	pub fn line_index(&self, char_index: usize) -> usize {
		self.line_starts.binary_search(&char_index).unwrap_or_else(|i| i - 1)
	}

	pub fn line_range(&self, line_index: usize) -> Range<usize> {
		let start = self.line_starts[line_index];
		let end = self
			.line_starts
			.get(line_index + 1)
			.copied()
			.unwrap_or(self.content.len());

		start..end
	}
}

pub struct FileDatabase {
	files: Vec<File>,
}

impl Default for FileDatabase {
	fn default() -> Self {
		Self::new()
	}
}

impl FileDatabase {
	pub fn new() -> Self {
		Self { files: Vec::new() }
	}

	pub fn add(&mut self, name: String, content: String) -> FileId {
		let id = self.files.len();
		self.files.push(File::new(name, content));
		id
	}

	pub fn get(&self, id: FileId) -> Option<&File> {
		self.files.get(id)
	}
}

impl<'a> Files<'a> for FileDatabase {
	type FileId = FileId;
	type Name = &'a str;
	type Source = &'a str;

	fn name(&'a self, id: Self::FileId) -> Result<Self::Name, codespan_reporting::files::Error> {
		match self.get(id) {
			Some(file) => Ok(file.name()),
			None => Err(Error::FileMissing),
		}
	}

	fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
		match self.get(id) {
			Some(file) => Ok(file.content()),
			None => Err(Error::FileMissing),
		}
	}

	fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
		match self.get(id) {
			Some(file) => Ok(file.line_index(byte_index)),
			None => Err(Error::FileMissing),
		}
	}

	fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
		match self.get(id) {
			Some(file) => Ok(file.line_range(line_index)),
			None => Err(Error::FileMissing),
		}
	}
}
