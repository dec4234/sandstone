//! Tests for the server parts of `sandstone::game::command`: merge and remove, requirements,
//! handlers, suggestion providers, argument types and error text.
//!
//! A server sends each player a graph with only the nodes that the player can use. If the parse
//! on the server does not use the same rules, a player can run a command that the client does
//! not show, or the client shows a command that the server rejects.

#[cfg(test)]
mod test {
    use sandstone::game::command::{
        ArgumentError, ArgumentType, ArgumentValue, BlockPosArgument, CommandGraph, CommandParseErrorKind, CommandSource, Coordinates, CustomValue, EntityArgument, EntitySelector, EnumArgument, GameModeArgument,
        Suggestion, Vec3Argument, WorldCoordinate, ask_server,
    };
    use sandstone::game::player::PlayerGamemode;
    use sandstone::protocol::packets::CommandsGraphPacket;
    use sandstone::protocol_types::datatypes::chat::{ComponentType, TextComponent};
    use sandstone::protocol_types::datatypes::command::Parser;
    use sandstone::protocol_types::datatypes::var_types::VarInt;

    /// The names of the root literals in a packet.
	fn root_names(packet: &CommandsGraphPacket) -> Vec<String> {
		let nodes = packet.nodes.slice();
		let root = &nodes[packet.root_index.0 as usize];
		root.children.iter().filter_map(|c| nodes[c.0 as usize].name.clone()).collect()
	}

	// ---- merge and remove ----

	/// Two plugins that add sub-commands to `/game` must make one `game` node. With two nodes,
	/// the parse uses only the first, and the commands of the second plugin cannot run.
	#[test]
	fn add_literal_merges_with_an_existing_literal() {
		let mut g: CommandGraph = CommandGraph::new();
		let game = g.add_literal(g.root(), "game");
		let start = g.add_literal(game, "start");
		g.set_executable(start, true);

		let game_again = g.add_literal(g.root(), "game");
		assert_eq!(game_again, game);
		let stop = g.add_literal(game_again, "stop");
		g.set_executable(stop, true);

		assert_eq!(g.node(g.root()).children.len(), 1);
		assert_eq!(g.parse(&(), "game start").unwrap().executed(), start);
		assert_eq!(g.parse(&(), "game stop").unwrap().executed(), stop);
		assert_eq!(g.child(g.root(), "game"), Some(game));
		assert_eq!(g.child(game, "missing"), None);

		let amount = g.add_argument(game, "amount", Parser::integer(None, None));
		assert_eq!(g.add_argument(game, "amount", Parser::integer(None, None)), amount);
	}

	#[test]
	#[should_panic(expected = "different parser")]
	fn same_argument_name_with_a_different_parser_panics() {
		// The old node is kept in Brigadier, so the new parser would be lost with no error.
		let mut g: CommandGraph = CommandGraph::new();
		let root = g.root();
		g.add_argument(root, "x", Parser::integer(None, None));
		g.add_argument(root, "x", Parser::BrigadierBool);
	}

	/// A plugin that unloads removes its commands while the server runs.
	#[test]
	fn remove_takes_the_subtree_out_of_parse_and_packet() {
		let mut g: CommandGraph<(), &str> = CommandGraph::new();
		let root = g.root();
		let a = g.add_literal(root, "a");
		g.set_handler(a, "a");
		let b = g.add_literal(root, "b");
		let b_x = g.add_argument(b, "x", Parser::integer(None, None));
		g.set_handler(b_x, "b");
		let c = g.add_literal(root, "c");
		g.set_handler(c, "c");
		// A redirect into the removed subtree must not keep it alive.
		let alias = g.add_literal(root, "alias");
		g.set_redirect(alias, Some(b));

		assert_eq!(g.remove(root, "b"), Some(b));
		assert_eq!(g.remove(root, "b"), None, "a second remove finds nothing");

		let e = g.parse(&(), "b 1").unwrap_err();
		assert_eq!(e.kind, CommandParseErrorKind::UnknownCommand);
		assert_eq!(g.node(alias).redirect, None);
		assert_eq!(g.node(b_x).handler(), None, "the removed nodes lose their handlers");

		// Other ids do not change.
		assert_eq!(g.parse(&(), "c").unwrap().executed(), c);
		assert_eq!(g.parse(&(), "c").unwrap().handler, Some("c"));

		// The packet has no removed nodes, and the indices are packed.
		let packet = g.to_packet();
		assert_eq!(packet.nodes.slice().len(), 4, "root, a, c and alias");
		assert_eq!(root_names(&packet), vec!["a", "c", "alias"]);
		let received = <CommandGraph>::from_packet(&packet).expect("the packet must have valid indices");
		assert!(received.parse(&(), "c").is_ok());
	}

	// ---- requirements ----

	/// The sender is a permission level.
	struct Level;

	impl CommandSource for Level {
		type Sender<'a> = u8;
	}

	/// The handler is a name, so the test can see which handler the parse gives.
	fn permission_graph() -> CommandGraph<Level, &'static str> {
		let mut g = CommandGraph::new();
		let root = g.root();
		let help = g.add_literal(root, "help");
		g.set_handler(help, "help");
		let op = g.add_literal(root, "op");
		g.set_requirement(op, |level: &u8| *level >= 2);
		let target = g.add_argument(op, "target", Parser::BrigadierString(VarInt(0)));
		g.set_handler(target, "op");
		g
	}

	/// A player without permission must not see `op` in tab-complete, and must get the same error
	/// as for a command that does not exist, as in vanilla.
	#[test]
	fn requirement_hides_the_node_from_parse_suggest_and_packet() {
		let g = permission_graph();

		let e = g.parse(&0, "op steve").unwrap_err();
		assert_eq!((e.position, e.kind), (0, CommandParseErrorKind::UnknownCommand));
		assert_eq!(g.parse(&2, "op steve").unwrap().handler, Some("op"));

		assert_eq!(g.suggest(&0, "").texts(), vec!["help"]);
		assert_eq!(g.suggest(&4, "").texts(), vec!["help", "op"]);

		assert_eq!(root_names(&g.to_packet_for(&0)), vec!["help"]);
		assert_eq!(root_names(&g.to_packet_for(&2)), vec!["help", "op"]);
		assert_eq!(root_names(&g.to_packet()), vec!["help", "op"], "to_packet does not check requirements");

		// The filtered packet must have valid indices, or the client disconnects.
		let received = <CommandGraph>::from_packet(&g.to_packet_for(&0)).unwrap();
		assert_eq!(received.nodes().len(), 2);
	}

	/// A requirement on a redirect target hides the redirect, as vanilla does. The node then has
	/// no children, and the client cannot follow it.
	#[test]
	fn redirect_to_a_hidden_node_is_dropped_from_the_packet() {
		let mut g: CommandGraph<Level> = CommandGraph::new();
		let root = g.root();
		let secret = g.add_literal(root, "secret");
		g.set_executable(secret, true);
		g.set_requirement(secret, |level: &u8| *level >= 4);
		let alias = g.add_literal(root, "alias");
		g.set_redirect(alias, Some(secret));

		let packet = g.to_packet_for(&0);
		let alias_node = packet.nodes.slice().iter().find(|n| n.name.as_deref() == Some("alias")).unwrap();
		assert!(!alias_node.flags.has_redirect && alias_node.redirect_node.is_none());
		assert!(g.to_packet_for(&4).nodes.slice().iter().any(|n| n.flags.has_redirect));
	}

	// ---- handlers ----

	/// The handler comes from the node, so a server does not need a map from NodeId to handler.
	#[test]
	fn parse_returns_the_handler_of_the_executed_node() {
		let mut g: CommandGraph<(), fn(i32) -> i32> = CommandGraph::new();
		let root = g.root();
		let double = g.add_literal(root, "double");
		let n = g.add_argument(double, "n", Parser::integer(None, None));
		g.set_handler(n, |x| x * 2);
		assert!(g.node(n).executable, "set_handler must make the node executable");

		let p = g.parse(&(), "double 21").unwrap();
		let Some(ArgumentValue::Integer(x)) = p.argument("n") else { panic!() };
		assert_eq!((p.handler.unwrap())(*x), 42);

		// A node that is executable but has no handler gives None, for example in a client graph.
		g.set_executable(double, true);
		assert!(g.parse(&(), "double").unwrap().handler.is_none());
	}

	// ---- suggestions ----

	/// The sender borrows the names of the online players. A server keeps the graph in its game
	/// state, so the sender must be able to borrow data that lives shorter than the graph.
	struct Online;

	impl CommandSource for Online {
		type Sender<'a> = &'a [String];
	}

	fn names(names: &[&str]) -> Vec<String> {
		names.iter().map(|n| n.to_string()).collect()
	}

	/// The server must be able to suggest online player names. Without a provider, the client
	/// asks the server and gets nothing.
	#[test]
	fn suggestion_provider_gives_values_with_tooltips() {
		let mut g: CommandGraph<Online> = CommandGraph::new();
		let msg = g.add_literal(g.root(), "msg");
		let target = g.add_argument(msg, "target", Parser::BrigadierString(VarInt(0)));
		g.set_suggestion_provider(target, |online: &&[String], _partial| online.iter().map(|n| Suggestion::with_tooltip(n.as_str(), TextComponent::new("online"))).collect());

		// The client asks the server only if the node has a suggestions type.
		let packet = g.to_packet();
		let wire = packet.nodes.slice().iter().find(|n| n.name.as_deref() == Some("target")).unwrap();
		assert_eq!(wire.suggestions, Some(ask_server()));

		let online = names(&["Alice", "alfred", "Bob"]);
		let s = g.suggest(&online.as_slice(), "msg al");
		assert_eq!(s.texts(), vec!["Alice", "alfred"], "prefix filter that ignores case, sorted");
		assert_eq!(s.nodes, vec![target], "the node of the last word");
		assert_eq!(s.matches[0].tooltip, Some(TextComponent::new("online")));

		let response = s.into_packet(VarInt(3), "msg al");
		assert!(response.matches.slice().iter().all(|m| m.tooltip.is_present()));
	}

	#[test]
	fn builtin_and_typed_suggestions() {
		let mut g: CommandGraph = CommandGraph::new();
		let root = g.root();
		let gm = g.add_literal(root, "gm");
		g.add_typed_argument(gm, "mode", GameModeArgument);
		let color = g.add_literal(root, "color");
		let value = g.add_typed_argument(color, "value", EnumArgument::new(["Red", "Green", "Blue"]));

		assert_eq!(g.suggest(&(), "gm s").texts(), vec!["spectator", "survival"]);
		assert_eq!(g.suggest(&(), "color g").texts(), vec!["Green"]);
		assert_eq!(g.suggest(&(), "color ").nodes, vec![value]);
		assert!(g.suggest(&(), "color").nodes.is_empty(), "the last word is the literal, not the argument");
	}

	// ---- argument types ----

	/// A server type that accepts only online players.
	struct OnlinePlayer;

	impl ArgumentType<Online> for OnlinePlayer {
		fn parser(&self) -> Parser {
			Parser::MinecraftEntity(0x03)
		}

		fn read(&self, online: &&[String], rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
			let word = rest.split(' ').next().unwrap_or("");
			match online.iter().position(|n| *n == word) {
				Some(index) => Ok((word.len(), ArgumentValue::Custom(CustomValue::new(index)))),
				None => Err(ArgumentError::custom(format!("{word} is not online"))),
			}
		}
	}

	/// A value that is not valid in the game must fail the read, so that the parse tries the
	/// next argument child, as Brigadier does.
	#[test]
	fn typed_argument_can_reject_and_the_next_child_is_tried() {
		let mut g: CommandGraph<Online> = CommandGraph::new();
		let find = g.add_literal(g.root(), "find");
		let player = g.add_typed_argument(find, "player", OnlinePlayer);
		g.set_executable(player, true);
		let item = g.add_argument(find, "item", Parser::BrigadierString(VarInt(0)));
		g.set_executable(item, true);

		let online = names(&["alice", "bob"]);
		let p = g.parse(&online.as_slice(), "find bob").unwrap();
		assert_eq!(p.executed(), player);
		assert_eq!(p.custom::<usize>("player"), Some(&1));
		assert_eq!(p.custom::<String>("player"), None, "wrong type");

		let p = g.parse(&online.as_slice(), "find carol").unwrap();
		assert_eq!(p.executed(), item);
		assert_eq!(p.argument("item"), Some(&ArgumentValue::String("carol".to_string())));

		// The client gets the parser of the type.
		assert_eq!(g.to_packet().nodes.slice()[player.0].parser, Some(Parser::MinecraftEntity(0x03)));
	}

	fn world(relative: bool, value: f64) -> WorldCoordinate {
		WorldCoordinate {
			relative,
			value,
		}
	}

	#[test]
	fn coordinates_read_relative_and_local_values() {
		let mut g: CommandGraph = CommandGraph::new();
		let tp = g.add_literal(g.root(), "tp");
		let pos = g.add_typed_argument(tp, "pos", Vec3Argument);
		g.set_executable(pos, true);
		let set = g.add_literal(g.root(), "set");
		let block = g.add_typed_argument(set, "pos", BlockPosArgument);
		g.set_executable(block, true);

		// Like vanilla, a whole number for x and z in a vec3 is the center of the block.
		let p = g.parse(&(), "tp 1 2 ~-3.5").unwrap();
		assert_eq!(p.argument("pos"), Some(&ArgumentValue::Coordinates(Coordinates::World([world(false, 1.5), world(false, 2.0), world(true, -3.5)]))));
		let p = g.parse(&(), "tp 1.0 2 3.25").unwrap();
		assert_eq!(p.argument("pos"), Some(&ArgumentValue::Coordinates(Coordinates::World([world(false, 1.0), world(false, 2.0), world(false, 3.25)]))));
		let p = g.parse(&(), "tp ^ ^1 ^-2").unwrap();
		assert_eq!(p.argument("pos"), Some(&ArgumentValue::Coordinates(Coordinates::Local([0.0, 1.0, -2.0]))));

		// A block position has no center correction, and must have whole numbers.
		let p = g.parse(&(), "set 1 2 3").unwrap();
		assert_eq!(p.argument("pos"), Some(&ArgumentValue::Coordinates(Coordinates::World([world(false, 1.0), world(false, 2.0), world(false, 3.0)]))));
		// Brigadier reads "1.5" as one number, then it is not a valid integer.
		let e = g.parse(&(), "set 1.5 2 3").unwrap_err();
		assert_eq!((e.position, e.translation_key()), (4, Some("parsing.int.invalid")));

		let key = |input: &str| g.parse(&(), input).unwrap_err().translation_key().map(str::to_string);
		assert_eq!(key("tp ^ 1 ^").as_deref(), Some("argument.pos.mixed"));
		assert_eq!(key("tp 1 ^ 1").as_deref(), Some("argument.pos.mixed"));
		assert_eq!(key("tp 1 2").as_deref(), Some("argument.pos3d.incomplete"));
		assert_eq!(key("tp 1 x 3").as_deref(), Some("parsing.double.expected"));
	}

	/// Local coordinates are left, up and forwards from the view of the sender. With a wrong
	/// formula, `tp ^ ^ ^5` moves the player in the wrong direction.
	#[test]
	fn local_coordinates_resolve_with_the_vanilla_formula() {
		let close = |a: [f64; 3], b: [f64; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5);
		let local = Coordinates::Local([1.0, 2.0, 3.0]);
		// Yaw 0 looks to +z (south). Left is +x (east).
		assert!(close(local.resolve([10.0, 64.0, 10.0], 0.0, 0.0), [11.0, 66.0, 13.0]));
		// Yaw 90 looks to -x (west). Left is +z (south).
		assert!(close(local.resolve([0.0, 0.0, 0.0], 90.0, 0.0), [-3.0, 2.0, 1.0]));
		// Pitch -90 looks up. Forwards is +y.
		assert!(close(Coordinates::Local([0.0, 0.0, 1.0]).resolve([0.0, 0.0, 0.0], 0.0, -90.0), [0.0, 1.0, 0.0]));

		let rel = Coordinates::World([world(true, 1.0), world(false, 5.0), world(true, -0.5)]);
		assert_eq!(rel.resolve([2.0, 70.0, 3.0], 0.0, 0.0), [3.0, 5.0, 2.5]);
		assert_eq!(rel.resolve_block([2.0, 70.0, 3.0], 0.0, 0.0), [3, 5, 2]);
		assert_eq!(Coordinates::World([world(false, -0.5), world(false, 0.0), world(false, 0.0)]).resolve_block([0.0; 3], 0.0, 0.0), [-1, 0, 0]);
	}

	/// A selector with a space in quotes, such as `@e[name="a b"]`, must be one argument. Before,
	/// the text was cut at the space.
	#[test]
	fn entity_selectors_read_options_and_check_flags() {
		let mut g: CommandGraph = CommandGraph::new();
		let kill = g.add_literal(g.root(), "kill");
		let targets = g.add_typed_argument(kill, "targets", EntityArgument::entities());
		let reason = g.add_argument(targets, "reason", Parser::MinecraftMessage);
		g.set_executable(reason, true);

		let p = g.parse(&(), r#"kill @e[name="a b",type=!cow,scores={x=1..}] bye"#).unwrap();
		assert_eq!(p.executed(), reason);
		let expected = EntitySelector::Selector {
			kind: 'e',
			options: vec![("name".into(), r#""a b""#.into()), ("type".into(), "!cow".into()), ("scores".into(), "{x=1..}".into())],
		};
		assert_eq!(p.argument("targets"), Some(&ArgumentValue::Entity(expected)));
		assert_eq!(p.argument("reason"), Some(&ArgumentValue::Raw("bye".to_string())));

		let p = g.parse(&(), "kill Steve x").unwrap();
		assert_eq!(p.argument("targets"), Some(&ArgumentValue::Entity(EntitySelector::Name("Steve".into()))));
		let p = g.parse(&(), "kill 0a1b2c3d-0000-0000-0000-000000000001 x").unwrap();
		assert!(matches!(p.argument("targets"), Some(ArgumentValue::Entity(EntitySelector::Uuid(_)))));

		let key = |input: &str| g.parse(&(), input).unwrap_err().translation_key().map(str::to_string);
		assert_eq!(key("kill @x y").as_deref(), Some("argument.entity.selector.unknown"));
		assert!(g.parse(&(), "kill @e[name=a y").is_err(), "a bracket that is not closed is an error");

		// The rules of the single and players-only flags, as in vanilla.
		let check = |arg: EntityArgument, input: &str| arg.read_selector(input).err().and_then(|e| e.key.map(|k| k.into_owned()));
		assert_eq!(check(EntityArgument::entity(), "@a").as_deref(), Some("argument.entity.toomany"));
		assert_eq!(check(EntityArgument::player(), "@a").as_deref(), Some("argument.player.toomany"));
		assert_eq!(check(EntityArgument::entity(), "@a[limit=1]"), None);
		assert_eq!(check(EntityArgument::players(), "@e").as_deref(), Some("argument.player.entities"));
		assert_eq!(check(EntityArgument::players(), "@e[type=player]"), None);
		assert_eq!(check(EntityArgument::player(), "@s"), None, "@s is allowed for players-only");
		assert_eq!(check(EntityArgument::player(), "@p"), None);
	}

	/// Raw tokens stop at a space only outside quotes and brackets, so NBT with spaces stays in
	/// one argument.
	#[test]
	fn raw_tokens_keep_brackets_and_quotes_together() {
		let mut g: CommandGraph = CommandGraph::new();
		let give = g.add_literal(g.root(), "give");
		let item = g.add_argument(give, "item", Parser::MinecraftItemStack);
		let count = g.add_argument(item, "count", Parser::integer(None, None));
		g.set_executable(count, true);

		let p = g.parse(&(), r#"give stone[custom_name="a b",lore=['x y']] 5"#).unwrap();
		assert_eq!(p.argument("item"), Some(&ArgumentValue::Raw(r#"stone[custom_name="a b",lore=['x y']]"#.to_string())));
		assert_eq!(p.argument("count"), Some(&ArgumentValue::Integer(5)));
		assert!(g.parse(&(), "give stone[a=b 5").is_err());
	}

	#[test]
	fn game_mode_and_enum_values() {
		let mut g: CommandGraph = CommandGraph::new();
		let gm = g.add_literal(g.root(), "gm");
		let mode = g.add_typed_argument(gm, "mode", GameModeArgument);
		g.set_executable(mode, true);
		let color = g.add_literal(g.root(), "color");
		let value = g.add_typed_argument(color, "value", EnumArgument::new(["Red", "Green"]));
		g.set_executable(value, true);

		assert_eq!(g.parse(&(), "gm creative").unwrap().argument("mode"), Some(&ArgumentValue::GameMode(PlayerGamemode::CREATIVE)));
		assert_eq!(g.parse(&(), "gm hard").unwrap_err().translation_key(), Some("argument.gamemode.invalid"));
		assert_eq!(g.parse(&(), "color RED").unwrap().argument("value"), Some(&ArgumentValue::String("Red".into())));
		assert_eq!(g.parse(&(), "color pink").unwrap_err().translation_key(), Some("argument.enum.invalid"));

		let packet = g.to_packet();
		assert_eq!(packet.nodes.slice()[mode.0].parser, Some(Parser::MinecraftGamemode));
		assert_eq!(packet.nodes.slice()[value.0].suggestions, Some(ask_server()), "an enum asks the server");
	}

	// ---- errors ----

	fn translate(component: &TextComponent) -> (&str, Option<&str>, Vec<String>) {
		let ComponentType::Translatable(t) = &component.content else { panic!("not translatable: {component:?}") };
		let args = t.with.iter().flatten().map(|c| match &c.content {
			ComponentType::Text { text } => text.clone(),
			_ => panic!(),
		});
		(&t.translate, t.fallback.as_deref(), args.collect())
	}

	/// The client shows a translated message only if the server sends the vanilla key and the
	/// arguments in the vanilla order. The fallback is the English text.
	#[test]
	fn errors_have_vanilla_keys_and_arguments() {
		let mut g: CommandGraph = CommandGraph::new();
		let give = g.add_literal(g.root(), "give");
		let count = g.add_argument(give, "count", Parser::integer(Some(1), Some(64)));
		g.set_executable(count, true);

		let e = g.parse(&(), "give 0").unwrap_err();
		let message = e.message();
		assert_eq!(translate(&message), ("argument.integer.low", Some("Integer must not be less than 1, found 0"), vec!["1".into(), "0".into()]));
		assert_eq!(message.color.as_deref(), Some("red"));

		let e = g.parse(&(), "give x").unwrap_err();
		assert_eq!(translate(&e.message()).0, "parsing.int.expected");
		let e = g.parse(&(), "give 1-").unwrap_err();
		assert_eq!(translate(&e.message()).2, vec!["1-".to_string()], "parsing.int.invalid has the text");

		assert_eq!(translate(&g.parse(&(), "nothing").unwrap_err().message()).0, "command.unknown.command");
		assert_eq!(translate(&g.parse(&(), "give").unwrap_err().message()).0, "command.unknown.command");
		assert_eq!(translate(&g.parse(&(), "give 5x").unwrap_err().message()).0, "command.expected.separator");

		// A custom error has no key, and the message is literal text.
		let e = ArgumentError::custom("nope");
		assert_eq!(e.to_component().content, ComponentType::Text { text: "nope".into() });
	}

	fn plain(component: &TextComponent) -> String {
		let mut out = match &component.content {
			ComponentType::Text { text } => text.clone(),
			ComponentType::Translatable(t) => t.fallback.clone().unwrap_or_default(),
			_ => String::new(),
		};
		for child in component.extra.iter().flatten() {
			out.push_str(&plain(child));
		}
		out
	}

	/// The vanilla context line: at most 10 characters before the error, then the rest in red,
	/// then `<--[HERE]`.
	#[test]
	fn context_shows_the_input_up_to_the_error() {
		let mut g: CommandGraph = CommandGraph::new();
		let give = g.add_literal(g.root(), "giveallitems");
		let count = g.add_argument(give, "count", Parser::integer(None, None));
		g.set_executable(count, true);

		let input = "giveallitems abc";
		let e = g.parse(&(), input).unwrap_err();
		assert_eq!(e.position, 13);
		let line = e.context(input);
		assert_eq!(plain(&line), "...eallitems abc<--[HERE]");
		let extra = line.extra.as_ref().unwrap();
		assert_eq!(extra[2], {
			let mut rest = TextComponent::new("abc");
			rest.color = Some("red".into());
			rest.underlined = Some(true);
			rest
		});
		assert!(line.click_event.is_some(), "a click puts the command in the chat box");

		let e = g.parse(&(), "nope").unwrap_err();
		assert_eq!(plain(&e.context("nope")), "nope<--[HERE]");
	}

	/// Errors and suggestions both use byte positions, so a server can slice the input with them.
	#[test]
	fn error_positions_are_bytes() {
		let mut g: CommandGraph = CommandGraph::new();
		let say = g.add_literal(g.root(), "say");
		let word = g.add_argument(say, "word", Parser::BrigadierString(VarInt(1)));
		let n = g.add_argument(word, "n", Parser::integer(None, None));
		g.set_executable(n, true);

		let input = r#"say "é😀" x"#;
		let e = g.parse(&(), input).unwrap_err();
		// "say " (4) + quoted string (1 + 2 + 4 + 1) + " " (1) = 13 bytes.
		assert_eq!(e.position, 13);
		assert_eq!(&input[e.position..], "x");
		assert_eq!(plain(&e.context(input)), "say \"é😀\" x<--[HERE]", "10 characters or less: no ...");
	}
}
