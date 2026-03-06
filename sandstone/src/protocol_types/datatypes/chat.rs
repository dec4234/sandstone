//! This file defines the TextComponent type in the Minecraft network API.
//! Seen in books, disconnect messages, chat messages, action bar, etc.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt::{NbtCompound, NbtList, NbtTag};
use sandstone_derive::McDefault;
use serde::{Deserialize, Serialize};

/// A TextComponent is a fancy way to display text inside the game. This is most commonly seen
/// in chat messages and book messages. The only thing that is required to be included is a String
/// representing the text to be displayed. Everything else is an optional modifier.
///
/// See https://minecraft.wiki/w/Text_component_format for more information.
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TextComponent {
	pub text: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "type")]
	pub typ: Option<String>, // TODO: replace with ComponentType enum
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

impl TextComponent {
	pub fn new<T: Into<String>>(text: T) -> Self {
		Self {
			text: text.into(),
			typ: None,
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
	
	pub fn set_type<T: Into<String>>(&mut self, typ: T) {
		self.typ = Some(typ.into());
	}
	
	pub fn set_extra(&mut self, extra: Vec<TextComponent>) {
		self.extra = Some(extra);
	}

	/// True if this TextComponent has no modifiers and is just a plain string.
	pub fn is_plain(&self) -> bool {
		self.typ.is_none()
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

impl From<NbtTag> for TextComponent {
	fn from(tag: NbtTag) -> Self {
		match tag {
			NbtTag::String(s) => Self::new(s),
			NbtTag::Compound(compound) => {
				let text = match compound.get("text") {
					Some(NbtTag::String(s)) => s.clone(),
					_ => String::new(),
				};
				let typ = match compound.get("type") {
					Some(NbtTag::String(s)) => Some(s.clone()),
					_ => None,
				};
				let extra = match compound.get("extra") {
					Some(NbtTag::List(list)) => {
						let components: Vec<TextComponent> = list.list.iter()
							.map(|tag| TextComponent::from(tag.clone()))
							.collect();
						Some(components)
					}
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
				Self { text, typ, extra, color, bold, italic, underlined, strikethrough, obfuscated, font, insertion }
			}
			_ => Self::new(String::new()),
		}
	}
}

impl From<TextComponent> for NbtTag {
	fn from(component: TextComponent) -> Self {
		if component.is_plain() {
			return NbtTag::String(component.text);
		}

		let mut compound = NbtCompound::new_no_name();
		compound.add("text", NbtTag::String(component.text));
		if let Some(typ) = component.typ {
			compound.add("type", NbtTag::String(typ));
		}
		if let Some(extra) = component.extra {
			let tags: Vec<NbtTag> = extra.into_iter().map(NbtTag::from).collect();
			if let Ok(list) = NbtList::from_vec(tags) {
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
				nbt.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for TextComponent {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> Result<Self, SerializingErr> {
		let type_id = u8::mc_deserialize(deserializer)?;
		let nbt = NbtTag::deserialize_specific(deserializer, type_id)?;
		Ok(TextComponent::from(nbt))
	}
}

impl From<String> for TextComponent {
	fn from(s: String) -> Self {
		Self::new(s)
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentType {
	#[serde(rename = "text")]
	Text,
	#[serde(rename = "translatable")]
	Translatable,
	#[serde(rename = "keybind")]
	Keybind,
	#[serde(rename = "score")]
	Score,
	#[serde(rename = "selector")]
	Selector,
	#[serde(rename = "nbt")]
	Nbt,
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
		serde_json::to_string(self).map_err(|e| SerializingErr::UniqueFailure(format!("Failed to serialize JSON: {}", e)))?.mc_serialize(serializer)?;
		
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
		serde_json::to_string(self).map_err(|e| SerializingErr::UniqueFailure(format!("Failed to serialize JSON: {}", e)))?.mc_serialize(serializer)?;
		
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