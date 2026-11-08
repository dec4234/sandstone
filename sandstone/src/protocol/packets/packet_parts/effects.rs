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
#[cfg(test)]
mod test {
	use super::*;

	fn ser<T: McSerialize>(v: &T) -> Vec<u8> {
		let mut s = McSerializer::new();
		v.mc_serialize(&mut s).unwrap();
		s.as_bytes().to_vec()
	}

	fn round_trip<T: McSerialize + McDeserialize + PartialEq + std::fmt::Debug>(v: T) {
		let bytes = ser(&v);
		let mut d = McDeserializer::new(&bytes);
		assert_eq!(T::mc_deserialize(&mut d).unwrap(), v);
		assert!(d.is_at_end(), "all bytes must be used");
	}

	#[test]
	fn potion_detail_field_order_matches_wiki() {
		// Wiki (Potion Effect): Amplifier, Duration, Ambient, Show Particles, Show Icon, Hidden Effect.
		assert_eq!(ser(&PotionEffectDetail::mc_default()), vec![1, 100, 0, 1, 1, 0]);
	}

	#[test]
	fn nested_hidden_effects_round_trip() {
		// A hidden effect is the effect that comes back when the current one ends. It can nest.
		let inner = PotionEffectDetail { amplifier: VarInt(0), duration: VarInt(-1), ambient: true, show_particles: false, show_icon: false, hidden_effect: PrefixedOptional::new(None) };
		let middle = PotionEffectDetail { hidden_effect: PrefixedOptional::new(Some(Box::new(inner))), ..PotionEffectDetail::mc_default() };
		let outer = PotionEffectDetail { amplifier: VarInt(4), hidden_effect: PrefixedOptional::new(Some(Box::new(middle))), ..PotionEffectDetail::mc_default() };
		round_trip(PotionEffect { type_id: VarInt(10), detail: outer });
	}

	#[test]
	fn consume_effect_ids_match_wiki() {
		assert_eq!(ser(&ConsumeEffect::ClearAllEffects), vec![2]);
		let mut teleport = vec![3];
		teleport.extend_from_slice(&16.0f32.to_be_bytes());
		assert_eq!(ser(&ConsumeEffect::TeleportRandomly(16.0)), teleport);
		assert_eq!(ser(&ConsumeEffect::PlaySound(IDorX::Registry(VarInt(0)))), vec![4, 1]);
	}

	#[test]
	fn consume_effects_round_trip() {
		round_trip(ConsumeEffect::ApplyEffects(PrefixedArray::new(vec![PotionEffect::mc_default()]), 0.5));
		round_trip(ConsumeEffect::RemoveEffects(IDSet { typ: VarInt(3), tag_name: None, ids: Some(vec![VarInt(1), VarInt(2)]) }));
		round_trip(ConsumeEffect::RemoveEffects(IDSet { typ: VarInt(0), tag_name: Some("minecraft:harmful".into()), ids: None }));
		round_trip(ConsumeEffect::ClearAllEffects);
		round_trip(ConsumeEffect::TeleportRandomly(8.0));
		round_trip(ConsumeEffect::PlaySound(IDorX::Inline(SoundEvent { name: "minecraft:entity.player.burp".into(), has_fixed_range: false, fixed_range: None })));
	}

	#[test]
	fn firework_colors_are_ints() {
		// Wiki (Firework Explosion): colors are Prefixed Arrays of Int (4 bytes each), not VarInt.
		let f = FireworkExplosion { shape: VarInt(1), colors: PrefixedArray::new(vec![0xFF0000]), fade_colors: PrefixedArray::new(vec![]), has_trail: true, has_twinkle: false };
		assert_eq!(ser(&f), vec![1, 1, 0x00, 0xFF, 0x00, 0x00, 0, 1, 0]);
		round_trip(f);
	}
}
