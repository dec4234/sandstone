//! Reads command strings against a [CommandGraph], and makes tab-complete suggestions.
//!
//! The rules follow Brigadier, the command library of the vanilla game:
//! - One space separates two nodes.
//! - If a literal child matches the next word, only that literal is tried. Otherwise all argument
//!   children are tried, in order.
//! - A command is complete only if the input stops at an executable node.
//! - A node whose requirement is false for the sender is skipped.
//!
//! All positions in this module are byte indices in the input. Only
//! [Suggestions::into_packet] changes them to UTF-16 indices, as the protocol uses Java string
//! indices.

use super::argument::{ArgumentError, ArgumentValue, Suggestion, builtin_suggestions, read_argument, translatable};
use super::{CommandGraph, CommandNodeKind, CommandSource, NodeId};
use crate::protocol::packets::CommandSuggestionsResponsePacket;
use crate::protocol::packets::packet_parts::TooltipMatch;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol_types::datatypes::chat::{ClickEvent, TextComponent};
use crate::protocol_types::datatypes::var_types::VarInt;
use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// One argument in a [ParsedCommand].
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArgument {
	pub node: NodeId,
	pub name: String,
	pub value: ArgumentValue,
}

/// The result of [CommandGraph::parse].
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand<H = ()> {
	/// The nodes that the input matched, in order. The first node is the root.
	pub path: Vec<NodeId>,
	/// The argument values, in the order of the input.
	pub arguments: Vec<ParsedArgument>,
	/// The handler of the node where the input stopped. `None` if that node is executable but has
	/// no handler, for example in a graph from a packet.
	pub handler: Option<H>,
}

impl<H> ParsedCommand<H> {
	/// The executable node where the input stopped.
	pub fn executed(&self) -> NodeId {
		*self.path.last().expect("path always contains the root")
	}

	/// The value of the first argument with this name.
	pub fn argument(&self, name: &str) -> Option<&ArgumentValue> {
		self.arguments.iter().find(|a| a.name == name).map(|a| &a.value)
	}

	/// The value of an [ArgumentValue::Custom] argument, if it has the type `T`.
	pub fn custom<T: Any>(&self, name: &str) -> Option<&T> {
		match self.argument(name)? {
			ArgumentValue::Custom(value) => value.get(),
			_ => None,
		}
	}
}

/// An error from [CommandGraph::parse]. `position` is a byte index in the input.
#[derive(Error, Debug, Clone, PartialEq)]
#[error("{kind} at position {position}")]
pub struct CommandParseError {
	pub position: usize,
	pub kind: CommandParseErrorKind,
}

/// The vanilla client shows a different message for each kind. See
/// [CommandParseError::translation_key].
#[derive(Debug, Clone, PartialEq)]
pub enum CommandParseErrorKind {
	/// No command has the first word.
	UnknownCommand,
	/// The command is known, but no child matches the next word.
	UnknownArgument,
	/// The input stops at a node that is not executable.
	Incomplete,
	/// An argument is followed by a character that is not a space.
	ExpectedSeparator,
	/// The argument `name` has a value that is not valid.
	InvalidValue { name: String, error: ArgumentError },
}

impl Display for CommandParseErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			CommandParseErrorKind::UnknownCommand | CommandParseErrorKind::Incomplete => f.write_str(UNKNOWN_COMMAND),
			CommandParseErrorKind::UnknownArgument => f.write_str(UNKNOWN_ARGUMENT),
			CommandParseErrorKind::ExpectedSeparator => f.write_str(EXPECTED_SEPARATOR),
			CommandParseErrorKind::InvalidValue {
				error,
				..
			} => f.write_str(&error.message),
		}
	}
}

const UNKNOWN_COMMAND: &str = "Unknown or incomplete command. See below for error";
const UNKNOWN_ARGUMENT: &str = "Incorrect argument for command";
const EXPECTED_SEPARATOR: &str = "Expected whitespace to end one argument, but found trailing data";

impl CommandParseError {
	/// The vanilla translation key of the message. An incomplete command has the same key as an
	/// unknown command, as in vanilla. A custom [ArgumentError] has no key.
	pub fn translation_key(&self) -> Option<&str> {
		match &self.kind {
			CommandParseErrorKind::UnknownCommand | CommandParseErrorKind::Incomplete => Some("command.unknown.command"),
			CommandParseErrorKind::UnknownArgument => Some("command.unknown.argument"),
			CommandParseErrorKind::ExpectedSeparator => Some("command.expected.separator"),
			CommandParseErrorKind::InvalidValue {
				error,
				..
			} => error.key.as_deref(),
		}
	}

	/// The message as a red text component. It is translatable, so the client shows it in the
	/// language of the player, with the English text as fallback.
	pub fn message(&self) -> TextComponent {
		let mut component = match &self.kind {
			CommandParseErrorKind::InvalidValue {
				error,
				..
			} => error.to_component(),
			kind => translatable(self.translation_key().unwrap_or_default(), &[], &kind.to_string()),
		};
		component.color = Some("red".to_string());
		component
	}

	/// The second line of the vanilla error: up to 10 characters of `input` before the error, the
	/// rest of the input in red, then `<--[HERE]`. Clicking it puts the command in the chat box.
	/// `input` is the text that was given to [CommandGraph::parse].
	pub fn context(&self, input: &str) -> TextComponent {
		let mut position = self.position.min(input.len());
		while !input.is_char_boundary(position) {
			position -= 1;
		}
		let before = &input[..position];
		let count = before.chars().count();
		let shown: String = before.chars().skip(count.saturating_sub(10)).collect();

		let mut extra = Vec::new();
		if count > 10 {
			extra.push(TextComponent::new("..."));
		}
		extra.push(TextComponent::new(shown));
		if position < input.len() {
			let mut rest = TextComponent::new(&input[position..]);
			rest.color = Some("red".to_string());
			rest.underlined = Some(true);
			extra.push(rest);
		}
		let mut here = translatable("command.context.here", &[], "<--[HERE]");
		here.color = Some("red".to_string());
		here.italic = Some(true);
		extra.push(here);

		let mut line = TextComponent::new("");
		line.color = Some("gray".to_string());
		line.set_extra(extra);
		let command = if input.starts_with('/') {
			input.to_string()
		} else {
			format!("/{input}")
		};
		line.set_click_event(ClickEvent::suggest_command(command));
		line
	}
}

/// The result of [CommandGraph::suggest].
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestions {
	/// The byte index of the last word in the text.
	pub start: usize,
	/// The length of the last word in bytes.
	pub length: usize,
	/// Sorted by text, with no duplicates.
	pub matches: Vec<Suggestion>,
	/// The argument nodes that the last word can be, in child order. A server can use them to add
	/// its own values.
	pub nodes: Vec<NodeId>,
}

impl Suggestions {
	/// The text of each match.
	pub fn texts(&self) -> Vec<&str> {
		self.matches.iter().map(|m| m.text.as_str()).collect()
	}

	/// Makes the response to a Command Suggestions Request. `id` is the transaction ID of the
	/// request, and `text` is the text that was given to [CommandGraph::suggest]. The byte range
	/// is changed to UTF-16 indices here.
	///
	/// # Panics
	/// If `start` and `length` are not a range of `text` on character boundaries.
	pub fn into_packet(self, id: VarInt, text: &str) -> CommandSuggestionsResponsePacket {
		let start = text[..self.start].encode_utf16().count();
		let length = text[self.start..self.start + self.length].encode_utf16().count();
		let matches = self
			.matches
			.into_iter()
			.map(|m| TooltipMatch {
				matc: m.text,
				tooltip: PrefixedOptional::new(m.tooltip),
			})
			.collect();
		CommandSuggestionsResponsePacket::new(id, VarInt(start as i32), VarInt(length as i32), PrefixedArray::new(matches))
	}
}

type ParseResult = Result<(), CommandParseError>;

fn error(position: usize, kind: CommandParseErrorKind) -> CommandParseError {
	CommandParseError {
		position,
		kind,
	}
}

/// The position after an optional `/` at the start of the input.
fn command_start(input: &str) -> usize {
	usize::from(input.starts_with('/'))
}

/// The values that a parse or a suggestion needs, other than the position.
struct Env<'a, 's, S: CommandSource, H> {
	graph: &'a CommandGraph<S, H>,
	sender: &'a S::Sender<'s>,
	input: &'a str,
	command_start: usize,
}

impl<S: CommandSource, H> Env<'_, '_, S, H> {
	/// If a usable literal child matches the next word, only that literal. Otherwise all usable
	/// argument children.
	fn relevant_children(&self, source: NodeId, start: usize) -> Vec<NodeId> {
		let word = self.input[start..].split(' ').next().unwrap_or("");
		let graph = self.graph;
		let usable = || graph.node(source).children.iter().copied().filter(|c| graph.node(*c).can_use(self.sender));
		if let Some(literal) = usable().find(|c| matches!(&graph.node(*c).kind, CommandNodeKind::Literal(name) if name == word)) {
			return vec![literal];
		}
		usable().filter(|c| matches!(graph.node(*c).kind, CommandNodeKind::Argument { .. })).collect()
	}

	/// Reads the argument node `id` at `start`, with its [ArgumentType](super::ArgumentType) or the
	/// built-in rules of its parser. Returns the byte index after the argument.
	fn read(&self, id: NodeId, start: usize) -> Result<(usize, ArgumentValue), ArgumentError> {
		let node = self.graph.node(id);
		let rest = &self.input[start..];
		let (len, value) = match (&node.argument_type, &node.kind) {
			(Some(ty), _) => ty.read(self.sender, rest)?,
			(
				None,
				CommandNodeKind::Argument {
					parser,
					..
				},
			) => read_argument(parser, rest)?,
			(None, _) => return Err(ArgumentError::custom("not an argument node")),
		};
		// A wrong length from an ArgumentType must not make the parse panic.
		if len > rest.len() || !rest.is_char_boundary(len) {
			return Err(ArgumentError::custom(format!("argument type read {len} bytes, which is not a valid length")));
		}
		Ok((start + len, value))
	}
}

impl<S: CommandSource, H: Clone> CommandGraph<S, H> {
	/// Reads a command, for example the text of a Chat Command packet. A `/` at the start is
	/// optional. Nodes that `sender` cannot use are skipped.
	///
	/// If more than one path can match, the first path in child order that reaches an executable
	/// node is used. If no path matches, the error is the one that is farthest in the input.
	pub fn parse(&self, sender: &S::Sender<'_>, input: &str) -> Result<ParsedCommand<H>, CommandParseError> {
		let env = Env {
			graph: self,
			sender,
			input,
			command_start: command_start(input),
		};
		let mut parsed = ParsedCommand {
			path: vec![self.root],
			arguments: Vec::new(),
			handler: None,
		};
		if env.command_start == input.len() {
			if !self.node(self.root).executable {
				return Err(error(env.command_start, CommandParseErrorKind::Incomplete));
			}
		} else {
			self.parse_children(&env, self.root, env.command_start, &mut parsed)?;
		}
		parsed.handler = self.node(parsed.executed()).handler.clone();
		Ok(parsed)
	}

	/// Tries the children of `id`. `start` is the first byte of the next word.
	fn parse_children(&self, env: &Env<'_, '_, S, H>, id: NodeId, start: usize, parsed: &mut ParsedCommand<H>) -> ParseResult {
		let source = self.node(id).redirect.unwrap_or(id);
		let mut best: Option<CommandParseError> = None;
		for child in env.relevant_children(source, start) {
			let saved = (parsed.path.len(), parsed.arguments.len());
			match self.parse_child(env, child, start, parsed) {
				Ok(()) => return Ok(()),
				Err(e) => {
					parsed.path.truncate(saved.0);
					parsed.arguments.truncate(saved.1);
					if best.as_ref().is_none_or(|b| e.position > b.position) {
						best = Some(e);
					}
				}
			}
		}
		Err(best.unwrap_or_else(|| {
			let kind = if start == env.command_start {
				CommandParseErrorKind::UnknownCommand
			} else {
				CommandParseErrorKind::UnknownArgument
			};
			error(start, kind)
		}))
	}

	fn parse_child(&self, env: &Env<'_, '_, S, H>, child: NodeId, start: usize, parsed: &mut ParsedCommand<H>) -> ParseResult {
		let node = self.node(child);
		let input = env.input;
		let end = match &node.kind {
			CommandNodeKind::Root => return Err(error(start, CommandParseErrorKind::UnknownArgument)),
			// relevant_children only gives a literal that matches the next word.
			CommandNodeKind::Literal(name) => start + name.len(),
			CommandNodeKind::Argument {
				name,
				..
			} => {
				let (end, value) = env.read(child, start).map_err(|e| {
					error(
						start + e.offset,
						CommandParseErrorKind::InvalidValue {
							name: name.clone(),
							error: e,
						},
					)
				})?;
				parsed.arguments.push(ParsedArgument {
					node: child,
					name: name.clone(),
					value,
				});
				end
			}
		};
		parsed.path.push(child);

		if end == input.len() {
			return if node.executable {
				Ok(())
			} else {
				Err(error(end, CommandParseErrorKind::Incomplete))
			};
		}
		if input.as_bytes()[end] != b' ' {
			return Err(error(end, CommandParseErrorKind::ExpectedSeparator));
		}
		self.parse_children(env, child, end + 1, parsed)
	}
}

impl<S: CommandSource, H> CommandGraph<S, H> {
	/// Makes tab-complete suggestions for the last word of `text`, for example the text of a
	/// Command Suggestions Request. A `/` at the start is optional. Nodes that `sender` cannot use
	/// are skipped.
	///
	/// For an argument, the values come from its suggestion provider, else from its
	/// [ArgumentType](super::ArgumentType), else from the built-in values of its parser (`true` and
	/// `false` for a bool, the game modes for a game mode). Only values that start with the last
	/// word are kept (case is ignored).
	pub fn suggest(&self, sender: &S::Sender<'_>, text: &str) -> Suggestions {
		let env = Env {
			graph: self,
			sender,
			input: text,
			command_start: command_start(text),
		};
		let word_start = text.rfind(' ').map_or(env.command_start, |i| i + 1).max(env.command_start);
		let mut found = BTreeMap::new();
		let mut nodes = Vec::new();
		self.collect_suggestions(&env, self.root, env.command_start, word_start, &mut found, &mut nodes);
		Suggestions {
			start: word_start,
			length: text.len() - word_start,
			matches: found
				.into_iter()
				.map(|(text, tooltip)| Suggestion {
					text,
					tooltip,
				})
				.collect(),
			nodes,
		}
	}

	/// Follows the input from `id` until `start` is the last word, then adds the matching children.
	fn collect_suggestions(&self, env: &Env<'_, '_, S, H>, id: NodeId, start: usize, word_start: usize, found: &mut BTreeMap<String, Option<TextComponent>>, nodes: &mut Vec<NodeId>) {
		let text = env.input;
		let source = self.node(id).redirect.unwrap_or(id);
		if start == word_start {
			let partial = &text[start..];
			let lower = partial.to_lowercase();
			for child in self.node(source).children.iter().copied() {
				let node = self.node(child);
				if !node.can_use(env.sender) {
					continue;
				}
				let values = match &node.kind {
					CommandNodeKind::Root => continue,
					CommandNodeKind::Literal(name) => vec![Suggestion::new(name.clone())],
					CommandNodeKind::Argument {
						parser,
						..
					} => {
						if !nodes.contains(&child) {
							nodes.push(child);
						}
						match (&node.suggestion_provider, &node.argument_type) {
							(Some(provider), _) => provider(env.sender, partial),
							(None, Some(ty)) => ty.suggest(env.sender, partial),
							(None, None) => builtin_suggestions(parser),
						}
					}
				};
				for value in values.into_iter().filter(|v| v.text.to_lowercase().starts_with(&lower)) {
					let entry = found.entry(value.text).or_insert(None);
					if entry.is_none() {
						*entry = value.tooltip;
					}
				}
			}
			return;
		}
		for child in env.relevant_children(source, start) {
			let end = match &self.node(child).kind {
				CommandNodeKind::Root => continue,
				CommandNodeKind::Literal(name) => start + name.len(),
				CommandNodeKind::Argument {
					..
				} => match env.read(child, start) {
					Ok((end, _)) => end,
					Err(_) => continue,
				},
			};
			// end < word_start, so each step moves forward and a redirect loop stops.
			if end < word_start && text.as_bytes()[end] == b' ' {
				self.collect_suggestions(env, child, end + 1, word_start, found, nodes);
			}
		}
	}
}
