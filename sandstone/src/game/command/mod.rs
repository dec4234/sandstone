//! Command graphs above the wire format of the Commands packet.
//!
//! A [CommandGraph] is a tree of literal and argument nodes, kept in one `Vec`. A [NodeId] is the
//! index of a node in that `Vec`, and it is the same as the node index in the Commands packet.
//!
//! - A server builds a graph with [CommandGraph::add_literal] and [CommandGraph::add_argument], then
//!   sends [CommandGraph::to_packet].
//! - A client makes a graph from a received packet with [CommandGraph::from_packet].
//! - The two sides can then use [CommandGraph::parse] and [CommandGraph::suggest].
//!
//! <https://minecraft.wiki/w/Java_Edition_protocol/Command_data>

mod parse;

pub use parse::{ArgumentValue, CommandParseError, CommandParseErrorKind, ParsedArgument, ParsedCommand, Suggestions};

use crate::protocol::packets::CommandsGraphPacket;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol_types::datatypes::command::{Node, NodeFlags, NodeType, Parser};
use crate::protocol_types::datatypes::var_types::VarInt;
use thiserror::Error;

/// The index of a node in a [CommandGraph]. It is the same as the node index in the Commands packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

/// The type of a [CommandNode], with the data that only that type has.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandNodeKind {
	Root,
	/// A fixed word, for example `tp` or `creative`.
	Literal(String),
	/// A value that the [Parser] reads, for example a number.
	Argument {
		name: String,
		parser: Parser,
		/// The suggestions type, for example `minecraft:ask_server`.
		///
		/// <https://minecraft.wiki/w/Java_Edition_protocol/Command_data#Suggestions_Types>
		suggestions: Option<String>,
	},
}

/// One node in a [CommandGraph].
#[derive(Debug, Clone, PartialEq)]
pub struct CommandNode {
	pub kind: CommandNodeKind,
	/// The command is complete if the input stops after this node.
	pub executable: bool,
	/// Only players with elevated permissions can use this node.
	pub restricted: bool,
	pub children: Vec<NodeId>,
	/// If the input continues after this node, the parse continues at the children of this node
	/// instead. For example, `execute ... run` goes back to the root.
	pub redirect: Option<NodeId>,
}

impl CommandNode {
	fn new(kind: CommandNodeKind) -> Self {
		Self {
			kind,
			executable: false,
			restricted: false,
			children: Vec::new(),
			redirect: None,
		}
	}

	/// The name of a literal or argument node. The root has no name.
	pub fn name(&self) -> Option<&str> {
		match &self.kind {
			CommandNodeKind::Root => None,
			CommandNodeKind::Literal(name) | CommandNodeKind::Argument { name, .. } => Some(name),
		}
	}
}

/// An error when a Commands packet does not make a valid graph.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CommandGraphError {
	#[error("root index {0} is not the index of a root node")]
	InvalidRoot(i32),
	#[error("node {node} points to index {index}, but the graph has {len} nodes")]
	IndexOutOfRange { node: usize, index: i32, len: usize },
	#[error("node {node} is not valid: {reason}")]
	MalformedNode { node: usize, reason: &'static str },
}

/// A tree of commands. See the [module documentation](self).
///
/// All [NodeId] values that the methods return are valid for this graph. The methods panic if they
/// get a [NodeId] that is not in this graph.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandGraph {
	nodes: Vec<CommandNode>,
	root: NodeId,
}

impl Default for CommandGraph {
	fn default() -> Self {
		Self::new()
	}
}

impl CommandGraph {
	/// Makes a graph that has only a root node.
	pub fn new() -> Self {
		Self {
			nodes: vec![CommandNode::new(CommandNodeKind::Root)],
			root: NodeId(0),
		}
	}

	pub fn root(&self) -> NodeId {
		self.root
	}

	pub fn node(&self, id: NodeId) -> &CommandNode {
		&self.nodes[id.0]
	}

	/// All nodes, in packet order.
	pub fn nodes(&self) -> &[CommandNode] {
		&self.nodes
	}

	/// Adds a literal node as a child of `parent`.
	///
	/// # Panics
	/// If `parent` has a redirect. Brigadier does not allow children on a redirected node, because
	/// the parse uses the children of the redirect target.
	pub fn add_literal(&mut self, parent: NodeId, name: impl Into<String>) -> NodeId {
		self.add(parent, CommandNodeKind::Literal(name.into()))
	}

	/// Adds an argument node as a child of `parent`.
	///
	/// # Panics
	/// If `parent` has a redirect. See [Self::add_literal].
	pub fn add_argument(&mut self, parent: NodeId, name: impl Into<String>, parser: Parser) -> NodeId {
		self.add(
			parent,
			CommandNodeKind::Argument {
				name: name.into(),
				parser,
				suggestions: None,
			},
		)
	}

	fn add(&mut self, parent: NodeId, kind: CommandNodeKind) -> NodeId {
		assert!(self.nodes[parent.0].redirect.is_none(), "cannot add children to a redirected node");
		let id = NodeId(self.nodes.len());
		self.nodes.push(CommandNode::new(kind));
		self.nodes[parent.0].children.push(id);
		id
	}

	pub fn set_executable(&mut self, id: NodeId, executable: bool) {
		self.nodes[id.0].executable = executable;
	}

	pub fn set_restricted(&mut self, id: NodeId, restricted: bool) {
		self.nodes[id.0].restricted = restricted;
	}

	/// # Panics
	/// If `target` is not in this graph, or if `id` has children (see [Self::add_literal]).
	pub fn set_redirect(&mut self, id: NodeId, target: Option<NodeId>) {
		if let Some(target) = target {
			assert!(target.0 < self.nodes.len(), "redirect target {target:?} is not in this graph");
			assert!(self.nodes[id.0].children.is_empty(), "cannot redirect a node that has children");
		}
		self.nodes[id.0].redirect = target;
	}

	/// Sets the suggestions type of an argument node, for example `minecraft:ask_server`.
	///
	/// # Panics
	/// If `id` is not an argument node. Only argument nodes have a suggestions type.
	pub fn set_suggestions(&mut self, id: NodeId, suggestions_type: Option<String>) {
		match &mut self.nodes[id.0].kind {
			CommandNodeKind::Argument { suggestions, .. } => *suggestions = suggestions_type,
			_ => panic!("only argument nodes have a suggestions type"),
		}
	}

	/// Makes the Commands packet for this graph.
	pub fn to_packet(&self) -> CommandsGraphPacket {
		let nodes = self
			.nodes
			.iter()
			.map(|n| {
				let (typ, name, parser, suggestions) = match &n.kind {
					CommandNodeKind::Root => (NodeType::Root, None, None, None),
					CommandNodeKind::Literal(name) => (NodeType::Literal, Some(name.clone()), None, None),
					CommandNodeKind::Argument { name, parser, suggestions } => (NodeType::Argument, Some(name.clone()), Some(parser.clone()), suggestions.clone()),
				};
				Node {
					flags: NodeFlags {
						typ,
						is_executable: n.executable,
						has_redirect: n.redirect.is_some(),
						has_suggestions: suggestions.is_some(),
						is_restricted: n.restricted,
					},
					children_count: VarInt(n.children.len() as i32),
					children: n.children.iter().map(|c| VarInt(c.0 as i32)).collect(),
					redirect_node: n.redirect.map(|r| VarInt(r.0 as i32)),
					name,
					parser,
					suggestions,
				}
			})
			.collect();
		CommandsGraphPacket::new(PrefixedArray::new(nodes), VarInt(self.root.0 as i32))
	}

	/// Makes a graph from a received Commands packet. The [NodeId] of each node is its index in
	/// the packet.
	///
	/// This checks that all indices are in range, that the root index is a root node, and that the
	/// optional fields of each node agree with its flags. The packet reader does not check indices,
	/// and a packet made by hand can have fields that disagree with the flags.
	pub fn from_packet(packet: &CommandsGraphPacket) -> Result<Self, CommandGraphError> {
		let wire = packet.nodes.slice();
		let len = wire.len();
		let index = |node: usize, v: &VarInt| -> Result<NodeId, CommandGraphError> {
			if v.0 < 0 || v.0 as usize >= len {
				return Err(CommandGraphError::IndexOutOfRange { node, index: v.0, len });
			}
			Ok(NodeId(v.0 as usize))
		};
		let malformed = |node: usize, reason: &'static str| CommandGraphError::MalformedNode { node, reason };

		let mut nodes = Vec::with_capacity(len);
		for (i, n) in wire.iter().enumerate() {
			if n.children_count.0 as usize != n.children.len() || n.children_count.0 < 0 {
				return Err(malformed(i, "children count does not match the children"));
			}
			if n.flags.has_redirect != n.redirect_node.is_some() {
				return Err(malformed(i, "redirect flag does not match the redirect field"));
			}
			if n.flags.has_suggestions != n.suggestions.is_some() {
				return Err(malformed(i, "suggestions flag does not match the suggestions field"));
			}
			let kind = match (n.flags.typ, &n.name, &n.parser, &n.suggestions) {
				(NodeType::Root, None, None, None) => CommandNodeKind::Root,
				(NodeType::Literal, Some(name), None, None) => CommandNodeKind::Literal(name.clone()),
				(NodeType::Argument, Some(name), Some(parser), suggestions) => CommandNodeKind::Argument {
					name: name.clone(),
					parser: parser.clone(),
					suggestions: suggestions.clone(),
				},
				_ => return Err(malformed(i, "name, parser or suggestions do not match the node type")),
			};
			nodes.push(CommandNode {
				kind,
				executable: n.flags.is_executable,
				restricted: n.flags.is_restricted,
				children: n.children.iter().map(|c| index(i, c)).collect::<Result<_, _>>()?,
				redirect: n.redirect_node.as_ref().map(|r| index(i, r)).transpose()?,
			});
		}

		let root_index = packet.root_index.0;
		if root_index < 0 || root_index as usize >= len || nodes[root_index as usize].kind != CommandNodeKind::Root {
			return Err(CommandGraphError::InvalidRoot(root_index));
		}

		Ok(Self {
			nodes,
			root: NodeId(root_index as usize),
		})
	}
}
