use super::*;

#[allow(unused_variables)]
pub trait AstVisitor<'a> {
	fn visit_ast(&mut self, ast: &'a Ast<'a>) {}
	fn visit_opts(&mut self, opts: &'a Opts<'a>) {}
	fn visit_decl(&mut self, decl: &'a AstDecl<'a>) {}
	fn visit_config_struct(&mut self, config: &'a AstConfigStruct<'a>) {}
	fn visit_config_value(&mut self, value: &'a AstConfigValue<'a>) {}
	fn visit_ty(&mut self, ty: &'a AstTy<'a>) {}
	fn visit_enum(&mut self, en: &'a AstEnum<'a>) {}
	fn visit_struct(&mut self, st: &'a AstStruct<'a>) {}
	fn visit_range(&mut self, range: &'a AstRange) {}
	fn visit_string(&mut self, string: &'a AstString<'a>) {}
	fn visit_number(&mut self, number: &'a AstNumber) {}
	fn visit_bool(&mut self, boolean: &'a AstBool) {}
	fn visit_ident(&mut self, ident: &'a AstIdent<'a>) {}
}

trait AstNode<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V);
}

pub fn visit<'a, V: AstVisitor<'a>>(ast: &'a Ast<'a>, visitor: &mut V) {
	ast.visit(visitor);
}

impl<'a> AstNode<'a> for Ast<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		for opts in &self.opts {
			opts.visit(visitor);
		}

		for decl in &self.decls {
			decl.visit(visitor);
		}
	}
}

impl<'a> AstNode<'a> for Opts<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_opts(self);
	}
}

impl<'a> AstNode<'a> for AstDecl<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_decl(self);

		match self {
			Self::Ty { name, ty, .. } => {
				name.visit(visitor);
				ty.visit(visitor);
			}

			Self::Ev { name, config, data, .. } => {
				name.visit(visitor);
				config.visit(visitor);

				for ty in data {
					ty.visit(visitor);
				}
			}

			Self::Fn {
				name,
				config,
				args,
				rets,
				..
			} => {
				name.visit(visitor);
				config.visit(visitor);

				for ty in args {
					ty.visit(visitor);
				}

				for ty in rets {
					ty.visit(visitor);
				}
			}

			Self::Ch { name, config, .. } => {
				name.visit(visitor);
				config.visit(visitor);
			}

			Self::Ns { name, body, .. } => {
				name.visit(visitor);

				for decl in body {
					decl.visit(visitor);
				}
			}
		}
	}
}

impl<'a> AstNode<'a> for AstConfigValue<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_config_value(self);
	}
}

impl<'a> AstNode<'a> for AstConfigStruct<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_config_struct(self);

		for (_, value) in &self.fields {
			value.visit(visitor);
		}
	}
}

impl<'a> AstNode<'a> for AstTy<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_ty(self);

		match self {
			Self::Error(_) => {}

			Self::Reference { name, .. } => {
				for ident in name {
					ident.visit(visitor);
				}
			}

			Self::Instance { class, .. } => {
				if let Some(class) = class {
					class.visit(visitor);
				}
			}

			Self::Optional { ty, .. } => ty.visit(visitor),

			Self::Number { name, range, .. } => {
				name.visit(visitor);

				if let Some(range) = range {
					range.visit(visitor);
				}
			}

			Self::String { range, .. } => {
				if let Some(range) = range {
					range.visit(visitor);
				}
			}

			Self::Buffer { range, .. } => {
				if let Some(range) = range {
					range.visit(visitor);
				}
			}

			Self::Array { ty, range, .. } => {
				ty.visit(visitor);

				if let Some(range) = range {
					range.visit(visitor);
				}
			}

			Self::Map {
				key_ty, val_ty, range, ..
			} => {
				key_ty.visit(visitor);
				val_ty.visit(visitor);

				if let Some(range) = range {
					range.visit(visitor);
				}
			}

			Self::Struct(_, st) => st.visit(visitor),
			Self::Enum(_, en) => en.visit(visitor),
		}
	}
}

impl<'a> AstNode<'a> for AstEnum<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_enum(self);

		if let Self::Tagged {
			variants, catch_all, ..
		} = self
		{
			for (_, st) in variants {
				st.visit(visitor);
			}

			if let Some(catch_all) = catch_all {
				catch_all.visit(visitor);
			}
		}
	}
}

impl<'a> AstNode<'a> for AstStruct<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_struct(self);

		for (_, ty) in &self.fields {
			ty.visit(visitor);
		}
	}
}

impl<'a> AstNode<'a> for AstRange {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_range(self);

		match self {
			Self::WithMinMax(_, min, max) => {
				min.visit(visitor);
				max.visit(visitor);
			}

			Self::WithMax(_, max) => max.visit(visitor),
			Self::WithMin(_, min) => min.visit(visitor),
			Self::Exact(_, num) => num.visit(visitor),
			Self::None(_) => {}
		}
	}
}

impl<'a> AstNode<'a> for AstString<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_string(self);
	}
}

impl<'a> AstNode<'a> for AstNumber {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_number(self);
	}
}

impl<'a> AstNode<'a> for AstBool {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_bool(self);
	}
}

impl<'a> AstNode<'a> for AstIdent<'a> {
	fn visit<V: AstVisitor<'a>>(&'a self, visitor: &mut V) {
		visitor.visit_ident(self);
	}
}
