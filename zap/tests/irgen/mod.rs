use std::collections::HashMap;

use lune::Runtime;
use zap::{
	config::{Config, Parameter, TyDecl},
	irgen::{des, ser},
	output::luau::Output,
	parser::parse,
};

struct TestOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
	var_occurrences: HashMap<String, usize>,
	default_values: HashMap<&'src str, Vec<&'src str>>,
}

impl Output for TestOutput<'_> {
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
		let name = &tydecl.name;
		let ty = &tydecl.ty;

		self.push_indent();
		self.push(&format!("export type {name} = "));
		self.push_ty(ty);
		self.push("\n");

		self.push_line(&format!("function types.write_{name}(value: {name})"));
		self.indent();
		let statements = &ser::gen(
			&[ty.clone()],
			&["value".to_string()],
			self.config.write_checks,
			&mut self.var_occurrences,
		);
		self.push_stmts(statements);
		self.dedent();
		self.push_line("end");

		self.push_line(&format!("function types.read_{name}()"));
		self.indent();
		self.push_line("local value;");
		let statements = &des::gen(&[ty.clone()], &["value".to_string()], true, &mut self.var_occurrences);
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

		let ser_statements = ser::gen(
			parameters.iter().map(|parameter| &parameter.ty),
			&ser_names,
			self.config.write_checks,
			&mut self.var_occurrences,
		);

		self.push_stmts(&ser_statements);
		self.push_line("incoming_buff = outgoing_buff");
		self.push_line("incoming_inst = outgoing_inst");
		self.push_line("incoming_read = 0");
		self.push_line("incoming_ipos = 0");
		self.push_line("load_empty()");

		let des_statements = des::gen(
			parameters.iter().map(|parameter| &parameter.ty),
			&des_names,
			self.config.write_checks,
			&mut self.var_occurrences,
		);

		self.push_stmts(&des_statements);

		for (ser_name, des_name) in ser_names.iter().zip(des_names.iter()) {
			self.push_line(&format!("assert(deepEquals({ser_name}, {des_name}))"));
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

		for evdecl in self.config.evdecls.iter() {
			self.push_event_callback(&evdecl.data, evdecl.name);
		}

		for fndecl in self.config.fndecls.iter() {
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
