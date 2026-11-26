//! Tests for the high level command graph in `sandstone::game::command`.
//!
//! A server sends its graph to the client, and the client uses it for tab-complete and to check
//! the command before it sends it. The server must then read the command with the same rules,
//! or a command that the client accepts can fail on the server.

#[cfg(test)]
mod test {
    use sandstone::game::command::{ArgumentValue, CommandGraph, CommandGraphError, CommandNodeKind, CommandParseErrorKind, NodeId};
    use sandstone::protocol::packets::CommandsGraphPacket;
    use sandstone::protocol::serialization::serializer_types::PrefixedArray;
    use sandstone::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
    use sandstone::protocol_types::datatypes::command::{Node, NodeFlags, NodeType, Parser};
    use sandstone::protocol_types::datatypes::var_types::VarInt;

    /// The nodes of [sample] that the tests compare against.
	struct Ids {
		tp_z: NodeId,
		tp_target: NodeId,
		gm_creative: NodeId,
		give_item: NodeId,
		give_count: NodeId,
		say: NodeId,
		run: NodeId,
		setblock_block: NodeId,
		tag_name: NodeId,
		toggle_value: NodeId,
		kill_all: NodeId,
		kill_target: NodeId,
	}

	fn double() -> Parser {
		Parser::BrigadierDouble { flags: 0, min: None, max: None }
	}

	/// A graph with the same shapes as vanilla commands.
	fn sample() -> (CommandGraph, Ids) {
		let mut g = CommandGraph::new();
		let root = g.root();

		let tp = g.add_literal(root, "tp");
		let x = g.add_argument(tp, "x", double());
		let y = g.add_argument(x, "y", double());
		let tp_z = g.add_argument(y, "z", double());
		g.set_executable(tp_z, true);
		let tp_target = g.add_argument(tp, "target", Parser::MinecraftEntity(0x01));
		g.set_executable(tp_target, true);

		let gm = g.add_literal(root, "gamemode");
		let gm_creative = g.add_literal(gm, "creative");
		g.set_executable(gm_creative, true);
		let gm_survival = g.add_literal(gm, "survival");
		g.set_executable(gm_survival, true);

		let give = g.add_literal(root, "give");
		let give_target = g.add_argument(give, "targets", Parser::MinecraftEntity(0));
		let give_item = g.add_argument(give_target, "item", Parser::MinecraftItemStack);
		g.set_executable(give_item, true);
		let give_count = g.add_argument(give_item, "count", Parser::BrigadierInteger { flags: 0x03, min: Some(1), max: Some(6400) });
		g.set_executable(give_count, true);

		let say_lit = g.add_literal(root, "say");
		let say = g.add_argument(say_lit, "message", Parser::MinecraftMessage);
		g.set_executable(say, true);

		let execute = g.add_literal(root, "execute");
		let run = g.add_literal(execute, "run");
		g.set_redirect(run, Some(root));

		let setblock = g.add_literal(root, "setblock");
		let pos = g.add_argument(setblock, "pos", Parser::MinecraftBlockPos);
		let setblock_block = g.add_argument(pos, "block", Parser::MinecraftBlockState);
		g.set_executable(setblock_block, true);

		let tag = g.add_literal(root, "tag");
		let tag_name = g.add_argument(tag, "name", Parser::BrigadierString(VarInt(1)));
		g.set_executable(tag_name, true);

		let toggle = g.add_literal(root, "toggle");
		let toggle_value = g.add_argument(toggle, "value", Parser::BrigadierBool);
		g.set_executable(toggle_value, true);
		g.set_suggestions(toggle_value, Some("minecraft:ask_server".parse().unwrap()));

		// The argument is added first, so the literal must win by priority, not by order.
		let kill = g.add_literal(root, "kill");
		let kill_target = g.add_argument(kill, "target", Parser::BrigadierString(VarInt(0)));
		g.set_executable(kill_target, true);
		let kill_all = g.add_literal(kill, "all");
		g.set_executable(kill_all, true);
		g.set_restricted(kill_all, true);

		(
			g,
			Ids {
				tp_z,
				tp_target,
				gm_creative,
				give_item,
				give_count,
				say,
				run,
				setblock_block,
				tag_name,
				toggle_value,
				kill_all,
				kill_target,
			},
		)
	}

	fn wire_round_trip(packet: &CommandsGraphPacket) -> CommandsGraphPacket {
		let mut s = McSerializer::new();
		packet.mc_serialize(&mut s).unwrap();
		let mut d = McDeserializer::new(&s.output);
		let out = CommandsGraphPacket::mc_deserialize(&mut d).unwrap();
		assert!(d.is_at_end(), "the packet reader must use all bytes");
		out
	}

	// ---- packet conversion ----

	/// The client must get the same graph that the server built. If a flag, index or field is
	/// lost, the client shows wrong tab-complete or rejects valid commands.
	#[test]
	fn graph_survives_packet_bytes_round_trip() {
		let (g, _) = sample();
		let received = wire_round_trip(&g.to_packet());
		assert_eq!(CommandGraph::from_packet(&received).unwrap(), g);
	}

	/// The flags on the wire come from the node fields, not from defaults.
	#[test]
	fn to_packet_sets_flags_from_node_fields() {
		let (g, ids) = sample();
		let packet = g.to_packet();
		let nodes = packet.nodes.slice();
		assert_eq!(packet.root_index, VarInt(g.root().0 as i32));

		let run = &nodes[ids.run.0];
		assert!(run.flags.has_redirect);
		assert_eq!(run.redirect_node, Some(VarInt(g.root().0 as i32)));
		assert!(!run.flags.is_executable);

		let toggle = &nodes[ids.toggle_value.0];
		assert!(toggle.flags.has_suggestions && toggle.flags.is_executable);
		assert_eq!(toggle.suggestions, Some("minecraft:ask_server".parse().unwrap()));

		assert!(nodes[ids.kill_all.0].flags.is_restricted);
		assert!(!nodes[ids.kill_target.0].flags.is_restricted);
	}

	fn node(typ: NodeType, children: Vec<i32>, name: Option<&str>) -> Node {
		Node {
			flags: NodeFlags {
				typ,
				is_executable: false,
				has_redirect: false,
				has_suggestions: false,
				is_restricted: false,
			},
			children_count: VarInt(children.len() as i32),
			children: children.into_iter().map(VarInt).collect(),
			redirect_node: None,
			name: name.map(String::from),
			parser: None,
			suggestions: None,
		}
	}

	fn packet(nodes: Vec<Node>, root: i32) -> CommandsGraphPacket {
		CommandsGraphPacket::new(PrefixedArray::new(nodes), VarInt(root))
	}

	/// The packet reader does not check indices. A bad index from a server must be an error, not
	/// a panic later in parse.
	#[test]
	fn from_packet_rejects_index_out_of_range() {
		let p = packet(vec![node(NodeType::Root, vec![1, 2], None), node(NodeType::Literal, vec![], Some("a"))], 0);
		assert_eq!(CommandGraph::from_packet(&p), Err(CommandGraphError::IndexOutOfRange { node: 0, index: 2, len: 2 }));

		let mut lit = node(NodeType::Literal, vec![], Some("a"));
		lit.flags.has_redirect = true;
		lit.redirect_node = Some(VarInt(-1));
		let p = packet(vec![node(NodeType::Root, vec![1], None), lit], 0);
		assert_eq!(CommandGraph::from_packet(&p), Err(CommandGraphError::IndexOutOfRange { node: 1, index: -1, len: 2 }));
	}

	#[test]
	fn from_packet_rejects_bad_root_index() {
		let nodes = || vec![node(NodeType::Root, vec![1], None), node(NodeType::Literal, vec![], Some("a"))];
		assert_eq!(CommandGraph::from_packet(&packet(nodes(), 5)), Err(CommandGraphError::InvalidRoot(5)));
		// Index 1 is in range, but it is a literal.
		assert_eq!(CommandGraph::from_packet(&packet(nodes(), 1)), Err(CommandGraphError::InvalidRoot(1)));
	}

	/// The serializer writes optional fields if they are present, whatever the flags say. A node
	/// whose fields disagree with its flags makes bytes that the client cannot read.
	#[test]
	fn from_packet_rejects_fields_that_disagree_with_flags() {
		let mut no_name = node(NodeType::Literal, vec![], None);
		no_name.name = None;
		let mut redirect_no_target = node(NodeType::Literal, vec![], Some("a"));
		redirect_no_target.flags.has_redirect = true;
		let mut bad_count = node(NodeType::Literal, vec![], Some("a"));
		bad_count.children_count = VarInt(3);

		for bad in [no_name, redirect_no_target, bad_count] {
			let p = packet(vec![node(NodeType::Root, vec![1], None), bad], 0);
			assert!(matches!(CommandGraph::from_packet(&p), Err(CommandGraphError::MalformedNode { node: 1, .. })), "{p:?}");
		}
	}

	#[test]
	#[should_panic(expected = "redirected node")]
	fn children_on_a_redirected_node_panic() {
		// The parse uses the children of the redirect target, so these children could never match.
		let (mut g, ids) = sample();
		g.add_literal(ids.run, "never");
	}

	// ---- parse ----

	#[test]
	fn parses_typed_brigadier_arguments() {
		let (g, ids) = sample();
		let p = g.parse("tp 1 -2.5 3").unwrap();
		assert_eq!(p.executed(), ids.tp_z);
		assert_eq!(p.argument("x"), Some(&ArgumentValue::Double(1.0)));
		assert_eq!(p.argument("y"), Some(&ArgumentValue::Double(-2.5)));
		assert_eq!(p.argument("z"), Some(&ArgumentValue::Double(3.0)));
		assert_eq!(p.path.first(), Some(&g.root()));

		assert_eq!(g.parse("toggle false").unwrap().argument("value"), Some(&ArgumentValue::Bool(false)));
		assert_eq!(g.parse("give @s stone 64").unwrap().argument("count"), Some(&ArgumentValue::Integer(64)));
	}

	/// `tp Steve` must not stop at the failed `x` argument. The next argument child must be tried.
	#[test]
	fn tries_the_next_argument_when_one_fails() {
		let (g, ids) = sample();
		let p = g.parse("tp Steve").unwrap();
		assert_eq!(p.executed(), ids.tp_target);
		assert_eq!(p.argument("target"), Some(&ArgumentValue::Raw("Steve".to_string())));
		assert_eq!(p.argument("x"), None, "values from the failed branch must be removed");
	}

	/// Brigadier tries only the literal when the word matches it, so `kill all` never becomes a
	/// player named "all".
	#[test]
	fn literal_has_priority_over_argument() {
		let (g, ids) = sample();
		let p = g.parse("kill all").unwrap();
		assert_eq!(p.executed(), ids.kill_all);
		assert!(p.arguments.is_empty());
		assert_eq!(g.parse("kill bob").unwrap().executed(), ids.kill_target);
	}

	/// Chat Command packets have no `/`, but typed text and suggestion requests can have one.
	#[test]
	fn slash_at_start_is_optional() {
		let (g, ids) = sample();
		assert_eq!(g.parse("/gamemode creative").unwrap().executed(), ids.gm_creative);
		assert_eq!(g.parse("gamemode creative").unwrap().executed(), ids.gm_creative);
	}

	#[test]
	fn stops_only_at_executable_nodes() {
		let (g, ids) = sample();
		assert_eq!(g.parse("gamemode").unwrap_err().kind, CommandParseErrorKind::Incomplete);
		assert_eq!(g.parse("gamemode").unwrap_err().position, 8);
		assert_eq!(g.parse("").unwrap_err().kind, CommandParseErrorKind::Incomplete);
		// item is executable and count is optional.
		assert_eq!(g.parse("give @s stone").unwrap().executed(), ids.give_item);
	}

	#[test]
	fn reports_unknown_command_at_its_position() {
		let (g, _) = sample();
		let e = g.parse("gamemod creative").unwrap_err();
		assert_eq!((e.position, e.kind), (0, CommandParseErrorKind::Unknown));
		let e = g.parse("gamemode hard").unwrap_err();
		assert_eq!((e.position, e.kind), (9, CommandParseErrorKind::Unknown));
	}

	/// The min and max from the packet must be enforced, or the server accepts values that the
	/// client would reject.
	#[test]
	fn enforces_number_range() {
		let (g, _) = sample();
		for bad in ["give @s stone 0", "give @s stone 6401"] {
			let e = g.parse(bad).unwrap_err();
			assert_eq!(e.position, 14, "{bad}");
			assert!(matches!(&e.kind, CommandParseErrorKind::InvalidValue { name, .. } if name == "count"), "{bad}: {e:?}");
		}
		assert!(g.parse("give @s stone 1").is_ok());
		assert!(g.parse("give @s stone 6400").is_ok());
	}

	/// When all paths fail, the error must be the one farthest in the input. That is the most
	/// useful error for the player.
	#[test]
	fn reports_the_farthest_error() {
		let (g, _) = sample();
		let e = g.parse("tp 1 2 3x").unwrap_err();
		assert_eq!((e.position, e.kind), (8, CommandParseErrorKind::ExpectedSeparator));
	}

	/// `execute run` redirects to the root, so any command can follow it, also another
	/// `execute run`.
	#[test]
	fn follows_redirects() {
		let (g, ids) = sample();
		let p = g.parse("execute run execute run tp 1 2 3").unwrap();
		assert_eq!(p.executed(), ids.tp_z);
		assert_eq!(p.path.iter().filter(|n| **n == ids.run).count(), 2);
		assert_eq!(g.parse("execute run").unwrap_err().kind, CommandParseErrorKind::Incomplete);
	}

	#[test]
	fn reads_strings_as_brigadier_does() {
		let (g, ids) = sample();
		let p = g.parse(r#"tag "a b\" \\ c""#).unwrap();
		assert_eq!(p.executed(), ids.tag_name);
		assert_eq!(p.argument("name"), Some(&ArgumentValue::String(r#"a b" \ c"#.to_string())));
		assert_eq!(g.parse("tag 'x'").unwrap().argument("name"), Some(&ArgumentValue::String("x".to_string())));
		assert_eq!(g.parse("tag plain").unwrap().argument("name"), Some(&ArgumentValue::String("plain".to_string())));

		assert!(matches!(g.parse(r#"tag "open"#).unwrap_err().kind, CommandParseErrorKind::InvalidValue { .. }));
		assert!(matches!(g.parse(r#"tag "bad\n""#).unwrap_err().kind, CommandParseErrorKind::InvalidValue { .. }));
	}

	/// Raw parsers must take the correct number of words, or `setblock ~ ~1 ~ stone` reads `~1`
	/// as the block.
	#[test]
	fn raw_parsers_read_the_correct_number_of_words() {
		let (g, ids) = sample();
		let p = g.parse("setblock ~ ~1 ~ stone").unwrap();
		assert_eq!(p.executed(), ids.setblock_block);
		assert_eq!(p.argument("pos"), Some(&ArgumentValue::Raw("~ ~1 ~".to_string())));
		assert_eq!(p.argument("block"), Some(&ArgumentValue::Raw("stone".to_string())));

		let p = g.parse("say hello  world").unwrap();
		assert_eq!(p.executed(), ids.say);
		assert_eq!(p.argument("message"), Some(&ArgumentValue::Raw("hello  world".to_string())));

		assert!(g.parse("setblock ~ ~").is_err());
	}

	// ---- suggestions ----

	#[test]
	fn suggests_root_literals_for_partial_word() {
		let (g, _) = sample();
		let s = g.suggest("/g");
		assert_eq!(s.matches, vec!["gamemode", "give"]);
		// The client replaces the text from `start`, which is after the `/`.
		assert_eq!((s.start, s.length), (1, 1));

		assert_eq!(g.suggest("GAM").matches, vec!["gamemode"], "the match must ignore case");
		assert_eq!(g.suggest("").matches.len(), 9, "every root literal for empty input");
	}

	#[test]
	fn suggests_children_after_complete_words() {
		let (g, _) = sample();
		let s = g.suggest("gamemode c");
		assert_eq!(s.matches, vec!["creative"]);
		assert_eq!((s.start, s.length), (9, 1));
		assert_eq!(g.suggest("gamemode ").matches, vec!["creative", "survival"]);
		assert_eq!(g.suggest("toggle t").matches, vec!["true"]);
		assert_eq!(g.suggest("execute run gam").matches, vec!["gamemode"], "must follow the redirect");
		assert!(g.suggest("nothing ").matches.is_empty());
	}

	/// The protocol uses Java string indices (UTF-16 code units), not UTF-8 bytes. A wrong start
	/// makes the client replace the wrong part of the chat text.
	#[test]
	fn suggestion_range_is_in_utf16_units() {
		let mut g = CommandGraph::new();
		let greet = g.add_literal(g.root(), "greet");
		let name = g.add_argument(greet, "name", Parser::MinecraftEntity(0));
		g.add_literal(name, "loudly");

		// "é" is 2 bytes in UTF-8 and 1 unit in UTF-16. "😀" is 4 bytes and 2 units.
		let s = g.suggest("greet é😀 l");
		assert_eq!(s.matches, vec!["loudly"]);
		// "greet " (6) + "é" (1) + "😀" (2) + " " (1) = 10 units. In UTF-8 bytes it is 13.
		assert_eq!((s.start, s.length), (10, 1));
	}

	#[test]
	fn suggestions_make_a_response_packet() {
		let (g, _) = sample();
		let p = g.suggest("gamemode ").into_packet(VarInt(7));
		assert_eq!((p.id, p.start, p.length), (VarInt(7), VarInt(9), VarInt(0)));
		let names: Vec<_> = p.matches.slice().iter().map(|m| m.matc.as_str()).collect();
		assert_eq!(names, vec!["creative", "survival"]);
		assert!(p.matches.slice().iter().all(|m| !m.tooltip.is_present()));
	}

	/// The graph from a received packet must parse as the graph that the server built.
	#[test]
	fn received_graph_parses_like_the_original() {
		let (g, ids) = sample();
		let received = CommandGraph::from_packet(&wire_round_trip(&g.to_packet())).unwrap();
		assert_eq!(received.parse("execute run tp 1 2 3").unwrap().executed(), ids.tp_z);
		assert_eq!(received.node(ids.give_count).kind, g.node(ids.give_count).kind);
		assert!(matches!(&received.node(ids.give_count).kind, CommandNodeKind::Argument { name, .. } if name == "count"));
	}
}
