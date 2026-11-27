//! This file defines the TextComponent type in the Minecraft network API.
//! Seen in books, disconnect messages, chat messages, action bar, etc.
//!
//! <https://minecraft.wiki/w/Text_component_format>

use crate::protocol::game::player::interface::dialog::Dialog;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtIntArray, NbtList, NbtTag};
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A TextComponent is a fancy way to display text inside the game. This is most commonly seen
/// in chat messages and book messages. The only thing that is required to be included is a String
/// representing the text to be displayed. Everything else is an optional modifier.
///
/// See <https://minecraft.wiki/w/Text_component_format> for more information.
///
/// Not `Eq` or `Hash`: events and object components can hold raw NBT, which has float tags.
#[derive(Serialize, Debug, Clone, PartialEq)]
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

	/// The text shadow color as an ARGB integer (`alpha << 24 | red << 16 | green << 8 | blue`).
	/// A reader also accepts a list of 4 floats (red, green, blue, alpha, each 0 to 1).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub shadow_color: Option<i32>,

	/// Text put into the chat input when the player shift-clicks this text. Works only in chat.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub insertion: Option<String>,

	/// What happens when the player clicks this text. Boxed because most components have none.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub click_event: Option<Box<ClickEvent>>,

	/// What shows when the player puts the mouse on this text. Boxed because most components have none.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hover_event: Option<Box<HoverEvent>>,
}

/// Hand-written because a derived default fills `extra` with more default components,
/// and each of those does the same, so the derive recurses without end.
impl McDefault for TextComponent {
	#[cfg(any(test, feature = "testing"))]
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
			shadow_color: Option::mc_default(),
			insertion: Option::mc_default(),
			click_event: None,
			hover_event: None,
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
			shadow_color: None,
			insertion: None,
			click_event: None,
			hover_event: None,
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

	pub fn set_click_event(&mut self, event: ClickEvent) {
		self.click_event = Some(Box::new(event));
	}

	pub fn set_hover_event(&mut self, event: HoverEvent) {
		self.hover_event = Some(Box::new(event));
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
			&& self.shadow_color.is_none()
			&& self.insertion.is_none()
			&& self.click_event.is_none()
			&& self.hover_event.is_none()
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
				let shadow_color = compound.get("shadow_color").and_then(shadow_color_from_tag);
				// An event that is present but not valid is an error, as in vanilla.
				let click_event = match compound.get("click_event") {
					Some(NbtTag::Compound(c)) => Some(Box::new(ClickEvent::from_compound(c)?)),
					Some(_) => return Err(NbtError::InvalidType),
					None => None,
				};
				let hover_event = match compound.get("hover_event") {
					Some(NbtTag::Compound(c)) => Some(Box::new(HoverEvent::from_compound(c)?)),
					Some(_) => return Err(NbtError::InvalidType),
					None => None,
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
					shadow_color,
					insertion,
					click_event,
					hover_event,
				})
			}
			_ => Err(NbtError::InvalidType),
		}
	}
}

impl From<TextComponent> for NbtTag {
	fn from(component: TextComponent) -> Self {
		if component.is_plain()
			&& let ComponentType::Text {
				text,
			} = component.content
		{
			return NbtTag::String(text);
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
		if let Some(shadow_color) = component.shadow_color {
			compound.add("shadow_color", NbtTag::Int(shadow_color));
		}
		if let Some(insertion) = component.insertion {
			compound.add("insertion", NbtTag::String(insertion));
		}
		if let Some(click_event) = component.click_event {
			compound.add("click_event", click_event.into_compound());
		}
		if let Some(hover_event) = component.hover_event {
			compound.add("hover_event", hover_event.into_compound());
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
	#[serde(default, deserialize_with = "shadow_color_json")]
	shadow_color: Option<i32>,
	insertion: Option<String>,
	click_event: Option<Box<ClickEvent>>,
	hover_event: Option<Box<HoverEvent>>,
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
			shadow_color: obj.shadow_color,
			insertion: obj.insertion,
			click_event: obj.click_event,
			hover_event: obj.hover_event,
		}
	}
}

/// Read a `shadow_color` tag: an ARGB int, or a list of 4 floats (red, green, blue, alpha) from 0 to 1.
/// Each float becomes `floor(f * 255)`, as in vanilla.
fn shadow_color_from_tag(tag: &NbtTag) -> Option<i32> {
	match tag {
		NbtTag::Int(argb) => Some(*argb),
		NbtTag::List(list) if list.list.len() == 4 => {
			let channels = list
				.list
				.iter()
				.map(|t| match t {
					NbtTag::Float(f) => Some(*f),
					NbtTag::Double(d) => Some(*d as f32),
					NbtTag::Int(i) => Some(*i as f32),
					_ => None,
				})
				.map(|f| f.map(|f| ((f * 255.0).floor() as i32) & 0xFF))
				.collect::<Option<Vec<i32>>>()?;
			Some(channels[3] << 24 | channels[0] << 16 | channels[1] << 8 | channels[2])
		}
		_ => None,
	}
}

fn shadow_color_json<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<i32>, D::Error> {
	let tag: NbtTag = nbt_json::deserialize(deserializer)?;
	shadow_color_from_tag(&tag).map(Some).ok_or_else(|| de::Error::custom("shadow_color must be an int or a list of 4 floats"))
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
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq)]
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
/// Last is `object`, found from its own fields (`object`, `sprite`, `player`).
///
/// See <https://minecraft.wiki/w/Text_component_format#Java_Edition>
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq)]
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
	/// A sprite from a texture atlas, or a player head, shown in the text (1.21.9+).
	/// Boxed because this is a rare, large variant.
	Object(Box<ObjectContent>),
}

/// The fields of a [`ComponentType::Translatable`] component. Held behind a [`Box`] in the enum to
/// keep the common (text/keybind) variants small. Serialized untagged, so its fields sit flat in
/// the parent component exactly as the inline variant did.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NbtContent {
	pub nbt: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub interpret: Option<bool>,
	/// Show the value as plain text, not pretty-printed (26.1+). Cannot be used with `interpret`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub plain: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub separator: Option<Box<TextComponent>>,
	/// Which of `block`, `entity` or `storage` to read. When absent, the reader checks entity, then
	/// block, then storage.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub source: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub entity: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub storage: Option<String>,
}

/// The fields of a [`ComponentType::Object`] component.
///
/// <https://minecraft.wiki/w/Text_component_format#Object>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ObjectContent {
	#[serde(flatten)]
	pub info: ObjectInfo,
	/// Shown in place of the object where it cannot be drawn (26.1+).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fallback: Option<Box<TextComponent>>,
}

/// What a [`ComponentType::Object`] component shows, keyed by the `object` field.
/// When `object` is absent, the reader uses `atlas`.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "object", rename_all = "snake_case")]
pub enum ObjectInfo {
	/// A sprite from a texture atlas.
	Atlas {
		/// The atlas id. Vanilla uses `minecraft:blocks` when absent.
		#[serde(skip_serializing_if = "Option::is_none")]
		atlas: Option<String>,
		/// The sprite id in the atlas, for example `block/emerald_block`.
		sprite: String,
	},
	/// The head of a player.
	Player {
		/// A player name (String tag) or a profile compound, the same as the `profile` item component.
		#[serde(serialize_with = "nbt_json::serialize")]
		player: NbtTag,
		/// Show the hat layer of the skin. Vanilla uses `true` when absent.
		#[serde(skip_serializing_if = "Option::is_none")]
		hat: Option<bool>,
	},
}

/// Read form of [`ObjectInfo`]. Serde cannot give a default to the `object` tag, so the fields are read
/// first and the variant is chosen after.
#[derive(Deserialize)]
struct ObjectInfoFields {
	object: Option<String>,
	atlas: Option<String>,
	sprite: Option<String>,
	#[serde(default, with = "nbt_json::option")]
	player: Option<NbtTag>,
	hat: Option<bool>,
}

impl<'de> Deserialize<'de> for ObjectInfo {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let f = ObjectInfoFields::deserialize(deserializer)?;
		match f.object.as_deref() {
			None | Some("atlas") => Ok(ObjectInfo::Atlas {
				atlas: f.atlas,
				sprite: f.sprite.ok_or_else(|| de::Error::missing_field("sprite"))?,
			}),
			Some("player") => Ok(ObjectInfo::Player {
				player: f.player.ok_or_else(|| de::Error::missing_field("player"))?,
				hat: f.hat,
			}),
			Some(other) => Err(de::Error::unknown_variant(other, &["atlas", "player"])),
		}
	}
}

impl ObjectContent {
	fn from_compound(compound: &NbtCompound) -> Result<Self, NbtError> {
		let info = match compound.get_string("object").as_deref() {
			None | Some("atlas") => ObjectInfo::Atlas {
				atlas: compound.get_string("atlas"),
				sprite: require_string(compound, "sprite")?,
			},
			Some("player") => ObjectInfo::Player {
				player: compound.get("player").cloned().ok_or_else(|| NbtError::MissingField("player".to_string()))?,
				hat: compound.get_bool("hat"),
			},
			Some(_) => return Err(NbtError::InvalidType),
		};
		let fallback = compound.get("fallback").map(|t| TextComponent::try_from(t.clone()).map(Box::new)).transpose()?;
		Ok(Self {
			info,
			fallback,
		})
	}
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
					plain: compound.get_bool("plain"),
					separator: get_separator("separator")?,
					source: compound.get_string("source"),
					block: compound.get_string("block"),
					entity: compound.get_string("entity"),
					storage: compound.get_string("storage"),
				})));
			}
			Some("object") => {
				return Ok(ComponentType::Object(Box::new(ObjectContent::from_compound(compound)?)));
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
				plain: compound.get_bool("plain"),
				separator: get_separator("separator")?,
				source: compound.get_string("source"),
				block: compound.get_string("block"),
				entity: compound.get_string("entity"),
				storage: compound.get_string("storage"),
			}))
		} else if ["object", "sprite", "player"].iter().any(|key| compound.get(*key).is_some()) {
			ComponentType::Object(Box::new(ObjectContent::from_compound(compound)?))
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
					plain,
					separator,
					source,
					block,
					entity,
					storage,
				} = *content;
				compound.add("type", NbtTag::String("nbt".to_string()));
				compound.add("nbt", NbtTag::String(nbt));
				if let Some(interpret) = interpret {
					compound.add("interpret", NbtTag::from(interpret));
				}
				if let Some(plain) = plain {
					compound.add("plain", NbtTag::from(plain));
				}
				if let Some(separator) = separator {
					compound.add("separator", NbtTag::from(*separator));
				}
				if let Some(source) = source {
					compound.add("source", NbtTag::String(source));
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
			ComponentType::Object(content) => {
				let ObjectContent {
					info,
					fallback,
				} = *content;
				compound.add("type", NbtTag::String("object".to_string()));
				match info {
					ObjectInfo::Atlas {
						atlas,
						sprite,
					} => {
						compound.add("object", NbtTag::String("atlas".to_string()));
						if let Some(atlas) = atlas {
							compound.add("atlas", NbtTag::String(atlas));
						}
						compound.add("sprite", NbtTag::String(sprite));
					}
					ObjectInfo::Player {
						player,
						hat,
					} => {
						compound.add("object", NbtTag::String("player".to_string()));
						compound.add("player", player);
						if let Some(hat) = hat {
							compound.add("hat", NbtTag::from(hat));
						}
					}
				}
				if let Some(fallback) = fallback {
					compound.add("fallback", NbtTag::from(*fallback));
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

/// Read a required string field, or give [`NbtError::MissingField`].
fn require_string(compound: &NbtCompound, key: &str) -> Result<String, NbtError> {
	compound.get_string(key).ok_or_else(|| NbtError::MissingField(key.to_string()))
}

/// Read a required field of any tag type, or give [`NbtError::MissingField`].
fn require_tag<'a>(compound: &'a NbtCompound, key: &str) -> Result<&'a NbtTag, NbtError> {
	compound.get(key).ok_or_else(|| NbtError::MissingField(key.to_string()))
}

/// Read an int field. As in vanilla, any whole-number tag is accepted.
fn get_int(compound: &NbtCompound, key: &str) -> Option<i32> {
	match compound.get(key)? {
		NbtTag::Byte(b) => Some(*b as i32),
		NbtTag::Short(s) => Some(*s as i32),
		NbtTag::Int(i) => Some(*i),
		NbtTag::Long(l) => Some(*l as i32),
		_ => None,
	}
}

/// What happens when the player clicks a [`TextComponent`]. Each action has its own fields
/// (1.21.5+ format).
///
/// <https://minecraft.wiki/w/Text_component_format#Click_events>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClickEvent {
	/// Open a URL in the browser of the player. Vanilla accepts only `http://` and `https://`.
	OpenUrl { url: String },
	/// Open a file on the computer of the player. Vanilla clients do not accept this from a server.
	OpenFile { path: String },
	/// Run a command as the player. The leading `/` is optional.
	RunCommand { command: String },
	/// Put a command into the chat input of the player. Does not work in books.
	SuggestCommand { command: String },
	/// Go to a page of a written book. Works only in books. Must be 1 or more.
	ChangePage { page: i32 },
	/// Copy text to the clipboard of the player.
	CopyToClipboard { value: String },
	/// Open a dialog (1.21.6+). The value is a dialog id (String tag) or a dialog compound.
	ShowDialog {
		#[serde(with = "nbt_json")]
		dialog: NbtTag,
	},
	/// Send a custom payload to the server (1.21.6+). Vanilla servers ignore it.
	Custom {
		id: String,
		#[serde(default, skip_serializing_if = "Option::is_none", with = "nbt_json::option")]
		payload: Option<NbtTag>,
	},
}

impl ClickEvent {
	pub fn open_url<T: Into<String>>(url: T) -> Self {
		Self::OpenUrl {
			url: url.into(),
		}
	}

	/// Internal to clients only - doesn't work over server connections
	pub fn open_file<T: Into<String>>(path: T) -> Self {
		Self::OpenFile {
			path: path.into(),
		}
	}

	pub fn run_command<T: Into<String>>(command: T) -> Self {
		Self::RunCommand {
			command: command.into(),
		}
	}

	pub fn suggest_command<T: Into<String>>(command: T) -> Self {
		Self::SuggestCommand {
			command: command.into(),
		}
	}

	pub fn change_page(page: i32) -> Self {
		Self::ChangePage {
			page,
		}
	}

	pub fn copy_to_clipboard<T: Into<String>>(value: T) -> Self {
		Self::CopyToClipboard {
			value: value.into(),
		}
	}

	/// Open a dialog from the dialog registry, by its id.
	pub fn show_dialog_id<T: Into<String>>(id: T) -> Self {
		Self::ShowDialog {
			dialog: NbtTag::String(id.into()),
		}
	}

	/// Open a dialog that is sent in full with the text.
	pub fn show_dialog(dialog: &Dialog) -> Self {
		Self::ShowDialog {
			dialog: NbtTag::Compound(dialog.as_nbt()),
		}
	}

	pub fn custom<T: Into<String>>(id: T, payload: Option<NbtTag>) -> Self {
		Self::Custom {
			id: id.into(),
			payload,
		}
	}

	/// The vanilla id of the action, written in the `action` field.
	pub fn action(&self) -> &'static str {
		match self {
			Self::OpenUrl { .. } => "open_url",
			Self::OpenFile { .. } => "open_file",
			Self::RunCommand { .. } => "run_command",
			Self::SuggestCommand { .. } => "suggest_command",
			Self::ChangePage { .. } => "change_page",
			Self::CopyToClipboard { .. } => "copy_to_clipboard",
			Self::ShowDialog { .. } => "show_dialog",
			Self::Custom { .. } => "custom",
		}
	}

	fn from_compound(compound: &NbtCompound) -> Result<Self, NbtError> {
		Ok(match require_string(compound, "action")?.as_str() {
			"open_url" => Self::open_url(require_string(compound, "url")?),
			"open_file" => Self::open_file(require_string(compound, "path")?),
			"run_command" => Self::run_command(require_string(compound, "command")?),
			"suggest_command" => Self::suggest_command(require_string(compound, "command")?),
			"change_page" => Self::change_page(get_int(compound, "page").ok_or_else(|| NbtError::MissingField("page".to_string()))?),
			"copy_to_clipboard" => Self::copy_to_clipboard(require_string(compound, "value")?),
			"show_dialog" => Self::ShowDialog {
				dialog: require_tag(compound, "dialog")?.clone(),
			},
			"custom" => Self::custom(require_string(compound, "id")?, compound.get("payload").cloned()),
			_ => return Err(NbtError::InvalidType),
		})
	}

	fn into_compound(self) -> NbtCompound {
		let mut compound = NbtCompound::new_no_name();
		compound.add("action", self.action());
		match self {
			Self::OpenUrl {
				url,
			} => compound.add("url", url),
			Self::OpenFile {
				path,
			} => compound.add("path", path),
			Self::RunCommand {
				command,
			}
			| Self::SuggestCommand {
				command,
			} => compound.add("command", command),
			Self::ChangePage {
				page,
			} => compound.add("page", page),
			Self::CopyToClipboard {
				value,
			} => compound.add("value", value),
			Self::ShowDialog {
				dialog,
			} => compound.add("dialog", dialog),
			Self::Custom {
				id,
				payload,
			} => {
				compound.add("id", id);
				if let Some(payload) = payload {
					compound.add("payload", payload);
				}
			}
		}
		compound
	}
}

/// What shows when the player puts the mouse on a [`TextComponent`]. Each action has its own fields
/// (1.21.5+ format).
///
/// <https://minecraft.wiki/w/Text_component_format#Hover_events>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum HoverEvent {
	/// Show a text tooltip. Click and hover events inside the tooltip do not work.
	ShowText { value: Box<TextComponent> },
	/// Show the tooltip of an item.
	ShowItem(ItemHover),
	/// Show the type, UUID and name of an entity.
	ShowEntity(EntityHover),
}

impl HoverEvent {
	pub fn show_text<T: Into<String>>(text: T) -> Self {
		Self::show_text_component(TextComponent::new(text))
	}

	pub fn show_text_component(text: TextComponent) -> Self {
		Self::ShowText {
			value: Box::new(text),
		}
	}

	/// Show an item with no data components.
	pub fn show_item<T: Into<String>>(id: T, count: i32) -> Self {
		Self::ShowItem(ItemHover {
			id: id.into(),
			count,
			components: None,
		})
	}

	pub fn show_entity<T: Into<String>>(id: T, uuid: Uuid, name: Option<TextComponent>) -> Self {
		Self::ShowEntity(EntityHover {
			id: id.into(),
			uuid,
			name: name.map(Box::new),
		})
	}

	/// The vanilla id of the action, written in the `action` field.
	pub fn action(&self) -> &'static str {
		match self {
			Self::ShowText { .. } => "show_text",
			Self::ShowItem(_) => "show_item",
			Self::ShowEntity(_) => "show_entity",
		}
	}

	fn from_compound(compound: &NbtCompound) -> Result<Self, NbtError> {
		Ok(match require_string(compound, "action")?.as_str() {
			"show_text" => Self::show_text_component(TextComponent::try_from(require_tag(compound, "value")?.clone())?),
			"show_item" => Self::ShowItem(ItemHover {
				id: require_string(compound, "id")?,
				count: get_int(compound, "count").unwrap_or(1),
				components: match compound.get("components") {
					Some(NbtTag::Compound(c)) => Some(c.clone()),
					Some(_) => return Err(NbtError::InvalidType),
					None => None,
				},
			}),
			"show_entity" => Self::ShowEntity(EntityHover {
				id: require_string(compound, "id")?,
				uuid: uuid_from_tag(require_tag(compound, "uuid")?)?,
				name: compound.get("name").map(|t| TextComponent::try_from(t.clone()).map(Box::new)).transpose()?,
			}),
			_ => return Err(NbtError::InvalidType),
		})
	}

	fn into_compound(self) -> NbtCompound {
		let mut compound = NbtCompound::new_no_name();
		compound.add("action", self.action());
		match self {
			Self::ShowText {
				value,
			} => compound.add("value", NbtTag::from(*value)),
			Self::ShowItem(item) => {
				compound.add("id", item.id);
				compound.add("count", item.count);
				if let Some(components) = item.components {
					compound.add("components", components);
				}
			}
			Self::ShowEntity(entity) => {
				compound.add("id", entity.id);
				compound.add("uuid", uuid_to_tag(entity.uuid));
				if let Some(name) = entity.name {
					compound.add("name", NbtTag::from(*name));
				}
			}
		}
		compound
	}
}

/// The fields of a [`HoverEvent::ShowItem`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemHover {
	/// The item id, for example `minecraft:diamond_sword`.
	pub id: String,
	/// The stack size. Vanilla uses 1 when absent.
	#[serde(default = "default_item_count")]
	pub count: i32,
	/// The data component patch of the item, as raw NBT: component id to value, and `!id` to remove
	/// a default component.
	#[serde(default, skip_serializing_if = "Option::is_none", with = "nbt_json::option")]
	pub components: Option<NbtCompound>,
}

fn default_item_count() -> i32 {
	1
}

/// The fields of a [`HoverEvent::ShowEntity`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EntityHover {
	/// The entity type id, for example `minecraft:pig`.
	pub id: String,
	/// Written as 4 ints. A reader also accepts a UUID string with hyphens, or a list of 4 ints.
	#[serde(with = "uuid_json")]
	pub uuid: Uuid,
	/// The entity name. Hidden when absent.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<Box<TextComponent>>,
}

/// Vanilla writes a UUID as an int array of 4, most significant first.
fn uuid_to_tag(uuid: Uuid) -> NbtTag {
	let v = uuid.as_u128();
	NbtTag::IntArray(NbtIntArray::new(vec![(v >> 96) as i32, (v >> 64) as i32, (v >> 32) as i32, v as i32]))
}

/// Read a UUID as an int array of 4, a list of 4 ints, or a string.
fn uuid_from_tag(tag: &NbtTag) -> Result<Uuid, NbtError> {
	let ints: Vec<i32> = match tag {
		NbtTag::String(s) => return Uuid::parse_str(s).map_err(|_| NbtError::InvalidType),
		NbtTag::IntArray(array) => array.list.clone(),
		NbtTag::List(list) => list
			.list
			.iter()
			.map(|t| match t {
				NbtTag::Int(i) => Ok(*i),
				_ => Err(NbtError::InvalidType),
			})
			.collect::<Result<_, _>>()?,
		_ => return Err(NbtError::InvalidType),
	};
	let [a, b, c, d]: [i32; 4] = ints.try_into().map_err(|_| NbtError::InvalidType)?;
	Ok(Uuid::from_u128((a as u32 as u128) << 96 | (b as u32 as u128) << 64 | (c as u32 as u128) << 32 | d as u32 as u128))
}

/// Serde helpers for a UUID in the JSON form of a text component.
mod uuid_json {
    use super::{nbt_json, uuid_from_tag, uuid_to_tag};
    use serde::de::Error;
    use serde::{Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S: Serializer>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error> {
		nbt_json::serialize(&uuid_to_tag(*uuid), serializer)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Uuid, D::Error> {
		let tag: crate::protocol_types::datatypes::nbt::NbtTag = nbt_json::deserialize(deserializer)?;
		uuid_from_tag(&tag).map_err(D::Error::custom)
	}
}

/// Serde helpers for raw NBT fields in the JSON form of a text component (status response and
/// login disconnect). NBT is written as plain JSON values. On read, a whole number becomes an Int
/// (or a Long if it is too large), any other number becomes a Double, a bool becomes a Byte, and an
/// array becomes a list.
mod nbt_json {
    use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtList, NbtTag};
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_json::Value;

    pub fn to_json(tag: &NbtTag) -> Value {
		let number = |f: f64| serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null);
		match tag {
			NbtTag::End => Value::Null,
			NbtTag::Byte(b) => Value::from(*b),
			NbtTag::Short(s) => Value::from(*s),
			NbtTag::Int(i) => Value::from(*i),
			NbtTag::Long(l) => Value::from(*l),
			NbtTag::Float(f) => number(*f as f64),
			NbtTag::Double(d) => number(*d),
			NbtTag::ByteArray(a) => a.list.iter().map(|b| Value::from(*b)).collect(),
			NbtTag::String(s) => Value::String(s.clone()),
			NbtTag::List(l) => l.list.iter().map(|t| to_json(&t.clone().unwrap_list_item())).collect(),
			NbtTag::Compound(c) => Value::Object(c.map.iter().map(|(k, v)| (k.clone(), to_json(v))).collect()),
			NbtTag::IntArray(a) => a.list.iter().map(|i| Value::from(*i)).collect(),
			NbtTag::LongArray(a) => a.list.iter().map(|l| Value::from(*l)).collect(),
		}
	}

	pub fn from_json(value: Value) -> Result<NbtTag, String> {
		Ok(match value {
			Value::Null => return Err("null is not an NBT value".to_string()),
			Value::Bool(b) => NbtTag::from(b),
			Value::Number(n) => match n.as_i64() {
				Some(i) => i32::try_from(i).map(NbtTag::Int).unwrap_or(NbtTag::Long(i)),
				None => NbtTag::Double(n.as_f64().unwrap_or_default()),
			},
			Value::String(s) => NbtTag::String(s),
			Value::Array(items) => {
				let tags = items.into_iter().map(from_json).collect::<Result<Vec<_>, _>>()?;
				NbtTag::List(NbtList::from_vec_wrapping(tags).map_err(|e| e.to_string())?)
			}
			Value::Object(map) => {
				let mut compound = NbtCompound::new_no_name();
				for (k, v) in map {
					compound.add(k, from_json(v)?);
				}
				NbtTag::Compound(compound)
			}
		})
	}

	pub fn serialize<T: Clone + Into<NbtTag>, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
		to_json(&value.clone().into()).serialize(serializer)
	}

	pub fn deserialize<'de, T: TryFrom<NbtTag>, D: Deserializer<'de>>(deserializer: D) -> Result<T, D::Error> {
		let tag = from_json(Value::deserialize(deserializer)?).map_err(D::Error::custom)?;
		T::try_from(tag).map_err(|_| D::Error::custom("wrong NBT type"))
	}

	pub mod option {
        use super::*;

        pub fn serialize<T: Clone + Into<NbtTag>, S: Serializer>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error> {
			match value {
				Some(v) => super::serialize(v, serializer),
				None => serializer.serialize_none(),
			}
		}

		pub fn deserialize<'de, T: TryFrom<NbtTag>, D: Deserializer<'de>>(deserializer: D) -> Result<Option<T>, D::Error> {
			super::deserialize(deserializer).map(Some)
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PlayerChatSignature {
	message_id: VarInt,
	#[mc(deserialize_if = message_id.0 == 0)]
	signature: Option<[u8; 256]>,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
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
		// Since 1.21.5 each action has its own field name. A wrong name makes vanilla fail the whole
		// component, so the player sees no text.
		for (event, action, field, value) in [
			(ClickEvent::open_url("https://a.b"), "open_url", "url", NbtTag::from("https://a.b")),
			(ClickEvent::open_file("f.txt"), "open_file", "path", NbtTag::from("f.txt")),
			(ClickEvent::run_command("/say hi"), "run_command", "command", NbtTag::from("/say hi")),
			(ClickEvent::suggest_command("/tp"), "suggest_command", "command", NbtTag::from("/tp")),
			(ClickEvent::change_page(3), "change_page", "page", NbtTag::Int(3)),
			(ClickEvent::copy_to_clipboard("x"), "copy_to_clipboard", "value", NbtTag::from("x")),
			(ClickEvent::show_dialog_id("minecraft:server_links"), "show_dialog", "dialog", NbtTag::from("minecraft:server_links")),
			(ClickEvent::custom("my:id", None), "custom", "id", NbtTag::from("my:id")),
		] {
			let compound = event.clone().into_compound();
			assert_eq!(compound.get_string("action").as_deref(), Some(action));
			assert_eq!(compound.get(field), Some(&value), "{action} must write `{field}`");
			assert_eq!(ClickEvent::from_compound(&compound).unwrap(), event, "{action} must read back");
		}
	}

	/// The JSON form (status response, login disconnect) uses the same names. Each action must
	/// write its own field, not the old `value` field.
	#[test]
	fn click_event_json_uses_action_fields() {
		let json = serde_json::to_value(ClickEvent::change_page(2)).unwrap();
		assert_eq!(json, serde_json::json!({"action": "change_page", "page": 2}));

		let custom = ClickEvent::custom("my:id", Some(NbtTag::Compound(crate::nbt_compound! {"k" => 1})));
		let json = serde_json::to_value(&custom).unwrap();
		assert_eq!(json, serde_json::json!({"action": "custom", "id": "my:id", "payload": {"k": 1}}));
		assert_eq!(serde_json::from_value::<ClickEvent>(json).unwrap(), custom);
	}

	/// An inline dialog must be sent as its NBT compound, so the client can open it with no registry entry.
	#[test]
	fn show_dialog_inline_writes_dialog_compound() {
		let dialog = Dialog::mc_default();
		match ClickEvent::show_dialog(&dialog) {
			ClickEvent::ShowDialog {
				dialog: NbtTag::Compound(c),
			} => assert_eq!(c, dialog.as_nbt()),
			other => panic!("expected an inline dialog, got {other:?}"),
		}
	}

	/// An unknown action or a missing field is an error, as in vanilla. If the reader gave a default,
	/// a click could run a command that the server did not send.
	#[test]
	fn invalid_click_event_is_an_error() {
		let unknown = crate::nbt_compound! {"action" => "explode", "value" => "x"};
		assert_eq!(ClickEvent::from_compound(&unknown), Err(NbtError::InvalidType));

		let old_format = crate::nbt_compound! {"action" => "run_command", "value" => "/x"};
		assert_eq!(ClickEvent::from_compound(&old_format), Err(NbtError::MissingField("command".to_string())));

		let mut root = crate::nbt_compound! {"text" => "a"};
		root.add("click_event", unknown);
		assert!(TextComponent::try_from(NbtTag::Compound(root)).is_err());
	}

	/// Every hover action must survive the wire, or the tooltip is lost or wrong.
	#[test]
	fn hover_events_survive_nbt_round_trip() {
		let uuid = Uuid::from_u128(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
		let mut item = HoverEvent::show_item("minecraft:diamond_sword", 1);
		if let HoverEvent::ShowItem(i) = &mut item {
			i.components = Some(crate::nbt_compound! {"minecraft:damage" => 5});
		}
		for event in [
			HoverEvent::show_text("tip"),
			item,
			HoverEvent::show_entity("minecraft:pig", uuid, Some(TextComponent::new("Bob"))),
			HoverEvent::show_entity("minecraft:cow", uuid, None),
		] {
			let mut component = TextComponent::new("hover me");
			component.set_hover_event(event);
			assert_eq!(nbt_round_trip(&component), component);
		}
	}

	/// Vanilla writes the UUID as 4 ints, most significant first. It reads a string or a list too.
	#[test]
	fn entity_hover_uuid_formats() {
		let uuid = Uuid::parse_str("00000001-0000-0002-0000-0003ffffffff").unwrap();
		assert_eq!(uuid_to_tag(uuid), NbtTag::IntArray(NbtIntArray::new(vec![1, 2, 3, -1])));

		let from_list = NbtTag::List(NbtList::from_vec(vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3), NbtTag::Int(-1)]).unwrap());
		assert_eq!(uuid_from_tag(&from_list), Ok(uuid));
		assert_eq!(uuid_from_tag(&NbtTag::from("00000001-0000-0002-0000-0003ffffffff")), Ok(uuid));
		assert_eq!(uuid_from_tag(&NbtTag::IntArray(NbtIntArray::new(vec![1, 2, 3]))), Err(NbtError::InvalidType));
	}

	/// The count of a hover item is optional, and vanilla uses 1. Without the default, a vanilla item
	/// tooltip with no count would fail.
	#[test]
	fn show_item_count_defaults_to_one() {
		let compound = crate::nbt_compound! {"action" => "show_item", "id" => "minecraft:stone"};
		assert_eq!(HoverEvent::from_compound(&compound).unwrap(), HoverEvent::show_item("minecraft:stone", 1));
		let json: HoverEvent = serde_json::from_str(r#"{"action":"show_item","id":"minecraft:stone"}"#).unwrap();
		assert_eq!(json, HoverEvent::show_item("minecraft:stone", 1));
	}

	/// A vanilla JSON component with events and a shadow color must parse into the same value that
	/// the NBT reader gives, so both paths of the protocol agree.
	#[test]
	fn vanilla_json_with_events_parses() {
		let json = r#"{
			"text": "click",
			"shadow_color": [1.0, 0.0, 0.0, 0.5],
			"click_event": {"action": "run_command", "command": "say hi"},
			"hover_event": {"action": "show_entity", "id": "minecraft:pig", "uuid": [1, 2, 3, 4], "name": "Bob"}
		}"#;
		let parsed: TextComponent = serde_json::from_str(json).unwrap();

		let mut expected = TextComponent::new("click");
		// floor(0.5 * 255) = 127 alpha, full red.
		expected.shadow_color = Some(0x7FFF0000);
		expected.set_click_event(ClickEvent::run_command("say hi"));
		expected.set_hover_event(HoverEvent::show_entity("minecraft:pig", Uuid::from_u128(1 << 96 | 2 << 64 | 3 << 32 | 4), Some(TextComponent::new("Bob"))));
		assert_eq!(parsed, expected);

		let written = serde_json::to_string(&expected).unwrap();
		assert!(written.contains(r#""click_event":{"action":"run_command","command":"say hi"}"#), "{written}");
		assert_eq!(serde_json::from_str::<TextComponent>(&written).unwrap(), expected);
		assert_eq!(nbt_round_trip(&expected), expected);
	}

	/// A shadow color is an ARGB int on the wire. The float list form must give the same int, with
	/// each channel as floor(f * 255), as in vanilla.
	#[test]
	fn shadow_color_float_list_becomes_argb() {
		let list = |v: [f32; 4]| NbtTag::List(NbtList::from_vec(v.iter().map(|f| NbtTag::Float(*f)).collect()).unwrap());
		assert_eq!(shadow_color_from_tag(&list([1.0, 1.0, 1.0, 1.0])), Some(-1));
		assert_eq!(shadow_color_from_tag(&list([0.0, 0.5, 1.0, 0.25])), Some(0x3F007FFF));
		assert_eq!(shadow_color_from_tag(&NbtTag::Int(0x11223344)), Some(0x11223344));
		assert_eq!(shadow_color_from_tag(&NbtTag::from("red")), None);
	}

	/// A component with an event is not plain. If it were, the writer would send a bare string and
	/// the event would be lost.
	#[test]
	fn component_with_event_is_not_plain() {
		let mut component = TextComponent::new("a");
		component.set_click_event(ClickEvent::open_url("https://a.b"));
		assert!(!component.is_plain());
		assert!(matches!(NbtTag::from(component), NbtTag::Compound(_)));
	}

	/// Object components can come with no `type`. Without inference, a sprite or head would show as
	/// empty text. With no `object` field, vanilla uses `atlas`.
	#[test]
	fn object_content_inferred_without_type() {
		match content_of(vec![("sprite", "item/apple".into())]) {
			ComponentType::Object(o) => assert_eq!(
				o.info,
				ObjectInfo::Atlas {
					atlas: None,
					sprite: "item/apple".to_string()
				}
			),
			other => panic!("expected object content, got {other:?}"),
		}
		assert!(matches!(content_of(vec![("object", "player".into()), ("player", "Steve".into())]), ComponentType::Object(_)));

		let json: TextComponent = serde_json::from_str(r#"{"object":"player","player":{"name":"Steve"},"hat":false}"#).unwrap();
		match json.content {
			ComponentType::Object(o) => assert!(matches!(o.info, ObjectInfo::Player { hat: Some(false), .. })),
			other => panic!("expected object content, got {other:?}"),
		}
	}

	/// A player object must write `"object": "player"`. Vanilla uses `atlas` when the field is
	/// absent, and then fails because there is no `sprite`.
	#[test]
	fn player_object_writes_object_field() {
		let component = TextComponent::from_content(ComponentType::Object(Box::new(ObjectContent {
			info: ObjectInfo::Player {
				player: NbtTag::from("Steve"),
				hat: None,
			},
			fallback: None,
		})));
		let json = serde_json::to_value(&component).unwrap();
		assert_eq!(json, serde_json::json!({"object": "player", "player": "Steve"}));
		assert_eq!(serde_json::from_value::<TextComponent>(json).unwrap(), component);
		match NbtTag::from(component) {
			NbtTag::Compound(c) => assert_eq!(c.get_string("object").as_deref(), Some("player")),
			other => panic!("expected a compound, got {other:?}"),
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
				plain: Some(false),
				separator: None,
				source: Some("entity".to_string()),
				block: None,
				entity: Some("@p".to_string()),
				storage: None,
			}))),
			TextComponent::from_content(ComponentType::Object(Box::new(ObjectContent {
				info: ObjectInfo::Atlas {
					atlas: Some("minecraft:items".to_string()),
					sprite: "item/diamond".to_string(),
				},
				fallback: Some(Box::new(TextComponent::new("[diamond]"))),
			}))),
			TextComponent::from_content(ComponentType::Object(Box::new(ObjectContent {
				info: ObjectInfo::Player {
					player: NbtTag::String("Steve".to_string()),
					hat: Some(false),
				},
				fallback: None,
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
