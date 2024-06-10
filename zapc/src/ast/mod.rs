use self::decl::AstDecl;

pub mod decl;
pub mod primitive;
pub mod range;
pub mod ty;

#[derive(Debug, Clone)]
pub struct Ast {
	decls: Vec<AstDecl>,
}

impl Ast {
	pub fn new(decls: Vec<AstDecl>) -> Self {
		Self { decls }
	}

	pub fn into_decls(self) -> Vec<AstDecl> {
		self.decls
	}
}
