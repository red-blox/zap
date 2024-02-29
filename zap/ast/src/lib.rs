use meta::Span;

mod visitor;

pub use visitor::{visit, AstVisitor};

#[derive(Debug, Clone)]
pub struct Ast<'a> {
	opts: Vec<Opts<'a>>,
	decls: Vec<AstDecl<'a>>,
}

impl<'a> Ast<'a> {
	pub fn new(opts: Vec<Opts<'a>>, decls: Vec<AstDecl<'a>>) -> Self {
		Self { opts, decls }
	}

	pub fn opts(&self) -> &[Opts<'a>] {
		&self.opts
	}

	pub fn decls(&self) -> &[AstDecl<'a>] {
		&self.decls
	}
}

#[derive(Debug, Clone)]
pub struct Opts<'a> {
	span: Span,
	name: Option<AstString<'a>>,
	opts: AstConfigStruct<'a>,
}

impl<'a> Opts<'a> {
	pub fn new(span: Span, name: Option<AstString<'a>>, opts: AstConfigStruct<'a>) -> Self {
		Self { span, name, opts }
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn name(&self) -> Option<AstString<'a>> {
		self.name
	}

	pub fn opts(&self) -> &AstConfigStruct<'a> {
		&self.opts
	}
}

#[derive(Debug, Clone)]
pub enum AstDecl<'a> {
	Ty {
		span: Span,
		name: AstIdent<'a>,
		ty: AstTy<'a>,
	},

	Ev {
		span: Span,
		name: AstIdent<'a>,
		config: AstConfigStruct<'a>,
		data: Vec<AstTy<'a>>,
	},

	Fn {
		span: Span,
		name: AstIdent<'a>,
		config: AstConfigStruct<'a>,
		args: Vec<AstTy<'a>>,
		rets: Vec<AstTy<'a>>,
	},

	Ch {
		span: Span,
		name: AstIdent<'a>,
		config: AstConfigStruct<'a>,
	},

	Ns {
		span: Span,
		name: AstIdent<'a>,
		body: Vec<AstDecl<'a>>,
	},
}

#[derive(Debug, Clone)]
pub struct AstConfigStruct<'a> {
	span: Span,
	fields: Vec<(AstIdent<'a>, AstConfigValue<'a>)>,
}

#[derive(Debug, Clone)]
pub enum AstConfigValue<'a> {
	Bool(AstBool),
	Number(AstNumber),
	String(AstString<'a>),
	Struct(AstConfigStruct<'a>),
	Enum(Span, AstIdent<'a>, Vec<AstConfigValue<'a>>),
	Reference(Span, Vec<AstIdent<'a>>),
}

impl<'a> AstConfigValue<'a> {
	pub fn span(&self) -> Span {
		match self {
			AstConfigValue::Bool(b) => b.span(),
			AstConfigValue::Number(n) => n.span(),
			AstConfigValue::String(s) => s.span(),
			AstConfigValue::Struct(s) => s.span(),
			AstConfigValue::Enum(span, ..) => *span,
			AstConfigValue::Reference(span, ..) => *span,
		}
	}
}

impl<'a> AstConfigStruct<'a> {
	pub fn new(span: Span, fields: Vec<(AstIdent<'a>, AstConfigValue<'a>)>) -> Self {
		Self { span, fields }
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn fields(&self) -> &[(AstIdent<'a>, AstConfigValue<'a>)] {
		&self.fields
	}
}

#[derive(Debug, Clone)]
pub enum AstTy<'a> {
	Error(Span),

	Reference {
		span: Span,
		name: Vec<AstIdent<'a>>,
	},

	Instance {
		span: Span,
		class: Option<AstString<'a>>,
	},

	Optional {
		span: Span,
		ty: Box<AstTy<'a>>,
	},

	Number {
		span: Span,
		name: AstIdent<'a>,
		range: Option<AstRange>,
	},

	String {
		span: Span,
		range: Option<AstRange>,
	},

	Buffer {
		span: Span,
		range: Option<AstRange>,
	},

	Array {
		span: Span,
		ty: Box<AstTy<'a>>,
		range: Option<AstRange>,
	},

	Map {
		span: Span,
		range: Option<AstRange>,
		key_ty: Box<AstTy<'a>>,
		val_ty: Box<AstTy<'a>>,
	},

	Struct(Span, AstStruct<'a>),
	Enum(Span, AstEnum<'a>),
}

impl<'a> AstTy<'a> {
	pub fn span(&self) -> Span {
		match self {
			AstTy::Error(span) => *span,
			AstTy::Reference { span, .. } => *span,
			AstTy::Instance { span, .. } => *span,
			AstTy::Optional { span, .. } => *span,
			AstTy::Number { span, .. } => *span,
			AstTy::String { span, .. } => *span,
			AstTy::Buffer { span, .. } => *span,
			AstTy::Array { span, .. } => *span,
			AstTy::Map { span, .. } => *span,
			AstTy::Struct(span, ..) => *span,
			AstTy::Enum(span, ..) => *span,
		}
	}
}

#[derive(Debug, Clone)]
pub enum AstEnum<'a> {
	Unit {
		span: Span,
		variants: Vec<AstIdent<'a>>,
	},

	Tagged {
		span: Span,
		field: AstString<'a>,
		variants: Vec<(AstIdent<'a>, AstStruct<'a>)>,
		catch_all: Option<AstStruct<'a>>,
	},
}

impl<'a> AstEnum<'a> {
	pub fn span(&self) -> Span {
		match self {
			AstEnum::Unit { span, .. } => *span,
			AstEnum::Tagged { span, .. } => *span,
		}
	}
}

#[derive(Debug, Clone)]
pub struct AstStruct<'a> {
	span: Span,
	fields: Vec<(AstIdent<'a>, AstTy<'a>)>,
}

impl<'a> AstStruct<'a> {
	pub fn new(span: Span, fields: Vec<(AstIdent<'a>, AstTy<'a>)>) -> Self {
		Self { span, fields }
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn fields(&self) -> &[(AstIdent<'a>, AstTy<'a>)] {
		&self.fields
	}

	pub fn field(&self, name: &str) -> Option<(&AstIdent<'a>, &AstTy<'a>)> {
		self.fields
			.iter()
			.find_map(|(field, ty)| if field.value() == name { Some((field, ty)) } else { None })
	}
}

#[derive(Debug, Clone, Copy)]
pub enum AstRange {
	WithMinMax(Span, AstNumber, AstNumber),
	WithMax(Span, AstNumber),
	WithMin(Span, AstNumber),
	Exact(Span, AstNumber),
	None(Span),
}

impl AstRange {
	pub fn span(&self) -> Span {
		match self {
			AstRange::WithMinMax(span, _, _) => *span,
			AstRange::WithMax(span, _) => *span,
			AstRange::WithMin(span, _) => *span,
			AstRange::Exact(span, _) => *span,
			AstRange::None(span) => *span,
		}
	}

	pub fn min(&self) -> Option<AstNumber> {
		match self {
			AstRange::WithMinMax(_, min, _) => Some(*min),
			AstRange::WithMin(_, min) => Some(*min),
			_ => None,
		}
	}

	pub fn max(&self) -> Option<AstNumber> {
		match self {
			AstRange::WithMinMax(_, _, max) => Some(*max),
			AstRange::WithMax(_, max) => Some(*max),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstString<'a> {
	value: &'a str,
	span: Span,
}

impl<'a> AstString<'a> {
	pub fn new(value: &'a str, span: Span) -> Self {
		Self { span, value }
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn value_with_quotes(&self) -> &str {
		self.value
	}

	pub fn value_without_quotes(&self) -> &str {
		&self.value[1..self.value.len() - 1]
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstNumber {
	span: Span,
	value: f64,
}

impl AstNumber {
	pub fn new(span: Span, value: f64) -> Self {
		Self { span, value }
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn value(&self) -> f64 {
		self.value
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstBool {
	span: Span,
	value: bool,
}

impl AstBool {
	pub fn new(span: Span, value: bool) -> Self {
		Self { span, value }
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn value(&self) -> bool {
		self.value
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AstIdent<'a> {
	value: &'a str,
	span: Span,
}

impl<'a> AstIdent<'a> {
	pub fn new(value: &'a str, span: Span) -> Self {
		Self { span, value }
	}

	pub fn span(&self) -> Span {
		self.span
	}

	pub fn value(&self) -> &str {
		self.value
	}
}
