//! Tests for the command graph types sent in the Commands packet.
//!
//! Reference: <https://minecraft.wiki/w/Java_Edition_protocol/Command_data>.
//! The client builds its tab-complete tree from these bytes. One wrong bit or a missing
//! optional field shifts every node after it, so the client cannot read the full packet.

#[cfg(test)]
mod test {
	use sandstone::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use sandstone::protocol_types::datatypes::command::{Node, NodeFlags, NodeType, Parser};
	use sandstone::protocol_types::datatypes::var_types::VarInt;

	fn to_bytes<T: McSerialize>(v: &T) -> Vec<u8> {
		let mut s = McSerializer::new();
		v.mc_serialize(&mut s).unwrap();
		s.output
	}

	fn flags(typ: NodeType, exec: bool, redirect: bool, suggest: bool, restricted: bool) -> NodeFlags {
		NodeFlags {
			typ,
			is_executable: exec,
			has_redirect: redirect,
			has_suggestions: suggest,
			is_restricted: restricted,
		}
	}

	#[test]
	fn node_flag_bits_match_wiki_layout() {
		// Wiki: 0x03 node type, 0x04 executable, 0x08 redirect, 0x10 suggestions, 0x20 restricted.
		assert_eq!(flags(NodeType::Root, false, false, false, false).to_byte(), 0x00);
		assert_eq!(flags(NodeType::Literal, false, false, false, false).to_byte(), 0x01);
		assert_eq!(flags(NodeType::Argument, false, false, false, false).to_byte(), 0x02);
		assert_eq!(flags(NodeType::Root, true, false, false, false).to_byte(), 0x04);
		assert_eq!(flags(NodeType::Root, false, true, false, false).to_byte(), 0x08);
		assert_eq!(flags(NodeType::Root, false, false, true, false).to_byte(), 0x10);
		assert_eq!(flags(NodeType::Root, false, false, false, true).to_byte(), 0x20);
		assert_eq!(flags(NodeType::Argument, true, true, true, true).to_byte(), 0x3E);
	}

	#[test]
	fn node_flags_round_trip_every_valid_byte() {
		// Every combination of the 6 defined bits with a valid type must survive a round trip.
		for byte in 0u8..0x40 {
			if byte & 0x03 == 3 {
				continue;
			}
			assert_eq!(NodeFlags::from_byte(byte).unwrap().to_byte(), byte, "byte {byte:#04x}");
		}
	}

	#[test]
	fn node_type_3_is_rejected() {
		// Type 3 is not defined. Accepting it would misread the rest of the node.
		assert!(NodeFlags::from_byte(0x03).is_err());
		assert!(NodeFlags::from_byte(0x3F).is_err());
	}

	#[test]
	fn root_node_has_no_name_or_parser() {
		// Bytes: flags=0, 2 children (1, 2). A root node must stop there.
		let bytes = [0x00, 0x02, 0x01, 0x02];
		let mut d = McDeserializer::new(&bytes);
		let node = Node::mc_deserialize(&mut d).unwrap();
		assert_eq!(node.flags.typ, NodeType::Root);
		assert_eq!(node.children, vec![VarInt(1), VarInt(2)]);
		assert_eq!(node.name, None);
		assert_eq!(node.parser, None);
		assert_eq!(to_bytes(&node), bytes);
	}

	#[test]
	fn literal_node_has_name_but_no_parser() {
		let node = Node {
			flags: flags(NodeType::Literal, true, false, false, false),
			children_count: VarInt(0),
			children: vec![],
			redirect_node: None,
			name: Some("gamemode".to_string()),
			parser: None,
			suggestions: None,
		};
		let bytes = to_bytes(&node);
		assert_eq!(bytes[0], 0x05);
		assert_eq!(bytes[2], 8, "name length");
		assert_eq!(&bytes[3..], b"gamemode");

		let mut d = McDeserializer::new(&bytes);
		assert_eq!(Node::mc_deserialize(&mut d).unwrap(), node);
	}

	#[test]
	fn argument_node_with_all_optional_fields_round_trips() {
		// Field order on the wire: flags, children, redirect, name, parser, suggestions.
		let node = Node {
			flags: flags(NodeType::Argument, true, true, true, false),
			children_count: VarInt(1),
			children: vec![VarInt(4)],
			redirect_node: Some(VarInt(9)),
			name: Some("amount".to_string()),
			parser: Some(Parser::BrigadierInteger {
				flags: 0x03,
				min: Some(1),
				max: Some(64),
			}),
			suggestions: Some("minecraft:ask_server".to_string()),
		};
		let bytes = to_bytes(&node);
		assert_eq!(&bytes[..4], &[0x1E, 0x01, 0x04, 0x09], "flags, child count, child, redirect");

		let mut d = McDeserializer::new(&bytes);
		assert_eq!(Node::mc_deserialize(&mut d).unwrap(), node);
	}

	#[test]
	fn several_nodes_in_sequence_stay_aligned() {
		// The Commands packet is an array of nodes. A wrong length in one node breaks all later nodes.
		let a = Node {
			flags: flags(NodeType::Argument, false, false, false, false),
			children_count: VarInt(0),
			children: vec![],
			redirect_node: None,
			name: Some("x".to_string()),
			parser: Some(Parser::BrigadierDouble {
				flags: 0x02,
				min: None,
				max: Some(1.5),
			}),
			suggestions: None,
		};
		let b = Node {
			flags: flags(NodeType::Literal, false, false, false, false),
			children_count: VarInt(0),
			children: vec![],
			redirect_node: None,
			name: Some("y".to_string()),
			parser: None,
			suggestions: None,
		};
		let mut bytes = to_bytes(&a);
		bytes.extend(to_bytes(&b));

		let mut d = McDeserializer::new(&bytes);
		assert_eq!(Node::mc_deserialize(&mut d).unwrap(), a);
		assert_eq!(Node::mc_deserialize(&mut d).unwrap(), b);
	}

	/// The extra bytes that follow each parser ID, as given in the wiki parser table.
	fn parser_payload(id: i32) -> Vec<u8> {
		match id {
			// number ranges: flags 0x03 means min and max are both present
			1 => [vec![0x03], 1.0f32.to_be_bytes().to_vec(), 2.0f32.to_be_bytes().to_vec()].concat(),
			2 => [vec![0x03], 1.0f64.to_be_bytes().to_vec(), 2.0f64.to_be_bytes().to_vec()].concat(),
			3 => [vec![0x03], 1i32.to_be_bytes().to_vec(), 2i32.to_be_bytes().to_vec()].concat(),
			4 => [vec![0x03], 1i64.to_be_bytes().to_vec(), 2i64.to_be_bytes().to_vec()].concat(),
			5 => vec![2],         // string behavior: greedy phrase
			6 | 31 => vec![0x01], // entity / score holder flags
			43 => 5i32.to_be_bytes().to_vec(), // time: minimum ticks
			44..=48 => [vec![17], b"minecraft:item_id".to_vec()].concat(),
			_ => vec![],
		}
	}

	#[test]
	fn every_parser_id_round_trips_to_identical_bytes() {
		// If serialize and deserialize disagree on any ID, the client reads garbage for the rest of the graph.
		for id in 0..=56 {
			let mut bytes = to_bytes(&VarInt(id));
			bytes.extend(parser_payload(id));

			let mut d = McDeserializer::new(&bytes);
			let parser = Parser::mc_deserialize(&mut d).unwrap_or_else(|e| panic!("parser id {id}: {e:?}"));
			assert_eq!(to_bytes(&parser), bytes, "parser id {id}");
		}
	}

	#[test]
	fn number_range_without_bounds_has_only_flags_byte() {
		// Wiki: min and max are present only if their flag bit is set.
		let parser = Parser::BrigadierLong {
			flags: 0,
			min: None,
			max: None,
		};
		assert_eq!(to_bytes(&parser), vec![4, 0]);

		let only_max = [vec![3, 0x02], 10i32.to_be_bytes().to_vec()].concat();
		let mut d = McDeserializer::new(&only_max);
		assert_eq!(
			Parser::mc_deserialize(&mut d).unwrap(),
			Parser::BrigadierInteger {
				flags: 0x02,
				min: None,
				max: Some(10)
			}
		);
	}

	#[test]
	fn unknown_parser_id_is_rejected() {
		let bytes = to_bytes(&VarInt(57));
		let mut d = McDeserializer::new(&bytes);
		assert!(Parser::mc_deserialize(&mut d).is_err());
	}
}
