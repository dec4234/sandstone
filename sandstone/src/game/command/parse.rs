//! Reads command strings against a [CommandGraph], and makes tab-complete suggestions.
//!
//! The rules follow Brigadier, the command library of the vanilla game:
//! - One space separates two nodes.
//! - If a literal child matches the next word, only that literal is tried. Otherwise all argument
//!   children are tried, in order.
//! - A command is complete only if the input stops at an executable node.

use super::{CommandGraph, CommandNodeKind, NodeId};
use crate::protocol::packets::CommandSuggestionsResponsePacket;
use crate::protocol::packets::packet_parts::TooltipMatch;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol_types::datatypes::command::Parser;
use crate::protocol_types::datatypes::var_types::VarInt;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;

/// The value of one argument in a parsed command.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentValue {
	Bool(bool),
	Integer(i32),
	Long(i64),
	Float(f32),
	Double(f64),
	/// From `brigadier:string`. Quotes and escapes are removed.
	String(String),
	/// The text of an argument whose parser is not a Brigadier parser. It is not checked.
	///
	/// Most of these parsers read one word. `minecraft:message` reads the rest of the input.
	/// `minecraft:block_pos` and `minecraft:vec3` read 3 words. `minecraft:vec2`,
	/// `minecraft:column_pos` and `minecraft:rotation` read 2 words.
	Raw(String),
}

/// One argument in a [ParsedCommand].
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArgument {
	pub node: NodeId,
	pub name: String,
	pub value: ArgumentValue,
}

/// The result of [CommandGraph::parse].
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand {
	/// The nodes that the input matched, in order. The first node is the root.
	pub path: Vec<NodeId>,
	/// The argument values, in the order of the input.
	pub arguments: Vec<ParsedArgument>,
}

impl ParsedCommand {
	/// The executable node where the input stopped. Compare it with the [NodeId] that the builder
	/// gave, to know which command to run.
	pub fn executed(&self) -> NodeId {
		*self.path.last().expect("path always contains the root")
	}

	/// The value of the first argument with this name.
	pub fn argument(&self, name: &str) -> Option<&ArgumentValue> {
		self.arguments.iter().find(|a| a.name == name).map(|a| &a.value)
	}
}

/// An error from [CommandGraph::parse]. `position` is a byte index in the input.
#[derive(Error, Debug, Clone, PartialEq)]
#[error("{kind} at position {position}")]
pub struct CommandParseError {
	pub position: usize,
	pub kind: CommandParseErrorKind,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CommandParseErrorKind {
	#[error("unknown command or argument")]
	Unknown,
	#[error("incomplete command")]
	Incomplete,
	#[error("expected a space after the argument")]
	ExpectedSeparator,
	#[error("invalid value for argument '{name}': {reason}")]
	InvalidValue { name: String, reason: String },
}

/// The result of [CommandGraph::suggest].
///
/// `start` and `length` are in UTF-16 code units, as the protocol uses Java string indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestions {
	pub start: usize,
	pub length: usize,
	/// Sorted, with no duplicates.
	pub matches: Vec<String>,
}

impl Suggestions {
	/// Makes the response to a Command Suggestions Request. `id` is the transaction ID of the request.
	pub fn into_packet(self, id: VarInt) -> CommandSuggestionsResponsePacket {
		let matches = self
			.matches
			.into_iter()
			.map(|m| TooltipMatch {
				matc: m,
				tooltip: PrefixedOptional::new(None),
			})
			.collect();
		CommandSuggestionsResponsePacket::new(id, VarInt(self.start as i32), VarInt(self.length as i32), PrefixedArray::new(matches))
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
	if input.starts_with('/') {
		1
	} else {
		0
	}
}

impl CommandGraph {
	/// Reads a command, for example the text of a Chat Command packet. A `/` at the start is
	/// optional.
	///
	/// If more than one path can match, the first path in child order that reaches an executable
	/// node is used. If no path matches, the error is the one that is farthest in the input.
	pub fn parse(&self, input: &str) -> Result<ParsedCommand, CommandParseError> {
		let start = command_start(input);
		let mut parsed = ParsedCommand {
			path: vec![self.root],
			arguments: Vec::new(),
		};
		if start == input.len() {
			return if self.node(self.root).executable {
				Ok(parsed)
			} else {
				Err(error(start, CommandParseErrorKind::Incomplete))
			};
		}
		self.parse_children(self.root, input, start, &mut parsed)?;
		Ok(parsed)
	}

	/// Tries the children of `id`. `start` is the first byte of the next word.
	fn parse_children(&self, id: NodeId, input: &str, start: usize, parsed: &mut ParsedCommand) -> ParseResult {
		let source = self.node(id).redirect.unwrap_or(id);
		let mut best: Option<CommandParseError> = None;
		for child in self.relevant_children(source, input, start) {
			let saved = (parsed.path.len(), parsed.arguments.len());
			match self.parse_child(child, input, start, parsed) {
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
		Err(best.unwrap_or(error(start, CommandParseErrorKind::Unknown)))
	}

	fn parse_child(&self, child: NodeId, input: &str, start: usize, parsed: &mut ParsedCommand) -> ParseResult {
		let node = self.node(child);
		let end = match &node.kind {
			CommandNodeKind::Root => return Err(error(start, CommandParseErrorKind::Unknown)),
			// relevant_children only gives a literal that matches the next word.
			CommandNodeKind::Literal(name) => start + name.len(),
			CommandNodeKind::Argument {
				name,
				parser,
				..
			} => {
				let (end, value) = read_argument(parser, input, start).map_err(|reason| {
					error(
						start,
						CommandParseErrorKind::InvalidValue {
							name: name.clone(),
							reason,
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
		self.parse_children(child, input, end + 1, parsed)
	}

	/// If a literal child matches the next word, only that literal. Otherwise all argument children.
	fn relevant_children(&self, source: NodeId, input: &str, start: usize) -> Vec<NodeId> {
		let word = input[start..].split(' ').next().unwrap_or("");
		let children = &self.node(source).children;
		if let Some(literal) = children.iter().find(|c| matches!(&self.node(**c).kind, CommandNodeKind::Literal(name) if name == word)) {
			return vec![*literal];
		}
		children.iter().copied().filter(|c| matches!(self.node(*c).kind, CommandNodeKind::Argument { .. })).collect()
	}

	/// Makes tab-complete suggestions for the last word of `text`, for example the text of a
	/// Command Suggestions Request. A `/` at the start is optional.
	///
	/// This suggests literal names and the values of `brigadier:bool`, with a case-insensitive
	/// prefix match. It does not suggest values for other argument parsers.
	pub fn suggest(&self, text: &str) -> Suggestions {
		let start = command_start(text);
		let word_start = text.rfind(' ').map_or(start, |i| i + 1).max(start);
		let mut found = BTreeSet::new();
		self.collect_suggestions(self.root, text, start, word_start, &mut found);
		Suggestions {
			start: text[..word_start].encode_utf16().count(),
			length: text[word_start..].encode_utf16().count(),
			matches: found.into_iter().collect(),
		}
	}

	/// Follows the input from `id` until `start` is the last word, then adds the matching children.
	fn collect_suggestions(&self, id: NodeId, text: &str, start: usize, word_start: usize, found: &mut BTreeSet<String>) {
		let source = self.node(id).redirect.unwrap_or(id);
		if start == word_start {
			let partial = text[start..].to_lowercase();
			for child in &self.node(source).children {
				match &self.node(*child).kind {
					CommandNodeKind::Literal(name) if name.to_lowercase().starts_with(&partial) => {
						found.insert(name.clone());
					}
					CommandNodeKind::Argument {
						parser: Parser::BrigadierBool,
						..
					} => {
						found.extend(["true", "false"].into_iter().filter(|v| v.starts_with(&partial)).map(String::from));
					}
					_ => {}
				}
			}
			return;
		}
		for child in self.relevant_children(source, text, start) {
			let end = match &self.node(child).kind {
				CommandNodeKind::Root => continue,
				CommandNodeKind::Literal(name) => start + name.len(),
				CommandNodeKind::Argument {
					parser,
					..
				} => match read_argument(parser, text, start) {
					Ok((end, _)) => end,
					Err(_) => continue,
				},
			};
			// end < word_start, so each step moves forward and a redirect loop stops.
			if end < word_start && text.as_bytes()[end] == b' ' {
				self.collect_suggestions(child, text, end + 1, word_start, found);
			}
		}
	}
}

/// Reads the argument at `start`. Returns the byte index after the argument, and its value.
/// The error is a reason for [CommandParseErrorKind::InvalidValue].
fn read_argument(parser: &Parser, input: &str, start: usize) -> Result<(usize, ArgumentValue), String> {
	let rest = &input[start..];
	let (len, value) = match parser {
		Parser::BrigadierBool => {
			let word = unquoted(rest);
			match word {
				"true" => (word.len(), ArgumentValue::Bool(true)),
				"false" => (word.len(), ArgumentValue::Bool(false)),
				_ => return Err(format!("expected true or false, found '{word}'")),
			}
		}
		Parser::BrigadierInteger {
			min,
			max,
			..
		} => read_number(rest, *min, *max).map(|(l, v)| (l, ArgumentValue::Integer(v)))?,
		Parser::BrigadierLong {
			min,
			max,
			..
		} => read_number(rest, *min, *max).map(|(l, v)| (l, ArgumentValue::Long(v)))?,
		Parser::BrigadierFloat {
			min,
			max,
			..
		} => read_number(rest, *min, *max).map(|(l, v)| (l, ArgumentValue::Float(v)))?,
		Parser::BrigadierDouble {
			min,
			max,
			..
		} => read_number(rest, *min, *max).map(|(l, v)| (l, ArgumentValue::Double(v)))?,
		Parser::BrigadierString(behavior) => match behavior.0 {
			// SINGLE_WORD
			0 => {
				let word = unquoted(rest);
				(word.len(), ArgumentValue::String(word.to_string()))
			}
			// QUOTABLE_PHRASE
			1 => read_quotable(rest).map(|(l, s)| (l, ArgumentValue::String(s)))?,
			// GREEDY_PHRASE
			2 => (rest.len(), ArgumentValue::String(rest.to_string())),
			other => return Err(format!("unknown string behavior {other}")),
		},
		Parser::MinecraftMessage => (rest.len(), ArgumentValue::Raw(rest.to_string())),
		Parser::MinecraftBlockPos | Parser::MinecraftVec3 => read_words(rest, 3)?,
		Parser::MinecraftVec2 | Parser::MinecraftColumnPos | Parser::MinecraftRotation => read_words(rest, 2)?,
		_ => read_words(rest, 1)?,
	};
	Ok((start + len, value))
}

/// The characters that Brigadier allows in a string with no quotes.
fn is_unquoted_char(c: char) -> bool {
	c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+')
}

/// The longest prefix of `rest` that has only unquoted characters. It can be empty.
fn unquoted(rest: &str) -> &str {
	let len = rest.find(|c: char| !is_unquoted_char(c)).unwrap_or(rest.len());
	&rest[..len]
}

/// Reads a number as Brigadier does: digits, `-` and `.`, then a range check.
fn read_number<T: FromStr + PartialOrd + Display + Copy>(rest: &str, min: Option<T>, max: Option<T>) -> Result<(usize, T), String> {
	let len = rest.find(|c: char| !(c.is_ascii_digit() || c == '-' || c == '.')).unwrap_or(rest.len());
	let text = &rest[..len];
	if text.is_empty() {
		return Err("expected a number".to_string());
	}
	let value: T = text.parse().map_err(|_| format!("'{text}' is not a valid number"))?;
	if let Some(min) = min
		&& value < min
	{
		return Err(format!("must not be less than {min}, found {value}"));
	}
	if let Some(max) = max
		&& value > max
	{
		return Err(format!("must not be more than {max}, found {value}"));
	}
	Ok((len, value))
}

/// Reads a string in `"` or `'` quotes, with `\\` and `\<quote>` escapes. With no quote at the
/// start, reads an unquoted string.
fn read_quotable(rest: &str) -> Result<(usize, String), String> {
	let quote = match rest.chars().next() {
		Some(q @ ('"' | '\'')) => q,
		_ => {
			let word = unquoted(rest);
			return Ok((word.len(), word.to_string()));
		}
	};
	let mut value = String::new();
	let mut escaped = false;
	for (i, c) in rest.char_indices().skip(1) {
		if escaped {
			if c != quote && c != '\\' {
				return Err(format!("invalid escape '\\{c}' in quoted string"));
			}
			value.push(c);
			escaped = false;
		} else if c == '\\' {
			escaped = true;
		} else if c == quote {
			return Ok((i + 1, value));
		} else {
			value.push(c);
		}
	}
	Err("quoted string has no closing quote".to_string())
}

/// Reads `count` words with one space between them, as raw text.
fn read_words(rest: &str, count: usize) -> Result<(usize, ArgumentValue), String> {
	let mut len = 0;
	for i in 0..count {
		if i > 0 {
			if rest.as_bytes().get(len) != Some(&b' ') {
				return Err(format!("expected {count} values"));
			}
			len += 1;
		}
		let word_len = rest[len..].find(' ').unwrap_or(rest.len() - len);
		if word_len == 0 {
			return Err(format!("expected {count} values"));
		}
		len += word_len;
	}
	Ok((len, ArgumentValue::Raw(rest[..len].to_string())))
}
