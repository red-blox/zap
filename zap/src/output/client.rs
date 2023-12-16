use std::fmt::Display;

use crate::{
	irgen::{gen_des, gen_ser},
	parser::{EvCall, EvSource, EvType, File},
	util::casing,
};

struct ClientFile<'a>(&'a File);

impl<'a> Display for ClientFile<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let file = self.0;

		if file.ev_decls.is_empty() {
			return Ok(());
		}

		write!(f, "{}", include_str!("base.luau"))?;
		write!(f, "{}", include_str!("client.luau"))?;

		writeln!(f, "local events = table.create({})", file.ev_decls.len())?;

		// Output Types
		for ty_decl in file.ty_decls.iter() {
			write!(f, "{}", ty_decl)?;
		}

		// Output Reliable Callbacks
		if file
			.ev_decls
			.iter()
			.any(|ev_decl| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Reliable)
		{
			writeln!(f, "reliable.OnClientEvent:Connect(function(buff, inst)")?;
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
				.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Reliable)
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
					EvCall::SingleSync => writeln!(f, "\t\t\tif events[{id}] then events[{id}](value) end")?,
					EvCall::SingleAsync => {
						writeln!(f, "\t\t\tif events[{id}] then task.spawn(events[{id}], value) end")?
					}

					EvCall::ManySync => writeln!(f, "\t\t\tfor _, cb in events[{id}] do cb(value) end")?,
					EvCall::ManyAsync => writeln!(f, "\t\t\tfor _, cb in events[{id}] do task.spawn(cb, value) end")?,
				}

				writeln!(f, "\t\tend")?;
			}

			writeln!(f, "\tend")?;
			writeln!(f, "end)")?;
		}

		// Output Unreliable Callbacks
		if file
			.ev_decls
			.iter()
			.any(|ev_decl| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Unreliable)
		{
			writeln!(f, "unreliable.OnClientEvent:Connect(function(buff, inst)")?;
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
				.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Unreliable)
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
					EvCall::SingleSync => writeln!(f, "\t\tif events[{id}] then events[{id}](value) end")?,
					EvCall::SingleAsync => writeln!(f, "\t\tif events[{id}] then task.spawn(events[{id}], value) end")?,

					EvCall::ManySync => writeln!(f, "\t\tfor _, cb in events[{id}] do cb(value) end")?,
					EvCall::ManyAsync => writeln!(f, "\t\tfor _, cb in events[{id}] do task.spawn(cb, value) end")?,
				}
			}

			writeln!(f, "\tend")?;
			writeln!(f, "end)")?;
		}

		// Output Event Declarations
		for (i, _) in file.ev_decls.iter().enumerate().filter(|(_, ev_decl)| {
			ev_decl.from == EvSource::Server && matches!(ev_decl.call, EvCall::ManyAsync | EvCall::ManySync)
		}) {
			let id = i + 1;

			writeln!(f, "events[{id}] = {{}}")?;
		}

		writeln!(f, "return {{")?;

		// Output Fire Return
		let value = casing(file.casing, "Value", "value", "value");

		for (i, ev_decl) in file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
		{
			let id = i + 1;
			let ty = &ev_decl.data;

			writeln!(f, "\t{name} = {{", name = ev_decl.name)?;

			writeln!(
				f,
				"\t\t{fire} = function({value}: {ty})",
				fire = casing(file.casing, "Fire", "fire", "fire")
			)?;

			match ev_decl.evty {
				EvType::Reliable => {}
				EvType::Unreliable => {
					writeln!(f, "\t\t\tlocal saved = save()")?;
					writeln!(f, "\t\t\tload_empty()")?;
				}
			}

			writeln!(f, "\t\t\tbuffer.write{}(buff, {id})", file.event_id_ty())?;

			for stmt in gen_ser(ty, value, file.write_checks) {
				writeln!(f, "\t\t\t{stmt}")?;
			}

			match ev_decl.evty {
				EvType::Reliable => {}
				EvType::Unreliable => {
					writeln!(f, "\t\t\tlocal buff = buffer.create(outgoing_used)")?;
					writeln!(f, "\t\t\tbuffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)")?;
					writeln!(f, "\t\t\tunreliable:FireServer(buff, outgoing_inst)")?;
					writeln!(f, "\t\t\tload(saved)")?;
				}
			}

			writeln!(f, "\t\tend,")?;
			writeln!(f, "\t}},")?;
		}

		// Output Listen Return
		for (i, ev_decl) in file
			.ev_decls
			.iter()
			.enumerate()
			.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
		{
			let id = i + 1;
			let ty = &ev_decl.data;

			writeln!(f, "\t{name} = {{", name = ev_decl.name,)?;

			match ev_decl.call {
				EvCall::SingleSync | EvCall::SingleAsync => {
					writeln!(
						f,
						"\t\t{set_callback} = function({callback}: ({ty}) -> ())",
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
						"{on} = function({callback}: ({ty}) -> ())",
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

pub fn code(file: &File) -> String {
	ClientFile(file).to_string()
}
