use crate::util::{EvCall, EvSource, EvType, NumTy};

use super::lexer::Token;

#[derive(Debug, Clone)]
pub struct SyntaxConfig<'a> {
	pub tydecls: Vec<SyntaxTyDecl<'a>>,
	pub evdecls: Vec<SyntaxEvDecl<'a>>,

	pub server_output: Option<&'a str>,
	pub client_output: Option<&'a str>,

	pub write_checks: bool,
	pub typescript: bool,
}

#[derive(Debug, Clone)]
pub struct SyntaxEvDecl<'a> {
	pub name: &'a str,
	pub from: EvSource,
	pub evty: EvType,
	pub call: EvCall,
	pub data: SyntaxTy<'a>,
}

#[derive(Debug, Clone)]
pub struct SyntaxTyDecl<'a> {
	pub name: &'a str,
	pub ty: SyntaxTy<'a>,
}

#[derive(Debug, Clone)]
pub enum SyntaxTy<'a> {
	// 0: NumTy 1: SyntaxRange
	Num(NumTy, Option<SyntaxRange<'a>>),

	// 0: "string" 1: SyntaxRange
	Str(Token<'a>, Option<SyntaxRange<'a>>),

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
