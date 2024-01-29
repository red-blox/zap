use crate::config::{Config, EvCall, EvDecl, EvSource, EvType, FnDecl, TyDecl, YieldType};

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
		self.push_ser("value", ty, self.config.write_checks);
		self.dedent();
		self.push_line("end");

		self.push_line(&format!("function types.read_{name}()"));
		self.indent();
		self.push_line("local value;");
		self.push_des("value", ty, false);
		self.push_line("return value");
		self.dedent();
		self.push_line("end");
	}

	fn push_tydecls(&mut self) {
		for tydecl in &self.config.tydecls {
			self.push_tydecl(tydecl);
		}
	}

	fn push_event_loop(&mut self) {
		self.push("\n");

		if self.config.manual_event_loop {
			let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

			self.push_line(&format!("local function {send_events}()"));
			self.indent();
		} else {
			self.push_line("RunService.Heartbeat:Connect(function(dt)");
			self.indent();
			self.push_line("time += dt");
			self.push("\n");
			self.push_line("if time >= (1 / 61) then");
			self.indent();
			self.push_line("time -= (1 / 61)");
			self.push("\n");
		}

		self.push_line("if outgoing_used ~= 0 then");
		self.indent();
		self.push_line("local buff = buffer.create(outgoing_used)");
		self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
		self.push("\n");
		self.push_line("reliable:FireServer(buff, outgoing_inst)");
		self.push("\n");
		self.push_line("outgoing_buff = buffer.create(64)");
		self.push_line("outgoing_used = 0");
		self.push_line("outgoing_size = 64");
		self.push_line("table.clear(outgoing_inst)");
		self.dedent();
		self.push_line("end");
		self.dedent();

		if self.config.manual_event_loop {
			self.push_line("end");
		} else {
			self.push_line("end");
			self.dedent();
			self.push_line("end)");
		}

		self.push("\n");
	}

	fn push_reliable_header(&mut self) {
		self.push_line("reliable.OnClientEvent:Connect(function(buff, inst)");
		self.indent();
		self.push_line("incoming_buff = buff");
		self.push_line("incoming_inst = inst");
		self.push_line("incoming_read = 0");
		self.push_line("incoming_ipos = 0");

		self.push_line("local len = buffer.len(buff)");
		self.push_line("while incoming_read < len do");

		self.indent();

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.config.event_id_ty(),
			self.config.event_id_ty().size()
		));
	}

	fn push_reliable_callback(&mut self, first: bool, ev: &EvDecl) {
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

		self.push_line("local value");

		if let Some(data) = &ev.data {
			self.push_des("value", data, false);
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"));
		} else {
			self.push_line(&format!("if events[{id}][1] then"));
		}

		self.indent();

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.push_line(&format!("for _, cb in events[{id}] do"));
			self.indent();
		}

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}](value)")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(events[{id}], value)")),
			EvCall::ManySync => self.push_line("cb(value)"),
			EvCall::ManyAsync => self.push_line("task.spawn(cb, value)"),
		}

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.dedent();
			self.push_line("end");
		}

		self.dedent();
		self.push_line("else");
		self.indent();

		if ev.data.is_some() {
			self.push_line(&format!("table.insert(event_queue[{id}], value)"));
			self.push_line(&format!("if #event_queue[{id}] > 64 then"));
		} else {
			self.push_line(&format!("event_queue[{id}] += 1"));
			self.push_line(&format!("if event_queue[{id}] > 16 then"));
		}

		self.indent();

		self.push_line(&format!(
			"warn(`[ZAP] {{#event_queue[{id}]}} events in queue for {}. Did you forget to attach a listener?`)",
			ev.name
		));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end");

		self.dedent();
	}

	fn push_fn_callback(&mut self, first: bool, fndecl: &FnDecl) {
		let id = fndecl.id;

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

		self.push_line("local value");

		if let Some(data) = &fndecl.rets {
			self.push_des("value", data, false);
		}

		match self.config.yield_type {
			YieldType::Yield | YieldType::Future => {
				self.push_line(&format!("task.spawn(event_queue[{id}][call_id], value)"));
			}
			YieldType::Promise => {
				self.push_line(&format!("event_queue[{id}][call_id](value)"));
			}
		}

		self.push_line(&format!("event_queue[{id}][call_id] = nil"));

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

		for evdecl in self
			.config
			.evdecls
			.iter()
			.filter(|evdecl| evdecl.from == EvSource::Server && evdecl.evty == EvType::Reliable)
		{
			self.push_reliable_callback(first, evdecl);
			first = false;
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_fn_callback(first, fndecl);
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
		self.push_line("incoming_ipos = 0");

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.config.event_id_ty(),
			self.config.event_id_ty().size()
		));
	}

	fn push_unreliable_callback(&mut self, first: bool, ev: &EvDecl) {
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

		self.push_line("local value");

		if let Some(data) = &ev.data {
			self.push_des("value", data, false);
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"));
		} else {
			self.push_line(&format!("if events[{id}][1] then"));
		}

		self.indent();

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.push_line(&format!("for _, cb in events[{id}] do"));
			self.indent();
		}

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}](value)")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(events[{id}], value)")),
			EvCall::ManySync => self.push_line("cb(value)"),
			EvCall::ManyAsync => self.push_line("task.spawn(cb, value)"),
		}

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.dedent();
			self.push_line("end");
		}

		self.dedent();
		self.push_line("else");
		self.indent();

		if ev.data.is_some() {
			self.push_line(&format!("table.insert(event_queue[{id}], value)"));
			self.push_line(&format!("if #event_queue[{id}] > 64 then"));
		} else {
			self.push_line(&format!("event_queue[{id}] += 1"));
			self.push_line(&format!("if event_queue[{id}] > 64 then"));
		}

		self.indent();

		self.push_line(&format!(
			"warn(`[ZAP] {{#event_queue[{}]}} events in queue for {}. Did you forget to attach a listener?`)",
			ev.id, ev.name
		));

		self.dedent();
		self.push_line("end");

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

		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Unreliable)
		{
			self.push_unreliable_callback(first, ev);
			first = false;
		}

		self.push_unreliable_footer();
	}

	fn push_callback_lists(&mut self) {
		self.push_line(&format!(
			"local events = table.create({})",
			self.config.evdecls.len() + self.config.fndecls.len()
		));
		self.push_line(&format!(
			"local event_queue: {{ [number]: {{ any }} }} = table.create({})",
			self.config.evdecls.len() + self.config.fndecls.len()
		));

		if !self.config.fndecls.is_empty() {
			self.push_line("local function_call_id = 0");

			if !self.config.async_lib.is_empty() {
				if self.config.typescript {
					self.push_line(&format!("local Promise = {}.Promise", self.config.async_lib))
				} else {
					self.push_line(&format!("local {} = {}", self.config.yield_type, self.config.async_lib))
				}
			}
		}

		for evdecl in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Server)
		{
			let id = evdecl.id;

			if evdecl.call == EvCall::ManyAsync || evdecl.call == EvCall::ManySync {
				self.push_line(&format!("events[{id}] = {{}}"));
			}

			if evdecl.data.is_some() {
				self.push_line(&format!("event_queue[{id}] = {{}}"));
			} else {
				self.push_line(&format!("event_queue[{id}] = 0"));
			}
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("event_queue[{}] = table.create(255)", fndecl.id));
		}
	}

	fn push_write_event_id(&mut self, id: usize) {
		self.push_line(&format!("alloc({})", self.config.event_id_ty().size()));
		self.push_line(&format!(
			"buffer.write{}(outgoing_buff, outgoing_apos, {id})",
			self.config.event_id_ty()
		));
	}

	fn push_return_fire(&mut self, ev: &EvDecl) {
		let fire = self.config.casing.with("Fire", "fire", "fire");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire} = function("));

		if let Some(data) = &ev.data {
			self.push(&format!("{value}: "));
			self.push_ty(data);
		}

		self.push(")\n");
		self.indent();

		if ev.evty == EvType::Unreliable {
			self.push_line("local saved = save()");
			self.push_line("load_empty()");
		}

		self.push_write_event_id(ev.id);

		if let Some(data) = &ev.data {
			self.push_ser(value, data, self.config.write_checks);
		}

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
		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Client)
		{
			self.push_line(&format!("{name} = {{", name = ev.name));
			self.indent();

			self.push_return_fire(ev);

			self.dedent();
			self.push_line("},");
		}
	}

	fn push_return_setcallback(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{set_callback} = function({callback}: ("));

		if let Some(data) = &ev.data {
			self.push_ty(data);
		}

		self.push(") -> ())\n");
		self.indent();

		self.push_line(&format!("events[{id}] = {callback}"));

		if ev.data.is_some() {
			self.push_line(&format!("for _, value in event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::SingleSync {
				self.push_line(&format!("{callback}(value)"))
			} else {
				self.push_line(&format!("task.spawn({callback}, value)"))
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = {{}}"));
		} else {
			self.push_line(&format!("for _ = 1, event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::SingleSync {
				self.push_line(&format!("{callback}()"))
			} else {
				self.push_line(&format!("task.spawn({callback})"))
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = 0"));
		}

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_on(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let on = self.config.casing.with("On", "on", "on");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{on} = function({callback}: ("));

		if let Some(data) = &ev.data {
			self.push_ty(data);
		}

		self.push(") -> ())\n");
		self.indent();

		self.push_line(&format!("table.insert(events[{id}], {callback})"));

		if ev.data.is_some() {
			self.push_line(&format!("for _, value in event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::ManySync {
				self.push_line(&format!("{callback}(value)"))
			} else {
				self.push_line(&format!("task.spawn({callback}, value)"))
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = {{}}"));
		} else {
			self.push_line(&format!("for _ = 1, event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::ManySync {
				self.push_line(&format!("{callback}()"))
			} else {
				self.push_line(&format!("task.spawn({callback})"))
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = 0"));
		}

		self.push_line("return function()");
		self.indent();

		self.push_line(&format!(
			"table.remove(events[{id}], table.find(events[{id}], {callback}))"
		));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end,");
	}

	pub fn push_return_listen(&mut self) {
		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Server)
		{
			self.push_line(&format!("{name} = {{", name = ev.name));
			self.indent();

			match ev.call {
				EvCall::SingleSync | EvCall::SingleAsync => self.push_return_setcallback(ev),
				EvCall::ManySync | EvCall::ManyAsync => self.push_return_on(ev),
			}

			self.dedent();
			self.push_line("},");
		}
	}

	fn push_return_functions(&mut self) {
		let call = self.config.casing.with("Call", "call", "call");
		let value = self.config.casing.with("Value", "value", "value");

		for fndecl in self.config.fndecls.iter() {
			let id = fndecl.id;

			self.push_line(&format!("{name} = {{", name = fndecl.name));
			self.indent();

			self.push_indent();
			self.push(&format!("{call} = function("));

			if let Some(ty) = &fndecl.args {
				self.push(&format!("{value}: "));
				self.push_ty(ty);
			}

			self.push(")\n");
			self.indent();

			self.push_write_event_id(fndecl.id);

			self.push_line("function_call_id += 1");

			self.push_line("function_call_id %= 256");

			self.push_line(&format!("if event_queue[{id}][function_call_id] then"));
			self.indent();

			self.push_line("function_call_id -= 1");
			self.push_line("error(\"Zap has more than 256 calls awaiting a response, and therefore this packet has been dropped\")");

			self.dedent();
			self.push_line("end");

			self.push_line("alloc(1)");
			self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, function_call_id)");

			if let Some(data) = &fndecl.args {
				self.push_ser(value, data, self.config.write_checks);
			}

			match self.config.yield_type {
				YieldType::Yield => {
					self.push_line(&format!("event_queue[{id}][function_call_id] = coroutine.running()",));
					self.push_line("local value = coroutine.yield()");
				}
				YieldType::Future => {
					self.push_line("local value = Future.new(function()");
					self.indent();

					self.push_line(&format!("event_queue[{id}][function_call_id] = coroutine.running()",));
					self.push_line("return coroutine.yield()");

					self.dedent();
					self.push_line("end)");
				}
				YieldType::Promise => {
					self.push_line("local value = Promise.new(function(resolve)");
					self.indent();

					self.push_line(&format!("event_queue[{id}][function_call_id] = resolve"));

					self.dedent();
					self.push_line("end)");
				}
			}

			self.push_line("return value");

			self.dedent();
			self.push_line("end,");

			self.dedent();
			self.push_line("},");
		}
	}

	pub fn push_return(&mut self) {
		self.push_line("return {");
		self.indent();

		if self.config.manual_event_loop {
			let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

			self.push_line(&format!("{send_events} = {send_events},"));
		}

		self.push_return_outgoing();
		self.push_return_listen();
		self.push_return_functions();

		self.dedent();
		self.push_line("}");
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Client");

		if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
			return self.buf;
		};

		self.push(include_str!("base.luau"));
		self.push(include_str!("client.luau"));

		self.push_tydecls();

		self.push_event_loop();

		self.push_callback_lists();

		if !self.config.fndecls.is_empty()
			|| self
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
