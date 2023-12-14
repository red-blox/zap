use super::{Casing, EvCall, EvDecl, EvSource, EvType, Lang, Ty, TyDecl};

#[derive(Debug, Clone)]
pub enum EvField {
	From(EvSource),
	Type(EvType),
	Call(EvCall),
	Data(Ty),
}

#[derive(Debug, Clone)]
pub enum Decl {
	Ev(EvDecl),
	Ty(TyDecl),
}

#[derive(Debug, Clone)]
pub enum Opt {
	Lang(Lang),
	Casing(Casing),
	WriteChecks(bool),
}
