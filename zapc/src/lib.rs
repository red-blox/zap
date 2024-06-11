use hir_builder::HirBuilder;

mod ast;
mod hir;
mod hir_builder;
mod lexer;
mod meta;
mod parser;
mod ty;

pub fn test() {
	let mut rodeo = lasso::Rodeo::new();
	let mut fdb = meta::FileDatabase::new();

	let fid = fdb.add("test.zap".to_string(), std::fs::read_to_string("test.zap").unwrap());

	let (tokens, reports) = lexer::tokenize(fid, fdb.get(fid).unwrap().code(), &mut rodeo);

	for report in reports {
		report.into_ariadne().eprint(&mut fdb).unwrap();
	}

	let parsed = parser::parse(&mut rodeo, tokens);

	match parsed {
		Ok(ast) => {
			let result = HirBuilder::new(&mut rodeo).init_ast(ast);

			if let Ok(hir) = result {
				println!("{:?}", hir);
			} else if let Err(reports) = result {
				for report in reports {
					report.into_ariadne().eprint(&mut fdb).unwrap();
				}
			}
		}

		Err(reports) => {
			for report in reports {
				report.into_ariadne().eprint(&mut fdb).unwrap();
			}
		}
	}
}
