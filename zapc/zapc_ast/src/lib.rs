use lasso::{Reader, Spur};
use zapc_meta::Span;

mod visitor;

pub use visitor::*;

#[rustfmt::skip]
pub const BUILTIN_TYPES: &[&str] = &[
	// primitives
	"u8", "i8", "u16", "i16", "u32", "i32", "f32", "f64", "string", "buffer", "vector", "boolean", "unknown",
	// composite
	"Map", "Array",
	// aliases
	"Vector3", "Vector2",
	// roblox
	"Color3", "CFrame", "Instance",
];

#[derive(Debug, Clone)]
pub struct Ast {
	decls: Vec<AstDecl>,
}

impl Ast {
	pub fn new(decls: Vec<AstDecl>) -> Self {
		Self { decls }
	}

	pub fn decls(&self) -> &[AstDecl] {
		&self.decls
	}
}

#[derive(Debug, Clone)]
pub enum AstDecl {
	Ty {
		name: AstWord,
		ty: AstTy,
		span: Span,
	},

	Mod {
		name: AstWord,
		decls: Vec<AstDecl>,
		span: Span,
	},

	Event {
		name: AstWord,
		config: AstConfigTable,
		tys: AstTys,
		span: Span,
	},

	Funct {
		name: AstWord,
		config: AstConfigTable,
		args: AstTys,
		rets: AstTys,
		span: Span,
	},

	Err(Span),
}

#[derive(Debug, Clone)]
pub enum AstConfigValue {
	Path { segments: Vec<AstWord>, span: Span },

	Number(AstNumber),
	String(AstString),
	Boolean(AstBoolean),
	Table(AstConfigTable),
}

impl AstConfigValue {
	pub fn span(&self) -> Span {
		match self {
			Self::Path { span, .. } => *span,
			Self::Number(number) => number.span(),
			Self::String(string) => string.span(),
			Self::Boolean(boolean) => boolean.span(),
			Self::Table(table) => table.span(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct AstConfigTable {
	fields: Vec<(AstWord, AstConfigValue)>,
	span: Span,
}

impl AstConfigTable {
	pub fn new(fields: Vec<(AstWord, AstConfigValue)>, span: Span) -> Self {
		Self { fields, span }
	}

	pub fn fields(&self) -> &[(AstWord, AstConfigValue)] {
		&self.fields
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone)]
pub struct AstTys {
	tys: Vec<AstTy>,
	span: Span,
}

impl AstTys {
	pub fn new(tys: Vec<AstTy>, span: Span) -> Self {
		Self { tys, span }
	}

	pub fn tys(&self) -> &[AstTy] {
		&self.tys
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone)]
pub enum AstTy {
	Path {
		segments: Vec<AstWord>,
		generics: Vec<AstGenericTy>,
		span: Span,
	},

	Struct {
		body: AstTyTable,
		span: Span,
	},

	UnitEnum {
		variants: Vec<AstWord>,
		span: Span,
	},

	TaggedEnum {
		tag: AstString,
		variants: Vec<(AstWord, AstTyTable)>,
		catch_all: Option<AstTyTable>,
		span: Span,
	},

	Optional {
		ty: Box<AstTy>,
		span: Span,
	},

	Err(Span),
}

impl AstTy {
	pub fn span(&self) -> Span {
		match self {
			Self::Path { span, .. } => *span,
			Self::Struct { span, .. } => *span,
			Self::UnitEnum { span, .. } => *span,
			Self::TaggedEnum { span, .. } => *span,
			Self::Optional { span, .. } => *span,
			Self::Err(span) => *span,
		}
	}
}

#[derive(Debug, Clone)]
pub struct AstTyTable {
	fields: Vec<(AstWord, AstTy)>,
	span: Span,
}

impl AstTyTable {
	pub fn new(fields: Vec<(AstWord, AstTy)>, span: Span) -> Self {
		Self { fields, span }
	}

	pub fn fields(&self) -> &[(AstWord, AstTy)] {
		&self.fields
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone)]
pub enum AstGenericTy {
	Ty(AstTy),
	Range(AstRange),
	String(AstString),
}

impl AstGenericTy {
	pub fn span(&self) -> Span {
		match self {
			Self::Ty(ty) => ty.span(),
			Self::Range(range) => range.span(),
			Self::String(range) => range.span(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum AstRange {
	WithMinMax(Span, AstNumber, AstNumber),
	WithMin(Span, AstNumber),
	WithMax(Span, AstNumber),
	Exact(Span, AstNumber),
	None(Span),

	Err(Span),
}

impl AstRange {
	pub fn span(&self) -> Span {
		match self {
			Self::WithMinMax(span, ..) => *span,
			Self::WithMin(span, ..) => *span,
			Self::WithMax(span, ..) => *span,
			Self::Exact(span, ..) => *span,
			Self::None(span) => *span,
			Self::Err(span) => *span,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstBoolean {
	value: bool,
	span: Span,
}

impl AstBoolean {
	pub fn new(value: bool, span: Span) -> Self {
		Self { value, span }
	}

	pub fn value(&self) -> bool {
		self.value
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstString {
	spur: Spur,
	span: Span,
}

impl AstString {
	pub fn new(spur: Spur, span: Span) -> Self {
		Self { spur, span }
	}

	pub fn spur(&self) -> Spur {
		self.spur
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn str<'a>(&self, rodeo: &'a impl Reader) -> &'a str {
		rodeo.resolve(&self.spur)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstNumber {
	value: f64,
	span: Span,
}

impl AstNumber {
	pub fn new(value: f64, span: Span) -> Self {
		Self { value, span }
	}

	pub fn value(&self) -> f64 {
		self.value
	}

	pub fn span(&self) -> Span {
		self.span
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstWord {
	spur: Spur,
	span: Span,
}

impl AstWord {
	pub fn new(spur: Spur, span: Span) -> Self {
		Self { spur, span }
	}

	pub fn spur(&self) -> Spur {
		self.spur
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn str<'a>(&self, rodeo: &'a impl Reader) -> &'a str {
		rodeo.resolve(&self.spur)
	}
}
