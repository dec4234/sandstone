//! The Minecraft Java "Dialog" format, modelled as NBT-serializable types.
//!
//! See <https://minecraft.wiki/w/Dialog#Dialog_format>.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtTag};
use sandstone_derive::{AsNbt, FromNbt, McDefault};

/// The root of a dialog. Holds the fields common to every dialog type.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct Dialog {
	/// Screen title, always visible regardless of dialog type.
	pub title: TextComponent,
	/// Name for a button leading to this dialog. Falls back to `title` when absent.
	pub external_title: Option<TextComponent>,
	/// Body elements shown between the title and the actions/inputs.
	pub body: Option<DialogBody>,
	/// Input controls used to collect information from the player.
	pub inputs: Option<Vec<InputControl>>,
	/// Whether the dialog can be dismissed with Escape. Defaults to true when absent.
	pub can_close_with_escape: Option<bool>,
	/// Whether the dialog pauses the game in singleplayer. Defaults to true when absent.
	pub pause: Option<bool>,
	/// Operation performed after a click/submit: `close`, `none`, or `wait_for_response`.
	pub after_action: Option<String>,
	/// The type-specific portion of the dialog, flattened to the root level.
	#[nbt(flatten)]
	pub dialog_type: DialogType,
}

/// The dialog kind and its type-specific fields, keyed by the `type` discriminant.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
#[nbt(tag = "type")]
pub enum DialogType {
	#[nbt(rename = "minecraft:notice")]
	Notice(NoticeDialog),
	#[nbt(rename = "minecraft:confirmation")]
	Confirmation(Box<ConfirmationDialog>),
	#[nbt(rename = "minecraft:multi_action")]
	MultiAction(MultiActionDialog),
	#[nbt(rename = "minecraft:server_links")]
	ServerLinks(ServerLinksDialog),
	#[nbt(rename = "minecraft:dialog_list")]
	DialogList(DialogListDialog),
}

/// A dialog with a single action button in the footer.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct NoticeDialog {
	/// Click action. Defaults to a `gui.ok` button when absent.
	pub action: Option<ActionButton>,
}

/// A dialog with two footer buttons for a positive/negative outcome.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct ConfirmationDialog {
	/// Click action for the positive outcome.
	pub yes: ActionButton,
	/// Click action for the negative outcome (also the exit action).
	pub no: ActionButton,
}

/// A dialog with a scrollable grid of action buttons.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct MultiActionDialog {
	/// Non-empty list of click actions.
	pub actions: Vec<ActionButton>,
	/// Number of columns. Defaults to 2 when absent.
	pub columns: Option<i32>,
	/// Footer/Escape action. The footer is hidden when absent.
	pub exit_action: Option<ActionButton>,
}

/// A dialog with a scrollable grid of the server's links.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct ServerLinksDialog {
	/// Footer/Escape action. The footer is hidden when absent.
	pub exit_action: Option<ActionButton>,
	/// Number of columns. Defaults to 2 when absent.
	pub columns: Option<i32>,
	/// Width of each link button. Defaults to 150 when absent.
	pub button_width: Option<i32>,
}

/// A dialog with a scrollable grid of buttons leading to other dialogs.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct DialogListDialog {
	/// The dialogs to list (IDs, tags, or inline definitions).
	pub dialogs: DialogReferences,
	/// Footer/Escape action. The footer is hidden when absent.
	pub exit_action: Option<ActionButton>,
	/// Number of columns. Defaults to 2 when absent.
	pub columns: Option<i32>,
	/// Width of each button. Defaults to 150 when absent.
	pub button_width: Option<i32>,
}

/// A button with a label, optional tooltip, and an optional action to run when clicked.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct ActionButton {
	/// Text shown on the button.
	pub label: TextComponent,
	/// Tooltip shown when the button is highlighted/hovered.
	pub tooltip: Option<TextComponent>,
	/// Button width between 1 and 1024. Defaults to 150 when absent.
	pub width: Option<i32>,
	/// Action performed when the button is clicked.
	pub action: Option<Action>,
}

/// A click action, keyed by the `type` discriminant. Covers both the static action types (which
/// mirror text-component click events) and the dynamic ones that build their event from input
/// control values.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
#[nbt(tag = "type")]
pub enum Action {
	#[nbt(rename = "open_url")]
	OpenUrl(OpenUrlAction),
	#[nbt(rename = "run_command")]
	RunCommand(RunCommandAction),
	#[nbt(rename = "suggest_command")]
	SuggestCommand(SuggestCommandAction),
	#[nbt(rename = "change_page")]
	ChangePage(ChangePageAction),
	#[nbt(rename = "copy_to_clipboard")]
	CopyToClipboard(CopyToClipboardAction),
	#[nbt(rename = "show_dialog")]
	ShowDialog(ShowDialogAction),
	#[nbt(rename = "custom")]
	Custom(CustomAction),
	#[nbt(rename = "dynamic/run_command")]
	DynamicRunCommand(DynamicRunCommandAction),
	#[nbt(rename = "dynamic/custom")]
	DynamicCustom(DynamicCustomAction),
}

/// Open a URL in the player's browser.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct OpenUrlAction {
	pub url: String,
}

/// Run a command as if typed in chat.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct RunCommandAction {
	pub command: String,
}

/// Fill the chat box with the given text/command.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct SuggestCommandAction {
	pub command: String,
}

/// Change to a page in a written book.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct ChangePageAction {
	pub page: i32,
}

/// Copy text to the clipboard.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct CopyToClipboardAction {
	pub value: String,
}

/// Open another dialog.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct ShowDialogAction {
	/// The dialog to show: an ID or an inline definition.
	pub dialog: DialogReference,
}

/// Send a custom event to the server (no effect on vanilla servers).
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct CustomAction {
	pub id: String,
	pub payload: Option<String>,
}

/// Build a `run_command` event from a macro template substituted with input values.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct DynamicRunCommandAction {
	pub template: String,
}

/// Build a `minecraft:custom` event from all input values, plus optional static additions.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct DynamicCustomAction {
	/// Static fields added to the payload.
	pub additions: Option<NbtCompound>,
	/// Namespaced ID of the event.
	pub id: String,
}

/// An input control, keyed by the `type` discriminant. Each variant carries the common `key` +
/// `label` plus its type-specific fields.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
#[nbt(tag = "type")]
pub enum InputControl {
	#[nbt(rename = "minecraft:text")]
	Text(TextInput),
	#[nbt(rename = "minecraft:boolean")]
	Boolean(BooleanInput),
	#[nbt(rename = "minecraft:single_option")]
	SingleOption(SingleOptionInput),
	#[nbt(rename = "minecraft:number_range")]
	NumberRange(NumberRangeInput),
}

/// A single-line (or, with `multiline`, multi-line) text input.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct TextInput {
	/// Identifier used when submitting this input's value.
	pub key: String,
	/// Label shown to the left of the input.
	pub label: TextComponent,
	/// Input width between 1 and 1024. Defaults to 200 when absent.
	pub width: Option<i32>,
	/// Whether the label is visible. Defaults to true when absent.
	pub label_visible: Option<bool>,
	/// Initial value of the input.
	pub initial: Option<String>,
	/// Maximum input length. Defaults to 32 when absent.
	pub max_length: Option<i32>,
	/// When present, enables multi-line input.
	pub multiline: Option<TextInputMultiline>,
}

/// Multi-line options for a [`TextInput`].
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct TextInputMultiline {
	/// Maximum number of lines, if limited.
	pub max_lines: Option<i32>,
	/// Height of the input between 1 and 512.
	pub height: Option<i32>,
}

/// A checkbox input.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct BooleanInput {
	/// Identifier used when submitting this input's value.
	pub key: String,
	/// Label shown to the left of the input.
	pub label: TextComponent,
	/// Initial checked state. Defaults to false when absent.
	pub initial: Option<bool>,
	/// String value sent when checked. Defaults to "true".
	pub on_true: Option<String>,
	/// String value sent when unchecked. Defaults to "false".
	pub on_false: Option<String>,
}

/// A preset option selection.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct SingleOptionInput {
	/// Identifier used when submitting this input's value.
	pub key: String,
	/// Label shown to the left of the input.
	pub label: TextComponent,
	/// Whether the label is visible. Defaults to true when absent.
	pub label_visible: Option<bool>,
	/// Input width between 1 and 1024. Defaults to 200 when absent.
	pub width: Option<i32>,
	/// Non-empty list of selectable options.
	pub options: Vec<SingleOption>,
}

/// One option of a [`SingleOptionInput`].
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct SingleOption {
	/// Value sent on submit.
	pub id: String,
	/// Text shown for the option. Falls back to `id` when absent.
	pub display: Option<TextComponent>,
	/// Whether this is the initially selected option. At most one option may set this.
	pub initial: Option<bool>,
}

/// A numeric slider input.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct NumberRangeInput {
	/// Identifier used when submitting this input's value.
	pub key: String,
	/// Label shown to the left of the input.
	pub label: TextComponent,
	/// Translation key used to build the label. Defaults to `options.generic_value`.
	pub label_format: Option<String>,
	/// Input width between 1 and 1024. Defaults to 200 when absent.
	pub width: Option<i32>,
	/// Minimum value of the slider.
	pub start: f32,
	/// Maximum value of the slider.
	pub end: f32,
	/// Step size. Any value in range is allowed when absent.
	pub step: Option<f32>,
	/// Initial value. Defaults to the middle of the range when absent.
	pub initial: Option<f32>,
}

/// A body element, keyed by the `type` discriminant.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
#[nbt(tag = "type")]
pub enum BodyElement {
	#[nbt(rename = "minecraft:plain_message")]
	PlainMessage(PlainMessageBody),
	#[nbt(rename = "minecraft:item")]
	Item(ItemBody),
}

/// A multiline text label.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct PlainMessageBody {
	/// The message text.
	pub contents: TextComponent,
	/// Maximum width between 1 and 1024. Defaults to 200 when absent.
	pub width: Option<i32>,
}

/// An item with an optional description.
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct ItemBody {
	/// The item stack to display.
	pub item: DialogItemStack,
	/// Description shown next to the item.
	pub description: Option<TextComponent>,
	/// Whether count/damage bar are rendered. Defaults to true when absent.
	pub show_decoration: Option<bool>,
	/// Whether the item tooltip shows on hover. Defaults to true when absent.
	pub show_tooltip: Option<bool>,
	/// Horizontal size between 1 and 256. Defaults to 16 when absent.
	pub width: Option<i32>,
	/// Vertical size between 1 and 256. Defaults to 16 when absent.
	pub height: Option<i32>,
}

/// An item stack as embedded in an [`ItemBody`].
#[derive(McDefault, Debug, Clone, PartialEq, AsNbt, FromNbt)]
pub struct DialogItemStack {
	/// Item identifier.
	pub id: String,
	/// Item count.
	pub count: Option<i32>,
	/// Additional item components.
	pub components: Option<NbtCompound>,
}

/// A dialog's `body`: either a single element or a list of elements.
#[derive(McDefault, Debug, Clone, PartialEq)]
pub enum DialogBody {
	Single(Box<BodyElement>),
	Multiple(Vec<BodyElement>),
}

impl From<DialogBody> for NbtTag {
	fn from(body: DialogBody) -> Self {
		match body {
			DialogBody::Single(element) => (*element).into(),
			DialogBody::Multiple(elements) => NbtTag::from(elements),
		}
	}
}

impl TryFrom<NbtTag> for DialogBody {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
		match tag {
			NbtTag::List(_) => Ok(DialogBody::Multiple(Vec::try_from(tag)?)),
			NbtTag::Compound(_) => Ok(DialogBody::Single(Box::from(BodyElement::try_from(tag)?))),
			_ => Err(NbtError::InvalidType),
		}
	}
}

/// A reference to a single dialog in `show_dialog` / `dialog_list`: either an ID (or `#tag`) string,
/// or an inline dialog definition.
#[derive(McDefault, Debug, Clone, PartialEq)]
pub enum DialogReference {
	Id(String),
	Inline(Box<Dialog>),
}

impl From<DialogReference> for NbtTag {
	fn from(reference: DialogReference) -> Self {
		match reference {
			DialogReference::Id(id) => NbtTag::String(id),
			DialogReference::Inline(dialog) => NbtTag::Compound(dialog.as_nbt()),
		}
	}
}

impl TryFrom<NbtTag> for DialogReference {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
		match tag {
			NbtTag::String(id) => Ok(DialogReference::Id(id)),
			NbtTag::Compound(compound) => Ok(DialogReference::Inline(Box::new(Dialog::try_from(compound)?))),
			_ => Err(NbtError::InvalidType),
		}
	}
}

/// The `dialogs` field of a `dialog_list`: a single reference or a list of references.
#[derive(McDefault, Debug, Clone, PartialEq)]
pub enum DialogReferences {
	Single(DialogReference),
	Multiple(Vec<DialogReference>),
}

impl From<DialogReferences> for NbtTag {
	fn from(references: DialogReferences) -> Self {
		match references {
			DialogReferences::Single(reference) => reference.into(),
			DialogReferences::Multiple(references) => NbtTag::from(references),
		}
	}
}

impl TryFrom<NbtTag> for DialogReferences {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
		match tag {
			NbtTag::List(_) => Ok(DialogReferences::Multiple(Vec::try_from(tag)?)),
			other => Ok(DialogReferences::Single(DialogReference::try_from(other)?)),
		}
	}
}

impl McSerialize for Dialog {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.as_nbt().mc_serialize(serializer)
	}
}

impl McDeserialize for Dialog {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let nbt = NbtCompound::mc_deserialize(deserializer)?;
		Dialog::try_from(nbt).map_err(|e| SerializingErr::DeserializationError(format!("Failed to deserialize dialog: {}", e)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtTag};

	fn button(label: &str) -> ActionButton {
		ActionButton {
			label: TextComponent::new(label),
			tooltip: None,
			width: None,
			action: Some(Action::RunCommand(RunCommandAction {
				command: "say hi".to_string(),
			})),
		}
	}

	/// Build a dialog with each common field populated plus the given type, round-trip it through
	/// NBT, and assert it is unchanged. The `type` discriminant and every nested field must survive,
	/// since the client relies on them to know which dialog to render and how.
	fn assert_nbt_round_trip(dialog_type: DialogType) {
		let dialog = Dialog {
			title: TextComponent::new("Title"),
			external_title: Some(TextComponent::new("External")),
			body: Some(DialogBody::Multiple(vec![
				BodyElement::PlainMessage(PlainMessageBody {
					contents: TextComponent::new("Hello"),
					width: Some(200),
				}),
				BodyElement::Item(ItemBody {
					item: DialogItemStack {
						id: "minecraft:stone".to_string(),
						count: Some(1),
						components: None,
					},
					description: Some(TextComponent::new("A stone")),
					show_decoration: Some(true),
					show_tooltip: Some(false),
					width: Some(16),
					height: Some(16),
				}),
			])),
			inputs: Some(vec![
				InputControl::Text(TextInput {
					key: "msg".to_string(),
					label: TextComponent::new("Message"),
					width: Some(200),
					label_visible: Some(true),
					initial: Some("hi".to_string()),
					max_length: Some(64),
					multiline: Some(TextInputMultiline {
						max_lines: Some(3),
						height: Some(64),
					}),
				}),
				InputControl::Boolean(BooleanInput {
					key: "flag".to_string(),
					label: TextComponent::new("Flag"),
					initial: Some(false),
					on_true: Some("yes".to_string()),
					on_false: Some("no".to_string()),
				}),
				InputControl::SingleOption(SingleOptionInput {
					key: "choice".to_string(),
					label: TextComponent::new("Choice"),
					label_visible: Some(true),
					width: Some(200),
					options: vec![SingleOption {
						id: "a".to_string(),
						display: Some(TextComponent::new("A")),
						initial: Some(true),
					}],
				}),
				InputControl::NumberRange(NumberRangeInput {
					key: "amount".to_string(),
					label: TextComponent::new("Amount"),
					label_format: Some("options.generic_value".to_string()),
					width: Some(200),
					start: 0.0,
					end: 10.0,
					step: Some(1.0),
					initial: Some(5.0),
				}),
			]),
			can_close_with_escape: Some(true),
			pause: Some(false),
			after_action: Some("close".to_string()),
			dialog_type,
		};

		let nbt: NbtCompound = dialog.clone().into();
		let restored = Dialog::try_from(nbt).expect("dialog should deserialize from NBT");
		assert_eq!(dialog, restored, "dialog did not survive NBT round trip");
	}

	#[test]
	fn notice_round_trips() {
		assert_nbt_round_trip(DialogType::Notice(NoticeDialog {
			action: Some(button("OK")),
		}));
	}

	#[test]
	fn confirmation_round_trips() {
		assert_nbt_round_trip(DialogType::Confirmation(Box::new(ConfirmationDialog {
			yes: button("Yes"),
			no: button("No"),
		})));
	}

	#[test]
	fn multi_action_round_trips() {
		assert_nbt_round_trip(DialogType::MultiAction(MultiActionDialog {
			actions: vec![button("One"), button("Two")],
			columns: Some(2),
			exit_action: Some(button("Exit")),
		}));
	}

	#[test]
	fn server_links_round_trips() {
		assert_nbt_round_trip(DialogType::ServerLinks(ServerLinksDialog {
			exit_action: Some(button("Exit")),
			columns: Some(2),
			button_width: Some(150),
		}));
	}

	#[test]
	fn dialog_list_round_trips() {
		assert_nbt_round_trip(DialogType::DialogList(DialogListDialog {
			dialogs: DialogReferences::Multiple(vec![DialogReference::Id("custom:other".to_string()), DialogReference::Id("#custom:tag".to_string())]),
			exit_action: Some(button("Exit")),
			columns: Some(2),
			button_width: Some(150),
		}));
	}

	/// The `type` discriminant must actually be written to NBT under the `type` key — without it a
	/// client cannot tell a notice from a confirmation.
	#[test]
	fn dialog_type_discriminant_is_written() {
		let dialog = Dialog {
			title: TextComponent::new("Title"),
			external_title: None,
			body: None,
			inputs: None,
			can_close_with_escape: None,
			pause: None,
			after_action: None,
			dialog_type: DialogType::Notice(NoticeDialog {
				action: None,
			}),
		};
		let nbt: NbtCompound = dialog.into();
		assert_eq!(nbt["type"], NbtTag::String("minecraft:notice".to_string()));
		assert_eq!(nbt["title"], NbtTag::String("Title".to_string()));
	}

	/// A dynamic action carries a `dynamic/...` discriminant which must round-trip exactly, since the
	/// server dispatches on it.
	#[test]
	fn dynamic_action_round_trips() {
		let action = Action::DynamicCustom(DynamicCustomAction {
			additions: None,
			id: "custom:event".to_string(),
		});
		let nbt: NbtTag = action.clone().into();
		assert_eq!(Action::try_from(nbt).unwrap(), action);
	}

	/// The dialog must travel through `McSerializer` as NBT and come back unchanged, since that is
	/// how it is actually sent on the network.
	#[test]
	fn dialog_survives_wire_round_trip() {
		let dialog = Dialog {
			title: TextComponent::new("Wire"),
			external_title: None,
			body: Some(DialogBody::Single(Box::from(BodyElement::PlainMessage(PlainMessageBody {
				contents: TextComponent::new("Body"),
				width: None,
			})))),
			inputs: None,
			can_close_with_escape: Some(true),
			pause: Some(true),
			after_action: None,
			dialog_type: DialogType::Notice(NoticeDialog {
				action: Some(button("OK")),
			}),
		};

		let mut serializer = McSerializer::new();
		dialog.mc_serialize(&mut serializer).expect("serialize dialog");
		let mut deserializer = McDeserializer::new(&serializer.output);
		let restored = Dialog::mc_deserialize(&mut deserializer).expect("deserialize dialog");
		assert_eq!(dialog, restored, "dialog did not survive wire round trip");
	}
}
