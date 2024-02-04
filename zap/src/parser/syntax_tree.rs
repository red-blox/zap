use crate::config::{EvCall, EvSource, EvType, FnCall, NumTy};

use super::reports::Span;

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

impl<'src> Spanned for SyntaxConfig<'src> {
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

impl<'src> Spanned for SyntaxOpt<'src> {
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

impl<'src> Spanned for SyntaxOptValue<'src> {
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
	Ty(SyntaxTyDecl<'src>),
	Ev(SyntaxEvDecl<'src>),
	Fn(SyntaxFnDecl<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxFnDecl<'src> {
	pub start: usize,
	pub name: SyntaxIdentifier<'src>,
	pub call: FnCall,
	pub args: Option<SyntaxTy<'src>>,
	pub rets: Option<SyntaxTy<'src>>,
	pub end: usize,
}

impl<'src> Spanned for SyntaxFnDecl<'src> {
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
	pub call: EvCall,
	pub data: Option<SyntaxTy<'src>>,
	pub end: usize,
}

impl<'src> Spanned for SyntaxEvDecl<'src> {
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

impl<'src> Spanned for SyntaxTyDecl<'src> {
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

impl<'src> Spanned for SyntaxTy<'src> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxTyKind<'src> {
	Num(NumTy, Option<SyntaxRange<'src>>),
	Str(Option<SyntaxRange<'src>>),
	Buf(Option<SyntaxRange<'src>>),
	Arr(Box<SyntaxTy<'src>>, Option<SyntaxRange<'src>>),
	Map(Box<SyntaxTy<'src>>, Box<SyntaxTy<'src>>),
	Opt(Box<SyntaxTy<'src>>),
	Ref(SyntaxIdentifier<'src>),

	Enum(SyntaxEnum<'src>),
	Struct(SyntaxStruct<'src>),
	Instance(Option<SyntaxIdentifier<'src>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxEnum<'src> {
	pub start: usize,
	pub kind: SyntaxEnumKind<'src>,
	pub end: usize,
}

impl<'src> Spanned for SyntaxEnum<'src> {
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
		catch_all: Option<SyntaxStruct<'src>>,
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

impl<'src> Spanned for SyntaxRange<'src> {
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

impl<'src> Spanned for SyntaxStrLit<'src> {
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

impl<'src> Spanned for SyntaxNumLit<'src> {
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

impl<'src> Spanned for SyntaxIdentifier<'src> {
	fn span(&self) -> Span {
		self.start..self.end
	}
}
