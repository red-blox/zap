use lasso::Rodeo;
use zapc_ast_builder::build;
use zapc_ast_validate::validate;
use zapc_meta::FileDatabase;
use zapc_token::tokenize;

fn main() {
	let source = std::fs::read_to_string("../test.zap").expect("Failed to read file");
	let mut files = FileDatabase::new();

	let id = files.add("test.zap".to_string(), source);

	let mut rodeo = Rodeo::new();

	let parse_result = build(
		id,
		tokenize(id, files.get(id).unwrap().code(), &mut rodeo).collect(),
		&mut rodeo,
	);

	let rodeo = rodeo.into_reader();

	if let Err(reports) = parse_result {
		let report_count = reports.len();

		for report in reports {
			report.into_ariadne().eprint(&mut files).unwrap();
		}

		eprint!("Compilation failed with {} error(s)", report_count);
		return;
	}

	let reports = validate(&parse_result.unwrap(), &rodeo);

	for report in reports {
		report.into_ariadne().eprint(&mut files).unwrap();
	}
}
