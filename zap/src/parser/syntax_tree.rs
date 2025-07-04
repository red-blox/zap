use crate::config::{EvCall, EvSource, EvType, FnCall, NumTy};

use super::reports::Span;

#[allow(dead_code)]
pub trait Spanned {
	fn span(&self) -> Span;

	fn start(&self) -> usize {
		self.span().start
	}

	fn end(&self) -> usize {
		self.span().end
	}

	fn len(&self) -> usize {
		self.end() - self.start()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxConfig<'src> {
	pub start: usize,
	pub opts: Vec<SyntaxOpt<'src>>,
	pub decls: Vec<SyntaxDecl<'src>>,
	pub end: usize,
}

impl Spanned for SyntaxConfig<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxOpt<'src> {
	pub start: usize,
	pub name: SyntaxIdentifier<'src>,
	pub value: SyntaxOptValue<'src>,
	pub end: usize,
}

impl Spanned for SyntaxOpt<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxOptValue<'src> {
	pub start: usize,
	pub kind: SyntaxOptValueKind<'src>,
	pub end: usize,
}

impl Spanned for SyntaxOptValue<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxOptValueKind<'src> {
	Str(SyntaxStrLit<'src>),
	Num(SyntaxNumLit<'src>),
	Bool(SyntaxBoolLit),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxDecl<'src> {
	Ns(SyntaxNsDecl<'src>),
	Ty(SyntaxTyDecl<'src>),
	Ev(SyntaxEvDecl<'src>),
	Fn(SyntaxFnDecl<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxNsDecl<'src> {
	pub start: usize,
	pub name: SyntaxIdentifier<'src>,
	pub decls: Vec<SyntaxDecl<'src>>,
	pub end: usize,
}

impl Spanned for SyntaxNsDecl<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxFnDecl<'src> {
	pub start: usize,
	pub name: SyntaxIdentifier<'src>,
	pub call: FnCall,
	pub args: Option<SyntaxParameters<'src>>,
	pub rets: Option<SyntaxParameters<'src>>,
	pub end: usize,
}

impl Spanned for SyntaxFnDecl<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxEvDecl<'src> {
	pub start: usize,
	pub name: SyntaxIdentifier<'src>,
	pub from: EvSource,
	pub evty: EvType,
	pub call: Option<EvCall>,
	pub data: Option<SyntaxParameters<'src>>,
	pub end: usize,
}

impl Spanned for SyntaxEvDecl<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxParameters<'src> {
	pub start: usize,
	pub parameters: Vec<(Option<SyntaxIdentifier<'src>>, SyntaxTy<'src>)>,
	pub end: usize,
}

impl Spanned for SyntaxParameters<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTyDecl<'src> {
	pub start: usize,
	pub name: SyntaxIdentifier<'src>,
	pub ty: SyntaxTy<'src>,
	pub end: usize,
}

impl Spanned for SyntaxTyDecl<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTy<'src> {
	pub start: usize,
	pub kind: SyntaxTyKind<'src>,
	pub end: usize,
}

impl Spanned for SyntaxTy<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

impl Spanned for Vec<SyntaxTy<'_>> {
	fn span(&self) -> Span {
		self.first().unwrap().start..self.last().unwrap().end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxTyKind<'src> {
	Num(NumTy, Option<SyntaxRange<'src>>),
	Str(Option<bool>, Option<SyntaxRange<'src>>),
	Buf(Option<SyntaxRange<'src>>),
	Vector(Box<SyntaxTy<'src>>, Box<SyntaxTy<'src>>, Option<Box<SyntaxTy<'src>>>),
	Arr(Box<SyntaxTy<'src>>, Option<SyntaxRange<'src>>),
	Map(Box<SyntaxTy<'src>>, Box<SyntaxTy<'src>>),
	Set(Box<SyntaxTy<'src>>),
	Opt(Box<SyntaxTy<'src>>),
	Ref(SyntaxIdentifier<'src>),
	Path(Vec<SyntaxIdentifier<'src>>),

	Enum(SyntaxEnum<'src>),
	Struct(SyntaxStruct<'src>),
	Instance(bool, Option<SyntaxIdentifier<'src>>),
	Or(Vec<SyntaxTy<'src>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxEnum<'src> {
	pub start: usize,
	pub kind: SyntaxEnumKind<'src>,
	pub end: usize,
}

impl Spanned for SyntaxEnum<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxEnumKind<'src> {
	Unit(Vec<SyntaxIdentifier<'src>>),

	Tagged {
		tag: SyntaxStrLit<'src>,
		variants: Vec<(SyntaxIdentifier<'src>, SyntaxStruct<'src>)>,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxStruct<'src> {
	pub start: usize,
	pub fields: Vec<(SyntaxIdentifier<'src>, SyntaxTy<'src>)>,
	pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxRange<'src> {
	pub start: usize,
	pub kind: SyntaxRangeKind<'src>,
	pub end: usize,
}

impl Spanned for SyntaxRange<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxRangeKind<'src> {
	None,
	Exact(SyntaxNumLit<'src>),
	WithMin(SyntaxNumLit<'src>),
	WithMax(SyntaxNumLit<'src>),
	WithMinMax(SyntaxNumLit<'src>, SyntaxNumLit<'src>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxStrLit<'src> {
	pub start: usize,
	pub value: &'src str,
	pub end: usize,
}

impl Spanned for SyntaxStrLit<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxNumLit<'src> {
	pub start: usize,
	pub value: &'src str,
	pub end: usize,
}

impl Spanned for SyntaxNumLit<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxBoolLit {
	pub start: usize,
	pub value: bool,
	pub end: usize,
}

impl Spanned for SyntaxBoolLit {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxIdentifier<'src> {
	pub start: usize,
	pub name: &'src str,
	pub end: usize,
}

impl Spanned for SyntaxIdentifier<'_> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}
