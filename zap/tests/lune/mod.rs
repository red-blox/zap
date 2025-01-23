use std::fs;

use insta::{assert_debug_snapshot, glob, Settings};
use lune::Runtime;

pub fn run_lune_test(input: &str, no_warnings: bool, insta_settings: Settings) {
	let script = zap::run(&format!("opt tooling = true\n{input}"), no_warnings);

	assert!(script.code.is_some(), "No code generated!");

	let code = script.code.as_ref().unwrap();
	let mut runtime = Runtime::new().with_args(vec![
		&code.server.code,
		&code.client.code,
		&code.tooling.as_ref().unwrap().code,
	]);

	let result = tokio::runtime::Runtime::new()
		.expect("Unable to setup tokio")
		.block_on(async { runtime.run("zap", include_str!("./base.luau")).await })
		.map(|value| value.0);

	insta_settings.bind(|| assert_debug_snapshot!(result))
}

#[test]
fn test_compile() {
	glob!(env!("CARGO_MANIFEST_DIR"), "tests/files/*.zap", |path| {
		let input = fs::read_to_string(path).unwrap();

		let mut insta_settings = Settings::new();
		insta_settings.set_prepend_module_to_snapshot(false);
		insta_settings.set_sort_maps(true);
		insta_settings.set_input_file(path);
		insta_settings.set_snapshot_suffix(path.file_stem().unwrap().to_string_lossy());

		run_lune_test(&input, true, insta_settings)
	});
}
