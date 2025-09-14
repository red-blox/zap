use std::{collections::HashMap, slice};

use insta::{Settings, assert_debug_snapshot};
use lune::Runtime;
use zap::{
	config::{Config, Parameter, TyDecl},
	irgen::{des, ser},
	output::{ConfigProvider, luau::Output},
	parser::parse,
};

struct TestOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
	var_occurrences: HashMap<String, usize>,
	default_values: HashMap<&'src str, Vec<&'src str>>,
}

impl<'src> Output<'src> for TestOutput<'src> {
	fn push(&mut self, s: &str) {
		self.buf.push_str(s);
	}

	fn indent(&mut self) {
		self.tabs += 1;
	}

	fn dedent(&mut self) {
		self.tabs -= 1;
	}

	fn push_indent(&mut self) {
		for _ in 0..self.tabs {
			self.push("\t");
		}
	}
}

impl<'src> ConfigProvider<'src> for TestOutput<'src> {
	fn get_config(&self) -> &'src Config<'src> {
		self.config
	}
}

impl<'src> TestOutput<'src> {
	pub fn new(config: &'src Config<'src>, default_values: HashMap<&'src str, Vec<&'src str>>) -> Self {
		Self {
			config,
			default_values,
			tabs: 0,
			buf: String::new(),
			var_occurrences: HashMap::new(),
		}
	}

	fn push_tydecl(&mut self, tydecl: &TyDecl) {
		let ty = &*tydecl.ty.borrow();

		self.push_indent();
		if tydecl.path.is_empty() {
			self.push("export ");
		}
		self.push(&format!("type {tydecl} = "));
		self.push_ty(ty);
		self.push("\n");

		if tydecl.inline {
			return;
		}

		self.push_line(&format!("function types.write_{tydecl}(value: {tydecl})"));
		self.indent();
		let statements = &ser::generate(
			slice::from_ref(ty),
			&["value".to_string()],
			self.config.write_checks,
			&mut HashMap::new(),
			self.config.typescript_enum,
		);
		self.push_stmts(statements);
		self.dedent();
		self.push_line("end");

		self.push_line(&format!("function types.read_{tydecl}()"));
		self.indent();
		self.push_line("local value;");
		let statements = &des::generate(
			slice::from_ref(ty),
			&["value".to_string()],
			true,
			&mut HashMap::new(),
			self.config.typescript_enum,
		);
		self.push_stmts(statements);
		self.push_line("return value");
		self.dedent();
		self.push_line("end");
	}

	fn push_event_callback(&mut self, parameters: &Vec<Parameter<'src>>, default_value_index: &str) {
		self.push_line("do");
		self.indent();

		let ser_names = parameters
			.iter()
			.enumerate()
			.map(|parameters| format!("ser_{}", parameters.0 + 1))
			.collect::<Vec<String>>();

		let des_names = parameters
			.iter()
			.enumerate()
			.map(|parameters| format!("des_{}", parameters.0 + 1))
			.collect::<Vec<String>>();

		let default_values = self
			.default_values
			.get(default_value_index)
			.unwrap_or_else(|| panic!("No default values for {default_value_index}"))
			.clone();

		for (ser_name, default_value) in ser_names.iter().zip(default_values) {
			self.push_line(&format!("local {ser_name} = {default_value}"));
		}

		for des_name in &des_names {
			self.push_line(&format!("local {des_name}"));
		}

		let ser_statements = ser::generate(
			parameters.iter().map(|parameter| &parameter.ty),
			&ser_names,
			self.config.write_checks,
			&mut self.var_occurrences,
			self.config.typescript_enum,
		);

		self.push_stmts(&ser_statements);
		self.push_line("incoming_buff = outgoing_buff");
		self.push_line("incoming_inst = outgoing_inst");
		self.push_line("incoming_read = 0");
		self.push_line("incoming_ipos = 0");
		self.push_line("load_empty()");

		let des_statements = des::generate(
			parameters.iter().map(|parameter| &parameter.ty),
			&des_names,
			self.config.write_checks,
			&mut self.var_occurrences,
			self.config.typescript_enum,
		);

		self.push_stmts(&des_statements);

		for (ser_name, des_name) in ser_names.iter().zip(des_names.iter()) {
			self.push_line(&format!(
				r#"assert(deepEquals({ser_name}, {des_name}), "deserialised value differs from original in {default_value_index}!")"#
			));
		}

		self.dedent();
		self.push_line("end");
	}

	pub fn output(mut self) -> String {
		self.push(include_str!("./polyfill.luau"));
		self.push(include_str!("../../src/output/luau/base.luau"));

		for tydecl in &self.config.tydecls {
			self.push_tydecl(tydecl);
		}

		for evdecl in self.config.evdecls().iter() {
			self.push_event_callback(&evdecl.data, evdecl.name);
		}

		for fndecl in self.config.fndecls().iter() {
			self.push_event_callback(&fndecl.args, &format!("{}__ARGS", fndecl.name));

			if let Some(rets) = &fndecl.rets {
				let parameters = &rets
					.iter()
					.map(|ty| Parameter {
						name: None,
						ty: ty.to_owned(),
					})
					.collect::<Vec<Parameter<'src>>>();

				self.push_event_callback(parameters, &format!("{}__RETS", fndecl.name));
			}
		}

		self.buf
	}
}

#[tokio::test]
async fn test_function() {
	let (config, reports) = parse(include_str!("../files/function.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([
		("Test__ARGS", vec!["0", "\"foo\""]),
		("Test__RETS", vec!["\"Success\"", "\"bar\""]),
	]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_nested_complex() {
	let (config, reports) = parse(include_str!("../files/nested_complex.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	// This was not fun to write a test for.
	// Hopefully for future readers - it'll be clear enough what goes where :)
	let setup_script = r#"local numbers_1 = { 0, 4, 51 }
local numbers_2 = { 200, 157, 55, 94, 75 }
local numbers_3 = { 108, 242, 73, 19, 151, 194 }
local numbers_4 = { 255, 157, 78, 132 }

local key_1 = {
    [numbers_1] = "foo",
    [numbers_2] = "bar",
    [numbers_3] = "baz"
}
local key_2 = {
    [numbers_4] = "but"
}
local key_3 = {}

local value_1 = {{ x = 215 }, { x = 38 }, { x = 86 }}
local value_2 = {}
local value_3 = {{ x = 27 }, { x = 184 }, { x = 249 }}

default_value = {
    [key_1] = value_1,
    [key_2] = value_2,
    [key_3] = value_3,
}"#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("NestedComplex", vec!["default_value"])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Setup", setup_script).await.unwrap();
	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_simple_struct() {
	let (config, reports) = parse(include_str!("../files/simple_struct.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_value = r#"{
		foo = "baz",
		bar = 21
	}"#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("MyEvent", vec![default_value])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_simple() {
	let (config, reports) = parse(include_str!("../files/simple.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("Test", vec!["100"])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_tuples() {
	let (config, reports) = parse(include_str!("../files/tuples.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("MyEvent", vec!["true", "250", "\"baz\""])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_no_data() {
	let (config, reports) = parse(include_str!("../files/no_data.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([
		("Test1", vec![]),
		("Test2", vec![]),
		("Test3__ARGS", vec![]),
		("Test3__RETS", vec![]),
		("Test4__ARGS", vec!["47"]),
		("Test4__RETS", vec![]),
		("Test5__ARGS", vec![]),
		("Test5__RETS", vec!["183"]),
	]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_unit_enum_quoted() {
	let (config, reports) = parse(include_str!("../files/unit_enum_quoted.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_value = r#""Foo Bar""#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("MyEvent", vec![default_value])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_struct_quoted_fields() {
	let (config, reports) = parse(include_str!("../files/struct_quoted_fields.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_value = r#"{
		["foo bar"] = "baz",
		buzz = 21
	}"#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("MyEvent", vec![default_value])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_tagged_enum_quoted() {
	let (config, reports) = parse(include_str!("../files/tagged_enum_quoted.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_value = r#"{
		["tag with spaces"] = "foo",
		["value with spaces"] = "buzz",
		bar = 21
	}"#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("MyEvent", vec![default_value])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_or_simple() {
	let (config, reports) = parse(include_str!("../files/or_simple.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_value = r#""hello world""#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("MyEvent", vec![default_value])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_or_unknown() {
	let (config, reports) = parse(include_str!("../files/or_unknown.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_value = r#"buffer.create(64)"#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("MyEvent", vec![default_value])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[tokio::test]
async fn test_or_complex() {
	let (config, reports) = parse(include_str!("../files/or_complex.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_value = r#"{ test = "a", b = 127 }"#;

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("Test", vec![default_value])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

#[test]
fn test_unbounded_recursive_type() {
	let input = r#"type foo = foo
event Simple = {
    from: Client,
    type: Reliable,
    call: SingleSync,
    data: foo
}"#;

	let (config, reports) = parse(input);

	assert!(config.is_some());
	assert!(!reports.is_empty());

	let mut insta_settings = Settings::new();
	insta_settings.set_prepend_module_to_snapshot(false);
	insta_settings.set_sort_maps(true);
	insta_settings.set_input_file("unbounded_recursive_type.zap");

	insta_settings.bind(|| assert_debug_snapshot!(reports))
}

#[tokio::test]
async fn test_bitpacking() {
	let (config, reports) = parse(include_str!("../files/bitpacking.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([
		(
			"Event1",
			vec![r#"{ true, false, false, true, true, true, false, false }"#],
		),
		(
			"Event2",
			vec![
				r#"{ true, false, false, true, true, true, false, false, true, false, false, true, true, true, false, false, true }"#,
			],
		),
		(
			"Event3",
			vec![
				r#"{ bools = { false, true, true, false, false, false, false, true, true, false, false, false, true, false }, enum = "world" }"#,
			],
		),
		(
			"Event4",
			vec![
				r#"{ bools = { false, true, true, false, false, false, false, true, true, false, false, false, true, false }, enum = "there" }"#,
			],
		),
		(
			"Event5",
			vec![r#"{ type = "a", value = true }"#, r#"{ type = "b", value = true }"#],
		),
	]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}

const VALID_UTF8_STR: &str = r#""abcdefg1234ąśłź𒂓""#;

#[tokio::test]
async fn test_string_kinds() {
	let (config, reports) = parse(include_str!("../files/string_kinds.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let invalid_utf = [
		r#""\xc3\x28""#,
		r#""\xa0\xa1""#,
		r#""\xe2\x28\xa1""#,
		r#""\xe2\x82\x28""#,
		r#""\xf0\x28\x8c\xbc""#,
		r#""\xf0\x90\x28\xbc""#,
		r#""\xf0\x28\x8c\x28""#,
	];

	let config = config.unwrap();

	for (valid, bin_values, utf_values) in invalid_utf.into_iter().flat_map(|data| {
		[
			(true, vec![data], vec![VALID_UTF8_STR]),
			(false, vec![VALID_UTF8_STR], vec![data]),
		]
	}) {
		let default_values = HashMap::from([("Binary", bin_values), ("Utf8", utf_values)]);
		let output = TestOutput::new(&config, default_values).output();
		let mut runtime = Runtime::new();

		if valid != runtime.run("Zap", output).await.is_ok() {
			unreachable!()
		}
	}
}

#[tokio::test]
async fn test_typescript_enum() {
	let (config, reports) = parse(include_str!("../files/typescript_enum.zap"));

	assert!(config.is_some());
	assert!(reports.is_empty());

	let default_values: HashMap<&str, Vec<&str>> = HashMap::from([("Event", vec!["0", "2"])]);
	let output = TestOutput::new(&config.unwrap(), default_values).output();
	let mut runtime = Runtime::new();

	runtime.run("Zap", output).await.unwrap();
}
