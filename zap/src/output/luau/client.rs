use crate::{
	config::{Config, EvCall, EvDecl, EvSource, EvType, TyDecl},
	irgen::{des, ser},
};

use super::Output;

struct ClientOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl<'a> Output for ClientOutput<'a> {
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

impl<'src> ClientOutput<'src> {
	pub fn new(config: &'src Config<'src>) -> Self {
		Self {
			config,
			tabs: 0,
			buf: String::new(),
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
		self.push_stmts(&ser::gen(ty, "value", self.config.write_checks));
		self.dedent();
		self.push_line("end");

		self.push_line(&format!("function types.read_{name}()"));
		self.indent();
		self.push_line("local value;");
		self.push_stmts(&des::gen(ty, "value", false));
		self.push_line("return value");
		self.dedent();
		self.push_line("end");
	}

	fn push_tydecls(&mut self) {
		for tydecl in &self.config.tydecls {
			self.push_tydecl(tydecl);
		}
	}

	fn push_reliable_header(&mut self) {
		self.push_line("reliable.OnClientEvent:Connect(function(buff, inst)");
		self.indent();
		self.push_line("incoming_buff = buff");
		self.push_line("incoming_inst = inst");
		self.push_line("incoming_read = 0");

		self.push_line("local len = buffer.len(buff)");
		self.push_line("while incoming_read < len do");

		self.indent();

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.config.event_id_ty(),
			self.config.event_id_ty().size()
		));
	}
	fn push_reliable_callback(&mut self, first: bool, ev: &EvDecl, id: usize) {
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

		self.push_line("local value");
		self.push_stmts(&des::gen(&ev.data, "value", true));

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"))
		} else {
			self.push_line(&format!("for _, cb in events[{id}] do"))
		}

		self.indent();

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}](value)")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(events[{id}], value)")),
			EvCall::ManySync => self.push_line("cb(value)"),
			EvCall::ManyAsync => self.push_line("task.spawn(cb, value)"),
		}

		self.dedent();
		self.push_line("end");

		self.dedent();
	}

	fn push_reliable_footer(&mut self) {
		self.push_line("else");
		self.indent();
		self.push_line("error(\"Unknown event id\")");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end)");
	}

	fn push_reliable(&mut self) {
		self.push_reliable_header();

		let mut first = true;

		for (i, ev) in self
			.config
			.evdecls
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
		self.push_line("unreliable.OnClientEvent:Connect(function(buff, inst)");
		self.indent();
		self.push_line("incoming_buff = buff");
		self.push_line("incoming_inst = inst");
		self.push_line("incoming_read = 0");

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.config.event_id_ty(),
			self.config.event_id_ty().size()
		));
	}

	fn push_unreliable_callback(&mut self, first: bool, ev: &EvDecl, id: usize) {
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

		self.push_line("local value");
		self.push_stmts(&des::gen(&ev.data, "value", self.config.write_checks));

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"))
		} else {
			self.push_line(&format!("for _, cb in events[{id}] do"))
		}

		self.indent();

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}](value)")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(events[{id}], value)")),
			EvCall::ManySync => self.push_line("cb(value)"),
			EvCall::ManyAsync => self.push_line("task.spawn(cb, value)"),
		}

		self.dedent();
		self.push_line("end");

		self.dedent();
	}

	fn push_unreliable_footer(&mut self) {
		self.push_line("else");
		self.indent();
		self.push_line("error(\"Unknown event id\")");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end)");
	}

	fn push_unreliable(&mut self) {
		self.push_unreliable_header();

		let mut first = true;

		for (i, ev) in self
			.config
			.evdecls
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
		self.push_line(&format!("local events = table.create({})", self.config.evdecls.len()));

		for (i, _) in self.config.evdecls.iter().enumerate().filter(|(_, ev_decl)| {
			ev_decl.from == EvSource::Server && matches!(ev_decl.call, EvCall::ManyAsync | EvCall::ManySync)
		}) {
			let id = i + 1;

			self.push_line(&format!("events[{id}] = {{}}"));
		}
	}

	fn push_write_event_id(&mut self, id: usize) {
		self.push_line(&format!("local pos = alloc({})", self.config.event_id_ty().size()));
		self.push_line(&format!(
			"buffer.write{}(outgoing_buff, pos, {id})",
			self.config.event_id_ty()
		));
	}

	fn push_return_fire(&mut self, ev: &EvDecl, id: usize) {
		let ty = &ev.data;

		let fire = self.config.casing.with("Fire", "fire", "fire");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire} = function({value}: "));
		self.push_ty(ty);
		self.push(")\n");
		self.indent();

		if ev.evty == EvType::Unreliable {
			self.push_line("local saved = save()");
			self.push_line("load_empty()");
		}

		self.push_write_event_id(id);

		self.push_stmts(&ser::gen(ty, value, self.config.write_checks));

		if ev.evty == EvType::Unreliable {
			self.push_line("local buff = buffer.create(outgoing_used)");
			self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
			self.push_line("unreliable:FireServer(buff, outgoing_inst)");
			self.push_line("load(saved)");
		}

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_outgoing(&mut self) {
		for (i, ev) in self
			.config
			.evdecls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			let id = i + 1;

			self.push_line(&format!("{name} = {{", name = ev.name));
			self.indent();

			self.push_return_fire(ev, id);

			self.dedent();
			self.push_line("},");
		}
	}

	fn push_return_setcallback(&mut self, ev: &EvDecl, id: usize) {
		let ty = &ev.data;

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{set_callback} = function({callback}: ("));
		self.push_ty(ty);
		self.push(") -> ())\n");
		self.indent();

		self.push_line(&format!("events[{id}] = {callback}"));

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_on(&mut self, ev: &EvDecl, id: usize) {
		let ty = &ev.data;

		let on = self.config.casing.with("On", "on", "on");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{on} = function({callback}: ("));
		self.push_ty(ty);
		self.push(") -> ())\n");
		self.indent();

		self.push_line(&format!("table.insert(events[{id}], {callback})"));

		self.dedent();
		self.push_line("end,");
	}

	pub fn push_return_listen(&mut self) {
		for (i, ev) in self
			.config
			.evdecls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
		{
			let id = i + 1;

			self.push_line(&format!("{name} = {{", name = ev.name));
			self.indent();

			match ev.call {
				EvCall::SingleSync | EvCall::SingleAsync => self.push_return_setcallback(ev, id),
				EvCall::ManySync | EvCall::ManyAsync => self.push_return_on(ev, id),
			}

			self.dedent();
			self.push_line("},");
		}
	}

	pub fn push_return(&mut self) {
		self.push_line("return {");
		self.indent();

		self.push_return_outgoing();
		self.push_return_listen();

		self.dedent();
		self.push_line("}");
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Client");

		if self.config.evdecls.is_empty() {
			return self.buf;
		};

		self.push(include_str!("base.luau"));
		self.push(include_str!("client.luau"));

		self.push_tydecls();

		self.push_callback_lists();

		if self
			.config
			.evdecls
			.iter()
			.any(|ev| ev.evty == EvType::Reliable && ev.from == EvSource::Server)
		{
			self.push_reliable();
		}

		if self
			.config
			.evdecls
			.iter()
			.any(|ev| ev.evty == EvType::Unreliable && ev.from == EvSource::Server)
		{
			self.push_unreliable();
		}

		self.push_return();

		self.buf
	}
}

pub fn code(config: &Config) -> String {
	ClientOutput::new(config).output()
}
