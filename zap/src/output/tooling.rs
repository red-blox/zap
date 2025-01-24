use std::collections::HashMap;

use crate::{
	config::{Config, EvDecl, EvSource, EvType, FnDecl, NumTy, TyDecl},
	irgen::{des, Stmt},
	output::get_unnamed_values,
	Output,
};

struct ToolingOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
	var_occurrences: HashMap<String, usize>,
}

impl<'src> ToolingOutput<'src> {
	pub fn new(config: &'src Config) -> Self {
		Self {
			config,
			tabs: 0,
			buf: String::new(),
			var_occurrences: HashMap::new(),
		}
	}

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

	fn push_line(&mut self, s: &str) {
		self.push_indent();
		self.push(s);
		self.push("\n");
	}

	fn push_stmt(&mut self, stmt: &Stmt) {
		if matches!(stmt, Stmt::ElseIf(..) | Stmt::Else | Stmt::End) {
			self.dedent();
		}

		match &stmt {
			Stmt::Local(name, expr) => {
				if let Some(expr) = expr {
					self.push_line(&format!("local {name} = {expr}"));
				} else {
					self.push_line(&format!("local {name}"));
				}
			}
			Stmt::LocalTuple(var, expr) => {
				let items = var.join(", ");

				if let Some(expr) = expr {
					self.push_line(&format!("local {items} = {expr}"));
				} else {
					self.push_line(&format!("local {items}"));
				}
			}

			Stmt::Assign(var, expr) => self.push_line(&format!("{var} = {expr}")),
			Stmt::Error(msg) => self.push_line(&format!("error(\"{msg}\")")),
			Stmt::Assert(cond, msg) => match msg {
				Some(msg) => self.push_line(&format!("assert({cond}, \"{msg}\")")),
				None => self.push_line(&format!("assert({cond})")),
			},

			Stmt::Call(var, method, args) => match method {
				Some(method) => self.push_line(&format!(
					"{var}:{method}({})",
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				)),

				None => self.push_line(&format!(
					"{var}({})",
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				)),
			},

			Stmt::NumFor { var, from, to } => self.push_line(&format!("for {var} = {from}, {to} do")),
			Stmt::GenFor { key, val, obj } => self.push_line(&format!("for {key}, {val} in {obj} do")),
			Stmt::If(cond) => self.push_line(&format!("if {cond} then")),
			Stmt::ElseIf(cond) => self.push_line(&format!("elseif {cond} then")),
			Stmt::Else => self.push_line("else"),

			Stmt::End => self.push_line("end"),
		};

		if matches!(
			stmt,
			Stmt::NumFor { .. } | Stmt::GenFor { .. } | Stmt::If(..) | Stmt::ElseIf(..) | Stmt::Else
		) {
			self.indent();
		};
	}

	fn push_stmts(&mut self, stmts: &[Stmt]) {
		for stmt in stmts {
			self.push_stmt(stmt);
		}
	}

	fn push_tydecl(&mut self, tydecl: &TyDecl) {
		let name = &tydecl.name;
		let ty = &tydecl.ty;

		self.push_line(&format!("function types.read_{name}()"));
		self.indent();
		self.push_line("local value;");
		let statements = &des::gen(
			std::iter::once(ty),
			&get_unnamed_values("value", 1),
			true,
			&mut self.var_occurrences,
		);
		self.push_stmts(statements);
		self.push_line("return value");
		self.dedent();
		self.push_line("end");
	}

	fn push_tydecls(&mut self) {
		self.push_line("local types = {}");

		for tydecl in self.config.tydecls.iter() {
			self.push_tydecl(tydecl);
		}
	}

	fn push_event_callback(&mut self, first: bool, ev: &EvDecl) {
		let id = ev.id;

		self.push_indent();

		if first {
			self.push("if ");
		} else {
			self.push("elseif ");
		}

		// push_line is not used here as indent was pushed above
		// and we don't want to push it twice, especially after
		// the if/elseif
		self.push(&format!("id == {id} then"));
		self.push("\n");

		self.indent();

		let values = get_unnamed_values("value", ev.data.len());

		self.push_line(&format!("local {}", values.join(", ")));

		if !ev.data.is_empty() {
			let statements = &des::gen(
				ev.data.iter().map(|parameter| &parameter.ty),
				&values,
				true,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
		}

		self.push_line("table.insert(events, {");
		self.indent();

		self.push_line(&format!("Name = \"{}\",", ev.name));

		self.push_indent();
		self.push("Arguments = { ");

		if self.config.tooling_show_internal_data {
			self.push(&format!(
				"{{ {} = id }}, ",
				self.config.casing.with("EventId", "eventId", "event_id")
			));
		}

		self.push(&format!("{} }}", values.join(", ")));
		self.push("\n");

		self.dedent();
		self.push_line("})");

		self.dedent();
	}

	fn push_function_callback(&mut self, first: bool, is_server: bool, fn_decl: &FnDecl) {
		let id = if is_server {
			fn_decl.server_id
		} else {
			fn_decl.client_id
		};
		let event_id = self.config.casing.with("EventId", "eventId", "event_id");
		let call_id = self.config.casing.with("CallId", "callId", "call_id");

		self.push_indent();

		if first {
			self.push("if ");
		} else {
			self.push("elseif ");
		}

		// push_line is not used here as indent was pushed above
		// and we don't want to push it twice, especially after
		// the if/elseif
		self.push(&format!("id == {id} then"));
		self.push("\n");

		self.indent();

		self.push_line("local call_id = buffer.readu8(incoming_buff, read(1))");

		if is_server {
			let values = get_unnamed_values("value", fn_decl.args.len());
			self.push_line(&format!("local {}", values.join(", ")));

			if !fn_decl.args.is_empty() {
				let statements = &des::gen(
					fn_decl.args.iter().map(|parameter| &parameter.ty),
					&values,
					true,
					&mut self.var_occurrences,
				);
				self.push_stmts(statements);
			}

			self.push_line("table.insert(events, {");
			self.indent();

			self.push_line(&format!("Name = \"{} (request)\",", fn_decl.name));

			self.push_indent();
			self.push("Arguments = { ");

			if self.config.tooling_show_internal_data {
				self.push(&format!("{{ {} = id, {} = call_id }}, ", event_id, call_id));
			}

			self.push(&format!("{} }}", values.join(", ")));
			self.push("\n");

			self.dedent();
			self.push_line("})");
		} else {
			let values = get_unnamed_values("value", fn_decl.rets.as_ref().unwrap_or(&vec![]).len());
			self.push_line(&format!("local {}", values.join(", ")));

			if let Some(data) = &fn_decl.rets {
				let statements = &des::gen(
					data,
					&get_unnamed_values("value", data.len()),
					true,
					&mut self.var_occurrences,
				);
				self.push_stmts(statements);
			}

			self.push_line("table.insert(events, {");
			self.indent();

			self.push_line(&format!("Name = \"{} (callback)\",", fn_decl.name));

			self.push_indent();
			self.push("Arguments = { ");

			if self.config.tooling_show_internal_data {
				self.push(&format!("{{ {} = id, {} = call_id }}, ", event_id, call_id));
			}

			self.push(&format!("{} }}", values.join(", ")));
			self.push("\n");

			self.dedent();
			self.push_line("})");
		}

		self.dedent();
	}

	pub fn output(mut self) -> String {
		self.push_line("--!native");
		self.push_line("--!optimize 2");
		self.push_line("--!nocheck");
		self.push_line("--!nolint");
		self.push_line("--#selene: allow(if_same_then_else, unused_variable)");

		self.push_line(&format!(
			"-- Tooling generated by Zap v{} (https://github.com/red-blox/zap)",
			env!("CARGO_PKG_VERSION")
		));

		// if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
		// 	return self.buf;
		// };

		self.push_line("local ReplicatedStorage = game:GetService(\"ReplicatedStorage\")");
		self.push("\n");

		self.push_line("-- thanks to https://dom.rojo.space/binary.html#cframe");
		self.push_line("local CFrameSpecialCases = {");
		self.indent();

		self.push_line("CFrame.Angles(0, 0, 0),");
		self.push_line("CFrame.Angles(math.rad(90), 0, 0),");
		self.push_line("CFrame.Angles(0, math.rad(180), math.rad(180)),");
		self.push_line("CFrame.Angles(math.rad(-90), 0, 0),");
		self.push_line("CFrame.Angles(0, math.rad(180), math.rad(90)),");
		self.push_line("CFrame.Angles(0, math.rad(90), math.rad(90)),");
		self.push_line("CFrame.Angles(0, 0, math.rad(90)),");
		self.push_line("CFrame.Angles(0, math.rad(-90), math.rad(90)),");
		self.push_line("CFrame.Angles(math.rad(-90), math.rad(-90), 0),");
		self.push_line("CFrame.Angles(0, math.rad(-90), 0),");
		self.push_line("CFrame.Angles(math.rad(90), math.rad(-90), 0),");
		self.push_line("CFrame.Angles(0, math.rad(90), math.rad(180)),");
		self.push_line("CFrame.Angles(0, math.rad(-90), math.rad(180)),");
		self.push_line("CFrame.Angles(0, math.rad(180), math.rad(0)),");
		self.push_line("CFrame.Angles(math.rad(-90), math.rad(-180), math.rad(0)),");
		self.push_line("CFrame.Angles(0, math.rad(0), math.rad(180)),");
		self.push_line("CFrame.Angles(math.rad(90), math.rad(180), math.rad(0)),");
		self.push_line("CFrame.Angles(0, math.rad(0), math.rad(-90)),");
		self.push_line("CFrame.Angles(0, math.rad(-90), math.rad(-90)),");
		self.push_line("CFrame.Angles(0, math.rad(-180), math.rad(-90)),");
		self.push_line("CFrame.Angles(0, math.rad(90), math.rad(-90)),");
		self.push_line("CFrame.Angles(math.rad(90), math.rad(90), 0),");
		self.push_line("CFrame.Angles(0, math.rad(90), 0),");
		self.push_line("CFrame.Angles(math.rad(-90), math.rad(90), 0),");

		self.dedent();
		self.push_line("}\n");

		self.push_line("return function(remote_instance, player, incoming_buff, incoming_inst)");
		self.indent();

		self.push_line(&format!(
			"local remotes = ReplicatedStorage:FindFirstChild(\"{}\")",
			self.config.remote_folder
		));

		self.push_line("if not remotes then");
		self.indent();

		self.push_line("return");

		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line(&format!(
			"local reliable = remotes:FindFirstChild(\"{}_RELIABLE\")",
			self.config.remote_scope
		));
		self.push_line(&format!(
			"local unreliable = remotes:FindFirstChild(\"{}_UNRELIABLE\")",
			self.config.remote_scope
		));
		self.push("\n");

		self.push_line("if not reliable or not unreliable then");
		self.indent();

		self.push_line("return");

		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line("if remote_instance ~= reliable and remote_instance ~= unreliable then");
		self.indent();

		self.push_line("return");

		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line("local isServer = true");
		self.push_line("if type(player) == \"buffer\" then");
		self.indent();

		self.push_line("isServer = false");
		self.push_line("incoming_inst = incoming_buff");
		self.push_line("incoming_buff = player");
		self.push_line("player = nil");

		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line("local incoming_read = 0");
		self.push_line("local incoming_ipos = 0");

		self.push_line("local function read(len: number)");
		self.indent();

		self.push_line("local pos = incoming_read");
		self.push_line("incoming_read = incoming_read + len");
		self.push("\n");
		self.push_line("return pos");

		self.dedent();
		self.push_line("end");
		self.push_line("local len = buffer.len(incoming_buff)");
		self.push("\n");

		self.push_tydecls();
		self.push("\n");

		self.push_line("local events = {}");
		self.push_line("while incoming_read < len do");

		self.indent();

		self.push_line("if isServer and remote_instance == reliable then");
		self.indent();
		self.push_events(
			true,
			self.config.server_reliable_ty(),
			EvSource::Client,
			EvType::Reliable,
		);
		self.dedent();

		self.push_line("elseif isServer and remote_instance == unreliable then");
		self.indent();
		self.push_events(
			true,
			self.config.server_unreliable_ty(),
			EvSource::Client,
			EvType::Unreliable,
		);
		self.dedent();

		self.push_line("elseif not isServer and remote_instance == reliable then");
		self.indent();
		self.push_events(
			false,
			self.config.client_reliable_ty(),
			EvSource::Server,
			EvType::Reliable,
		);
		self.dedent();

		self.push_line("elseif not isServer and remote_instance == unreliable then");
		self.indent();
		self.push_events(
			false,
			self.config.client_unreliable_ty(),
			EvSource::Server,
			EvType::Unreliable,
		);
		self.dedent();

		self.push_line("end");

		self.dedent();
		self.push_line("end");

		self.push("\n");
		self.push_line("return events");

		self.dedent();
		self.push_line("end");

		self.buf
	}

	fn push_events(
		&mut self,
		is_server: bool,
		event_ty: NumTy,
		expected_ev_source: EvSource,
		expected_ev_type: EvType,
	) {
		self.push_line(&format!(
			"local id = buffer.read{}(incoming_buff, read({}))",
			event_ty,
			event_ty.size()
		));

		let mut first = true;

		for ev in self.config.evdecls.iter() {
			if ev.from != expected_ev_source || ev.evty != expected_ev_type {
				continue;
			}

			self.push_event_callback(first, ev);
			first = false;
		}

		if expected_ev_type == EvType::Reliable {
			for fn_decl in self.config.fndecls.iter() {
				self.push_function_callback(first, is_server, fn_decl);
				first = false;
			}
		}

		if !first {
			self.push_line("else");
			self.indent();
		}

		self.push_line("error(\"Unknown event id\")");

		if !first {
			self.dedent();
			self.push_line("end");
		}
	}
}

pub fn output(config: &Config) -> Option<Output> {
	if !config.tooling {
		return None;
	}

	#[cfg(not(target_arch = "wasm32"))]
	let output = Output {
		code: ToolingOutput::new(config).output(),
		defs: None,
		path: config.tooling_output.into(),
	};

	#[cfg(target_arch = "wasm32")]
	let output = Output {
		code: ToolingOutput::new(config).output(),
		defs: None,
	};

	Some(output)
}
