//! Argument types: how the server reads an argument, the parser that the client gets, and the
//! default suggestions.
//!
//! A node made with [CommandGraph::add_argument](super::CommandGraph::add_argument) has only a
//! [Parser]. The graph reads it with the built-in rules of that parser (see [ArgumentValue]). A node
//! made with [CommandGraph::add_typed_argument](super::CommandGraph::add_typed_argument) has an
//! [ArgumentType], which can use the sender and can reject values that are not valid in the game.

use crate::game::player::PlayerGamemode;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::command::Parser;
use crate::protocol_types::datatypes::internal_types::Identifier;
use crate::protocol_types::datatypes::var_types::VarInt;
use std::any::Any;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

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
	/// From `minecraft:entity` and `minecraft:game_profile`.
	Entity(EntitySelector),
	/// From `minecraft:vec3` and `minecraft:block_pos`. Use [Coordinates::resolve] to get the
	/// position for a sender.
	Coordinates(Coordinates),
	/// From `minecraft:gamemode`.
	GameMode(PlayerGamemode),
	/// A value from an [ArgumentType] of the server. Get it with
	/// [ParsedCommand::custom](super::ParsedCommand::custom).
	Custom(CustomValue),
	/// The text of an argument whose parser has no built-in reader. It is not checked.
	///
	/// Most of these parsers read one token. A token stops at a space that is not in quotes or in
	/// `[]`, `{}` or `()`. `minecraft:message` reads the rest of the input. `minecraft:vec2`,
	/// `minecraft:column_pos` and `minecraft:rotation` read 2 words.
	Raw(String),
}

/// A value of any type, made by an [ArgumentType]. Two values are equal only if they are the same
/// allocation.
#[derive(Clone)]
pub struct CustomValue(pub Arc<dyn Any + Send + Sync>);

impl CustomValue {
	pub fn new<T: Any + Send + Sync>(value: T) -> Self {
		Self(Arc::new(value))
	}

	pub fn get<T: Any>(&self) -> Option<&T> {
		self.0.downcast_ref()
	}
}

impl Debug for CustomValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("CustomValue(..)")
	}
}

impl PartialEq for CustomValue {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

/// Why an argument is not valid.
///
/// A vanilla error has a translation key and arguments, so the client shows the text in the
/// language of the player. `message` is the English text, for logs and for the fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentError {
	/// The vanilla translation key, for example `parsing.int.invalid`. `None` for a custom error.
	pub key: Option<Cow<'static, str>>,
	/// The values for the `%s` parts of the translation.
	pub args: Vec<String>,
	/// The English text.
	pub message: String,
	/// The byte offset of the error from the start of the argument.
	pub offset: usize,
}

impl ArgumentError {
	/// An error with a vanilla translation key.
	pub fn translatable(key: impl Into<Cow<'static, str>>, args: Vec<String>, message: impl Into<String>) -> Self {
		Self {
			key: Some(key.into()),
			args,
			message: message.into(),
			offset: 0,
		}
	}

	/// An error with only a text.
	pub fn custom(message: impl Into<String>) -> Self {
		Self {
			key: None,
			args: Vec::new(),
			message: message.into(),
			offset: 0,
		}
	}

	/// Moves the error position `offset` bytes after the start of the argument.
	pub fn at(mut self, offset: usize) -> Self {
		self.offset = offset;
		self
	}

	/// Moves the error position `by` bytes later. A reader uses it for an error from a part of
	/// the argument.
	pub fn shift(mut self, by: usize) -> Self {
		self.offset += by;
		self
	}

	/// The error as a text component. A translatable component has the English text as fallback.
	pub fn to_component(&self) -> TextComponent {
		match &self.key {
			Some(key) => translatable(key, &self.args, &self.message),
			None => TextComponent::new(self.message.clone()),
		}
	}
}

impl Display for ArgumentError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.message)
	}
}

impl std::error::Error for ArgumentError {}

/// A translatable component with arguments and an English fallback.
pub(super) fn translatable(key: &str, args: &[String], fallback: &str) -> TextComponent {
	let mut component = TextComponent::translatable(key);
	if let crate::protocol_types::datatypes::chat::ComponentType::Translatable(content) = &mut component.content {
		content.fallback = Some(fallback.to_string());
		if !args.is_empty() {
			content.with = Some(args.iter().map(|a| TextComponent::new(a.clone())).collect());
		}
	}
	component
}

/// One tab-complete value.
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
	pub text: String,
	/// Text that the client shows when the player selects the value.
	pub tooltip: Option<TextComponent>,
}

impl Suggestion {
	pub fn new(text: impl Into<String>) -> Self {
		Self {
			text: text.into(),
			tooltip: None,
		}
	}

	pub fn with_tooltip(text: impl Into<String>, tooltip: TextComponent) -> Self {
		Self {
			text: text.into(),
			tooltip: Some(tooltip),
		}
	}
}

impl<T: Into<String>> From<T> for Suggestion {
	fn from(text: T) -> Self {
		Self::new(text)
	}
}

/// The type of an argument node. `S` is the sender type of the graph.
///
/// Implement it to read a value that the game must check, for example the name of an online
/// player. If `read` fails, the parse tries the next argument child, as Brigadier does.
pub trait ArgumentType<S>: Send + Sync {
	/// The parser that the client gets in the Commands packet.
	fn parser(&self) -> Parser;

	/// Reads the argument at the start of `rest`. Returns the length in bytes and the value.
	fn read(&self, sender: &S, rest: &str) -> Result<(usize, ArgumentValue), ArgumentError>;

	/// The values for tab-complete. `partial` is the text of the last word. The graph keeps only
	/// the values that start with `partial` (case is ignored).
	fn suggest(&self, _sender: &S, _partial: &str) -> Vec<Suggestion> {
		Vec::new()
	}

	/// The suggestions type that the client gets, for example `minecraft:ask_server`. With
	/// `None`, the client makes its own suggestions for the parser.
	fn suggestions_type(&self) -> Option<Identifier> {
		None
	}
}

/// `minecraft:ask_server`: the client sends a Command Suggestions Request for this argument.
pub fn ask_server() -> Identifier {
	Identifier::minecraft("ask_server").expect("ask_server is a valid identifier")
}

// ---- coordinates ----

/// One world coordinate: a number, or `~` with an optional offset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCoordinate {
	/// True for `~`. The value is then an offset from the position of the sender.
	pub relative: bool,
	pub value: f64,
}

impl WorldCoordinate {
	fn resolve(&self, origin: f64) -> f64 {
		if self.relative {
			origin + self.value
		} else {
			self.value
		}
	}
}

/// The value of a `minecraft:vec3` or `minecraft:block_pos` argument.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coordinates {
	/// `x y z`, where each part is a number or `~`. In a `vec3`, a whole number for x or z is
	/// already moved to the center of the block (+0.5), as in vanilla.
	World([WorldCoordinate; 3]),
	/// `^left ^up ^forwards`, relative to the direction that the sender looks.
	Local([f64; 3]),
}

impl Coordinates {
	/// The position for a sender at `position` with this rotation (in degrees). This uses the
	/// vanilla formula for local coordinates.
	pub fn resolve(&self, position: [f64; 3], yaw: f32, pitch: f32) -> [f64; 3] {
		match self {
			Coordinates::World(c) => [c[0].resolve(position[0]), c[1].resolve(position[1]), c[2].resolve(position[2])],
			Coordinates::Local([left, up, forwards]) => {
				let rad = std::f32::consts::PI / 180.0;
				let f = ((yaw + 90.0) * rad).cos();
				let g = ((yaw + 90.0) * rad).sin();
				let h = (-pitch * rad).cos();
				let i = (-pitch * rad).sin();
				let j = ((-pitch + 90.0) * rad).cos();
				let k = ((-pitch + 90.0) * rad).sin();
				let forward = [(f * h) as f64, i as f64, (g * h) as f64];
				let upward = [(f * j) as f64, k as f64, (g * j) as f64];
				// left = -(forward x upward)
				let leftward = [
					-(forward[1] * upward[2] - forward[2] * upward[1]),
					-(forward[2] * upward[0] - forward[0] * upward[2]),
					-(forward[0] * upward[1] - forward[1] * upward[0]),
				];
				let mut out = position;
				for (axis, value) in out.iter_mut().enumerate() {
					*value += forward[axis] * forwards + upward[axis] * up + leftward[axis] * left;
				}
				out
			}
		}
	}

	/// The block that contains the position from [Self::resolve].
	pub fn resolve_block(&self, position: [f64; 3], yaw: f32, pitch: f32) -> [i32; 3] {
		self.resolve(position, yaw, pitch).map(|v| v.floor() as i32)
	}
}

/// `minecraft:vec3`: three coordinates, with `~` or `^`. Whole numbers for x and z are moved to
/// the center of the block.
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3Argument;

/// `minecraft:block_pos`: three whole coordinates, with `~` or `^`.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockPosArgument;

impl<S> ArgumentType<S> for Vec3Argument {
	fn parser(&self) -> Parser {
		Parser::MinecraftVec3
	}

	fn read(&self, _sender: &S, rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
		read_coordinates(rest, false).map(|(l, c)| (l, ArgumentValue::Coordinates(c)))
	}
}

impl<S> ArgumentType<S> for BlockPosArgument {
	fn parser(&self) -> Parser {
		Parser::MinecraftBlockPos
	}

	fn read(&self, _sender: &S, rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
		read_coordinates(rest, true).map(|(l, c)| (l, ArgumentValue::Coordinates(c)))
	}
}

fn incomplete_coordinates() -> ArgumentError {
	ArgumentError::translatable("argument.pos3d.incomplete", vec![], "Incomplete (expected 3 coordinates)")
}

fn mixed_coordinates(offset: usize) -> ArgumentError {
	ArgumentError::translatable("argument.pos.mixed", vec![], "Cannot mix world & local coordinates (everything must either use ^ or not)").at(offset)
}

/// Reads 3 coordinates as vanilla does. `block` is true for `block_pos` (whole numbers, no center).
fn read_coordinates(rest: &str, block: bool) -> Result<(usize, Coordinates), ArgumentError> {
	let local = rest.starts_with('^');
	let mut pos = 0;
	let mut world = [WorldCoordinate {
		relative: false,
		value: 0.0,
	}; 3];
	let mut locals = [0.0; 3];
	for axis in 0..3 {
		if axis > 0 {
			if rest.as_bytes().get(pos) != Some(&b' ') {
				return Err(incomplete_coordinates());
			}
			pos += 1;
		}
		let part = &rest[pos..];
		if local {
			let Some(after) = part.strip_prefix('^') else {
				return Err(mixed_coordinates(pos));
			};
			let (len, value) = read_offset(after).map_err(|e| e.shift(pos + 1))?;
			locals[axis] = value;
			pos += 1 + len;
		} else {
			let (len, coordinate) = read_world_coordinate(part, block, axis != 1).map_err(|e| e.shift(pos))?;
			world[axis] = coordinate;
			pos += len;
		}
	}
	Ok((pos, if local { Coordinates::Local(locals) } else { Coordinates::World(world) }))
}

/// The optional number after `~` or `^`. With no number (end or space), the value is 0.
fn read_offset(rest: &str) -> Result<(usize, f64), ArgumentError> {
	if rest.is_empty() || rest.starts_with(' ') {
		return Ok((0, 0.0));
	}
	read_number(rest, &DOUBLE, None, None)
}

fn read_world_coordinate(rest: &str, block: bool, center: bool) -> Result<(usize, WorldCoordinate), ArgumentError> {
	if rest.starts_with('^') {
		return Err(mixed_coordinates(0));
	}
	let missing = || {
		if block {
			ArgumentError::translatable("argument.pos.missing.int", vec![], "Expected a block position")
		} else {
			ArgumentError::translatable("argument.pos.missing.double", vec![], "Expected a coordinate")
		}
	};
	if rest.is_empty() {
		return Err(missing());
	}
	if let Some(after) = rest.strip_prefix('~') {
		let (len, value) = read_offset(after).map_err(|e| e.shift(1))?;
		return Ok((
			1 + len,
			WorldCoordinate {
				relative: true,
				value,
			},
		));
	}
	if rest.starts_with(' ') {
		return Err(missing());
	}
	let (len, value) = if block {
		read_number::<i32>(rest, &INTEGER, None, None).map(|(l, v)| (l, v as f64))?
	} else {
		let (len, mut value) = read_number::<f64>(rest, &DOUBLE, None, None)?;
		if center && !rest[..len].contains('.') {
			value += 0.5;
		}
		(len, value)
	};
	Ok((
		len,
		WorldCoordinate {
			relative: false,
			value,
		},
	))
}

// ---- entity selectors ----

/// The value of a `minecraft:entity` or `minecraft:game_profile` argument. The graph does not know
/// the entities of the server, so it does not find them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitySelector {
	/// A player name.
	Name(String),
	/// An entity UUID, in the text form with `-`.
	Uuid(Uuid),
	/// A selector such as `@a[distance=..5]`.
	Selector {
		/// The letter after `@`: `p`, `a`, `r`, `s`, `e` or `n`.
		kind: char,
		/// The options in `[]`, in input order, as `(key, value)` text. Quotes are not removed.
		/// A `!` at the start of the value stays in the value.
		options: Vec<(String, String)>,
	},
}

impl EntitySelector {
	/// The maximum number of entities that the selector can give, or `None` for no limit.
	pub fn max_results(&self) -> Option<u32> {
		match self {
			EntitySelector::Name(_) | EntitySelector::Uuid(_) => Some(1),
			EntitySelector::Selector {
				kind,
				options,
			} => {
				if matches!(kind, 'p' | 'r' | 's' | 'n') {
					return Some(1);
				}
				options.iter().rev().find(|(k, _)| k == "limit").and_then(|(_, v)| v.parse().ok())
			}
		}
	}

	/// True if the selector can give entities that are not players.
	pub fn includes_entities(&self) -> bool {
		match self {
			EntitySelector::Name(_) => false,
			EntitySelector::Uuid(_) => true,
			EntitySelector::Selector {
				kind,
				options,
			} => {
				matches!(kind, 'e' | 'n' | 's')
					&& !options.iter().any(|(k, v)| k == "type" && matches!(v.as_str(), "player" | "minecraft:player"))
			}
		}
	}
}

/// `minecraft:entity`: a player name, a UUID or a selector. A server that must find the entities
/// can wrap this type in its own [ArgumentType], and fail the read if no entity matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityArgument {
	/// Only one entity is allowed.
	pub single: bool,
	/// Only players are allowed.
	pub players_only: bool,
}

impl EntityArgument {
	pub fn entity() -> Self {
		Self {
			single: true,
			players_only: false,
		}
	}

	pub fn entities() -> Self {
		Self {
			single: false,
			players_only: false,
		}
	}

	pub fn player() -> Self {
		Self {
			single: true,
			players_only: true,
		}
	}

	pub fn players() -> Self {
		Self {
			single: false,
			players_only: true,
		}
	}

	/// The flags byte of the `minecraft:entity` parser.
	fn flags(&self) -> u8 {
		u8::from(self.single) | (u8::from(self.players_only) << 1)
	}

	fn from_flags(flags: u8) -> Self {
		Self {
			single: flags & 0x01 != 0,
			players_only: flags & 0x02 != 0,
		}
	}

	/// Reads a name, UUID or selector, then checks the single and players-only rules.
	pub fn read_selector(&self, rest: &str) -> Result<(usize, EntitySelector), ArgumentError> {
		let (len, selector) = read_entity(rest)?;
		if self.single && selector.max_results().is_none_or(|m| m > 1) {
			return Err(if self.players_only {
				ArgumentError::translatable("argument.player.toomany", vec![], "Only one player is allowed, but the provided selector allows more than one")
			} else {
				ArgumentError::translatable("argument.entity.toomany", vec![], "Only one entity is allowed, but the provided selector allows more than one")
			});
		}
		let is_self = matches!(selector, EntitySelector::Selector { kind: 's', .. });
		if self.players_only && selector.includes_entities() && !is_self {
			return Err(ArgumentError::translatable("argument.player.entities", vec![], "Only players may be affected by this command, but the provided selector includes entities"));
		}
		Ok((len, selector))
	}
}

impl<S> ArgumentType<S> for EntityArgument {
	fn parser(&self) -> Parser {
		Parser::MinecraftEntity(self.flags())
	}

	fn read(&self, _sender: &S, rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
		self.read_selector(rest).map(|(l, s)| (l, ArgumentValue::Entity(s)))
	}
}

/// Reads `@x`, `@x[...]`, a UUID or a name.
fn read_entity(rest: &str) -> Result<(usize, EntitySelector), ArgumentError> {
	let Some(after) = rest.strip_prefix('@') else {
		let (len, text) = read_quotable(rest)?;
		if let Ok(uuid) = Uuid::parse_str(&text)
			&& text.contains('-')
		{
			return Ok((len, EntitySelector::Uuid(uuid)));
		}
		if text.is_empty() || text.len() > 16 {
			return Err(ArgumentError::translatable("argument.entity.invalid", vec![], "Invalid name or UUID"));
		}
		return Ok((len, EntitySelector::Name(text)));
	};
	let Some(kind) = after.chars().next() else {
		return Err(ArgumentError::translatable("argument.entity.selector.missing", vec![], "Missing selector type").at(1));
	};
	if !matches!(kind, 'p' | 'a' | 'r' | 's' | 'e' | 'n') {
		return Err(ArgumentError::translatable("argument.entity.selector.unknown", vec![format!("@{kind}")], format!("Unknown selector type '@{kind}'")).at(1));
	}
	let mut pos = 1 + kind.len_utf8();
	let mut options = Vec::new();
	if rest[pos..].starts_with('[') {
		let len = read_token(&rest[pos..]).map_err(|e| e.shift(pos))?;
		let body = &rest[pos + 1..pos + len - 1];
		for option in split_top_level(body, ',').into_iter().map(str::trim).filter(|o| !o.is_empty()) {
			match option.split_once('=') {
				Some((key, value)) => options.push((key.trim().to_string(), value.trim().to_string())),
				None => return Err(ArgumentError::translatable("argument.entity.options.valueless", vec![option.to_string()], format!("Expected value for option '{option}'")).at(pos)),
			}
		}
		pos += len;
	}
	Ok((
		pos,
		EntitySelector::Selector {
			kind,
			options,
		},
	))
}

// ---- game mode and enum ----

/// `minecraft:gamemode`: `survival`, `creative`, `adventure` or `spectator`.
#[derive(Debug, Clone, Copy, Default)]
pub struct GameModeArgument;

const GAME_MODES: [(&str, PlayerGamemode); 4] = [
	("survival", PlayerGamemode::SURVIVAL),
	("creative", PlayerGamemode::CREATIVE),
	("adventure", PlayerGamemode::ADVENTURE),
	("spectator", PlayerGamemode::SPECTATOR),
];

fn read_game_mode(rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
	let word = unquoted(rest);
	match GAME_MODES.iter().find(|(name, _)| *name == word) {
		Some((_, mode)) => Ok((word.len(), ArgumentValue::GameMode(*mode))),
		None => Err(ArgumentError::translatable("argument.gamemode.invalid", vec![word.to_string()], format!("Unknown game mode: {word}"))),
	}
}

impl<S> ArgumentType<S> for GameModeArgument {
	fn parser(&self) -> Parser {
		Parser::MinecraftGamemode
	}

	fn read(&self, _sender: &S, rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
		read_game_mode(rest)
	}

	fn suggest(&self, _sender: &S, _partial: &str) -> Vec<Suggestion> {
		GAME_MODES.iter().map(|(name, _)| Suggestion::new(*name)).collect()
	}
}

/// One word from a fixed list. Case is ignored, and the value is a
/// [ArgumentValue::String] with the word from the list. The client gets a `brigadier:string`
/// (single word) and asks the server for suggestions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumArgument(pub Vec<String>);

impl EnumArgument {
	pub fn new<T: Into<String>>(values: impl IntoIterator<Item = T>) -> Self {
		Self(values.into_iter().map(Into::into).collect())
	}
}

impl<S> ArgumentType<S> for EnumArgument {
	fn parser(&self) -> Parser {
		Parser::BrigadierString(VarInt(0))
	}

	fn read(&self, _sender: &S, rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
		let word = unquoted(rest);
		match self.0.iter().find(|v| v.eq_ignore_ascii_case(word)) {
			Some(value) => Ok((word.len(), ArgumentValue::String(value.clone()))),
			None => Err(ArgumentError::translatable("argument.enum.invalid", vec![word.to_string()], format!("Invalid value \"{word}\""))),
		}
	}

	fn suggest(&self, _sender: &S, _partial: &str) -> Vec<Suggestion> {
		self.0.iter().map(Suggestion::new).collect()
	}

	fn suggestions_type(&self) -> Option<Identifier> {
		Some(ask_server())
	}
}

// ---- built-in rules for each parser ----

/// Reads the argument at the start of `rest` with the built-in rules of `parser`. Returns the
/// length in bytes and the value.
pub(super) fn read_argument(parser: &Parser, rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
	match parser {
		Parser::BrigadierBool => read_bool(rest),
		Parser::BrigadierInteger {
			min,
			max,
		} => read_number(rest, &INTEGER, *min, *max).map(|(l, v)| (l, ArgumentValue::Integer(v))),
		Parser::BrigadierLong {
			min,
			max,
		} => read_number(rest, &LONG, *min, *max).map(|(l, v)| (l, ArgumentValue::Long(v))),
		Parser::BrigadierFloat {
			min,
			max,
		} => read_number(rest, &FLOAT, *min, *max).map(|(l, v)| (l, ArgumentValue::Float(v))),
		Parser::BrigadierDouble {
			min,
			max,
		} => read_number(rest, &DOUBLE, *min, *max).map(|(l, v)| (l, ArgumentValue::Double(v))),
		Parser::BrigadierString(behavior) => match behavior.0 {
			// SINGLE_WORD
			0 => {
				let word = unquoted(rest);
				Ok((word.len(), ArgumentValue::String(word.to_string())))
			}
			// QUOTABLE_PHRASE
			1 => read_quotable(rest).map(|(l, s)| (l, ArgumentValue::String(s))),
			// GREEDY_PHRASE
			2 => Ok((rest.len(), ArgumentValue::String(rest.to_string()))),
			other => Err(ArgumentError::custom(format!("unknown string behavior {other}"))),
		},
		Parser::MinecraftEntity(flags) => EntityArgument::from_flags(*flags).read_selector(rest).map(|(l, s)| (l, ArgumentValue::Entity(s))),
		Parser::MinecraftGameProfile => read_game_profile(rest),
		Parser::MinecraftVec3 => read_coordinates(rest, false).map(|(l, c)| (l, ArgumentValue::Coordinates(c))),
		Parser::MinecraftBlockPos => read_coordinates(rest, true).map(|(l, c)| (l, ArgumentValue::Coordinates(c))),
		Parser::MinecraftGamemode => read_game_mode(rest),
		Parser::MinecraftMessage => Ok((rest.len(), ArgumentValue::Raw(rest.to_string()))),
		Parser::MinecraftVec2 | Parser::MinecraftColumnPos | Parser::MinecraftRotation => read_words(rest, 2),
		_ => {
			let len = read_token(rest)?;
			if len == 0 {
				return Err(ArgumentError::custom("expected a value"));
			}
			Ok((len, ArgumentValue::Raw(rest[..len].to_string())))
		}
	}
}

/// The built-in default suggestions of a parser, before the prefix filter.
pub(super) fn builtin_suggestions(parser: &Parser) -> Vec<Suggestion> {
	match parser {
		Parser::BrigadierBool => vec![Suggestion::new("true"), Suggestion::new("false")],
		Parser::MinecraftGamemode => GAME_MODES.iter().map(|(name, _)| Suggestion::new(*name)).collect(),
		_ => Vec::new(),
	}
}

/// A selector (players only) or any text up to the next space, as vanilla reads a game profile.
fn read_game_profile(rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
	if rest.starts_with('@') {
		let (len, selector) = EntityArgument::players().read_selector(rest)?;
		return Ok((len, ArgumentValue::Entity(selector)));
	}
	let word = rest.split(' ').next().unwrap_or("");
	Ok((word.len(), ArgumentValue::Entity(EntitySelector::Name(word.to_string()))))
}

/// Reads a Brigadier bool: a quotable string that is `true` or `false`.
fn read_bool(rest: &str) -> Result<(usize, ArgumentValue), ArgumentError> {
	let (len, word) = read_quotable(rest)?;
	match word.as_str() {
		"" => Err(ArgumentError::translatable("parsing.bool.expected", vec![], "Expected boolean")),
		"true" => Ok((len, ArgumentValue::Bool(true))),
		"false" => Ok((len, ArgumentValue::Bool(false))),
		_ => Err(ArgumentError::translatable("parsing.bool.invalid", vec![word.clone()], format!("Invalid boolean, expected 'true' or 'false' but found '{word}'"))),
	}
}

/// The vanilla names and keys of one number type.
struct NumberKind {
	/// The word in `parsing.<word>.expected`, for example `int`.
	parsing: &'static str,
	/// The word in `argument.<word>.low`, for example `integer`.
	argument: &'static str,
	/// The English name, for example `Integer`.
	english: &'static str,
}

const INTEGER: NumberKind = NumberKind {
	parsing: "int",
	argument: "integer",
	english: "Integer",
};
const LONG: NumberKind = NumberKind {
	parsing: "long",
	argument: "long",
	english: "Long",
};
const FLOAT: NumberKind = NumberKind {
	parsing: "float",
	argument: "float",
	english: "Float",
};
const DOUBLE: NumberKind = NumberKind {
	parsing: "double",
	argument: "double",
	english: "Double",
};

/// Reads a number as Brigadier does: digits, `-` and `.`, then a range check.
fn read_number<T: FromStr + PartialOrd + Display + Copy>(rest: &str, kind: &NumberKind, min: Option<T>, max: Option<T>) -> Result<(usize, T), ArgumentError> {
	let len = rest.find(|c: char| !(c.is_ascii_digit() || c == '-' || c == '.')).unwrap_or(rest.len());
	let text = &rest[..len];
	if text.is_empty() {
		return Err(ArgumentError::translatable(format!("parsing.{}.expected", kind.parsing), vec![], format!("Expected {}", kind.english.to_lowercase())));
	}
	let value: T = text
		.parse()
		.map_err(|_| ArgumentError::translatable(format!("parsing.{}.invalid", kind.parsing), vec![text.to_string()], format!("Invalid {} '{text}'", kind.english.to_lowercase())))?;
	if let Some(min) = min
		&& value < min
	{
		return Err(ArgumentError::translatable(
			format!("argument.{}.low", kind.argument),
			vec![min.to_string(), value.to_string()],
			format!("{} must not be less than {min}, found {value}", kind.english),
		));
	}
	if let Some(max) = max
		&& value > max
	{
		return Err(ArgumentError::translatable(
			format!("argument.{}.big", kind.argument),
			vec![max.to_string(), value.to_string()],
			format!("{} must not be more than {max}, found {value}", kind.english),
		));
	}
	Ok((len, value))
}

/// The characters that Brigadier allows in a string with no quotes.
fn is_unquoted_char(c: char) -> bool {
	c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+')
}

/// The longest prefix of `rest` that has only unquoted characters. It can be empty.
pub(super) fn unquoted(rest: &str) -> &str {
	let len = rest.find(|c: char| !is_unquoted_char(c)).unwrap_or(rest.len());
	&rest[..len]
}

/// Reads a string in `"` or `'` quotes, with `\\` and `\<quote>` escapes. With no quote at the
/// start, reads an unquoted string.
fn read_quotable(rest: &str) -> Result<(usize, String), ArgumentError> {
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
				return Err(ArgumentError::translatable("parsing.quote.escape", vec![c.to_string()], format!("Invalid escape sequence '\\{c}' in quoted string")).at(i));
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
	Err(ArgumentError::translatable("parsing.quote.expected.end", vec![], "Unclosed quoted string").at(rest.len()))
}

/// The length of the token at the start of `rest`. The token stops at a space that is not in
/// quotes and not in `[]`, `{}` or `()`. A bracket at depth 0 that closes nothing also stops it.
fn read_token(rest: &str) -> Result<usize, ArgumentError> {
	let mut stack: Vec<char> = Vec::new();
	let mut quote: Option<char> = None;
	let mut escaped = false;
	for (i, c) in rest.char_indices() {
		if let Some(q) = quote {
			if escaped {
				escaped = false;
			} else if c == '\\' {
				escaped = true;
			} else if c == q {
				quote = None;
			}
			continue;
		}
		match c {
			'"' | '\'' => quote = Some(c),
			'[' => stack.push(']'),
			'{' => stack.push('}'),
			'(' => stack.push(')'),
			']' | '}' | ')' => {
				if stack.last() == Some(&c) {
					stack.pop();
					// A selector or block state ends at its closing bracket.
					if stack.is_empty() && c == ']' && rest[..i].starts_with(['@', '[']) {
						return Ok(i + 1);
					}
				} else if stack.is_empty() {
					return Ok(i);
				} else {
					return Err(ArgumentError::custom(format!("unexpected '{c}'")).at(i));
				}
			}
			' ' if stack.is_empty() => return Ok(i),
			_ => {}
		}
	}
	if quote.is_some() {
		return Err(ArgumentError::translatable("parsing.quote.expected.end", vec![], "Unclosed quoted string").at(rest.len()));
	}
	if let Some(close) = stack.last() {
		return Err(ArgumentError::custom(format!("expected '{close}'")).at(rest.len()));
	}
	Ok(rest.len())
}

/// Splits `text` at `sep` where it is not in quotes or brackets.
fn split_top_level(text: &str, sep: char) -> Vec<&str> {
	let mut parts = Vec::new();
	let mut depth = 0usize;
	let mut quote: Option<char> = None;
	let mut escaped = false;
	let mut start = 0;
	for (i, c) in text.char_indices() {
		if let Some(q) = quote {
			if escaped {
				escaped = false;
			} else if c == '\\' {
				escaped = true;
			} else if c == q {
				quote = None;
			}
			continue;
		}
		match c {
			'"' | '\'' => quote = Some(c),
			'[' | '{' | '(' => depth += 1,
			']' | '}' | ')' => depth = depth.saturating_sub(1),
			c if c == sep && depth == 0 => {
				parts.push(&text[start..i]);
				start = i + c.len_utf8();
			}
			_ => {}
		}
	}
	parts.push(&text[start..]);
	parts
}

/// Reads `count` words with one space between them, as raw text.
fn read_words(rest: &str, count: usize) -> Result<(usize, ArgumentValue), ArgumentError> {
	let incomplete = || ArgumentError::translatable("argument.pos2d.incomplete", vec![], "Incomplete (expected 2 coordinates)");
	let mut len = 0;
	for i in 0..count {
		if i > 0 {
			if rest.as_bytes().get(len) != Some(&b' ') {
				return Err(incomplete());
			}
			len += 1;
		}
		let word_len = rest[len..].find(' ').unwrap_or(rest.len() - len);
		if word_len == 0 {
			return Err(incomplete());
		}
		len += word_len;
	}
	Ok((len, ArgumentValue::Raw(rest[..len].to_string())))
}
