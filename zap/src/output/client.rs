use crate::{
	irgen::{gen_des, gen_ser, Stmt},
	parser::{EvCall, EvDecl, EvSource, EvType, File, TyDecl},
	util::casing,
};

struct ClientOutput<'a> {
	file: &'a File,
	buff: String,
	tabs: u32,
}

impl<'a> ClientOutput<'a> {
	pub fn new(file: &'a File) -> Self {
		Self {
			file,
			buff: String::new(),
			tabs: 0,
		}
	}

	fn push(&mut self, s: &str) {
		self.buff.push_str(s);
	}

	fn push_tab(&mut self) {
		for _ in 0..self.tabs {
			self.buff.push('\t');
		}
	}

	fn push_line(&mut self, s: &str) {
		self.push_tab();
		self.push(s);
		self.buff.push('\n');
	}

	fn push_stmt(&mut self, stmt: &Stmt) {
		if matches!(stmt, Stmt::ElseIf(..) | Stmt::Else | Stmt::End) {
			self.tabs -= 1;
		}

		match &stmt {
			Stmt::Local(name, expr) => {
				if let Some(expr) = expr {
					self.push_line(&format!("local {name} = {expr}"));
				} else {
					self.push_line(&format!("local {name}"));
				}
			}

			Stmt::Assign(var, expr) => self.push_line(&format!("{var} = {expr}")),
			Stmt::Error(msg) => self.push_line(&format!("error({msg})")),
			Stmt::Assert(cond, msg) => match msg {
				Some(msg) => self.push_line(&format!("assert({cond}, {msg})")),
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
			self.tabs += 1;
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

		self.push_line(&format!("export type {name} = {ty}"));

		self.push_line(&format!("function types.write_{name}(value: {name})"));
		self.push_stmts(&gen_ser(ty, "value".into(), self.file.write_checks));
		self.push_line("end");

		self.push_line(&format!("function types.read_{name}()"));
		self.push_line("local value;");
		self.push_stmts(&gen_des(ty, "value".into(), false));
		self.push_line("return value");
		self.push_line("end");
	}

	fn push_tydecls(&mut self) {
		for tydecl in self.file.ty_decls.iter() {
			self.push_tydecl(tydecl);
		}
	}

	fn push_reliable_header(&mut self) {
		self.push_line("reliable.OnClientEvent:Connect(function(buff, inst)");
		self.tabs += 1;
		self.push_line("incoming_buff = buff");
		self.push_line("incoming_inst = inst");
		self.push_line("incoming_read = 0");

		self.push_line("local len = buffer.len(buff)");
		self.push_line("while incoming_read < len do");

		self.tabs += 1;

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.file.event_id_ty(),
			self.file.event_id_ty().size()
		));
	}
	fn push_reliable_callback(&mut self, first: bool, ev: &EvDecl, id: usize) {
		self.push_tab();

		if first {
			self.push("if ");
		} else {
			self.push("elseif ");
		}

		self.push(&format!("id == {id} then"));
		self.push("\n");

		self.tabs += 1;

		self.push_line("local value");
		self.push_stmts(&gen_des(&ev.data, "value".into(), false));

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("if events[{id}] then events[{id}](value) end")),
			EvCall::SingleAsync => self.push_line(&format!("if events[{id}] then task.spawn(events[{id}], value) end")),
			EvCall::ManySync => self.push_line(&format!("for _, cb in events[{id}] do cb(value) end")),
			EvCall::ManyAsync => self.push_line(&format!("for _, cb in events[{id}] do task.spawn(cb, value) end")),
		}

		self.tabs -= 1;
	}

	fn push_reliable_footer(&mut self) {
		self.push_line("else");
		self.tabs += 1;
		self.push_line("error(\"Unknown event id\")");
		self.tabs -= 1;
		self.push_line("end");
		self.tabs -= 1;
		self.push_line("end");
		self.tabs -= 1;
		self.push_line("end)");
	}

	fn push_reliable(&mut self) {
		self.push_reliable_header();

		let mut first = true;

		for (i, ev) in self
			.file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Reliable)
		{
			let id = i + 1;

			self.push_reliable_callback(first, ev, id);
			first = false;
		}

		self.push_reliable_footer();
	}

	fn push_unreliable_header(&mut self) {
		self.push_line("unreliable.OnServerEvent:Connect(function(buff, inst)");
		self.tabs += 1;
		self.push_line("incoming_buff = buff");
		self.push_line("incoming_inst = inst");
		self.push_line("incoming_read = 0");

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.file.event_id_ty(),
			self.file.event_id_ty().size()
		));
	}

	fn push_unreliable_callback(&mut self, first: bool, ev: &EvDecl, id: usize) {
		self.push_tab();

		if first {
			self.push("if ");
		} else {
			self.push("elseif ");
		}

		self.push(&format!("id == {id} then"));
		self.push("\n");

		self.tabs += 1;

		self.push_line("local value");
		self.push_stmts(&gen_des(&ev.data, "value".into(), true));

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("if events[{id}] then events[{id}](value) end")),
			EvCall::SingleAsync => self.push_line(&format!("if events[{id}] then task.spawn(events[{id}], value) end")),
			EvCall::ManySync => self.push_line(&format!("for _, cb in events[{id}] do cb(value) end")),
			EvCall::ManyAsync => self.push_line(&format!("for _, cb in events[{id}] do task.spawn(cb, value) end")),
		}

		self.tabs -= 1;
	}

	fn push_unreliable_footer(&mut self) {
		self.push_line("else");
		self.tabs += 1;
		self.push_line("error(\"Unknown event id\")");
		self.tabs -= 1;
		self.push_line("end");
		self.tabs -= 1;
		self.push_line("end)");
	}

	fn push_unreliable(&mut self) {
		self.push_unreliable_header();

		let mut first = true;

		for (i, ev) in self
			.file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Unreliable)
		{
			let id = i + 1;

			self.push_unreliable_callback(first, ev, id);
			first = false;
		}

		self.push_unreliable_footer();
	}

	fn push_callback_lists(&mut self) {
		for (i, _) in self.file.ev_decls.iter().enumerate().filter(|(_, ev_decl)| {
			ev_decl.from == EvSource::Server && matches!(ev_decl.call, EvCall::ManyAsync | EvCall::ManySync)
		}) {
			let id = i + 1;

			self.push_line(&format!("events[{id}] = {{}}"));
		}
	}

	fn push_return_fire(&mut self, ev: &EvDecl, id: usize) {
		let ty = &ev.data;

		let fire = casing(self.file.casing, "Fire", "fire", "fire");
		let value = casing(self.file.casing, "Value", "value", "value");

		self.push_line(&format!("{fire} = function({value}: {ty})"));
		self.tabs += 1;

		if ev.evty == EvType::Unreliable {
			self.push_line("local saved = save()");
			self.push_line("load_empty()");
		}

		self.push_line(&format!("buffer.write{}(outgoing_buff, {id})", self.file.event_id_ty()));
		self.push_stmts(&gen_ser(ty, value.into(), self.file.write_checks));

		if ev.evty == EvType::Unreliable {
			self.push_line("local buff = buffer.create(outgoing_used)");
			self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
			self.push_line("unreliable:FireServer(buff, outgoing_inst)");
			self.push_line("load(saved)");
		}

		self.tabs -= 1;
		self.push_line("end,");
	}

	fn push_return_outgoing(&mut self) {
		for (i, ev) in self
			.file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			let id = i + 1;

			self.push_line(&format!("{name} = {{", name = ev.name));
			self.tabs += 1;

			self.push_return_fire(ev, id);

			self.tabs -= 1;
			self.push_line("}},");
		}
	}

	fn push_return_setcallback(&mut self, ev: &EvDecl, id: usize) {
		let ty = &ev.data;

		let set_callback = casing(self.file.casing, "SetCallback", "setCallback", "set_callback");
		let callback = casing(self.file.casing, "Callback", "callback", "callback");

		self.push_line(&format!("{set_callback} = function({callback}: ({ty}) -> ())",));
		self.tabs += 1;

		self.push_line(&format!("events[{id}] = {callback}"));

		self.tabs -= 1;
		self.push_line("end,");
	}

	fn push_return_on(&mut self, ev: &EvDecl, id: usize) {
		let ty = &ev.data;

		let on = casing(self.file.casing, "On", "on", "on");
		let callback = casing(self.file.casing, "Callback", "callback", "callback");

		self.push_line(&format!("{on} = function({callback}: ({ty}) -> ())",));
		self.tabs += 1;

		self.push_line(&format!("table.insert(events[{id}], {callback})",));

		self.tabs -= 1;
		self.push_line("end,");
	}

	pub fn push_return_listen(&mut self) {
		for (i, ev) in self
			.file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			let id = i + 1;

			self.push_line(&format!("{name} = {{", name = ev.name));
			self.tabs += 1;

			match ev.call {
				EvCall::SingleSync | EvCall::SingleAsync => self.push_return_setcallback(ev, id),
				EvCall::ManySync | EvCall::ManyAsync => self.push_return_on(ev, id),
			}

			self.tabs -= 1;
			self.push_line("}},");
		}
	}

	pub fn push_return(&mut self) {
		self.push_line("return {");
		self.tabs += 1;

		self.push_return_outgoing();
		self.push_return_listen();

		self.tabs -= 1;
		self.push_line("}");
	}

	pub fn output(&mut self) {
		self.push_line("--!native");
		self.push_line("--!optimize 2");

		self.push_line(&format!("-- Generated by Zap v{}", env!("CARGO_PKG_VERSION")));

		self.push(include_str!("base.luau"));
		self.push(include_str!("client.luau"));

		self.push_tydecls();

		self.push_reliable();
		self.push_unreliable();

		self.push_callback_lists();

		self.push_return();
	}

	pub fn into_inner(self) -> String {
		self.buff
	}
}

pub fn code(file: &File) -> String {
	let mut output = ClientOutput::new(file);
	output.output();
	output.into_inner()
}
