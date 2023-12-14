use crate::{
	irgen::{gen_des, gen_ser, Expr, Op, Var},
	line,
	output::casing,
	parser::{EvCall, EvSource, EvType, File, Ty, TyDecl},
	util::NumTy,
};

use super::Output;

const LUAU_BASE: &str = include_str!("luau_base.luau");
const LUAU_SERVER: &str = include_str!("luau_server.luau");
const LUAU_CLIENT: &str = include_str!("luau_client.luau");

fn luau_expr(expr: Expr) -> String {
	match expr {
		Expr::False => "false".to_string(),
		Expr::True => "true".to_string(),
		Expr::Nil => "nil".to_string(),

		Expr::Num(num) => num.to_string(),
		Expr::Str(string) => format!("\"{}\"", string),
		Expr::Var(var) => luau_var(*var),

		Expr::EmptyArr => "{}".to_string(),
		Expr::EmptyObj => "{}".to_string(),

		Expr::Len(expr) => format!("#{}", luau_expr(*expr)),

		Expr::Lt(lhs, rhs) => format!("{} < {}", luau_expr(*lhs), luau_expr(*rhs)),
		Expr::Gt(lhs, rhs) => format!("{} > {}", luau_expr(*lhs), luau_expr(*rhs)),
		Expr::Le(lhs, rhs) => format!("{} <= {}", luau_expr(*lhs), luau_expr(*rhs)),
		Expr::Ge(lhs, rhs) => format!("{} >= {}", luau_expr(*lhs), luau_expr(*rhs)),
		Expr::Eq(lhs, rhs) => format!("{} == {}", luau_expr(*lhs), luau_expr(*rhs)),
		Expr::Add(lhs, rhs) => format!("{} + {}", luau_expr(*lhs), luau_expr(*rhs)),
	}
}

fn luau_var(var: Var) -> String {
	match var {
		Var::Name(name) => name,

		Var::NameIndex(var, index) => format!("{}.{}", luau_var(*var), index),
		Var::ExprIndex(var, index) => format!("{}[{}]", luau_var(*var), luau_expr(*index)),
	}
}

fn luau_numty(numty: NumTy) -> &'static str {
	match numty {
		NumTy::F32 => "f32",
		NumTy::F64 => "f64",

		NumTy::U8 => "u8",
		NumTy::U16 => "u16",
		NumTy::U32 => "u32",

		NumTy::I8 => "i8",
		NumTy::I16 => "i16",
		NumTy::I32 => "i32",
	}
}

fn luau_op(output: &mut Output, op: Op) {
	match op {
		Op::Local { name } => line!(output, "local {};", name),
		Op::Assign { var, val } => line!(output, "{} = {};", luau_var(var), luau_expr(val)),
		Op::Throw { msg } => line!(output, "error(\"{}\");", msg),
		Op::Assert { cond, msg } => line!(output, "assert({}, \"{}\");", luau_expr(cond), msg),

		Op::BlockStart => output.line("do".into()).tab(),

		Op::NumFor { var, start, end } => {
			line!(output, "for {} = {}, {} do", var, luau_expr(start), luau_expr(end)).tab()
		}

		Op::GenFor { key, val, expr } => line!(output, "for {}, {} in {} do", key, val, luau_expr(expr)).tab(),

		Op::If { cond } => line!(output, "if {} then", luau_expr(cond)).tab(),

		Op::ElseIf { cond } => {
			output.untab();
			line!(output, "elseif {} then", luau_expr(cond)).tab()
		}

		Op::Else => output.untab().line("else".into()).tab(),

		Op::BlockEnd => output.untab().line("end".into()),

		Op::Alloc { into, len } => line!(output, "{} = alloc({});", luau_var(into), luau_expr(len)),

		Op::WriteNum { expr, ty, at } => {
			let at = match at {
				Some(at) => luau_expr(at),
				None => format!("alloc({})", ty.size()),
			};

			line!(
				output,
				"buffer.write{}(outgoing_buff, {}, {});",
				luau_numty(ty),
				at,
				luau_expr(expr),
			)
		}

		Op::WriteStr { expr, len } => line!(
			output,
			"buffer.writestring(outgoing_buff, alloc({len}), {val}, {len});",
			val = luau_expr(expr),
			len = luau_expr(len),
		),

		Op::WriteRef { expr, ref_name } => line!(output, "types.write_{}({});", ref_name, luau_expr(expr)),

		Op::ReadNum { into, ty, at } => {
			let at = match at {
				Some(at) => luau_expr(at),
				None => format!("read({})", ty.size()),
			};

			line!(
				output,
				"{} = buffer.read{}(incoming_buff, {});",
				luau_var(into),
				luau_numty(ty),
				at,
			)
		}

		Op::ReadStr { into, len } => line!(
			output,
			"{} = buffer.readstring(incoming_buff, read({len}), {len});",
			luau_var(into),
			len = luau_expr(len),
		),

		Op::ReadRef { into, ref_name } => line!(output, "{} = types.read_{}();", luau_var(into), ref_name),
	};
}

fn luau_ty(ty: &Ty) -> String {
	match ty {
		Ty::Bool => "boolean".into(),

		Ty::F32(..) => "number".into(),
		Ty::F64(..) => "number".into(),

		Ty::I8(..) => "number".into(),
		Ty::I16(..) => "number".into(),
		Ty::I32(..) => "number".into(),

		Ty::U8(..) => "number".into(),
		Ty::U16(..) => "number".into(),
		Ty::U32(..) => "number".into(),

		Ty::Str { .. } => "string".into(),
		Ty::Arr { ty, .. } => format!("{{ {} }}", luau_ty(ty)),
		Ty::Map { key, val } => format!("{{ [{}]: {} }}", luau_ty(key), luau_ty(val)),

		Ty::Struct { fields } => format!(
			"{{ {} }}",
			fields
				.iter()
				.map(|(name, ty)| format!("{}: {}", name, luau_ty(ty)))
				.collect::<Vec<_>>()
				.join(", ")
		),

		Ty::Enum { variants } => variants
			.iter()
			.map(|v| format!("\"{}\"", v))
			.collect::<Vec<_>>()
			.join(" | "),

		Ty::Ref(name) => name.clone(),

		Ty::Optional(ty) => format!("{}?", luau_ty(ty)),
	}
}

fn luau_ir(output: &mut Output, ir: Vec<Op>) {
	for op in ir {
		luau_op(output, op);
	}
}

fn luau_tydecl(output: &mut Output, tydecl: &TyDecl, write_check: bool, read_check: bool) {
	let name = &tydecl.name;
	let ty = &tydecl.ty;

	line!(output, "export type {name} = {};", luau_ty(ty));

	line!(output, "function types.write_{name}(value: {name})").tab();
	luau_ir(output, gen_ser(ty, "value", write_check));
	output.untab().line("end;".into());

	line!(output, "function types.read_{name}(): {name}").tab();
	line!(output, "local value;");
	luau_ir(output, gen_des(ty, "value", read_check));
	line!(output, "return value;");
	output.untab().line("end;".into());
}

fn server_callbacks(output: &mut Output, file: &File) {
	for (i, ev_decl) in file
		.ev_decls
		.iter()
		.enumerate()
		.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
	{
		let id = i + 1;

		line!(output, "event_callbacks[{id}] = function(player)").tab();
		line!(output, "local value;");
		luau_ir(output, gen_des(&ev_decl.data, "value", true));

		match ev_decl.call {
			EvCall::SingleSync => {
				line!(output, "event_user_call[{id}](player, value);");
			}

			EvCall::SingleAsync => {
				line!(output, "task.spawn(event_user_call[{id}], player, value);");
			}

			EvCall::ManySync => {
				line!(output, "for _, callback in event_user_call[{id}] do").tab();
				line!(output, "callback(player, value);");
				output.untab().line("end;".into());
			}

			EvCall::ManyAsync => {
				line!(output, "for _, callback in event_user_call[{id}] do").tab();
				line!(output, "task.spawn(callback, player, value);");
				output.untab().line("end;".into());
			}
		};

		output.untab().line("end;".into());
	}
}

fn server_return_fire(output: &mut Output, file: &File) {
	let load_empty = "outgoing_buff, outgoing_used, outgoing_size = buffer.create(64), 0, 64;";
	let value = casing(file.casing, "Value", "value", "value");

	for (i, ev_decl) in file
		.ev_decls
		.iter()
		.enumerate()
		.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
	{
		let id = i + 1;
		let ty = luau_ty(&ev_decl.data);

		line!(output, "{} = {{", ev_decl.name).tab();

		{
			// Fire
			line!(
				output,
				"{fire} = function({player}: Player, {value}: {ty})",
				fire = casing(file.casing, "Fire", "fire", "fire"),
				player = casing(file.casing, "Player", "player", "player"),
			)
			.tab();

			match ev_decl.evty {
				EvType::Reliable => line!(
					output,
					"load_player({player});",
					player = casing(file.casing, "Player", "player", "player")
				),

				EvType::Unreliable => output.line(load_empty.into()),
			};

			line!(output, "write_event_id({id});");

			luau_ir(output, gen_ser(&ev_decl.data, value, file.write_checks));

			match ev_decl.evty {
				EvType::Reliable => line!(
					output,
					"save_player({player});",
					player = casing(file.casing, "Player", "player", "player")
				),

				EvType::Unreliable => {
					line!(output, "local buff = buffer.create(outgoing_used);");
					line!(output, "buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used);");
					line!(
						output,
						"unreliable:FireClient({player}, buff);",
						player = casing(file.casing, "Player", "player", "player")
					)
				}
			};

			output.untab().line("end,".into());
		}

		{
			// FireAll
			line!(
				output,
				"{fire_all} = function({value}: {ty})",
				fire_all = casing(file.casing, "FireAll", "fireAll", "fire_all"),
			)
			.tab();

			output.line(load_empty.into());
			line!(output, "write_event_id({id});");

			luau_ir(output, gen_ser(&ev_decl.data, value, file.write_checks));

			match ev_decl.evty {
				EvType::Reliable => {
					line!(output, "local buff = outgoing_buff;");
					line!(output, "local used = outgoing_used;");

					line!(output, "for _, player in game:GetService(\"Players\"):GetPlayers() do").tab();
					line!(output, "load_player(player);");
					line!(output, "buffer.copy(outgoing_buff, alloc(used), buff, 0, used);");
					line!(output, "save_player(player);");
					output.untab().line("end;".into());
				}

				EvType::Unreliable => {
					line!(output, "local buff = buffer.create(outgoing_used);");
					line!(output, "buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used);");

					line!(output, "for _, player in game:GetService(\"Players\"):GetPlayers() do").tab();
					line!(output, "unreliable:FireClient(player, buff);");
					output.untab().line("end;".into());
				}
			}

			output.untab().line("end,".into());
		}

		{
			// FireAllExcept

			let except = casing(file.casing, "Except", "except", "except");

			line!(
				output,
				"{fire_all_except} = function({except}: Player, {value}: {ty})",
				fire_all_except = casing(file.casing, "FireAllExcept", "fireAllExcept", "fire_all_except"),
			)
			.tab();

			output.line(load_empty.into());
			line!(output, "write_event_id({id});");

			luau_ir(output, gen_ser(&ev_decl.data, value, file.write_checks));

			match ev_decl.evty {
				EvType::Reliable => {
					line!(output, "local buff = outgoing_buff;");
					line!(output, "local used = outgoing_used;");

					line!(output, "for _, player in game:GetService(\"Players\"):GetPlayers() do").tab();
					line!(output, "if player ~= {except} then").tab();
					line!(output, "load_player(player);");
					line!(output, "buffer.copy(outgoing_buff, alloc(used), buff, 0, used);");
					line!(output, "save_player(player);");
					output.untab().line("end;".into());
					output.untab().line("end;".into());
				}

				EvType::Unreliable => {
					line!(output, "local buff = buffer.create(outgoing_used);");
					line!(output, "buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used);");

					line!(output, "for _, player in game:GetService(\"Players\"):GetPlayers() do").tab();
					line!(output, "if player ~= {except} then").tab();
					line!(output, "unreliable:FireClient(player, buff);");
					output.untab().line("end;".into());
					output.untab().line("end;".into());
				}
			}

			output.untab().line("end,".into());
		}

		{
			// FireList
			let player_list = casing(file.casing, "PlayerList", "playerList", "player_list");

			line!(
				output,
				"{fire_list} = function({player_list}: {{ Player }}, {value}: {ty})",
				fire_list = casing(file.casing, "FireList", "fireList", "fire_list"),
			)
			.tab();

			output.line(load_empty.into());
			line!(output, "write_event_id({id});");

			luau_ir(output, gen_ser(&ev_decl.data, value, file.write_checks));

			match ev_decl.evty {
				EvType::Reliable => {
					line!(output, "local buff = outgoing_buff;");
					line!(output, "local used = outgoing_used;");

					line!(output, "for _, player in {player_list} do").tab();
					line!(output, "load_player(player);");
					line!(output, "buffer.copy(outgoing_buff, alloc(used), buff, 0, used);");
					line!(output, "save_player(player);");
					output.untab().line("end;".into());
				}

				EvType::Unreliable => {
					line!(output, "local buff = buffer.create(outgoing_used);");
					line!(output, "buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used);");

					line!(output, "for _, player in {player_list} do").tab();
					line!(output, "unreliable:FireClient(player, buff);");
					output.untab().line("end;".into());
				}
			}

			output.untab().line("end,".into());
		}

		output.untab().line("};".into());
	}
}

fn server_return_listen(output: &mut Output, file: &File) {
	let callback = casing(file.casing, "Callback", "callback", "callback");

	for (i, ev_decl) in file
		.ev_decls
		.iter()
		.enumerate()
		.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
	{
		let id = i + 1;
		let ty = luau_ty(&ev_decl.data);

		line!(output, "{} = {{", ev_decl.name).tab();

		match ev_decl.call {
			EvCall::SingleSync | EvCall::SingleAsync => {
				line!(
					output,
					"{set_callback} = function({callback}: (Player, {ty}) -> ())",
					set_callback = casing(file.casing, "SetCallback", "setCallback", "set_callback"),
				)
				.tab();

				line!(output, "event_user_call[{id}] = {callback};", id = id);

				output.untab().line("end,".into());
			}

			EvCall::ManySync | EvCall::ManyAsync => {
				line!(
					output,
					"{on} = function({callback}: (Player, {ty}) -> ())",
					on = casing(file.casing, "On", "on", "on"),
				)
				.tab();

				line!(output, "table.insert(event_user_call[{id}], {callback});", id = id);

				line!(output, "return function()").tab();
				line!(output, "local index = table.find(event_user_call[{id}], {callback});");
				line!(output, "if index then").tab();
				line!(output, "table.remove(event_user_call[{id}], index);");
				output.untab().line("end;".into());
				output.untab().line("end;".into());

				output.untab().line("end,".into());
			}
		}

		output.untab().line("};".into());
	}
}

pub fn server(file: &File) -> String {
	let mut output = Output::new(format!("{}\n{}", LUAU_BASE, LUAU_SERVER));

	let event_id_ty = NumTy::from_f64(0.0, file.ev_decls.len() as f64);

	line!(
		output,
		"read_event_id = function() return buffer.read{}(incoming_buff, read({})); end;",
		luau_numty(event_id_ty),
		event_id_ty.size()
	);

	line!(
		output,
		"write_event_id = function(id) buffer.write{}(outgoing_buff, alloc({}), id); end;",
		luau_numty(event_id_ty),
		event_id_ty.size()
	);

	for ty_decl in file.ty_decls.iter() {
		luau_tydecl(&mut output, ty_decl, file.write_checks, true);
	}

	line!(output, "event_callbacks = table.create({});", file.ev_decls.len());
	line!(output, "event_user_call = table.create({});", file.ev_decls.len());

	server_callbacks(&mut output, file);

	line!(output, "return {{").tab();

	server_return_fire(&mut output, file);
	server_return_listen(&mut output, file);

	output.untab().line("};".into());

	output.get()
}

fn client_callbacks(output: &mut Output, file: &File) {
	for (i, ev_decl) in file
		.ev_decls
		.iter()
		.enumerate()
		.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
	{
		let id = i + 1;

		line!(output, "event_callbacks[{id}] = function()").tab();
		line!(output, "local value;");
		luau_ir(output, gen_des(&ev_decl.data, "value", false));

		match ev_decl.call {
			EvCall::SingleSync => {
				line!(output, "event_user_call[{id}](value);");
			}

			EvCall::SingleAsync => {
				line!(output, "task.spawn(event_user_call[{id}], value);");
			}

			EvCall::ManySync => {
				line!(output, "for _, callback in event_user_call[{id}] do").tab();
				line!(output, "callback(value);");
				output.untab().line("end;".into());
			}

			EvCall::ManyAsync => {
				line!(output, "for _, callback in event_user_call[{id}] do").tab();
				line!(output, "task.spawn(callback, value);");
				output.untab().line("end;".into());
			}
		};

		output.untab().line("end;".into());
	}
}

fn client_return_fire(output: &mut Output, file: &File) {
	let value = casing(file.casing, "Value", "value", "value");

	for (i, ev_decl) in file
		.ev_decls
		.iter()
		.enumerate()
		.filter(|(_, ev_decl)| ev_decl.from == EvSource::Client)
	{
		let id = i + 1;
		let ty = luau_ty(&ev_decl.data);

		line!(output, "{name} = {{", name = ev_decl.name).tab();

		line!(
			output,
			"{fire} = function({value}: {ty})",
			fire = casing(file.casing, "Fire", "fire", "fire"),
		)
		.tab();

		match ev_decl.evty {
			EvType::Reliable => {}
			EvType::Unreliable => {
				line!(output, "local stored = save();");
				line!(output, "load_empty();");
			}
		};

		line!(output, "write_event_id({id});");

		luau_ir(output, gen_ser(&ev_decl.data, value, file.write_checks));

		match ev_decl.evty {
			EvType::Reliable => {}
			EvType::Unreliable => {
				line!(output, "local buff = buffer.create(outgoing_used);");
				line!(output, "buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used);");
				line!(output, "unreliable:FireServer(buff);");
				line!(output, "load(stored);");
			}
		};

		output.untab().line("end,".into());

		output.untab().line("},".into());
	}
}

fn client_return_listen(output: &mut Output, file: &File) {
	let callback = casing(file.casing, "Callback", "callback", "callback");

	for (i, ev_decl) in file
		.ev_decls
		.iter()
		.enumerate()
		.filter(|(_, ev_decl)| ev_decl.from == EvSource::Server)
	{
		let id = i + 1;
		let ty = luau_ty(&ev_decl.data);

		line!(output, "{name} = {{", name = ev_decl.name).tab();

		match ev_decl.call {
			EvCall::SingleSync | EvCall::SingleAsync => {
				line!(
					output,
					"{set_callback} = function({callback}: ({ty}) -> ())",
					set_callback = casing(file.casing, "SetCallback", "setCallback", "set_callback"),
				)
				.tab();

				line!(output, "event_user_call[{id}] = {callback};", id = id);

				output.untab().line("end,".into());
			}

			EvCall::ManySync | EvCall::ManyAsync => {
				line!(
					output,
					"{on} = function({callback}: ({ty}) -> ())",
					on = casing(file.casing, "On", "on", "on"),
				)
				.tab();

				line!(output, "table.insert(event_user_call[{id}], {callback});", id = id);

				line!(output, "return function()").tab();
				line!(output, "local index = table.find(event_user_call[{id}], {callback});");
				line!(output, "if index then").tab();
				line!(output, "table.remove(event_user_call[{id}], index);");
				output.untab().line("end;".into());
				output.untab().line("end;".into());

				output.untab().line("end,".into());
			}
		}

		output.untab().line("},".into());
	}
}

pub fn client(file: &File) -> String {
	let mut output = Output::new(format!("{}\n{}", LUAU_BASE, LUAU_CLIENT));

	let event_id_ty = NumTy::from_f64(0.0, file.ev_decls.len() as f64);

	line!(
		output,
		"read_event_id = function() return buffer.read{}(incoming_buff, read({})); end;",
		luau_numty(event_id_ty),
		event_id_ty.size()
	);

	line!(
		output,
		"write_event_id = function(id) buffer.write{}(outgoing_buff, alloc({}), id); end;",
		luau_numty(event_id_ty),
		event_id_ty.size()
	);

	for ty_decl in file.ty_decls.iter() {
		luau_tydecl(&mut output, ty_decl, file.write_checks, false);
	}

	line!(output, "event_callbacks = table.create({});", file.ev_decls.len());
	line!(output, "event_user_call = table.create({});", file.ev_decls.len());

	client_callbacks(&mut output, file);

	line!(output, "return {{").tab();

	client_return_fire(&mut output, file);
	client_return_listen(&mut output, file);

	output.untab().line("};".into());

	output.get()
}
