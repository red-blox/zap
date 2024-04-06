use ariadne::{Cache, Source};

pub type FileId = usize;

pub struct File {
	name: String,
	code: String,
	source: Option<Source>,
}

impl File {
	pub fn new(name: String, code: String) -> Self {
		Self {
			name,
			code,
			source: None,
		}
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn code(&self) -> &str {
		&self.code
	}

	pub fn source(&mut self) -> &Source {
		if self.source.is_none() {
			self.source = Some(Source::from(self.code.clone()));
		}

		self.source.as_ref().unwrap()
	}
}

#[derive(Default)]
pub struct FileDatabase {
	files: Vec<File>,
}

impl FileDatabase {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, id: FileId) -> Option<&File> {
		self.files.get(id)
	}

	pub fn has(&self, name: &str) -> bool {
		self.files.iter().any(|f| f.name() == name)
	}

	pub fn add(&mut self, name: String, code: String) -> FileId {
		debug_assert!(!self.has(&name));
		self.files.push(File::new(name, code));
		self.files.len() - 1
	}
}

impl Cache<FileId> for &mut FileDatabase {
	type Storage = String;

	fn fetch(&mut self, id: &FileId) -> Result<&Source<Self::Storage>, Box<dyn std::fmt::Debug + '_>> {
		Ok(self
			.files
			.get_mut(*id)
			.expect("attempt to `fetch` file that was not registered with the file database")
			.source())
	}

	fn display<'a>(&self, id: &'a FileId) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new(
			self.get(*id)
				.expect("attempt to `display` file that was not registered with the file database")
				.name()
				.to_string(),
		))
	}
}
