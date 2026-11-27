//! Command graphs above the wire format of the Commands packet.
//!
//! A [CommandGraph] is a tree of literal and argument nodes, kept in one `Vec`. A [NodeId] is the
//! index of a node in that `Vec`. It does not change when other nodes are added or removed.
//!
//! - A server builds a graph with [CommandGraph::add_literal] and [CommandGraph::add_argument] (or
//!   [CommandGraph::add_typed_argument]), then sends [CommandGraph::to_packet_for] to each player.
//! - A client makes a graph from a received packet with [CommandGraph::from_packet].
//! - The two sides can then use [CommandGraph::parse] and [CommandGraph::suggest].
//!
//! The graph has two type parameters:
//! - `S` is the sender. A requirement ([CommandGraph::set_requirement]) uses it to hide nodes from
//!   a sender, and an [ArgumentType] and a suggestion provider get it.
//! - `H` is the handler that the server stores on an executable node
//!   ([CommandGraph::set_handler]). [CommandGraph::parse] returns it. It is often an
//!   `Arc<dyn Fn(...)>` with the context type of the server.
//!
//! A client graph uses the defaults `()` for both.
//!
//! <https://minecraft.wiki/w/Java_Edition_protocol/Command_data>

mod argument;
mod parse;

pub use argument::{
    ArgumentError, ArgumentType, ArgumentValue, BlockPosArgument, Coordinates, CustomValue, EntityArgument, EntitySelector, EnumArgument, GameModeArgument, Suggestion, Vec3Argument, WorldCoordinate,
    ask_server,
};
pub use parse::{CommandParseError, CommandParseErrorKind, ParsedArgument, ParsedCommand, Suggestions};

use crate::protocol::packets::CommandsGraphPacket;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol_types::datatypes::command::{Node, NodeFlags, NodeType, Parser};
use crate::protocol_types::datatypes::internal_types::Identifier;
use crate::protocol_types::datatypes::var_types::VarInt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use thiserror::Error;

/// The index of a node in a [CommandGraph]. In a graph from [CommandGraph::from_packet], it is the
/// node index in the packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

/// A check on the sender. A node whose requirement is false is hidden from that sender.
pub type Requirement<S> = Arc<dyn Fn(&S) -> bool + Send + Sync>;

/// Gives tab-complete values for an argument. It gets the sender and the text of the last word.
pub type SuggestionProvider<S> = Arc<dyn Fn(&S, &str) -> Vec<Suggestion> + Send + Sync>;

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
		suggestions: Option<Identifier>,
	},
}

/// One node in a [CommandGraph].
pub struct CommandNode<S = (), H = ()> {
	pub kind: CommandNodeKind,
	/// The command is complete if the input stops after this node.
	pub executable: bool,
	/// Only players with elevated permissions can use this node. The client uses this flag.
	pub restricted: bool,
	pub children: Vec<NodeId>,
	/// If the input continues after this node, the parse continues at the children of this node
	/// instead. For example, `execute ... run` goes back to the root.
	pub redirect: Option<NodeId>,
	requirement: Option<Requirement<S>>,
	handler: Option<H>,
	suggestion_provider: Option<SuggestionProvider<S>>,
	argument_type: Option<Arc<dyn ArgumentType<S>>>,
}

impl<S, H> CommandNode<S, H> {
	fn new(kind: CommandNodeKind) -> Self {
		Self {
			kind,
			executable: false,
			restricted: false,
			children: Vec::new(),
			redirect: None,
			requirement: None,
			handler: None,
			suggestion_provider: None,
			argument_type: None,
		}
	}

	/// The name of a literal or argument node. The root has no name.
	pub fn name(&self) -> Option<&str> {
		match &self.kind {
			CommandNodeKind::Root => None,
			CommandNodeKind::Literal(name) | CommandNodeKind::Argument { name, .. } => Some(name),
		}
	}

	pub fn handler(&self) -> Option<&H> {
		self.handler.as_ref()
	}

	pub fn has_requirement(&self) -> bool {
		self.requirement.is_some()
	}

	pub fn has_suggestion_provider(&self) -> bool {
		self.suggestion_provider.is_some()
	}

	pub fn argument_type(&self) -> Option<&Arc<dyn ArgumentType<S>>> {
		self.argument_type.as_ref()
	}

	/// True if the node has no requirement, or if the requirement is true for `sender`.
	pub fn can_use(&self, sender: &S) -> bool {
		self.requirement.as_ref().is_none_or(|r| r(sender))
	}
}

impl<S, H: Clone> Clone for CommandNode<S, H> {
	fn clone(&self) -> Self {
		Self {
			kind: self.kind.clone(),
			executable: self.executable,
			restricted: self.restricted,
			children: self.children.clone(),
			redirect: self.redirect,
			requirement: self.requirement.clone(),
			handler: self.handler.clone(),
			suggestion_provider: self.suggestion_provider.clone(),
			argument_type: self.argument_type.clone(),
		}
	}
}

impl<S, H: Debug> Debug for CommandNode<S, H> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("CommandNode")
			.field("kind", &self.kind)
			.field("executable", &self.executable)
			.field("restricted", &self.restricted)
			.field("children", &self.children)
			.field("redirect", &self.redirect)
			.field("handler", &self.handler)
			.field("has_requirement", &self.requirement.is_some())
			.field("has_suggestion_provider", &self.suggestion_provider.is_some())
			.field("has_argument_type", &self.argument_type.is_some())
			.finish()
	}
}

/// Compares the data fields. Requirements, suggestion providers and argument types are equal only
/// if they are the same allocation.
impl<S, H: PartialEq> PartialEq for CommandNode<S, H> {
	fn eq(&self, other: &Self) -> bool {
		fn same<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
			match (a, b) {
				(None, None) => true,
				(Some(a), Some(b)) => Arc::ptr_eq(a, b),
				_ => false,
			}
		}
		self.kind == other.kind
			&& self.executable == other.executable
			&& self.restricted == other.restricted
			&& self.children == other.children
			&& self.redirect == other.redirect
			&& self.handler == other.handler
			&& same(&self.requirement, &other.requirement)
			&& same(&self.suggestion_provider, &other.suggestion_provider)
			&& same(&self.argument_type, &other.argument_type)
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
pub struct CommandGraph<S = (), H = ()> {
	nodes: Vec<CommandNode<S, H>>,
	root: NodeId,
}

impl<S, H: Clone> Clone for CommandGraph<S, H> {
	fn clone(&self) -> Self {
		Self {
			nodes: self.nodes.clone(),
			root: self.root,
		}
	}
}

impl<S, H: Debug> Debug for CommandGraph<S, H> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("CommandGraph").field("nodes", &self.nodes).field("root", &self.root).finish()
	}
}

impl<S, H: PartialEq> PartialEq for CommandGraph<S, H> {
	fn eq(&self, other: &Self) -> bool {
		self.root == other.root && self.nodes == other.nodes
	}
}

impl<S, H> Default for CommandGraph<S, H> {
	fn default() -> Self {
		Self::new()
	}
}

impl<S, H> CommandGraph<S, H> {
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

	pub fn node(&self, id: NodeId) -> &CommandNode<S, H> {
		&self.nodes[id.0]
	}

	/// All nodes, in [NodeId] order. This includes nodes that [Self::remove] took out of the tree.
	pub fn nodes(&self) -> &[CommandNode<S, H>] {
		&self.nodes
	}

	/// The first child of `parent` with this name, literal or argument.
	pub fn child(&self, parent: NodeId, name: &str) -> Option<NodeId> {
		self.nodes[parent.0].children.iter().copied().find(|c| self.nodes[c.0].name() == Some(name))
	}

	/// Adds a literal node as a child of `parent`. If `parent` already has a literal with this
	/// name, this returns that node (the Brigadier merge rule), so two plugins can add
	/// sub-commands to the same command.
	///
	/// # Panics
	/// If `parent` has a redirect. Brigadier does not allow children on a redirected node, because
	/// the parse uses the children of the redirect target.
	pub fn add_literal(&mut self, parent: NodeId, name: impl Into<String>) -> NodeId {
		let name = name.into();
		if let Some(existing) = self.find_child(parent, |k| matches!(k, CommandNodeKind::Literal(n) if *n == name)) {
			return existing;
		}
		self.add(parent, CommandNodeKind::Literal(name))
	}

	/// Adds an argument node as a child of `parent`. If `parent` already has an argument with
	/// this name and parser, this returns that node (the Brigadier merge rule).
	///
	/// The parse reads the argument with the built-in rules of `parser`. See [ArgumentValue].
	///
	/// # Panics
	/// If `parent` has a redirect (see [Self::add_literal]), or if `parent` already has an argument
	/// with this name and a different parser.
	pub fn add_argument(&mut self, parent: NodeId, name: impl Into<String>, parser: Parser) -> NodeId {
		let name = name.into();
		if let Some(existing) = self.find_argument(parent, &name) {
			if let CommandNodeKind::Argument {
				parser: old,
				..
			} = &self.nodes[existing.0].kind
			{
				assert!(*old == parser, "argument '{name}' already exists with a different parser");
			}
			return existing;
		}
		self.add(
			parent,
			CommandNodeKind::Argument {
				name,
				parser,
				suggestions: None,
			},
		)
	}

	/// Adds an argument node that reads its value with `ty`. The client gets [ArgumentType::parser]
	/// and [ArgumentType::suggestions_type]. If `parent` already has an argument with this name
	/// and parser, `ty` replaces its argument type, and this returns that node.
	///
	/// # Panics
	/// The same as [Self::add_argument].
	pub fn add_typed_argument(&mut self, parent: NodeId, name: impl Into<String>, ty: impl ArgumentType<S> + 'static) -> NodeId {
		let suggestions_type = ty.suggestions_type();
		let id = self.add_argument(parent, name, ty.parser());
		let node = &mut self.nodes[id.0];
		node.argument_type = Some(Arc::new(ty));
		if let CommandNodeKind::Argument {
			suggestions,
			..
		} = &mut node.kind
		{
			*suggestions = suggestions_type;
		}
		id
	}

	fn find_child(&self, parent: NodeId, f: impl Fn(&CommandNodeKind) -> bool) -> Option<NodeId> {
		self.nodes[parent.0].children.iter().copied().find(|c| f(&self.nodes[c.0].kind))
	}

	fn find_argument(&self, parent: NodeId, name: &str) -> Option<NodeId> {
		self.find_child(parent, |k| matches!(k, CommandNodeKind::Argument { name: n, .. } if n == name))
	}

	fn add(&mut self, parent: NodeId, kind: CommandNodeKind) -> NodeId {
		assert!(self.nodes[parent.0].redirect.is_none(), "cannot add children to a redirected node");
		let id = NodeId(self.nodes.len());
		self.nodes.push(CommandNode::new(kind));
		self.nodes[parent.0].children.push(id);
		id
	}

	/// Removes the child of `parent` with this name, and all nodes under it. Returns the removed
	/// node, or `None` if there is no child with this name.
	///
	/// The removed nodes stay in [Self::nodes], so other [NodeId] values do not change, but they
	/// lose their children, handlers and closures, and they are not in the tree or in the packet.
	/// A redirect to a removed node is cleared.
	pub fn remove(&mut self, parent: NodeId, name: &str) -> Option<NodeId> {
		let id = self.child(parent, name)?;
		self.nodes[parent.0].children.retain(|c| *c != id);

		let mut removed = vec![false; self.nodes.len()];
		let mut stack = vec![id];
		while let Some(next) = stack.pop() {
			if std::mem::replace(&mut removed[next.0], true) {
				continue;
			}
			let node = &mut self.nodes[next.0];
			stack.append(&mut node.children);
			node.redirect = None;
			node.executable = false;
			node.handler = None;
			node.requirement = None;
			node.suggestion_provider = None;
			node.argument_type = None;
		}
		for node in &mut self.nodes {
			if node.redirect.is_some_and(|r| removed[r.0]) {
				node.redirect = None;
			}
		}
		Some(id)
	}

	pub fn set_executable(&mut self, id: NodeId, executable: bool) {
		self.nodes[id.0].executable = executable;
	}

	pub fn set_restricted(&mut self, id: NodeId, restricted: bool) {
		self.nodes[id.0].restricted = restricted;
	}

	/// Sets the handler of a node, and makes it executable. [Self::parse] returns the handler of
	/// the node where the input stops.
	pub fn set_handler(&mut self, id: NodeId, handler: H) {
		let node = &mut self.nodes[id.0];
		node.handler = Some(handler);
		node.executable = true;
	}

	/// Only senders for which `requirement` is true can use this node and the nodes under it. The
	/// other senders do not get the node in [Self::to_packet_for], and [Self::parse] and
	/// [Self::suggest] skip it.
	pub fn set_requirement(&mut self, id: NodeId, requirement: impl Fn(&S) -> bool + Send + Sync + 'static) {
		self.nodes[id.0].requirement = Some(Arc::new(requirement));
	}

	pub fn clear_requirement(&mut self, id: NodeId) {
		self.nodes[id.0].requirement = None;
	}

	/// Sets the function that gives tab-complete values for an argument node. It replaces the
	/// suggestions of the [ArgumentType]. If the node has no suggestions type, this sets
	/// `minecraft:ask_server`, so that the client asks the server.
	///
	/// # Panics
	/// If `id` is not an argument node.
	pub fn set_suggestion_provider(&mut self, id: NodeId, provider: impl Fn(&S, &str) -> Vec<Suggestion> + Send + Sync + 'static) {
		let node = &mut self.nodes[id.0];
		match &mut node.kind {
			CommandNodeKind::Argument {
				suggestions,
				..
			} => {
				suggestions.get_or_insert_with(ask_server);
			}
			_ => panic!("only argument nodes have suggestions"),
		}
		node.suggestion_provider = Some(Arc::new(provider));
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
	pub fn set_suggestions(&mut self, id: NodeId, suggestions_type: Option<Identifier>) {
		match &mut self.nodes[id.0].kind {
			CommandNodeKind::Argument {
				suggestions,
				..
			} => *suggestions = suggestions_type,
			_ => panic!("only argument nodes have a suggestions type"),
		}
	}

	/// Makes the Commands packet with all nodes of the tree. Requirements are not checked.
	pub fn to_packet(&self) -> CommandsGraphPacket {
		self.packet_with(|_| true)
	}

	/// Makes the Commands packet with only the nodes that `sender` can use. Send it again when the
	/// result of a requirement changes, for example after a change to the permission level.
	pub fn to_packet_for(&self, sender: &S) -> CommandsGraphPacket {
		self.packet_with(|node| node.can_use(sender))
	}

	/// Includes the nodes that are in the tree (from the root, through children) and for which
	/// `visible` is true, also for all nodes above them. Packet indices are given in [NodeId]
	/// order, so with no removed or hidden nodes, the packet index is the [NodeId].
	fn packet_with(&self, visible: impl Fn(&CommandNode<S, H>) -> bool) -> CommandsGraphPacket {
		let mut included = vec![false; self.nodes.len()];
		let mut stack = vec![self.root];
		while let Some(id) = stack.pop() {
			if std::mem::replace(&mut included[id.0], true) {
				continue;
			}
			stack.extend(self.nodes[id.0].children.iter().copied().filter(|c| visible(&self.nodes[c.0])));
		}
		let mut index: Vec<Option<i32>> = vec![None; self.nodes.len()];
		let mut next = 0;
		for (i, inc) in included.iter().enumerate() {
			if *inc {
				index[i] = Some(next);
				next += 1;
			}
		}

		let nodes = self
			.nodes
			.iter()
			.enumerate()
			.filter(|(i, _)| included[*i])
			.map(|(_, n)| {
				let (typ, name, parser, suggestions) = match &n.kind {
					CommandNodeKind::Root => (NodeType::Root, None, None, None),
					CommandNodeKind::Literal(name) => (NodeType::Literal, Some(name.clone()), None, None),
					CommandNodeKind::Argument {
						name,
						parser,
						suggestions,
					} => (NodeType::Argument, Some(name.clone()), Some(parser.clone()), suggestions.clone()),
				};
				let children: Vec<VarInt> = n.children.iter().filter_map(|c| index[c.0]).map(VarInt).collect();
				// A redirect to a node that the sender cannot see is dropped, as in vanilla.
				let redirect_node = n.redirect.and_then(|r| index[r.0]).map(VarInt);
				Node {
					flags: NodeFlags {
						typ,
						is_executable: n.executable,
						has_redirect: redirect_node.is_some(),
						has_suggestions: suggestions.is_some(),
						is_restricted: n.restricted,
					},
					children_count: VarInt(children.len() as i32),
					children,
					redirect_node,
					name,
					parser,
					suggestions,
				}
			})
			.collect();
		let root = index[self.root.0].expect("the root is always included");
		CommandsGraphPacket::new(PrefixedArray::new(nodes), VarInt(root))
	}

	/// Makes a graph from a received Commands packet. The [NodeId] of each node is its index in
	/// the packet. The graph has no handlers, requirements or closures.
	///
	/// This checks that all indices are in range, that the root index is a root node, and that the
	/// optional fields of each node agree with its flags. The packet reader does not check indices,
	/// and a packet made by hand can have fields that disagree with the flags.
	pub fn from_packet(packet: &CommandsGraphPacket) -> Result<Self, CommandGraphError> {
		let wire = packet.nodes.slice();
		let len = wire.len();
		let index = |node: usize, v: &VarInt| -> Result<NodeId, CommandGraphError> {
			if v.0 < 0 || v.0 as usize >= len {
				return Err(CommandGraphError::IndexOutOfRange {
					node,
					index: v.0,
					len,
				});
			}
			Ok(NodeId(v.0 as usize))
		};
		let malformed = |node: usize, reason: &'static str| CommandGraphError::MalformedNode {
			node,
			reason,
		};

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
			let mut node = CommandNode::new(kind);
			node.executable = n.flags.is_executable;
			node.restricted = n.flags.is_restricted;
			node.children = n.children.iter().map(|c| index(i, c)).collect::<Result<_, _>>()?;
			node.redirect = n.redirect_node.as_ref().map(|r| index(i, r)).transpose()?;
			nodes.push(node);
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
