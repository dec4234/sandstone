use crate::protocol::game::effects::sound::SoundEvent;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::{IDSet, IDorX};
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct FireworkExplosion {
	pub shape: VarInt,
	pub colors: PrefixedArray<i32>,
	pub fade_colors: PrefixedArray<i32>,
	pub has_trail: bool,
	pub has_twinkle: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PotionEffectDetail {
	pub amplifier: VarInt,
	pub duration: VarInt,
	pub ambient: bool,
	pub show_particles: bool,
	pub show_icon: bool,
	pub hidden_effect: PrefixedOptional<Box<PotionEffectDetail>>,
}

impl McSerialize for PotionEffectDetail {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.amplifier.mc_serialize(serializer)?;
		self.duration.mc_serialize(serializer)?;
		self.ambient.mc_serialize(serializer)?;
		self.show_particles.mc_serialize(serializer)?;
		self.show_icon.mc_serialize(serializer)?;
		self.hidden_effect.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for PotionEffectDetail {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		Ok(Self {
			amplifier: VarInt::mc_deserialize(deserializer)?,
			duration: VarInt::mc_deserialize(deserializer)?,
			ambient: bool::mc_deserialize(deserializer)?,
			show_particles: bool::mc_deserialize(deserializer)?,
			show_icon: bool::mc_deserialize(deserializer)?,
			hidden_effect: PrefixedOptional::mc_deserialize(deserializer)?,
		})
	}
}

impl McDefault for PotionEffectDetail {
	fn mc_default() -> Self {
		Self {
			amplifier: VarInt(1),
			duration: VarInt(100),
			ambient: false,
			show_particles: true,
			show_icon: true,
			hidden_effect: PrefixedOptional::new(None),
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PotionEffect {
	pub type_id: VarInt,
	pub detail: PotionEffectDetail,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum ConsumeEffect {
	ApplyEffects(PrefixedArray<PotionEffect>, f32) = 0,
	RemoveEffects(IDSet) = 1,
	ClearAllEffects = 2,
	TeleportRandomly(f32) = 3,
	PlaySound(IDorX<SoundEvent>) = 4,
}