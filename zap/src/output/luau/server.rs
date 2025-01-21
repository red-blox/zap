use std::{collections::HashMap, hash::Hash};

use crate::{
	config::{Config, EvCall, EvDecl, EvSource, EvType, FnCall, FnDecl, NumTy, Parameter, TyDecl},
	irgen::{des, ser},
	output::{get_named_values, get_unnamed_values, luau::events_table_name},
};

use super::Output;

struct ServerOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
	var_occurrences: HashMap<String, usize>,
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
			var_occurrences: HashMap::new(),
		}
	}

	fn push_studio(&mut self) {
		self.push_line("if not RunService:IsRunning() then");
		self.indent();

		self.push_line("local noop = function() end");

		self.push_line("return table.freeze({");
		self.indent();

		let fire = self.config.casing.with("Fire", "fire", "fire");
		let fire_all = self.config.casing.with("FireAll", "fireAll", "fire_all");
		let fire_except = self.config.casing.with("FireExcept", "fireExcept", "fire_except");
		let fire_list = self.config.casing.with("FireList", "fireList", "fire_list");
		let fire_set = self.config.casing.with("FireSet", "fireSet", "fire_set");

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let on = self.config.casing.with("On", "on", "on");

		let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("{send_events} = noop,"));

		for ev in self.config.evdecls.iter() {
			self.push_line(&format!("{name} = table.freeze({{", name = ev.name));
			self.indent();

			if ev.from == EvSource::Client {
				match ev.call {
					EvCall::SingleSync | EvCall::SingleAsync => self.push_line(&format!("{set_callback} = noop")),
					EvCall::ManySync | EvCall::ManyAsync => self.push_line(&format!("{on} = noop")),
				}
			} else {
				self.push_line(&format!("{fire} = noop,"));

				if !self.config.disable_fire_all {
					self.push_line(&format!("{fire_all} = noop,"));
				}

				self.push_line(&format!("{fire_except} = noop,"));
				self.push_line(&format!("{fire_list} = noop,"));
				self.push_line(&format!("{fire_set} = noop"));
			}

			self.dedent();
			self.push_line("}),");
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("{name} = table.freeze({{", name = fndecl.name));
			self.indent();

			self.push_line(&format!("{set_callback} = noop"));

			self.dedent();
			self.push_line("}),");
		}

		self.dedent();
		self.push_line("}) :: Events");

		self.dedent();
		self.push_line("end");
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

	fn push_tydecls(&mut self) {
		for tydecl in self.config.tydecls.iter() {
			self.push_tydecl(tydecl);
		}
	}

	fn push_event_loop(&mut self) {
		self.push("\n");

		let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("local function {send_events}()"));
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
		self.push_line("end\n");

		if !self.config.manual_event_loop {
			self.push_line(&format!("RunService.Heartbeat:Connect({send_events})\n"));
		}
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

		let server_reliable_ty = self.config.server_reliable_ty();

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			server_reliable_ty,
			server_reliable_ty.size()
		));
	}

	fn get_values(&self, parameters: &[Parameter]) -> String {
		if !parameters.is_empty() {
			(1..=parameters.len())
				.map(|i| {
					if i == 1 {
						"value".to_string()
					} else {
						format!("value{}", i)
					}
				})
				.collect::<Vec<_>>()
				.join(", ")
		} else {
			"value".to_string()
		}
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

		let values = self.get_values(&ev.data);

		self.push_line(&format!("local {values}"));

		if !ev.data.is_empty() {
			let statements = &des::gen(
				ev.data.iter().map(|parameter| &parameter.ty),
				&get_unnamed_values("value", ev.data.len()),
				true,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if reliable_events[{id}] then"))
		} else {
			self.push_line(&format!("for _, cb in reliable_events[{id}] do"))
		}

		self.indent();

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("reliable_events[{id}](player, {values})")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(reliable_events[{id}], player, {values})")),
			EvCall::ManySync => self.push_line(&format!("cb(player, {values})")),
			EvCall::ManyAsync => self.push_line(&format!("task.spawn(cb, player, {values})")),
		}

		self.dedent();
		self.push_line("end");

		self.dedent();
	}

	fn push_fn_callback(&mut self, first: bool, fndecl: &FnDecl) {
		let server_id = fndecl.server_id;

		self.push_indent();

		if first {
			self.push("if ");
		} else {
			self.push("elseif ");
		}

		self.push(&format!("id == {server_id} then"));
		self.push("\n");

		self.indent();

		self.push_line("local call_id = buffer.readu8(buff, read(1))");

		let values = self.get_values(&fndecl.args);

		self.push_line(&format!("local {values}"));

		if !fndecl.args.is_empty() {
			let statements = &des::gen(
				fndecl.args.iter().map(|parameter| &parameter.ty),
				&get_unnamed_values("value", fndecl.args.len()),
				true,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
		}

		self.push_line(&format!("if reliable_events[{server_id}] then"));

		self.indent();

		let rets = if let Some(types) = &fndecl.rets { types } else { &vec![] };

		let rets_string = if !rets.is_empty() {
			(1..=rets.len())
				.map(|i| format!("ret_{}", i))
				.collect::<Vec<_>>()
				.join(", ")
		} else {
			"ret_1".to_string()
		};

		if fndecl.call == FnCall::Async {
			let args = if !fndecl.args.is_empty() {
				(1..=fndecl.args.len())
					.map(|i| format!("value_{}", i))
					.collect::<Vec<_>>()
					.join(", ")
			} else {
				"value_1".to_string()
			};

			// Avoid using upvalues as an optimization.
			self.push_line(&format!("task.spawn(function(player_2, call_id_2, {args})"));
			self.indent();

			self.push_line(&format!(
				"local {rets_string} = reliable_events[{server_id}](player_2, {args})"
			));

			self.push_line("load_player(player_2)");
			self.push_write_event_id(fndecl.client_id, self.config.client_reliable_ty());

			self.push_line("alloc(1)");
			self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, call_id_2)");

			if let Some(types) = &fndecl.rets {
				let names: Vec<String> = (0..types.len()).map(|i| format!("ret_{}", i + 1)).collect();
				let statements = &ser::gen(types, &names, self.config.write_checks, &mut self.var_occurrences);
				self.push_stmts(statements);
			}

			self.push_line("player_map[player_2] = save()");

			self.dedent();
			self.push_line(&format!("end, player, call_id, {values})"));
		} else {
			self.push_line(&format!(
				"local {rets_string} = reliable_events[{server_id}](player, {values})"
			));

			self.push_line("load_player(player)");
			self.push_write_event_id(fndecl.client_id, self.config.client_reliable_ty());

			self.push_line("alloc(1)");
			self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, call_id)");

			if let Some(types) = &fndecl.rets {
				let names: Vec<String> = (0..types.len()).map(|i| format!("ret_{}", i + 1)).collect();
				let statements = &ser::gen(types, &names, self.config.write_checks, &mut self.var_occurrences);
				self.push_stmts(statements);
			}

			self.push_line("player_map[player] = save()");
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

		let server_unreliable_ty = self.config.server_unreliable_ty();

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			server_unreliable_ty,
			server_unreliable_ty.size()
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

		let values = self.get_values(&ev.data);

		self.push_line(&format!("local {}", values));

		if !ev.data.is_empty() {
			let statements = &des::gen(
				ev.data.iter().map(|parameter| &parameter.ty),
				&get_unnamed_values("value", ev.data.len()),
				true,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if unreliable_events[{id}] then"))
		} else {
			self.push_line(&format!("for _, cb in unreliable_events[{id}] do"))
		}

		self.indent();

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("unreliable_events[{id}](player, {values})")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(unreliable_events[{id}], player, {values})")),
			EvCall::ManySync => self.push_line(&format!("cb(player, {values})")),
			EvCall::ManyAsync => self.push_line(&format!("task.spawn(cb, player, {values})")),
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
			"local reliable_events = table.create({})",
			self.config.server_reliable_count()
		));

		self.push_line(&format!(
			"local unreliable_events = table.create({})",
			self.config.server_unreliable_count()
		));

		for evdecl in self.config.evdecls.iter().filter(|ev_decl| {
			ev_decl.from == EvSource::Client && matches!(ev_decl.call, EvCall::ManyAsync | EvCall::ManySync)
		}) {
			match evdecl.evty {
				EvType::Reliable => self.push_line(&format!("reliable_events[{}] = {{}}", evdecl.id)),
				EvType::Unreliable => self.push_line(&format!("unreliable_events[{}] = {{}}", evdecl.id)),
			}
		}
	}

	fn push_write_event_id(&mut self, id: usize, num_ty: NumTy) {
		self.push_line(&format!("alloc({})", num_ty.size()));
		self.push_line(&format!("buffer.write{}(outgoing_buff, outgoing_apos, {id})", num_ty));
	}

	fn push_write_evdecl_event_id(&mut self, ev: &EvDecl) {
		let num_ty = match ev.evty {
			EvType::Reliable => self.config.server_reliable_ty(),
			EvType::Unreliable => self.config.server_unreliable_ty(),
		};

		self.push_write_event_id(ev.id, num_ty);
	}

	fn push_value_parameters(&mut self, parameters: &[Parameter]) {
		for (i, parameter) in parameters.iter().enumerate() {
			if i > 0 {
				self.push(", ");
			}

			if let Some(name) = parameter.name {
				self.push(&format!("{name}: "));
			} else {
				let value = format!(
					"{}{}",
					self.config.casing.with("Value", "value", "value"),
					if i == 0 { "".to_string() } else { (i + 1).to_string() }
				);

				self.push(&format!("{value}: "));
			}

			self.push_ty(&parameter.ty);
		}
	}

	fn push_return_fire(&mut self, ev: &EvDecl) {
		let parameters = &ev.data;

		let fire = self.config.casing.with("Fire", "fire", "fire");
		let player = self.config.casing.with("Player", "player", "player");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire} = function({player}: Player"));

		if !parameters.is_empty() {
			self.push(", ");
			self.push_value_parameters(parameters);
		}

		self.push(")\n");
		self.indent();

		match ev.evty {
			EvType::Reliable => self.push_line(&format!("load_player({player})")),
			EvType::Unreliable => self.push_line("load_empty()"),
		}

		self.push_write_evdecl_event_id(ev);

		if !parameters.is_empty() {
			let statements = &ser::gen(
				parameters.iter().map(|parameter| &parameter.ty),
				&get_named_values(value, parameters),
				self.config.write_checks,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
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
		let parameters = &ev.data;

		let fire_all = self.config.casing.with("FireAll", "fireAll", "fire_all");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_all} = function("));

		if !parameters.is_empty() {
			self.push_value_parameters(parameters);
		}

		self.push(")\n");
		self.indent();

		self.push_line("load_empty()");

		self.push_write_evdecl_event_id(ev);

		if !parameters.is_empty() {
			let statements = &ser::gen(
				parameters.iter().map(|parameter| &parameter.ty),
				&get_named_values(value, parameters),
				self.config.write_checks,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
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
		let parameters = &ev.data;

		let fire_except = self.config.casing.with("FireExcept", "fireExcept", "fire_except");
		let except = self.config.casing.with("Except", "except", "except");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_except} = function({except}: Player"));

		if !parameters.is_empty() {
			self.push(", ");
			self.push_value_parameters(parameters);
		}

		self.push(")\n");
		self.indent();

		self.push_line("load_empty()");

		self.push_write_evdecl_event_id(ev);

		if !parameters.is_empty() {
			let statements = &ser::gen(
				parameters.iter().map(|paramater| &paramater.ty),
				&get_named_values(value, parameters),
				self.config.write_checks,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
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
		let parameters = &ev.data;

		let fire_list = self.config.casing.with("FireList", "fireList", "fire_list");
		let list = self.config.casing.with("List", "list", "list");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_list} = function({list}: {{ Player }}"));

		if !parameters.is_empty() {
			self.push(", ");
			self.push_value_parameters(parameters);
		}

		self.push(")\n");
		self.indent();

		self.push_line("load_empty()");

		self.push_write_evdecl_event_id(ev);

		if !parameters.is_empty() {
			let statements = &ser::gen(
				parameters.iter().map(|parameter| &parameter.ty),
				&get_named_values(value, parameters),
				self.config.write_checks,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
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

	fn push_return_fire_set(&mut self, ev: &EvDecl) {
		let parameters = &ev.data;

		let fire_set = self.config.casing.with("FireSet", "fireSet", "fire_set");
		let set = self.config.casing.with("Set", "set", "set");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire_set} = function({set}: {{ [Player]: true }}"));

		if !parameters.is_empty() {
			self.push(", ");
			self.push_value_parameters(parameters);
		}

		self.push(")\n");
		self.indent();

		self.push_line("load_empty()");

		self.push_write_evdecl_event_id(ev);

		if !parameters.is_empty() {
			let statements = &ser::gen(
				parameters.iter().map(|parameter| &parameter.ty),
				&get_named_values(value, parameters),
				self.config.write_checks,
				&mut self.var_occurrences,
			);
			self.push_stmts(statements);
		}

		match ev.evty {
			EvType::Reliable => {
				self.push_line("local buff, used, inst = outgoing_buff, outgoing_used, outgoing_inst");
				self.push_line(&format!("for player in {set} do"));
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
				self.push_line(&format!("for player in {set} do"));
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

			if !self.config.disable_fire_all {
				self.push_return_fire_all(ev);
			}

			self.push_return_fire_except(ev);
			self.push_return_fire_list(ev);
			self.push_return_fire_set(ev);

			self.dedent();
			self.push_line("},");
		}
	}

	fn push_return_setcallback(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let callback = self.config.casing.with("Callback", "callback", "callback");
		let player = self.config.casing.with("Player", "player", "player");

		self.push_indent();
		self.push(&format!("{set_callback} = function({callback}: ({player}: Player"));

		if !ev.data.is_empty() {
			self.push(", ");
			self.push_value_parameters(&ev.data);
		}

		self.push(") -> ()): () -> ()\n");
		self.indent();

		self.push_line(&format!("{}[{id}] = {callback}", events_table_name(ev)));

		self.push_line("return function()");
		self.indent();

		self.push_line(&format!("{}[{id}] = nil", events_table_name(ev)));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_on(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let on = self.config.casing.with("On", "on", "on");
		let callback = self.config.casing.with("Callback", "callback", "callback");
		let player = self.config.casing.with("Player", "player", "player");

		self.push_indent();
		self.push(&format!("{on} = function({callback}: ({player}: Player"));

		if !ev.data.is_empty() {
			self.push(", ");
			self.push_value_parameters(&ev.data);
		}

		self.push(") -> ()): () -> ()\n");
		self.indent();

		let events_table = events_table_name(ev);

		self.push_line(&format!("table.insert({events_table}[{id}], {callback})"));

		self.push_line("return function()");
		self.indent();

		self.push_line(&format!(
			"table.remove({events_table}[{id}], table.find({events_table}[{id}], {callback}))",
		));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end,");
	}

	fn push_fn_return(&mut self, fndecl: &FnDecl) {
		let server_id = fndecl.server_id;

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let callback = self.config.casing.with("Callback", "callback", "callback");
		let player = self.config.casing.with("Player", "player", "player");

		self.push_indent();
		self.push(&format!("{set_callback} = function({callback}: ({player}: Player"));

		if !fndecl.args.is_empty() {
			self.push(", ");
			self.push_value_parameters(&fndecl.args);
		}

		self.push(") -> (");

		if let Some(types) = &fndecl.rets {
			for (i, ty) in types.iter().enumerate() {
				if i > 0 {
					self.push(", ");
				}
				self.push_ty(ty);
			}
		}

		self.push(")): () -> ()\n");
		self.indent();

		self.push_line(&format!("reliable_events[{server_id}] = {callback}"));

		self.push_line("return function()");
		self.indent();

		self.push_line(&format!("reliable_events[{server_id}] = nil"));

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
		self.push_line("local returns = {");
		self.indent();

		let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("{send_events} = {send_events},"));

		self.push_return_outgoing();
		self.push_return_listen();

		self.dedent();
		self.push_line("}");

		self.push_line("type Events = typeof(returns)");
		self.push_line("return returns");
	}

	pub fn push_check_client(&mut self) {
		self.push_line("local Players = game:GetService(\"Players\")");
		self.push("\n");
		self.push_line("if RunService:IsClient() then");
		self.indent();
		self.push_line("error(\"Cannot use the server module on the client!\")");
		self.dedent();
		self.push_line("end");
	}

	pub fn push_create_remotes(&mut self) {
		self.push("\n");
		self.push_line(&format!(
			"local remotes = ReplicatedStorage:FindFirstChild(\"{}\")",
			self.config.remote_folder
		));
		self.push_line("if remotes == nil then");
		self.indent();
		self.push_line("remotes = Instance.new(\"Folder\")");
		self.push_line(&format!("remotes.Name = \"{}\"", self.config.remote_folder));
		self.push_line("remotes.Parent = ReplicatedStorage");
		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line(&format!(
			"local reliable = remotes:FindFirstChild(\"{}_RELIABLE\")",
			self.config.remote_scope
		));
		self.push_line("if reliable == nil then");
		self.indent();
		self.push_line("reliable = Instance.new(\"RemoteEvent\")");
		self.push_line(&format!("reliable.Name = \"{}_RELIABLE\"", self.config.remote_scope));
		self.push_line("reliable.Parent = remotes");
		self.dedent();
		self.push_line("end");

		self.push("\n");

		self.push_line(&format!(
			"local unreliable = remotes:FindFirstChild(\"{}_UNRELIABLE\")",
			self.config.remote_scope
		));
		self.push_line("if unreliable == nil then");
		self.indent();
		self.push_line("unreliable = Instance.new(\"UnreliableRemoteEvent\")");
		self.push_line(&format!(
			"unreliable.Name = \"{}_UNRELIABLE\"",
			self.config.remote_scope
		));
		self.push_line("unreliable.Parent = remotes");
		self.dedent();
		self.push_line("end");
		self.push("\n");
	}

	pub fn push_player_map(&mut self) {
		self.push_line("local player_map = {}");
		self.push("\n");
		self.push_line("local function load_player(player: Player)");
		self.indent();
		self.push_line("if player_map[player] then");
		self.indent();
		self.push_line("load(player_map[player])");
		self.dedent();
		self.push_line("else");
		self.indent();
		self.push_line("load_empty()");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end");
		self.push("\n");
		self.push_line("Players.PlayerRemoving:Connect(function(player)");
		self.indent();
		self.push_line("player_map[player] = nil");
		self.dedent();
		self.push_line("end)");
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Server");

		if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
			return self.buf;
		};

		self.push(include_str!("base.luau"));

		self.push_studio();

		self.push_check_client();

		self.push_create_remotes();

		self.push_player_map();

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
