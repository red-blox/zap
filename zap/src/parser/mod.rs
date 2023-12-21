mod lexer;
mod syntax_parse;
mod syntax_tree;

#[derive(Debug, Clone)]
pub struct Struct(pub Vec<()>);
