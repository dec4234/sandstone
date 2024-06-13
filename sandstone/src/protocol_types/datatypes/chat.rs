use serde::{Deserialize, Serialize};

use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::serialization::serializer_error::SerializingErr;

/*
This file defines the TextComponent type in the Minecraft network API.
Seen in books, disconnect messages, chat messages, action bar, etc.
 */

// https://wiki.vg/Text_formatting#Text_components
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TextComponent {
	pub text: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "type")]
	pub typ: Option<String>,
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
}

impl McSerialize for TextComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		serde_json::to_string(self).map_err(|e| SerializingErr::UniqueFailure(format!("Failed to serialize JSON: {}", e)))?.mc_serialize(serializer)?;
		
		Ok(())
	}
}

impl McDeserialize for TextComponent {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> Result<Self, SerializingErr> {
		let json = String::mc_deserialize(deserializer)?;
		let deserialized = serde_json::from_str(&json).map_err(|e| SerializingErr::UniqueFailure(format!("Failed to deserialize JSON: {}", e)))?;
		
		Ok(deserialized)
	}
}

impl From<String> for TextComponent {
	fn from(s: String) -> Self {
		Self::new(s)
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