use crate::{Ast, AstDecl, AstGenericTy, AstRange, AstTy, AstTyTable};

#[allow(unused)]
pub trait AstVisitor {
	fn visit_ast(&mut self, ast: &Ast) {}
	fn visit_decl(&mut self, decl: &AstDecl) {}
	fn visit_ty(&mut self, ty: &AstTy) {}
	fn visit_ty_table(&mut self, table: &AstTyTable) {}
	fn visit_range(&mut self, range: &AstRange) {}
}

trait AstNode {
	fn visit(&self, visitor: &mut impl AstVisitor);
}

pub fn visit(ast: &Ast, visitor: &mut impl AstVisitor) {
	ast.visit(visitor);
}

impl AstNode for Ast {
	fn visit(&self, visitor: &mut impl AstVisitor) {
		visitor.visit_ast(self);

		for decl in &self.decls {
			decl.visit(visitor);
		}
	}
}

impl AstNode for AstDecl {
	fn visit(&self, visitor: &mut impl AstVisitor) {
		visitor.visit_decl(self);

		if let Self::Ty { ty, .. } = self {
			ty.visit(visitor);
		}
	}
}

impl AstNode for AstTy {
	fn visit(&self, visitor: &mut impl AstVisitor) {
		visitor.visit_ty(self);

		match self {
			Self::Path { generics, .. } => {
				for generic in generics {
					generic.visit(visitor);
				}
			}

			Self::Struct { body, .. } => body.visit(visitor),

			Self::TaggedEnum {
				variants, catch_all, ..
			} => {
				for (_, table) in variants {
					table.visit(visitor);
				}

				if let Some(table) = catch_all {
					table.visit(visitor);
				}
			}

			Self::Optional { ty, .. } => ty.visit(visitor),

			_ => {}
		}
	}
}

impl AstNode for AstTyTable {
	fn visit(&self, visitor: &mut impl AstVisitor) {
		visitor.visit_ty_table(self);

		for (_, ty) in self.fields() {
			ty.visit(visitor);
		}
	}
}

impl AstNode for AstGenericTy {
	fn visit(&self, visitor: &mut impl AstVisitor) {
		match self {
			Self::Ty(ty) => ty.visit(visitor),
			Self::Range(range) => range.visit(visitor),

			_ => {}
		}
	}
}

impl AstNode for AstRange {
	fn visit(&self, visitor: &mut impl AstVisitor) {
		visitor.visit_range(self);
	}
}
