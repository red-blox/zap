use crate::util::{EvCall, EvSource, EvType, NumTy};

use super::lexer::Token;

#[derive(Debug, Clone)]
pub struct SyntaxConfig<'a> {
	pub opts: Vec<SyntaxOpt<'a>>,
	pub decls: Vec<SyntaxDecl<'a>>,
}

#[derive(Debug, Clone)]
pub struct SyntaxOpt<'a> {
	pub name: Token<'a>,
	pub value: Token<'a>,
}

#[derive(Debug, Clone)]
pub enum SyntaxDecl<'a> {
	Ev(SyntaxEvDecl<'a>),
	Ty(SyntaxTyDecl<'a>),
}

#[derive(Debug, Clone)]
pub struct SyntaxEvDecl<'a> {
	pub name: Token<'a>,
	pub from: EvSource,
	pub evty: EvType,
	pub call: EvCall,
	pub data: SyntaxTy<'a>,
}

#[derive(Debug, Clone)]
pub struct SyntaxTyDecl<'a> {
	pub name: Token<'a>,
	pub ty: SyntaxTy<'a>,
}

#[derive(Debug, Clone)]
pub enum SyntaxTy<'a> {
	// 0: NumTy 1: SyntaxRange
	Num(NumTy, Option<SyntaxRange<'a>>),

	// 0: SyntaxRange
	Str(Option<SyntaxRange<'a>>),

	Arr(Box<SyntaxTy<'a>>, SyntaxRange<'a>),
	Map(Box<SyntaxTy<'a>>, Box<SyntaxTy<'a>>),

	Struct(SyntaxStruct<'a>),
	Enum(SyntaxEnum<'a>),

	Opt(Box<SyntaxTy<'a>>),
	Ref(Token<'a>),

	Instance { class: Option<Token<'a>>, strict: bool },
}

#[derive(Debug, Clone)]
pub enum SyntaxEnum<'a> {
	Unit(Vec<Token<'a>>),

	Tagged {
		tag: Token<'a>,
		variants: Vec<(Token<'a>, SyntaxStruct<'a>)>,
	},
}

#[derive(Debug, Clone)]
pub struct SyntaxStruct<'a>(pub Vec<(Token<'a>, SyntaxTy<'a>)>);

#[derive(Debug, Clone)]
pub enum SyntaxRange<'a> {
	None,

	// 0: numlit
	WithMin(Token<'a>),

	// 0: numlit
	WithMax(Token<'a>),

	// 0: numlit 1: numlit
	WithMinMax(Token<'a>, Token<'a>),
	Exact(Token<'a>),
}
