use std::fmt::Display;

use crate::{
	irgen::{gen_des, gen_ser},
	parser::{EvCall, EvSource, EvType, File},
	util::casing,
};

struct ServerFile<'a>(&'a File);

impl<'a> Display for ServerFile<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let file = &self.0;

		write!(f, "{}", include_str!("base.luau"))?;

		writeln!(f, "local events = table.create({})", file.ev_decls.len())?;

		// Output Types
		for ty_decl in file.ty_decls.iter() {
			write!(f, "{}", ty_decl)?;
		}

		// Output Reliable Callbacks
		writeln!(f, "reliable.OnServerEvent:Connect(function(player, buff, inst)")?;
		writeln!(f, "\tincoming_buff = buff")?;
		writeln!(f, "\tincoming_read = 0")?;
		writeln!(f, "\tincoming_inst = inst")?;

		writeln!(f, "\tlocal len = buffer.len(buff)")?;
		writeln!(f, "\twhile incoming_read < len do")?;

		writeln!(
			f,
			"\t\tlocal id = buffer.read{}(buff, read({}))",
			file.event_id_ty(),
			file.event_id_ty().size()
		)?;

		for (i, ev_decl) in file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client && ev_decl.evty == EvType::Reliable)
		{
			let id = i + 1;

			if i == 0 {
				writeln!(f, "\t\tif id == {id} then")?;
			} else {
				writeln!(f, "\t\telseif id == {id} then")?;
			}

			writeln!(f, "\t\t\tlocal value;")?;

			for stmt in gen_des(&ev_decl.data, "value", false) {
				writeln!(f, "\t\t\t{stmt}")?;
			}

			match ev_decl.call {
				EvCall::SingleSync => writeln!(f, "\t\tif events[{id}] then events[{id}](player, value) end")?,
				EvCall::SingleAsync => writeln!(
					f,
					"\t\tif events[{id}] then task.spawn(events[{id}], player, value) end"
				)?,

				EvCall::ManySync => writeln!(f, "\t\tfor _, cb in events[{id}] do cb(player, value) end")?,
				EvCall::ManyAsync => writeln!(f, "\t\tfor _, cb in events[{id}] do task.spawn(cb, player, value) end")?,
			}

			writeln!(f, "\t\tend")?;
		}

		writeln!(f, "\tend")?;
		writeln!(f, "end)")?;

		// Output Unreliable Callbacks
		writeln!(f, "unreliable.OnServerEvent:Connect(function(player, buff, inst)")?;
		writeln!(f, "\tincoming_buff = buff")?;
		writeln!(f, "\tincoming_read = 0")?;
		writeln!(f, "\tincoming_inst = inst")?;

		writeln!(
			f,
			"\tlocal id = buffer.read{}(buff, read({}))",
			file.event_id_ty(),
			file.event_id_ty().size()
		)?;

		for (i, ev_decl) in file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client && ev_decl.evty == EvType::Unreliable)
		{
			let id = i + 1;

			if i == 0 {
				writeln!(f, "\tif id == {id} then")?;
			} else {
				writeln!(f, "\telseif id == {id} then")?;
			}

			writeln!(f, "\t\tlocal value;")?;

			for stmt in gen_des(&ev_decl.data, "value", false) {
				writeln!(f, "\t\t{stmt}")?;
			}

			match ev_decl.call {
				EvCall::SingleSync => writeln!(f, "\t\tif events[{id}] then events[{id}](player, value) end")?,
				EvCall::SingleAsync => writeln!(
					f,
					"\t\tif events[{id}] then task.spawn(events[{id}], player, value) end"
				)?,

				EvCall::ManySync => writeln!(f, "\t\tfor _, cb in events[{id}] do cb(player, value) end")?,
				EvCall::ManyAsync => writeln!(f, "\t\tfor _, cb in events[{id}] do task.spawn(cb, player, value) end")?,
			}

			writeln!(f, "\t\tend")?;
		}

		writeln!(f, "\tend")?;
		writeln!(f, "end)")?;

		// Output Event Declarations
		for (i, ev_decl) in file.ev_decls.iter().enumerate().filter(|(_, ev_decl)| {
			ev_decl.from == EvSource::Client && matches!(ev_decl.call, EvCall::ManyAsync | EvCall::ManySync)
		}) {
			let id = i + 1;

			writeln!(f, "events[{id}] = {{}}")?;
		}

		// Fire Reliable
		write!(f, "{}", include_str!("server.luau"))?;

		writeln!(f, "return {{")?;

		// Output Fire Return
		let value = casing(file.casing, "Value", "value", "value");

		for (i, ev_decl) in file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
		{
			let id = i + 1;
			let ty = &ev_decl.data;

			let ser = &gen_ser(ty, value, false);

			writeln!(f, "\t{name} = {{", name = ev_decl.name)?;

			// Fire
			{
				writeln!(
					f,
					"\t\t{fire} = function({player}: Player, {value}: {ty})",
					fire = casing(file.casing, "Fire", "fire", "fire"),
					player = casing(file.casing, "Player", "player", "player"),
				)?;

				match ev_decl.evty {
					EvType::Reliable => writeln!(
						f,
						"\t\t\tload(player_map[{player}])",
						player = casing(file.casing, "Player", "player", "player")
					)?,

					EvType::Unreliable => writeln!(f, "\t\t\tload_empty()")?,
				}

				writeln!(f, "\t\t\tbuffer.write{}(outgoing_buff, {id})", file.event_id_ty())?;

				for stmt in ser {
					writeln!(f, "\t\t\t{stmt}")?;
				}

				match ev_decl.evty {
					EvType::Reliable => writeln!(
						f,
						"\t\t\tplayer_map[{player}] = save()",
						player = casing(file.casing, "Player", "player", "player")
					)?,

					EvType::Unreliable => {
						writeln!(f, "\t\t\tlocal buff = buffer.create(outgoing_used)")?;
						writeln!(f, "\t\t\tbuffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)")?;
						writeln!(f, "\t\t\tunreliable:FireClient(player, buff, outgoing_inst)")?;
					}
				}

				writeln!(f, "\t\tend,")?;
			}

			// FireAll
			{
				writeln!(
					f,
					"\t\t{fire_all} = function({value}: {ty})",
					fire_all = casing(file.casing, "FireAll", "fireAll", "fire_all"),
				)?;

				writeln!(f, "\t\t\tload_empty()")?;
				writeln!(f, "\t\t\tbuffer.write{}(outgoing_buff, {id})", file.event_id_ty())?;

				for stmt in ser {
					writeln!(f, "\t\t\t{stmt}")?;
				}

				match ev_decl.evty {
					EvType::Reliable => {
						writeln!(
							f,
							"\t\t\tlocal buff, used, inst = outgoing_buff, outgoing_used, outgoing_inst"
						)?;
						writeln!(f, "\t\t\tfor player, outgoing in player_map do")?;
						writeln!(f, "\t\t\t\tload(outgoing)")?;
						writeln!(f, "\t\t\t\tbuffer.copy(outgoing_buff, alloc(used), buff, 0, used)")?;
						writeln!(
							f,
							"\t\t\t\ttable.move(inst, 1, #inst, #outgoing_inst + 1, outgoing_inst)"
						)?;
						writeln!(f, "\t\t\t\tplayer_map[player] = save()")?;
						writeln!(f, "\t\t\tend")?;
					}

					EvType::Unreliable => {
						writeln!(f, "\t\t\tlocal buff = buffer.create(outgoing_used)")?;
						writeln!(f, "\t\t\tbuffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)")?;
						writeln!(f, "\t\t\tunreliable:FireAllClients(buff, outgoing_inst)")?;
					}
				}

				writeln!(f, "\t\tend,")?;
			}

			// FireExcept
			{
				writeln!(
					f,
					"\t\t{fire_all_except} = function({except}: Player, {value}: {ty})",
					fire_all_except = casing(file.casing, "FireExcept", "fireExcept", "fire_except"),
					except = casing(file.casing, "Except", "except", "except"),
				)?;

				writeln!(f, "\t\t\tload_empty()")?;
				writeln!(f, "\t\t\tbuffer.write{}(outgoing_buff, {id})", file.event_id_ty())?;

				for stmt in ser {
					writeln!(f, "\t\t\t{stmt}")?;
				}

				match ev_decl.evty {
					EvType::Reliable => {
						writeln!(
							f,
							"\t\t\tlocal buff, used, inst = outgoing_buff, outgoing_used, outgoing_inst"
						)?;
						writeln!(f, "\t\t\tfor player, outgoing in player_map do")?;
						writeln!(
							f,
							"\t\t\t\tif player ~= {except} then",
							except = casing(file.casing, "Except", "except", "except"),
						)?;
						writeln!(f, "\t\t\t\t\tload(outgoing)")?;
						writeln!(f, "\t\t\t\t\tbuffer.copy(outgoing_buff, alloc(used), buff, 0, used)")?;
						writeln!(
							f,
							"\t\t\t\ttable.move(inst, 1, #inst, #outgoing_inst + 1, outgoing_inst)"
						)?;
						writeln!(f, "\t\t\t\t\tplayer_map[player] = save()")?;
						writeln!(f, "\t\t\t\tend")?;
						writeln!(f, "\t\t\tend")?;
					}

					EvType::Unreliable => {
						writeln!(f, "\t\t\tlocal buff = buffer.create(outgoing_used)")?;
						writeln!(f, "\t\t\tbuffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)")?;
						writeln!(f, "\t\t\tfor player in player_map do")?;
						writeln!(
							f,
							"\t\t\t\tif player ~= {except} then",
							except = casing(file.casing, "Except", "except", "except"),
						)?;
						writeln!(f, "\t\t\t\t\tunreliable:FireClient(player, buff, outgoing_inst)")?;
						writeln!(f, "\t\t\t\tend")?;
					}
				}

				writeln!(f, "\t\tend,")?;
			}

			// FireList
			{
				writeln!(
					f,
					"\t\t{fire_list} = function({player_list}: {{ Player }}, {value}: {ty})",
					fire_list = casing(file.casing, "FireList", "fireList", "fire_list"),
					player_list = casing(file.casing, "PlayerList", "playerList", "player_list"),
				)?;

				writeln!(f, "\t\t\tload_empty()")?;
				writeln!(f, "\t\t\tbuffer.write{}(outgoing_buff, {id})", file.event_id_ty())?;

				for stmt in ser {
					writeln!(f, "\t\t\t{stmt}")?;
				}

				match ev_decl.evty {
					EvType::Reliable => {
						writeln!(
							f,
							"\t\t\tlocal buff, used, inst = outgoing_buff, outgoing_used, outgoing_inst"
						)?;
						writeln!(
							f,
							"\t\t\tfor _, player in {player_list} do",
							player_list = casing(file.casing, "PlayerList", "playerList", "player_list")
						)?;
						writeln!(f, "\t\t\t\tload(player_map[player])")?;
						writeln!(f, "\t\t\t\tbuffer.copy(outgoing_buff, alloc(used), buff, 0, used)")?;
						writeln!(
							f,
							"\t\t\t\ttable.move(inst, 1, #inst, #outgoing_inst + 1, outgoing_inst)"
						)?;
						writeln!(f, "\t\t\t\tplayer_map[player] = save()")?;
						writeln!(f, "\t\t\tend")?;
					}

					EvType::Unreliable => {
						writeln!(f, "\t\t\tlocal buff = buffer.create(outgoing_used)")?;
						writeln!(f, "\t\t\tbuffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)")?;
						writeln!(
							f,
							"\t\t\tfor _, player in {player_list} do",
							player_list = casing(file.casing, "PlayerList", "playerList", "player_list")
						)?;
						writeln!(f, "\t\t\t\tunreliable:FireClient(player, buff, outgoing_inst)")?;
						writeln!(f, "\t\t\tend")?;
					}
				}

				writeln!(f, "\t\tend,")?;
			}

			writeln!(f, "\t}},")?;
		}

		// Output Listen Return
		for (i, ev_decl) in file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			let id = i + 1;
			let ty = &ev_decl.data;

			writeln!(f, "\t{name} = {{", name = ev_decl.name,)?;

			match ev_decl.call {
				EvCall::SingleSync | EvCall::SingleAsync => {
					writeln!(
						f,
						"\t\t{set_callback} = function({callback}: (Player, {ty}) -> ())",
						set_callback = casing(file.casing, "SetCallback", "setCallback", "set_callback"),
						callback = casing(file.casing, "Callback", "callback", "callback"),
					)?;

					writeln!(
						f,
						"\t\t\tevents[{id}] = {callback}",
						callback = casing(file.casing, "Callback", "callback", "callback")
					)?;

					writeln!(f, "\t\tend,")?;
				}

				EvCall::ManySync | EvCall::ManyAsync => {
					writeln!(
						f,
						"{on} = function({callback}: (Player, {ty}) -> ())",
						on = casing(file.casing, "On", "on", "on"),
						callback = casing(file.casing, "Callback", "callback", "callback"),
					)?;

					writeln!(
						f,
						"\t\t\ttable.insert(events[{id}], {callback})",
						callback = casing(file.casing, "Callback", "callback", "callback")
					)?;

					writeln!(f, "\t\tend,")?;
				}
			}

			writeln!(f, "\t}},")?;
		}

		writeln!(f, "}}")?;

		Ok(())
	}
}
