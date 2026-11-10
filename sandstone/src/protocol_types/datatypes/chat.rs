//! This file defines the TextComponent type in the Minecraft network API.
//! Seen in books, disconnect messages, chat messages, action bar, etc.
//!
//! https://minecraft.wiki/w/Text_component_format

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtList, NbtTag};
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A TextComponent is a fancy way to display text inside the game. This is most commonly seen
/// in chat messages and book messages. The only thing that is required to be included is a String
/// representing the text to be displayed. Everything else is an optional modifier.
///
/// See https://minecraft.wiki/w/Text_component_format for more information.
#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TextComponent {
	/// The content of this component, determining how the displayed text is produced
	/// (literal text, translation, scoreboard value, etc). Flattened into the parent object
	/// so its fields (`text`, `translate`, ...) sit alongside the formatting fields, matching
	/// the wire format.
	#[serde(flatten)]
	pub content: ComponentType,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub extra: Option<Vec<TextComponent>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub color: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bold: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub italic: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub underlined: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub strikethrough: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub obfuscated: Option<bool>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub font: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub insertion: Option<String>,
}

/// Hand-written because a derived default fills `extra` with more default components,
/// and each of those does the same, so the derive recurses without end.
impl McDefault for TextComponent {
	fn mc_default() -> Self {
		Self {
			content: ComponentType::mc_default(),
			extra: None,
			color: Option::mc_default(),
			bold: Option::mc_default(),
			italic: Option::mc_default(),
			underlined: Option::mc_default(),
			strikethrough: Option::mc_default(),
			obfuscated: Option::mc_default(),
			font: Option::mc_default(),
			insertion: Option::mc_default(),
		}
	}
}

impl TextComponent {
	/// Create a plain literal-text component (`ComponentType::Text`).
	pub fn new<T: Into<String>>(text: T) -> Self {
		Self::from_content(ComponentType::Text {
			text: text.into(),
		})
	}

	/// Create a component wrapping the given content type, with no formatting applied.
	pub fn from_content(content: ComponentType) -> Self {
		Self {
			content,
			extra: None,
			color: None,
			bold: None,
			italic: None,
			underlined: None,
			strikethrough: None,
			obfuscated: None,
			font: None,
			insertion: None,
		}
	}

	/// Create a translatable component (`ComponentType::Translatable`).
	pub fn translatable<T: Into<String>>(translate: T) -> Self {
		Self::from_content(ComponentType::Translatable(Box::new(TranslatableContent {
			translate: translate.into(),
			fallback: None,
			with: None,
		})))
	}

	/// Create a keybind component (`ComponentType::Keybind`).
	pub fn keybind<T: Into<String>>(keybind: T) -> Self {
		Self::from_content(ComponentType::Keybind {
			keybind: keybind.into(),
		})
	}

	/// Create a scoreboard-value component (`ComponentType::Score`).
	pub fn score<N: Into<String>, O: Into<String>>(name: N, objective: O) -> Self {
		Self::from_content(ComponentType::Score {
			score: ScoreContent {
				name: name.into(),
				objective: objective.into(),
			},
		})
	}

	/// Create an entity-selector component (`ComponentType::Selector`).
	pub fn selector<T: Into<String>>(selector: T) -> Self {
		Self::from_content(ComponentType::Selector {
			selector: selector.into(),
			separator: None,
		})
	}

	pub fn set_extra(&mut self, extra: Vec<TextComponent>) {
		self.extra = Some(extra);
	}

	/// True if this TextComponent is just plain literal text with no modifiers.
	pub fn is_plain(&self) -> bool {
		matches!(self.content, ComponentType::Text { .. })
			&& self.extra.is_none()
			&& self.color.is_none()
			&& self.bold.is_none()
			&& self.italic.is_none()
			&& self.underlined.is_none()
			&& self.strikethrough.is_none()
			&& self.obfuscated.is_none()
			&& self.font.is_none()
			&& self.insertion.is_none()
	}
}

/// Read a list tag as text components, one for each item.
fn components_from_list(list: &NbtList) -> Result<Vec<TextComponent>, NbtError> {
	list.list.iter().cloned().map(TextComponent::try_from).collect()
}

/// A text component is a string, a list that is not empty, or a compound. Any other tag is an error,
/// as in vanilla, where the text codec does not decode it and the packet fails.
///
/// <https://minecraft.wiki/w/Text_component_format>
impl TryFrom<NbtTag> for TextComponent {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, NbtError> {
		match tag {
			NbtTag::String(s) => Ok(Self::new(s)),
			// An item of a list with mixed types, wrapped by vanilla as `{"": item}`.
			NbtTag::Compound(compound) if compound.is_list_wrapper() => Self::try_from(NbtTag::Compound(compound).unwrap_list_item()),
			// A list is shorthand for its first item, with the other items added to its `extra`. This is
			// the same rule as the JSON array form. Vanilla does not accept an empty list.
			NbtTag::List(list) => {
				let mut items = components_from_list(&list)?.into_iter();
				let mut base = items.next().ok_or(NbtError::InvalidType)?;
				let rest: Vec<TextComponent> = items.collect();
				if !rest.is_empty() {
					base.extra.get_or_insert_with(Vec::new).extend(rest);
				}
				Ok(base)
			}
			NbtTag::Compound(compound) => {
				let content = ComponentType::from_compound(&compound)?;
				let extra = match compound.get("extra") {
					Some(NbtTag::List(list)) => Some(components_from_list(list)?),
					_ => None,
				};
				let color = match compound.get("color") {
					Some(NbtTag::String(s)) => Some(s.clone()),
					_ => None,
				};
				let bold = match compound.get("bold") {
					Some(NbtTag::Byte(b)) => Some(*b != 0),
					_ => None,
				};
				let italic = match compound.get("italic") {
					Some(NbtTag::Byte(b)) => Some(*b != 0),
					_ => None,
				};
				let underlined = match compound.get("underlined") {
					Some(NbtTag::Byte(b)) => Some(*b != 0),
					_ => None,
				};
				let strikethrough = match compound.get("strikethrough") {
					Some(NbtTag::Byte(b)) => Some(*b != 0),
					_ => None,
				};
				let obfuscated = match compound.get("obfuscated") {
					Some(NbtTag::Byte(b)) => Some(*b != 0),
					_ => None,
				};
				let font = match compound.get("font") {
					Some(NbtTag::String(s)) => Some(s.clone()),
					_ => None,
				};
				let insertion = match compound.get("insertion") {
					Some(NbtTag::String(s)) => Some(s.clone()),
					_ => None,
				};
				Ok(Self {
					content,
					extra,
					color,
					bold,
					italic,
					underlined,
					strikethrough,
					obfuscated,
					font,
					insertion,
				})
			}
			_ => Err(NbtError::InvalidType),
		}
	}
}

impl From<TextComponent> for NbtTag {
	fn from(component: TextComponent) -> Self {
		if component.is_plain() {
			if let ComponentType::Text {
				text,
			} = component.content
			{
				return NbtTag::String(text);
			}
		}

		let mut compound = NbtCompound::new_no_name();
		component.content.write_to_compound(&mut compound);
		if let Some(extra) = component.extra {
			let tags: Vec<NbtTag> = extra.into_iter().map(NbtTag::from).collect();
			if let Ok(list) = NbtList::from_vec_wrapping(tags) {
				compound.add("extra", NbtTag::List(list));
			}
		}
		if let Some(color) = component.color {
			compound.add("color", NbtTag::String(color));
		}
		if let Some(bold) = component.bold {
			compound.add("bold", NbtTag::from(bold));
		}
		if let Some(italic) = component.italic {
			compound.add("italic", NbtTag::from(italic));
		}
		if let Some(underlined) = component.underlined {
			compound.add("underlined", NbtTag::from(underlined));
		}
		if let Some(strikethrough) = component.strikethrough {
			compound.add("strikethrough", NbtTag::from(strikethrough));
		}
		if let Some(obfuscated) = component.obfuscated {
			compound.add("obfuscated", NbtTag::from(obfuscated));
		}
		if let Some(font) = component.font {
			compound.add("font", NbtTag::String(font));
		}
		if let Some(insertion) = component.insertion {
			compound.add("insertion", NbtTag::String(insertion));
		}
		NbtTag::Compound(compound)
	}
}

impl McSerialize for TextComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let nbt = NbtTag::from(self.clone());
		match &nbt {
			NbtTag::Compound(compound) => {
				compound.mc_serialize(serializer)?;
			}
			_ => {
				nbt.get_type_id().mc_serialize(serializer)?;
				nbt.serialize_payload(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for TextComponent {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> Result<Self, SerializingErr> {
		let type_id = u8::mc_deserialize(deserializer)?;
		let nbt = NbtTag::deserialize_specific(deserializer, type_id)?;
		Ok(TextComponent::try_from(nbt)?)
	}
}

impl From<String> for TextComponent {
	fn from(s: String) -> Self {
		Self::new(s)
	}
}

/// Object-shaped mirror of [`TextComponent`] used only as a deserialization target. A text
/// component may arrive as a JSON string, array, or object (see
/// <https://minecraft.wiki/w/Text_component_format>); the manual `Deserialize` below dispatches on
/// that shape and delegates the object case here, where the derived impl handles the flattened
/// fields exactly as before.
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct TextComponentObject {
	#[serde(flatten)]
	content: ComponentType,
	extra: Option<Vec<TextComponent>>,
	color: Option<String>,
	bold: Option<bool>,
	italic: Option<bool>,
	underlined: Option<bool>,
	strikethrough: Option<bool>,
	obfuscated: Option<bool>,
	font: Option<String>,
	insertion: Option<String>,
}

impl From<TextComponentObject> for TextComponent {
	fn from(obj: TextComponentObject) -> Self {
		Self {
			content: obj.content,
			extra: obj.extra,
			color: obj.color,
			bold: obj.bold,
			italic: obj.italic,
			underlined: obj.underlined,
			strikethrough: obj.strikethrough,
			obfuscated: obj.obfuscated,
			font: obj.font,
			insertion: obj.insertion,
		}
	}
}

impl<'de> Deserialize<'de> for TextComponent {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct TextComponentVisitor;

		impl<'de> Visitor<'de> for TextComponentVisitor {
			type Value = TextComponent;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				f.write_str("a text component as a string, array, or object")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
				Ok(TextComponent::new(v.to_string()))
			}

			// A JSON array is shorthand for its first element with the remaining elements appended
			// to that element's `extra` list.
			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: SeqAccess<'de>,
			{
				let mut base: TextComponent = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
				let mut extras = Vec::new();
				while let Some(component) = seq.next_element::<TextComponent>()? {
					extras.push(component);
				}
				if !extras.is_empty() {
					match &mut base.extra {
						Some(existing) => existing.extend(extras),
						None => base.extra = Some(extras),
					}
				}
				Ok(base)
			}

			fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
			where
				A: MapAccess<'de>,
			{
				let obj = TextComponentObject::deserialize(MapAccessDeserializer::new(map))?;
				Ok(obj.into())
			}
		}

		deserializer.deserialize_any(TextComponentVisitor)
	}
}

/// A wrapper around TextComponent that is serialized as a JSON string.
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsonTextComponent(TextComponent);

impl From<TextComponent> for JsonTextComponent {
	fn from(component: TextComponent) -> Self {
		JsonTextComponent(component)
	}
}

impl McSerialize for JsonTextComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let json = serde_json::to_string(&self.0).map_err(|e| SerializingErr::FailedToSerializeJSON(format!("Failed to serialize JSON: {}", e)))?;
		json.mc_serialize(serializer)
	}
}

impl McDeserialize for JsonTextComponent {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> Result<Self, SerializingErr> {
		let json = String::mc_deserialize(deserializer)?;
		let text_component = serde_json::from_str(&json).map_err(|e| SerializingErr::DeserializationError(format!("Failed to deserialize JSON: {}", e)))?;
		Ok(JsonTextComponent(text_component))
	}
}

/// The content of a [`TextComponent`], i.e. the source of the text it displays.
///
/// Serialized untagged: which variant is used is determined by the content field that is present
/// (`text`, `translate`, `score`, `selector`, `keybind`, `nbt`), matching the wiki's rule that the
/// `type` field is optional and otherwise inferred from the content fields, in that priority order.
///
/// See https://minecraft.wiki/w/Text_component_format#Java_Edition
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ComponentType {
	/// Plain literal text.
	Text { text: String },
	/// Text resolved from a translation key, with optional fallback and substitution arguments.
	/// Boxed because this is a rare, large variant — keeping it inline would size the whole enum
	/// (and every embedding packet) to it.
	Translatable(Box<TranslatableContent>),
	/// A scoreboard value, resolved server-side.
	Score { score: ScoreContent },
	/// The name(s) of the entities matched by a selector, resolved server-side.
	Selector {
		selector: String,
		#[serde(skip_serializing_if = "Option::is_none")]
		separator: Option<Box<TextComponent>>,
	},
	/// The key currently bound to the given action.
	Keybind { keybind: String },
	/// An NBT value read from a block, entity, or command storage, resolved server-side.
	/// Boxed because this is a rare, large variant — keeping it inline would size the whole enum
	/// (and every embedding packet) to it.
	Nbt(Box<NbtContent>),
}

/// The fields of a [`ComponentType::Translatable`] component. Held behind a [`Box`] in the enum to
/// keep the common (text/keybind) variants small. Serialized untagged, so its fields sit flat in
/// the parent component exactly as the inline variant did.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TranslatableContent {
	pub translate: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fallback: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub with: Option<Vec<TextComponent>>,
}

/// The fields of a [`ComponentType::Nbt`] component. Held behind a [`Box`] in the enum to keep the
/// common (text/keybind) variants small. Serialized untagged, so its fields sit flat in the parent
/// component exactly as the inline variant did.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtContent {
	pub nbt: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub interpret: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub separator: Option<Box<TextComponent>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub entity: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub storage: Option<String>,
}

impl ComponentType {
	/// Extract the content type from an NBT compound, honoring an explicit `type` field if present
	/// and otherwise inferring the type from the content fields in priority order.
	fn from_compound(compound: &NbtCompound) -> Result<Self, NbtError> {
		let get_components = |key: &str| match compound.get(key) {
			Some(NbtTag::List(list)) => components_from_list(list).map(Some),
			_ => Ok(None),
		};
		let get_separator = |key: &str| compound.get(key).map(|t| TextComponent::try_from(t.clone()).map(Box::new)).transpose();

		let typ = match compound.get("type") {
			Some(NbtTag::String(s)) => Some(s.as_str()),
			_ => None,
		};

		// Resolve the type explicitly when given, otherwise fall back to detecting which content
		// field is present, in the wiki's documented priority order.
		match typ {
			Some("translatable") => {
				return Ok(ComponentType::Translatable(Box::new(TranslatableContent {
					translate: compound.get_string("translate").unwrap_or_default(),
					fallback: compound.get_string("fallback"),
					with: get_components("with")?,
				})));
			}
			Some("score") => {
				return Ok(ComponentType::Score {
					score: ScoreContent::from_compound(compound),
				});
			}
			Some("selector") => {
				return Ok(ComponentType::Selector {
					selector: compound.get_string("selector").unwrap_or_default(),
					separator: get_separator("separator")?,
				});
			}
			Some("keybind") => {
				return Ok(ComponentType::Keybind {
					keybind: compound.get_string("keybind").unwrap_or_default(),
				});
			}
			Some("nbt") => {
				return Ok(ComponentType::Nbt(Box::new(NbtContent {
					nbt: compound.get_string("nbt").unwrap_or_default(),
					interpret: compound.get_bool("interpret"),
					separator: get_separator("separator")?,
					block: compound.get_string("block"),
					entity: compound.get_string("entity"),
					storage: compound.get_string("storage"),
				})));
			}
			Some("text") => {
				return Ok(ComponentType::Text {
					text: compound.get_string("text").unwrap_or_default(),
				});
			}
			_ => {}
		}

		Ok(if let Some(text) = compound.get_string("text") {
			ComponentType::Text {
				text,
			}
		} else if let Some(translate) = compound.get_string("translate") {
			ComponentType::Translatable(Box::new(TranslatableContent {
				translate,
				fallback: compound.get_string("fallback"),
				with: get_components("with")?,
			}))
		} else if compound.get("score").is_some() {
			ComponentType::Score {
				score: ScoreContent::from_compound(compound),
			}
		} else if let Some(selector) = compound.get_string("selector") {
			ComponentType::Selector {
				selector,
				separator: get_separator("separator")?,
			}
		} else if let Some(keybind) = compound.get_string("keybind") {
			ComponentType::Keybind {
				keybind,
			}
		} else if let Some(nbt) = compound.get_string("nbt") {
			ComponentType::Nbt(Box::new(NbtContent {
				nbt,
				interpret: compound.get_bool("interpret"),
				separator: get_separator("separator")?,
				block: compound.get_string("block"),
				entity: compound.get_string("entity"),
				storage: compound.get_string("storage"),
			}))
		} else {
			ComponentType::Text {
				text: String::new(),
			}
		})
	}

	/// Write this content type's fields (including an explicit `type` tag for non-text variants)
	/// into the given NBT compound.
	fn write_to_compound(self, compound: &mut NbtCompound) {
		match self {
			ComponentType::Text {
				text,
			} => {
				compound.add("text", NbtTag::String(text));
			}
			ComponentType::Translatable(content) => {
				let TranslatableContent {
					translate,
					fallback,
					with,
				} = *content;
				compound.add("type", NbtTag::String("translatable".to_string()));
				compound.add("translate", NbtTag::String(translate));
				if let Some(fallback) = fallback {
					compound.add("fallback", NbtTag::String(fallback));
				}
				if let Some(with) = with {
					let tags: Vec<NbtTag> = with.into_iter().map(NbtTag::from).collect();
					if let Ok(list) = NbtList::from_vec_wrapping(tags) {
						compound.add("with", NbtTag::List(list));
					}
				}
			}
			ComponentType::Score {
				score,
			} => {
				compound.add("type", NbtTag::String("score".to_string()));
				compound.add("score", NbtTag::Compound(score.into()));
			}
			ComponentType::Selector {
				selector,
				separator,
			} => {
				compound.add("type", NbtTag::String("selector".to_string()));
				compound.add("selector", NbtTag::String(selector));
				if let Some(separator) = separator {
					compound.add("separator", NbtTag::from(*separator));
				}
			}
			ComponentType::Keybind {
				keybind,
			} => {
				compound.add("type", NbtTag::String("keybind".to_string()));
				compound.add("keybind", NbtTag::String(keybind));
			}
			ComponentType::Nbt(content) => {
				let NbtContent {
					nbt,
					interpret,
					separator,
					block,
					entity,
					storage,
				} = *content;
				compound.add("type", NbtTag::String("nbt".to_string()));
				compound.add("nbt", NbtTag::String(nbt));
				if let Some(interpret) = interpret {
					compound.add("interpret", NbtTag::from(interpret));
				}
				if let Some(separator) = separator {
					compound.add("separator", NbtTag::from(*separator));
				}
				if let Some(block) = block {
					compound.add("block", NbtTag::String(block));
				}
				if let Some(entity) = entity {
					compound.add("entity", NbtTag::String(entity));
				}
				if let Some(storage) = storage {
					compound.add("storage", NbtTag::String(storage));
				}
			}
		}
	}
}

/// The `score` object of a [`ComponentType::Score`] component.
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScoreContent {
	pub name: String,
	pub objective: String,
}

impl ScoreContent {
	fn from_compound(compound: &NbtCompound) -> Self {
		let score = match compound.get("score") {
			Some(NbtTag::Compound(c)) => c,
			_ => {
				return ScoreContent {
					name: String::new(),
					objective: String::new(),
				};
			}
		};
		let get = |key: &str| match score.get(key) {
			Some(NbtTag::String(s)) => s.clone(),
			_ => String::new(),
		};
		ScoreContent {
			name: get("name"),
			objective: get("objective"),
		}
	}
}

impl From<ScoreContent> for NbtCompound {
	fn from(score: ScoreContent) -> Self {
		let mut compound = NbtCompound::new_no_name();
		compound.add("name", NbtTag::String(score.name));
		compound.add("objective", NbtTag::String(score.objective));
		compound
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClickEvent {
	pub action: String,
	pub value: String,
}

impl ClickEvent {
	fn new<T: Into<String>>(action: T, value: String) -> Self {
		Self {
			action: action.into(),
			value,
		}
	}

	pub fn open_url<T: Into<String>>(url: T) -> Self {
		Self::new("open_url", url.into())
	}

	/// Internal to clients only - doesn't work over server connections
	pub fn open_file<T: Into<String>>(file: T) -> Self {
		Self::new("open_file", file.into())
	}

	pub fn run_command<T: Into<String>>(command: T) -> Self {
		Self::new("run_command", command.into())
	}

	pub fn suggest_command<T: Into<String>>(command: T) -> Self {
		Self::new("suggest_command", command.into())
	}

	pub fn change_page(page: usize) -> Self {
		Self::new("change_page", format!("{}", page))
	}

	pub fn copy_to_clipboard<T: Into<String>>(text: T) -> Self {
		Self::new("copy_to_clipboard", text.into())
	}
}

impl McSerialize for ClickEvent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		serde_json::to_string(self)
			.map_err(|e| SerializingErr::UniqueFailure(format!("Failed to serialize JSON: {}", e)))?
			.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for ClickEvent {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> Result<Self, SerializingErr> {
		let json = String::mc_deserialize(deserializer)?;
		let deserialized = serde_json::from_str(&json).map_err(|e| SerializingErr::UniqueFailure(format!("Failed to deserialize JSON: {}", e)))?;

		Ok(deserialized)
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HoverEvent {
	pub action: String,
	pub contents: HoverComponent,
}

impl HoverEvent {
	fn new<T: Into<String>>(action: T, contents: HoverComponent) -> Self {
		Self {
			action: action.into(),
			contents,
		}
	}

	pub fn show_text<T: Into<String>>(text: T) -> Self {
		Self::new("show_text", HoverComponent::String(text.into()))
	}

	pub fn show_item<T: Into<String>>(id: String, count: i32, tag: Option<String>) -> Self {
		let s = {
			if let Some(compound) = tag {
				compound
			} else {
				"".to_string()
			}
		};

		let item = ItemHover {
			id,
			count,
			tag: Some(s),
		};

		Self::new("show_item", HoverComponent::String(serde_json::to_string(&item).unwrap()))
	}

	pub fn show_entity<T: Into<String>>(entity: T) -> Self {
		Self::new("show_entity", HoverComponent::String(entity.into()))
	}

	pub fn show_text_component(text: TextComponent) -> Self {
		Self::new("show_text", HoverComponent::TextComponent(text))
	}

	pub fn show_achievement<T: Into<String>>(entity: T) -> Self {
		Self::new("show_achievement", HoverComponent::String(entity.into()))
	}
}

impl McSerialize for HoverEvent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		serde_json::to_string(self)
			.map_err(|e| SerializingErr::UniqueFailure(format!("Failed to serialize JSON: {}", e)))?
			.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for HoverEvent {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> Result<Self, SerializingErr> {
		let json = String::mc_deserialize(deserializer)?;
		let deserialized = serde_json::from_str(&json).map_err(|e| SerializingErr::UniqueFailure(format!("Failed to deserialize JSON: {}", e)))?;

		Ok(deserialized)
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HoverComponent {
	String(String),
	TextComponent(TextComponent),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemHover {
	pub id: String,
	pub count: i32,
	pub tag: Option<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PlayerChatSignature {
	message_id: VarInt,
	#[mc(deserialize_if = message_id.0 == 0)]
	signature: Option<[u8; 256]>,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum PlayerChatFilter {
	PassThrough = 0,
	FullyFiltered = 1,
	PartiallyFiltered = 2,
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Serialize via the NBT wire path and read it back. The content type must survive the round
	/// trip, since the receiving client relies on it to know how to render the component.
	fn nbt_round_trip(component: &TextComponent) -> TextComponent {
		let mut serializer = McSerializer::new();
		component.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&serializer.output);
		TextComponent::mc_deserialize(&mut deserializer).unwrap()
	}

	fn content_of(pairs: Vec<(&str, NbtTag)>) -> ComponentType {
		let mut compound = NbtCompound::new_no_name();
		for (k, v) in pairs {
			compound.add(k, v);
		}
		TextComponent::try_from(NbtTag::Compound(compound)).unwrap().content
	}

	/// The `type` field is optional on the wire. Without it, the reader must find the content type
	/// from which field is present, so that a vanilla component with no `type` renders correctly.
	#[test]
	fn nbt_content_detected_without_type_field() {
		assert_eq!(content_of(vec![("text", "a".into())]), ComponentType::Text { text: "a".to_string() });

		let with = NbtTag::List(NbtList::from_vec(vec![NbtTag::from("x")]).unwrap());
		assert_eq!(
			content_of(vec![("translate", "k".into()), ("fallback", "f".into()), ("with", with)]),
			ComponentType::Translatable(Box::new(TranslatableContent {
				translate: "k".to_string(),
				fallback: Some("f".to_string()),
				with: Some(vec![TextComponent::new("x")]),
			}))
		);

		let mut score = NbtCompound::new_no_name();
		score.add("name", "@p");
		score.add("objective", "kills");
		assert_eq!(
			content_of(vec![("score", NbtTag::Compound(score))]),
			ComponentType::Score { score: ScoreContent { name: "@p".to_string(), objective: "kills".to_string() } }
		);

		assert_eq!(content_of(vec![("selector", "@a".into())]), ComponentType::Selector { selector: "@a".to_string(), separator: None });
		assert_eq!(content_of(vec![("keybind", "key.jump".into())]), ComponentType::Keybind { keybind: "key.jump".to_string() });

		match content_of(vec![("nbt", "Health".into()), ("entity", "@s".into())]) {
			ComponentType::Nbt(n) => {
				assert_eq!(n.nbt, "Health");
				assert_eq!(n.entity.as_deref(), Some("@s"));
				assert_eq!(n.block, None);
			}
			other => panic!("expected nbt content, got {other:?}"),
		}
	}

	/// When more than one content field is present and there is no `type`, the wiki order decides:
	/// text, translatable, score, selector, keybind, nbt.
	#[test]
	fn nbt_content_detection_priority() {
		assert!(matches!(content_of(vec![("translate", "k".into()), ("text", "a".into())]), ComponentType::Text { .. }));
		assert!(matches!(content_of(vec![("keybind", "k".into()), ("translate", "t".into())]), ComponentType::Translatable(_)));
		assert!(matches!(content_of(vec![("nbt", "n".into()), ("keybind", "k".into())]), ComponentType::Keybind { .. }));
	}

	/// A compound with no content field gives empty text instead of an error.
	#[test]
	fn nbt_content_fallback_is_empty_text() {
		assert_eq!(content_of(vec![("color", "red".into())]), ComponentType::Text { text: String::new() });
	}

	/// Vanilla accepts only a string, a list that is not empty, or a compound. Any other tag is an
	/// error, as it is in vanilla and in the JSON reader of this crate. Before, it gave empty text
	/// with no error, so the text was lost and the caller did not know.
	#[test]
	fn nbt_tag_that_is_not_a_component_is_an_error() {
		for tag in [NbtTag::Int(5), NbtTag::Byte(1), NbtTag::Double(1.5), NbtTag::List(NbtList::new())] {
			assert_eq!(TextComponent::try_from(tag.clone()), Err(NbtError::InvalidType), "{tag:?}");
		}

		// The error must also come from a nested part, and from the packet reader.
		let mut root = NbtCompound::new_no_name();
		root.add("text", "base");
		root.add("extra", NbtList::from_vec(vec![NbtTag::Int(5)]).unwrap());
		assert_eq!(TextComponent::try_from(NbtTag::Compound(root.clone())), Err(NbtError::InvalidType));

		let mut serializer = McSerializer::new();
		root.mc_serialize(&mut serializer).unwrap();
		assert!(TextComponent::mc_deserialize(&mut McDeserializer::new(&serializer.output)).is_err());
	}

	/// Each constructor must use the vanilla action id, or the client ignores the click.
	#[test]
	fn click_event_constructors_use_vanilla_action_ids() {
		for (event, action, value) in [
			(ClickEvent::open_url("https://a.b"), "open_url", "https://a.b"),
			(ClickEvent::open_file("f.txt"), "open_file", "f.txt"),
			(ClickEvent::run_command("/say hi"), "run_command", "/say hi"),
			(ClickEvent::suggest_command("/tp"), "suggest_command", "/tp"),
			(ClickEvent::change_page(3), "change_page", "3"),
			(ClickEvent::copy_to_clipboard("x"), "copy_to_clipboard", "x"),
		] {
			assert_eq!(event.action, action);
			assert_eq!(event.value, value);
		}
	}

	/// A plain part becomes a String tag and a styled part becomes a Compound tag. Before, the mixed
	/// list failed and `extra` (and `with`) were dropped with no error, so the client lost text.
	#[test]
	fn mixed_plain_and_styled_parts_survive_nbt_round_trip() {
		let mut styled = TextComponent::new("styled");
		styled.bold = Some(true);
		let parts = vec![TextComponent::new("plain"), styled];

		let mut with_extra = TextComponent::new("base");
		with_extra.color = Some("red".to_string());
		with_extra.set_extra(parts.clone());
		assert_eq!(nbt_round_trip(&with_extra), with_extra);

		let mut translatable = TextComponent::translatable("chat.type.text");
		translatable.content = ComponentType::Translatable(Box::new(TranslatableContent {
			translate: "chat.type.text".to_string(),
			fallback: None,
			with: Some(parts),
		}));
		assert_eq!(nbt_round_trip(&translatable), translatable);
	}

	/// Vanilla sends a mixed `extra` as compounds, with each plain part as `{"": "text"}`. The reader
	/// must remove the wrapper. If not, the part has no content field and its text is lost.
	#[test]
	fn vanilla_wrapped_list_items_are_unwrapped() {
		let mut wrapper = NbtCompound::new_no_name();
		wrapper.add("", "plain");
		let mut styled = NbtCompound::new_no_name();
		styled.add("text", "styled");
		styled.add("bold", true);

		let mut root = NbtCompound::new_no_name();
		root.add("text", "base");
		root.add("extra", NbtList::from_vec(vec![NbtTag::Compound(wrapper), NbtTag::Compound(styled)]).unwrap());

		let parsed = TextComponent::try_from(NbtTag::Compound(root)).unwrap();
		let mut expected_styled = TextComponent::new("styled");
		expected_styled.bold = Some(true);
		assert_eq!(parsed.extra, Some(vec![TextComponent::new("plain"), expected_styled]));
	}

	/// A text component in NBT can be a list, as in JSON. Before, an NBT list gave empty text and all
	/// its text was lost. The first item is the base and the other items go into its `extra`.
	#[test]
	fn nbt_list_is_shorthand_for_first_with_extra() {
		let mut first = NbtCompound::new_no_name();
		first.add("text", "a");
		first.add("extra", NbtList::from_vec(vec![NbtTag::from("b")]).unwrap());
		let mut second = NbtCompound::new_no_name();
		second.add("text", "c");
		second.add("bold", true);
		let list = NbtList::from_vec(vec![NbtTag::Compound(first), NbtTag::Compound(second)]).unwrap();

		let parsed = TextComponent::try_from(NbtTag::List(list)).unwrap();
		let mut c = TextComponent::new("c");
		c.bold = Some(true);
		assert_eq!(parsed.content, ComponentType::Text { text: "a".to_string() });
		assert_eq!(parsed.extra, Some(vec![TextComponent::new("b"), c]), "the other items go after the existing extra");
	}

	#[test]
	fn all_content_types_survive_nbt_round_trip() {
		let mut translatable = TextComponent::translatable("chat.type.text");
		translatable.content = ComponentType::Translatable(Box::new(TranslatableContent {
			translate: "chat.type.text".to_string(),
			fallback: Some("%s: %s".to_string()),
			with: Some(vec![TextComponent::new("player"), TextComponent::new("hi")]),
		}));

		let components = vec![
			TextComponent::new("plain"),
			translatable,
			TextComponent::score("@p", "deaths"),
			TextComponent::selector("@e[type=cow]"),
			TextComponent::keybind("key.jump"),
			TextComponent::from_content(ComponentType::Nbt(Box::new(NbtContent {
				nbt: "Inventory[0].id".to_string(),
				interpret: Some(true),
				separator: None,
				block: None,
				entity: Some("@p".to_string()),
				storage: None,
			}))),
		];

		for component in components {
			assert_eq!(component, nbt_round_trip(&component), "content type did not survive NBT round trip");
		}
	}

	/// The variant must be inferred from which content field is present, matching the wiki's rule
	/// that `type` is optional. A regression here would silently mis-render server-sent components.
	#[test]
	fn json_infers_type_from_content_field() {
		let cases = [
			(
				r#"{"text":"hi"}"#,
				ComponentType::Text {
					text: "hi".to_string(),
				},
			),
			(
				r#"{"keybind":"key.jump"}"#,
				ComponentType::Keybind {
					keybind: "key.jump".to_string(),
				},
			),
			(
				r#"{"selector":"@a","separator":{"text":", "}}"#,
				ComponentType::Selector {
					selector: "@a".to_string(),
					separator: Some(Box::new(TextComponent::new(", "))),
				},
			),
			(
				r#"{"score":{"name":"@p","objective":"deaths"}}"#,
				ComponentType::Score {
					score: ScoreContent {
						name: "@p".to_string(),
						objective: "deaths".to_string(),
					},
				},
			),
		];

		for (json, expected) in cases {
			let parsed: TextComponent = serde_json::from_str(json).unwrap();
			assert_eq!(parsed.content, expected, "wrong content type inferred from {json}");
		}
	}

	/// A component may be a bare JSON string anywhere a component is expected — including inside a
	/// translatable component's `with` arguments. Servers send these (e.g. the version string in an
	/// incompatible-version disconnect), so failing to parse them drops the whole packet.
	#[test]
	fn string_shorthand_components_parse() {
		let json = r#"{"translate":"multiplayer.disconnect.incompatible","with":["26.1.2"]}"#;
		let parsed: TextComponent = serde_json::from_str(json).unwrap();

		assert_eq!(
			parsed.content,
			ComponentType::Translatable(Box::new(TranslatableContent {
				translate: "multiplayer.disconnect.incompatible".to_string(),
				fallback: None,
				with: Some(vec![TextComponent::new("26.1.2")]),
			}))
		);
	}

	/// An array is shorthand for its first element with the rest appended as `extra`. Mis-parsing it
	/// would silently drop the trailing components from a server-sent message.
	#[test]
	fn array_shorthand_appends_extra() {
		let json = r#"["first",{"text":"second"},"third"]"#;
		let parsed: TextComponent = serde_json::from_str(json).unwrap();

		assert_eq!(
			parsed.content,
			ComponentType::Text {
				text: "first".to_string()
			}
		);
		assert_eq!(parsed.extra, Some(vec![TextComponent::new("second"), TextComponent::new("third")]));
	}

	/// Formatting fields sit alongside the (flattened) content fields, not nested under it.
	#[test]
	fn formatting_stays_flat_with_content() {
		let mut component = TextComponent::translatable("multiplayer.player.joined");
		component.color = Some("yellow".to_string());

		let json = serde_json::to_string(&component).unwrap();
		assert!(json.contains(r#""translate":"multiplayer.player.joined""#));
		assert!(json.contains(r#""color":"yellow""#));

		assert_eq!(component, serde_json::from_str::<TextComponent>(&json).unwrap());
	}
}
