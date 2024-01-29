use crate::config::{Config, EvCall, EvDecl, EvSource, EvType, FnCall, FnDecl, TyDecl};

use super::Output;

struct ServerOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl<'a> Output for ServerOutput<'a> {
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

impl<'a> ServerOutput<'a> {
	pub fn new(config: &'a Config) -> Self {
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
		self.push_des("value", ty, true);
		self.push_line("return value");
		self.dedent();
		self.push_line("end");
	}

	fn push_tydecls(&mut self) {
		for tydecl in self.config.tydecls.iter() {
			self.push_tydecl(tydecl);
		}
	}

	fn push_event_loop(&mut self) {
		self.push("\n");

		if self.config.manual_event_loop {
			let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

			self.push_line(&format!("local function {send_events}()"));
		} else {
			self.push_line("RunService.Heartbeat:Connect(function()");
		}

		self.indent();
		self.push_line("for player, outgoing in player_map do");
		self.indent();
		self.push_line("if outgoing.used > 0 then");
		self.indent();
		self.push_line("local buff = buffer.create(outgoing.used)");
		self.push_line("buffer.copy(buff, 0, outgoing.buff, 0, outgoing.used)");
		self.push("\n");
		self.push_line("reliable:FireClient(player, buff, outgoing.inst)");
		self.push("\n");
		self.push_line("outgoing.buff = buffer.create(64)");
		self.push_line("outgoing.used = 0");
		self.push_line("outgoing.size = 64");
		self.push_line("table.clear(outgoing.inst)");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end");
		self.dedent();

		if self.config.manual_event_loop {
			self.push_line("end");
		} else {
			self.push_line("end)");
		}

		self.push("\n");
	}

	fn push_reliable_header(&mut self) {
		self.push_line("reliable.OnServerEvent:Connect(function(player, buff, inst)");
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
			self.push_des("value", data, true);
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"))
		} else {
			self.push_line(&format!("for _, cb in events[{id}] do"))
		}

		self.indent();

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}](player, value)")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(events[{id}], player, value)")),
			EvCall::ManySync => self.push_line("cb(player, value)"),
			EvCall::ManyAsync => self.push_line("task.spawn(cb, player, value)"),
		}

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

		self.push(&format!("id == {id} then"));
		self.push("\n");

		self.indent();

		self.push_line("local call_id = buffer.readu8(buff, read(1))");
		self.push_line("local value");

		if let Some(data) = &fndecl.args {
			self.push_des("value", data, true);
		}

		self.push_line(&format!("if events[{id}] then"));

		self.indent();

		if fndecl.call == FnCall::Async {
			self.push_line("task.spawn(function(player, call_id, value)");
			self.indent();
		}

		self.push_line(&format!("local rets = events[{id}](player, value)"));

		self.push_line("load_player(player)");
		self.push_write_event_id(fndecl.id);

		self.push_line("alloc(1)");
		self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, call_id)");

		if let Some(ty) = &fndecl.rets {
			self.push_ser("rets", ty, self.config.write_checks);
		}

		self.push_line("player_map[player] = save()");

		if fndecl.call == FnCall::Async {
			self.dedent();
			self.push_line("end, player, call_id, value)");
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

		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Client && ev_decl.evty == EvType::Reliable)
		{
			self.push_reliable_callback(first, ev);
			first = false;
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_fn_callback(first, fndecl);
			first = false;
		}

		self.push_reliable_footer();
	}

	fn push_unreliable_header(&mut self) {
		self.push_line("unreliable.OnServerEvent:Connect(function(player, buff, inst)");
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
			self.push_des("value", data, true);
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"))
		} else {
			self.push_line(&format!("for _, cb in events[{id}] do"))
		}

		self.indent();

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}](player, value)")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(events[{id}], player, value)")),
			EvCall::ManySync => self.push_line("cb(player, value)"),
			EvCall::ManyAsync => self.push_line("task.spawn(cb, player, value)"),
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

		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Client && ev_decl.evty == EvType::Unreliable)
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

		for evdecl in self.config.evdecls.iter().filter(|ev_decl| {
			ev_decl.from == EvSource::Client && matches!(ev_decl.call, EvCall::ManyAsync | EvCall::ManySync)
		}) {
			self.push_line(&format!("events[{}] = {{}}", evdecl.id));
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
		let ty = &ev.data;

		let fire = self.config.casing.with("Fire", "fire", "fire");
		let player = self.config.casing.with("Player", "player", "player");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire} = function({player}: Player"));

		if let Some(ty) = ty {
			self.push(&format!(", {value}: "));
			self.push_ty(ty);
		}

		self.push(")\n");
		self.indent();

		match ev.evty {
			EvType::Reliable => self.push_line(&format!("load_player({player})")),
			EvType::Unreliable => self.push_line("load_empty()"),
		}

		self.push_write_event_id(ev.id);

		if let Some(ty) = ty {
			self.push_ser(value, ty, self.config.write_checks);
		}

		match ev.evty {
			EvType::Reliable => self.push_line(&format!("player_map[{player}] = save()")),
			EvType::Unreliable => {
				self.push_line("local buff = buffer.create(outgoing_used)");
				self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
				self.push_line(&format!("unreliable:FireClient({player}, buff, outgoing_inst)"));
			}
		}

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_fire_all(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_all = self.config.casing.with("FireAll", "fireAll", "fire_all");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_all} = function("));

		if let Some(ty) = ty {
			self.push(&format!("{value}: "));
			self.push_ty(ty);
		}

		self.push(")\n");
		self.indent();

		self.push_line("load_empty()");

		self.push_write_event_id(ev.id);

		if let Some(ty) = ty {
			self.push_ser(value, ty, self.config.write_checks);
		}

		match ev.evty {
			EvType::Reliable => {
				self.push_line("local buff, used, inst = outgoing_buff, outgoing_used, outgoing_inst");
				self.push_line("for _, player in Players:GetPlayers() do");
				self.indent();
				self.push_line("load_player(player)");
				self.push_line("alloc(used)");
				self.push_line("buffer.copy(outgoing_buff, outgoing_apos, buff, 0, used)");
				self.push_line("table.move(inst, 1, #inst, #outgoing_inst + 1, outgoing_inst)");
				self.push_line("player_map[player] = save()");
				self.dedent();
				self.push_line("end");
			}

			EvType::Unreliable => {
				self.push_line("local buff = buffer.create(outgoing_used)");
				self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
				self.push_line("unreliable:FireAllClients(buff, outgoing_inst)")
			}
		}

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_fire_except(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_except = self.config.casing.with("FireExcept", "fireExcept", "fire_except");
		let except = self.config.casing.with("Except", "except", "except");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_except} = function({except}: Player"));

		if let Some(ty) = ty {
			self.push(&format!(", {value}: "));
			self.push_ty(ty);
		}

		self.push(")\n");
		self.indent();

		self.push_line("load_empty()");

		self.push_write_event_id(ev.id);

		if let Some(ty) = ty {
			self.push_ser(value, ty, self.config.write_checks);
		}

		match ev.evty {
			EvType::Reliable => {
				self.push_line("local buff, used, inst = outgoing_buff, outgoing_used, outgoing_inst");
				self.push_line("for _, player in Players:GetPlayers() do");
				self.indent();
				self.push_line(&format!("if player ~= {except} then"));
				self.indent();
				self.push_line("load_player(player)");
				self.push_line("alloc(used)");
				self.push_line("buffer.copy(outgoing_buff, outgoing_apos, buff, 0, used)");
				self.push_line("table.move(inst, 1, #inst, #outgoing_inst + 1, outgoing_inst)");
				self.push_line("player_map[player] = save()");
				self.dedent();
				self.push_line("end");
				self.dedent();
				self.push_line("end");
			}

			EvType::Unreliable => {
				self.push_line("local buff = buffer.create(outgoing_used)");
				self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
				self.push_line("for _, player in Players:GetPlayers() do");
				self.indent();
				self.push_line(&format!("if player ~= {except} then"));
				self.indent();
				self.push_line("unreliable:FireClient(player, buff, outgoing_inst)");
				self.dedent();
				self.push_line("end");
				self.dedent();
				self.push_line("end");
			}
		}

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_fire_list(&mut self, ev: &EvDecl) {
		let ty = &ev.data;

		let fire_list = self.config.casing.with("FireList", "fireList", "fire_list");
		let list = self.config.casing.with("List", "list", "list");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_list} = function({list}: {{ Player }}"));

		if let Some(ty) = ty {
			self.push(&format!(", {value}: "));
			self.push_ty(ty);
		}

		self.push(")\n");
		self.indent();

		self.push_line("load_empty()");

		self.push_write_event_id(ev.id);

		if let Some(ty) = ty {
			self.push_ser(value, ty, self.config.write_checks);
		}

		match ev.evty {
			EvType::Reliable => {
				self.push_line("local buff, used, inst = outgoing_buff, outgoing_used, outgoing_inst");
				self.push_line(&format!("for _, player in {list} do"));
				self.indent();
				self.push_line("load_player(player)");
				self.push_line("alloc(used)");
				self.push_line("buffer.copy(outgoing_buff, outgoing_apos, buff, 0, used)");
				self.push_line("table.move(inst, 1, #inst, #outgoing_inst + 1, outgoing_inst)");
				self.push_line("player_map[player] = save()");
				self.dedent();
				self.push_line("end");
			}

			EvType::Unreliable => {
				self.push_line("local buff = buffer.create(outgoing_used)");
				self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
				self.push_line(&format!("for _, player in {list} do"));
				self.indent();
				self.push_line("unreliable:FireClient(player, buff, outgoing_inst)");
				self.dedent();
				self.push_line("end");
			}
		}

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_outgoing(&mut self) {
		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Server)
		{
			self.push_line(&format!("{name} = {{", name = ev.name));
			self.indent();

			self.push_return_fire(ev);
			self.push_return_fire_all(ev);
			self.push_return_fire_except(ev);
			self.push_return_fire_list(ev);

			self.dedent();
			self.push_line("},");
		}
	}

	fn push_return_setcallback(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{set_callback} = function({callback}: (Player"));

		if let Some(ty) = &ev.data {
			self.push(", ");
			self.push_ty(ty);
		}

		self.push(") -> ())\n");
		self.indent();

		self.push_line(&format!("events[{id}] = {callback}"));

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_on(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let on = self.config.casing.with("On", "on", "on");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{on} = function({callback}: (Player"));

		if let Some(ty) = &ev.data {
			self.push(", ");
			self.push_ty(ty);
		}

		self.push(") -> ()): () -> ()\n");
		self.indent();

		self.push_line(&format!("table.insert(events[{id}], {callback})"));

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

	fn push_fn_return(&mut self, fndecl: &FnDecl) {
		let id = fndecl.id;

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{set_callback} = function({callback}: (Player"));

		if let Some(ty) = &fndecl.args {
			self.push(", ");
			self.push_ty(ty);
		}

		self.push(") -> (");

		if let Some(ty) = &fndecl.rets {
			self.push_ty(ty);
		}

		self.push("))\n");
		self.indent();

		self.push_line(&format!("events[{id}] = {callback}"));

		self.dedent();
		self.push_line("end,");
	}

	pub fn push_return_listen(&mut self) {
		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Client)
		{
			self.push_line(&format!("{} = {{", ev.name));
			self.indent();

			match ev.call {
				EvCall::SingleSync | EvCall::SingleAsync => self.push_return_setcallback(ev),
				EvCall::ManySync | EvCall::ManyAsync => self.push_return_on(ev),
			}

			self.dedent();
			self.push_line("},");
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("{} = {{", fndecl.name));
			self.indent();

			self.push_fn_return(fndecl);

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

		self.dedent();
		self.push_line("}");
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Server");

		if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
			return self.buf;
		};

		self.push(include_str!("base.luau"));
		self.push(include_str!("server.luau"));

		self.push_tydecls();

		self.push_event_loop();

		self.push_callback_lists();

		if !self.config.fndecls.is_empty()
			|| self
				.config
				.evdecls
				.iter()
				.any(|ev| ev.evty == EvType::Reliable && ev.from == EvSource::Client)
		{
			self.push_reliable();
		}

		if self
			.config
			.evdecls
			.iter()
			.any(|ev| ev.evty == EvType::Unreliable && ev.from == EvSource::Client)
		{
			self.push_unreliable();
		}

		self.push_return();

		self.buf
	}
}

pub fn code(config: &Config) -> String {
	ServerOutput::new(config).output()
}
