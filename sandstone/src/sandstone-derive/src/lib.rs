//! Derive traits for `McSerialize` and `McDeserialize` much like `serde` has for `Serialize` and `Deserialize`.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::__private::Span;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DataEnum, DeriveInput, Expr, Field, Fields, GenericArgument, Ident, LitStr, PathArguments, Type, Variant};

/// Derive the `McSerialize` trait for a struct. This implies that all fields of the struct also
/// implement `McSerialize`.
///
/// ```rust,ignore
/// #[derive(McSerialize)]
/// struct MyStruct {
///   field1: u32,
///   field2: bool,
/// }
/// ```
#[proc_macro_derive(McSerialize)]
pub fn derive_mc_serialize(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let fields = match &input.data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(fields) => fields
				.named
				.iter()
				.map(|field| {
					let field_name = field.ident.as_ref().unwrap();
					quote! {
						self.#field_name.mc_serialize(serializer)?;
					}
				})
				.collect(),
			Fields::Unnamed(fields) => fields
				.unnamed
				.iter()
				.enumerate()
				.map(|(i, _field)| {
					let field_name = Ident::new(&format!("__{}", i), Span::call_site());
					quote! {
						self.#field_name.mc_serialize(serializer)?;
					}
				})
				.collect(),
			Fields::Unit => vec![],
		},
		Data::Enum(enu) => {
			let mut match_arms = vec![];

			for variant in enu.variants.iter() {
				let variant_name = &variant.ident;

				let discriminant = &variant
					.discriminant
					.as_ref()
					.unwrap_or_else(|| panic!("McSerialize enum requires an explicit discriminant for variant {}", variant_name))
					.1;

				let pattern = match &variant.fields {
					Fields::Named(fields) => {
						let names: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
						quote! { #name::#variant_name { #(#names),* } }
					}
					Fields::Unnamed(fields) => {
						let vars: Vec<_> = (0..fields.unnamed.len()).map(|i| Ident::new(&format!("f{}", i), Span::call_site())).collect();
						quote! { #name::#variant_name(#(#vars),*) }
					}
					Fields::Unit => {
						quote! { #name::#variant_name }
					}
				};

				let serialize_fields = match &variant.fields {
					Fields::Named(fields) => {
						let names: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
						quote! { #(#names.mc_serialize(serializer)?;)* }
					}
					Fields::Unnamed(fields) => {
						let vars: Vec<_> = (0..fields.unnamed.len()).map(|i| Ident::new(&format!("f{}", i), Span::call_site())).collect();
						quote! { #(#vars.mc_serialize(serializer)?;)* }
					}
					Fields::Unit => quote! {},
				};

				match_arms.push(quote! {
					#pattern => {
						VarInt(#discriminant).mc_serialize(serializer)?;
						#serialize_fields
					}
				});
			}

			vec![quote! {
				match self {
					#(#match_arms),*
				}
			}]
		}
		Data::Union(_) => panic!("Unions are not supported"),
	};
	let expanded = quote! {
		impl McSerialize for #name {
			fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
				#(#fields)*
				Ok(())
			}
		}
	};
	TokenStream::from(expanded)
}

/// Derive the `McDeserialize` trait for a struct. This implies that all fields of the struct also implement
/// `McDeserialize`.
///
/// This macro supports the `#[mc(deserialize_if = ...)]` attribute on fields, which allows for conditional
/// deserialization of Option<T> fields according to if another boolean field is true.
///
/// ```rust,ignore
/// #[derive(McDeserialize)]
/// struct MyStruct {
///   field1: u32,
///   field2: bool,
///   #[mc(deserialize_if = field2)]
///   field3: Option<u64>,
/// }
/// ```
#[proc_macro_derive(McDeserialize, attributes(mc))]
pub fn derive_mc_deserialize(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	// Enums deserialize by reading a leading VarInt discriminant, then deserializing the body of
	// the matching variant. This mirrors the wire format written by the McSerialize enum derive.
	if let Data::Enum(data_enum) = &input.data {
		let mut deserialize_arms = Vec::new();

		for variant in &data_enum.variants {
			let variant_ident = &variant.ident;

			let discriminant = &variant
				.discriminant
				.as_ref()
				.unwrap_or_else(|| panic!("McDeserialize enum requires an explicit discriminant for variant {}", variant_ident))
				.1;

			let construct = match &variant.fields {
				Fields::Named(fields) => {
					let names: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
					let types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();
					quote! {
						#(let #names = <#types as McDeserialize>::mc_deserialize(deserializer)?;)*
						Ok(#name::#variant_ident { #(#names),* })
					}
				}
				Fields::Unnamed(fields) => {
					let vars: Vec<_> = (0..fields.unnamed.len()).map(|i| Ident::new(&format!("f{}", i), Span::call_site())).collect();
					let types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
					quote! {
						#(let #vars = <#types as McDeserialize>::mc_deserialize(deserializer)?;)*
						Ok(#name::#variant_ident(#(#vars),*))
					}
				}
				Fields::Unit => quote! { Ok(#name::#variant_ident) },
			};

			deserialize_arms.push(quote! {
				#discriminant => { #construct }
			});
		}

		let enum_name_str = name.to_string();

		let expanded = quote! {
			impl McDeserialize for #name {
				fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
					let __id = VarInt::mc_deserialize(deserializer)?.0;
					match __id {
						#(#deserialize_arms)*
						_ => Err(SerializingErr::OutOfBounds(format!("Invalid {} id: {}", #enum_name_str, __id))),
					}
				}
			}
		};

		return TokenStream::from(expanded);
	}

	let mut init_stmts = Vec::new();
	let mut field_names = Vec::new();

	match &input.data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(fields) => {
				for field in &fields.named {
					let field_name = field.ident.as_ref().unwrap();
					let mut current_ty = &field.ty;
					// Unwrap all Type::Group layers (e.g., types wrapped in parentheses)
					// This is needed to use "mc" attributes within packets!
					while let Type::Group(group) = current_ty {
						current_ty = &group.elem;
					}

					let mut condition: Option<Expr> = None;

					// Parse attributes to find #[mc(deserialize_if = ...)]
					for attr in &field.attrs {
						if !attr.path().is_ident("mc") {
							continue;
						}

						// Parse nested meta: like deserialize_if = field1
						attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("deserialize_if") {
								let value_expr = meta.value()?;
								condition = Some(value_expr.parse()?);
								Ok(())
							} else {
								Err(meta.error("unsupported mc attribute argument"))
							}
						})
						.unwrap_or_else(|e| panic!("Error parsing mc attribute: {}", e));
					}

					if let Some(cond) = condition {
						// Validate that the field is an Option<T>
						let inner_type = match current_ty {
							Type::Path(type_path) => {
								let segments = &type_path.path.segments;
								if let Some(segment) = segments.last() {
									// Check the last segment instead of the first
									if segment.ident == "Option" {
										match &segment.arguments {
											PathArguments::AngleBracketed(args) => {
												if let Some(GenericArgument::Type(ty)) = args.args.first() {
													ty
												} else {
													panic!("Option must have an inner type for field {field_name}");
												}
											}
											_ => panic!("Option must have angle bracketed arguments for field {field_name}"),
										}
									} else {
										panic!("deserialize_if can only be applied to Option fields, but field {field_name} is {}", segment.ident);
									}
								} else {
									panic!("Invalid type path for field {field_name}");
								}
							}
							_ => panic!(
								"deserialize_if can only be applied to Option fields with a type path for field {field_name} and field type {}",
								current_ty.to_token_stream()
							),
						};

						// Conditional deserialization
						init_stmts.push(quote! {
							let #field_name = if #cond {
								Some(<#inner_type as McDeserialize>::mc_deserialize(deserializer)?)
							} else {
								None
							};
						});
					} else {
						// Regular deserialization
						init_stmts.push(quote! {
							let #field_name = <#current_ty as McDeserialize>::mc_deserialize(deserializer)?;
						});
					}

					field_names.push(quote! { #field_name });
				}
			}
			Fields::Unnamed(fields) => {
				for (i, field) in fields.unnamed.iter().enumerate() {
					let field_ident = Ident::new(&format!("__{}", i), field.span());
					let field_type = &field.ty;

					init_stmts.push(quote! {
						let #field_ident = <#field_type as McDeserialize>::mc_deserialize(deserializer)?;
					});

					field_names.push(quote! { #field_ident });
				}
			}
			Fields::Unit => {}
		},
		_ => panic!("Only structs are supported"),
	}

	let struct_expr = match &input.data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(_) => quote! {
				Self {
					#(#field_names),*
				}
			},
			Fields::Unnamed(_) => quote! {
				Self(
					#(#field_names),*
				)
			},
			Fields::Unit => quote! { Self },
		},
		_ => unreachable!(),
	};

	let expanded = quote! {
		impl McDeserialize for #name {
			fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
				#(#init_stmts)*

				Ok(#struct_expr)
			}
		}
	};

	TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn mc(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

#[proc_macro_attribute]
pub fn nbt(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

/// The NBT key for a struct field, honoring `#[nbt(rename = "...")]`, plus whether the field is
/// marked `#[nbt(flatten)]` (its nested compound is merged into the parent instead of nested
/// under a key).
fn nbt_field_opts(f: &Field) -> (String, bool) {
	let raw_key = f.ident.as_ref().unwrap().to_string();
	let mut key = raw_key.strip_prefix("r#").unwrap_or(&raw_key).to_string();
	let mut flatten = false;

	for attr in &f.attrs {
		if attr.path().is_ident("nbt") {
			let mut rename_value: Option<String> = None;
			let _ = attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("rename") {
					let value = meta.value()?;
					let lit: LitStr = value.parse()?;
					rename_value = Some(lit.value());
					Ok(())
				} else if meta.path.is_ident("flatten") {
					flatten = true;
					Ok(())
				} else {
					Err(meta.error("unsupported nbt attribute"))
				}
			});
			if let Some(v) = rename_value {
				key = v;
			}
		}
	}

	(key, flatten)
}

/// The NBT key used to hold an enum's discriminant, from `#[nbt(tag = "...")]` on the enum.
/// Defaults to `"type"`, the Minecraft convention for internally-tagged compounds.
fn nbt_container_tag(attrs: &[Attribute]) -> String {
	for attr in attrs {
		if attr.path().is_ident("nbt") {
			let mut tag_value: Option<String> = None;
			let _ = attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("tag") {
					let value = meta.value()?;
					let lit: LitStr = value.parse()?;
					tag_value = Some(lit.value());
					Ok(())
				} else {
					Err(meta.error("unsupported nbt attribute"))
				}
			});
			if let Some(v) = tag_value {
				return v;
			}
		}
	}
	"type".to_string()
}

/// The discriminant string written for an enum variant, from `#[nbt(rename = "...")]`.
/// Defaults to the variant identifier.
fn nbt_variant_tag(variant: &Variant) -> String {
	for attr in &variant.attrs {
		if attr.path().is_ident("nbt") {
			let mut rename_value: Option<String> = None;
			let _ = attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("rename") {
					let value = meta.value()?;
					let lit: LitStr = value.parse()?;
					rename_value = Some(lit.value());
					Ok(())
				} else {
					Err(meta.error("unsupported nbt attribute"))
				}
			});
			if let Some(v) = rename_value {
				return v;
			}
		}
	}
	variant.ident.to_string()
}

/// Unwrap the invisible `Group`/`Paren` delimiters that wrap a type captured by a `macro_rules!`
/// `:ty` fragment, so the underlying type can be inspected structurally.
fn unwrap_type_groups(ty: &Type) -> &Type {
	match ty {
		Type::Group(group) => unwrap_type_groups(&group.elem),
		Type::Paren(paren) => unwrap_type_groups(&paren.elem),
		other => other,
	}
}

/// Extract `T` from a field type of the form `Option<T>`, returning `None` for any other type.
fn option_inner_type(ty: &Type) -> Option<&Type> {
	if let Type::Path(type_path) = unwrap_type_groups(ty) {
		let segment = type_path.path.segments.last()?;
		if segment.ident == "Option" {
			if let PathArguments::AngleBracketed(args) = &segment.arguments {
				if let Some(GenericArgument::Type(inner)) = args.args.first() {
					return Some(inner);
				}
			}
		}
	}
	None
}

/// Returns the inner type of a `Box<T>`, if `ty` is one. The orphan rule blocks a blanket
/// `TryFrom<NbtCompound> for Box<T>`, so reading into a boxed field is generated by converting the
/// inner type and wrapping the result in `Box::new`.
fn box_inner_type(ty: &Type) -> Option<&Type> {
	if let Type::Path(type_path) = unwrap_type_groups(ty) {
		let segment = type_path.path.segments.last()?;
		if segment.ident == "Box" {
			if let PathArguments::AngleBracketed(args) = &segment.arguments {
				if let Some(GenericArgument::Type(inner)) = args.args.first() {
					return Some(inner);
				}
			}
		}
	}
	None
}

/// Convert a struct or enum to an NbtCompound using the `as_nbt` method.
///
/// Structs map each field to a compound entry (keyed by the field name or `#[nbt(rename = "...")]`);
/// a `#[nbt(flatten)]` field has its own compound merged into the parent. Enums are written as
/// internally-tagged compounds: the variant's compound plus a discriminant entry under the
/// `#[nbt(tag = "...")]` key (default `"type"`) set to the variant's `#[nbt(rename = "...")]` value.
/// Only unit and single-field newtype variants are supported.
#[proc_macro_derive(AsNbt, attributes(nbt))]
pub fn as_nbt_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident.clone();

	match &input.data {
		Data::Struct(data) => as_nbt_struct(&name, &data.fields),
		Data::Enum(data) => as_nbt_enum(&name, &input.attrs, data),
		_ => panic!("AsNbt can only be derived for structs and enums"),
	}
}

fn as_nbt_struct(name: &Ident, fields: &Fields) -> TokenStream {
	let field_additions = fields.iter().map(|f| {
		let field_ident = f.ident.as_ref().unwrap();
		let (key, flatten) = nbt_field_opts(f);

		if flatten {
			quote! {
				let __flattened: NbtCompound = self.#field_ident.clone().into();
				compound.merge(__flattened);
			}
		} else if option_inner_type(&f.ty).is_some() {
			// An `Option` field maps to key-presence: only insert the entry when it is `Some`.
			quote! {
				if let Some(__v) = &self.#field_ident {
					compound.add(#key, __v.clone());
				}
			}
		} else {
			quote! {
				compound.add(#key, self.#field_ident.clone());
			}
		}
	});

	let expanded = quote! {
		impl Into<NbtCompound> for #name {
			fn into(self) -> NbtCompound {
				self.as_nbt()
			}
		}

		impl Into<NbtTag> for #name {
			fn into(self) -> NbtTag {
				NbtTag::Compound(self.as_nbt())
			}
		}

		impl #name {
			pub fn as_nbt(&self) -> NbtCompound {
				let mut compound = NbtCompound::new_no_name();
				#(#field_additions)*
				compound
			}
		}
	};

	TokenStream::from(expanded)
}

fn as_nbt_enum(name: &Ident, attrs: &[Attribute], data: &DataEnum) -> TokenStream {
	let tag_key = nbt_container_tag(attrs);

	let arms = data.variants.iter().map(|variant| {
		let variant_ident = &variant.ident;
		let tag_value = nbt_variant_tag(variant);

		match &variant.fields {
			Fields::Unit => quote! {
				#name::#variant_ident => {
					let mut compound = NbtCompound::new_no_name();
					compound.add(#tag_key, #tag_value);
					compound
				}
			},
			Fields::Unnamed(fields) if fields.unnamed.len() == 1 => quote! {
				#name::#variant_ident(inner) => {
					let mut compound: NbtCompound = inner.clone().into();
					compound.add(#tag_key, #tag_value);
					compound
				}
			},
			_ => panic!("AsNbt enum variants must be unit or single-field newtype variants, but '{}' is not", variant_ident),
		}
	});

	let expanded = quote! {
		impl Into<NbtCompound> for #name {
			fn into(self) -> NbtCompound {
				self.as_nbt()
			}
		}

		impl Into<NbtTag> for #name {
			fn into(self) -> NbtTag {
				NbtTag::Compound(self.as_nbt())
			}
		}

		impl #name {
			pub fn as_nbt(&self) -> NbtCompound {
				match self {
					#(#arms)*
				}
			}
		}
	};

	TokenStream::from(expanded)
}

/// Convert an NbtCompound into a struct or enum using the `TryFrom` trait.
///
/// Structs read each field from its keyed entry (missing required fields error; missing optional
/// fields become `None`); a `#[nbt(flatten)]` field is built from the whole compound. Enums read
/// the discriminant string under the `#[nbt(tag = "...")]` key (default `"type"`) and dispatch to
/// the matching variant by its `#[nbt(rename = "...")]` value.
#[proc_macro_derive(FromNbt, attributes(nbt))]
pub fn from_nbt_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident.clone();

	match &input.data {
		Data::Struct(data) => from_nbt_struct(&name, &data.fields),
		Data::Enum(data) => from_nbt_enum(&name, &input.attrs, data),
		_ => panic!("FromNbt can only be derived for structs and enums"),
	}
}

fn from_nbt_struct(name: &Ident, fields: &Fields) -> TokenStream {
	let name_str = name.to_string();

	let field_initializers = fields.iter().map(|f| {
		let field_ident = f.ident.as_ref().unwrap();
		let field_ty = &f.ty;
		let (key, flatten) = nbt_field_opts(f);
		let key_str = key.clone();
		let sname = name_str.clone();

		if flatten {
			// A flattened field is rebuilt from the entire compound rather than a single key.
			quote! {
				#field_ident: <#field_ty as ::std::convert::TryFrom<NbtCompound>>::try_from(nbt.clone())?,
			}
		} else if let Some(inner_ty) = option_inner_type(field_ty) {
			// Optional field: an absent key or a present-but-wrong-type value both resolve to `None`,
			// matching the lenient `Option<T>` conversions in nbt.rs while supporting custom inner types.
			quote! {
				#field_ident: match nbt.get(#key_str) {
					Some(tag) => {
						<#inner_ty as ::std::convert::TryFrom<NbtTag>>::try_from(tag.clone()).ok()
					}
					None => None,
				},
			}
		} else {
			quote! {
				#field_ident: match nbt.get(#key_str) {
					Some(tag) => {
						<#field_ty as ::std::convert::TryFrom<NbtTag>>::try_from(tag.clone())
							.map_err(|_| NbtError::MissingField(
								format!("Invalid type for field '{}' in '{}'", #key_str, #sname)
							))?
					}
					None => {
						return Err(NbtError::MissingField(
							format!("'{}' in '{}'", #key_str, #sname)
						));
					}
				},
			}
		}
	});

	let expanded = quote! {
		impl ::std::convert::TryFrom<NbtCompound> for #name {
			type Error = NbtError;

			fn try_from(nbt: NbtCompound) -> Result<Self, Self::Error> {
				Ok(Self {
					#(#field_initializers)*
				})
			}
		}

		impl ::std::convert::TryFrom<NbtTag> for #name {
			type Error = NbtError;

			fn try_from(value: NbtTag) -> Result<Self, Self::Error> {
				match value {
					NbtTag::Compound(nbt) => Self::try_from(nbt),
					_ => Err(NbtError::InvalidType),
				}
			}
		}

		impl #name {
			pub fn from_nbt(nbt: NbtCompound) -> Result<Self, NbtError> {
				Self::try_from(nbt)
			}
		}
	};

	TokenStream::from(expanded)
}

fn from_nbt_enum(name: &Ident, attrs: &[Attribute], data: &DataEnum) -> TokenStream {
	let tag_key = nbt_container_tag(attrs);
	let name_str = name.to_string();

	let arms = data.variants.iter().map(|variant| {
		let variant_ident = &variant.ident;
		let tag_value = nbt_variant_tag(variant);

		match &variant.fields {
			Fields::Unit => quote! {
				#tag_value => Ok(#name::#variant_ident),
			},
			Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
				let inner_ty = &fields.unnamed.first().unwrap().ty;
				// `Box<T>` can't impl `TryFrom<NbtCompound>` (orphan rule), so convert the boxed
				// value's type and re-box it.
				if let Some(boxed_ty) = box_inner_type(inner_ty) {
					quote! {
						#tag_value => Ok(#name::#variant_ident(
							::std::boxed::Box::new(
								<#boxed_ty as ::std::convert::TryFrom<NbtCompound>>::try_from(nbt.clone())?
							)
						)),
					}
				} else {
					quote! {
						#tag_value => Ok(#name::#variant_ident(
							<#inner_ty as ::std::convert::TryFrom<NbtCompound>>::try_from(nbt.clone())?
						)),
					}
				}
			}
			_ => panic!("FromNbt enum variants must be unit or single-field newtype variants, but '{}' is not", variant_ident),
		}
	});

	let expanded = quote! {
		impl ::std::convert::TryFrom<NbtCompound> for #name {
			type Error = NbtError;

			fn try_from(nbt: NbtCompound) -> Result<Self, Self::Error> {
				let __tag = nbt.get_string(#tag_key).ok_or_else(|| NbtError::MissingField(
					format!("'{}' in '{}'", #tag_key, #name_str)
				))?;

				match __tag.as_str() {
					#(#arms)*
					other => Err(NbtError::MissingField(
						format!("unknown {} '{}' for '{}'", #tag_key, other, #name_str)
					)),
				}
			}
		}

		impl ::std::convert::TryFrom<NbtTag> for #name {
			type Error = NbtError;

			fn try_from(value: NbtTag) -> Result<Self, Self::Error> {
				match value {
					NbtTag::Compound(nbt) => Self::try_from(nbt),
					_ => Err(NbtError::InvalidType),
				}
			}
		}

		impl #name {
			pub fn from_nbt(nbt: NbtCompound) -> Result<Self, NbtError> {
				Self::try_from(nbt)
			}
		}
	};

	TokenStream::from(expanded)
}

/// Derive `McSerialize` and `McDeserialize` for enums where each variant is identified on the
/// wire by a leading VarInt discriminant.
///
/// Each variant must have an explicit discriminant value. Unit variants serialize to just the
/// VarInt id. Tuple (data-carrying) variants serialize the VarInt id followed by each field in
/// declaration order; deserialization reads the fields back in the same order. Struct (named
/// field) variants are not supported.
///
/// `from_varint` is only generated when every variant is a unit variant, since data-carrying
/// variants cannot be reconstructed from the discriminant alone.
///
/// ```rust,ignore
/// #[derive(VarIntEnum)]
/// pub enum ClientStatusAction {
///     PerformRespawn = 0,
///     RequestStats = 1,
/// }
/// ```
#[proc_macro_derive(VarIntEnum)]
pub fn derive_var_int_enum(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	let data_enum = match &input.data {
		Data::Enum(data) => data,
		_ => panic!("VarIntEnum can only be derived for enums"),
	};

	let mut serialize_arms = Vec::new();
	let mut deserialize_arms = Vec::new();
	let mut from_varint_arms = Vec::new();
	let mut all_unit = true;

	for variant in &data_enum.variants {
		let discriminant = variant
			.discriminant
			.as_ref()
			.unwrap_or_else(|| panic!("VarIntEnum requires explicit discriminant for variant {}", variant.ident));

		let expr = &discriminant.1;
		let variant_ident = &variant.ident;

		match &variant.fields {
			Fields::Unit => {
				serialize_arms.push(quote! {
					#name::#variant_ident => {
						VarInt(#expr).mc_serialize(serializer)?;
						Ok(())
					}
				});

				deserialize_arms.push(quote! {
					#expr => Ok(#name::#variant_ident),
				});

				from_varint_arms.push(quote! {
					#expr => Ok(#name::#variant_ident),
				});
			}
			Fields::Unnamed(fields) => {
				all_unit = false;

				let bindings: Vec<Ident> = (0..fields.unnamed.len()).map(|i| Ident::new(&format!("f{}", i), Span::call_site())).collect();
				let field_types: Vec<&Type> = fields.unnamed.iter().map(|f| &f.ty).collect();

				serialize_arms.push(quote! {
					#name::#variant_ident( #(#bindings),* ) => {
						VarInt(#expr).mc_serialize(serializer)?;
						#( #bindings.mc_serialize(serializer)?; )*
						Ok(())
					}
				});

				deserialize_arms.push(quote! {
					#expr => {
						#( let #bindings = <#field_types>::mc_deserialize(deserializer)?; )*
						Ok(#name::#variant_ident( #(#bindings),* ))
					}
				});
			}
			Fields::Named(_) => {
				panic!("VarIntEnum does not support struct (named field) variants, but {} has them", variant_ident);
			}
		}
	}

	let enum_name_str = name.to_string();

	let from_varint_impl = if all_unit {
		quote! {
			impl #name {
				pub fn from_varint(value: i32) -> SerializingResult<'static, Self> {
					match value {
						#(#from_varint_arms)*
						_ => Err(SerializingErr::OutOfBounds(format!("Invalid {} id: {}", #enum_name_str, value))),
					}
				}
			}
		}
	} else {
		quote! {}
	};

	let expanded = quote! {
		impl McSerialize for #name {
			fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
				match self {
					#(#serialize_arms)*
				}
			}
		}

		impl McDeserialize for #name {
			fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
				let id = VarInt::mc_deserialize(deserializer)?.0;
				match id {
					#(#deserialize_arms)*
					_ => Err(SerializingErr::OutOfBounds(format!("Invalid {} id: {}", #enum_name_str, id))),
				}
			}
		}

		#from_varint_impl
	};

	TokenStream::from(expanded)
}

/// Derive `McSerialize` and `McDeserialize` for unit enums where each variant maps to a
/// primitive type specified via `#[type_enum(T)]` (e.g. `u8`, `i8`, `i32`).
///
/// Each variant must be a unit variant with an explicit discriminant value.
///
/// ```rust,ignore
/// #[derive(TypeEnum)]
/// #[type_enum(u8)]
/// pub enum ModifierOperation {
///     AddSubtractAmount = 0,
///     AddSubtractPercentage = 1,
///     MultiplyPercentage = 2,
/// }
/// ```
#[proc_macro_derive(TypeEnum, attributes(type_enum))]
pub fn derive_type_enum(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	let underlying_type: syn::Type = input
		.attrs
		.iter()
		.find(|attr| attr.path().is_ident("type_enum"))
		.expect("TypeEnum requires a #[type_enum(T)] attribute")
		.parse_args()
		.expect("Expected a type inside #[type_enum(...)], e.g. #[type_enum(u8)]");

	let data_enum = match &input.data {
		Data::Enum(data) => data,
		_ => panic!("TypeEnum can only be derived for enums"),
	};

	let mut serialize_arms = Vec::new();
	let mut deserialize_arms = Vec::new();

	for variant in &data_enum.variants {
		assert!(matches!(variant.fields, Fields::Unit), "TypeEnum only supports unit variants, but {:?} has fields", variant.ident);

		let discriminant = variant.discriminant.as_ref().unwrap_or_else(|| panic!("TypeEnum requires explicit discriminant for variant {}", variant.ident));

		let expr = &discriminant.1;
		let variant_ident = &variant.ident;

		serialize_arms.push(quote! {
			#name::#variant_ident => #expr,
		});

		deserialize_arms.push(quote! {
			#expr => Ok(#name::#variant_ident),
		});
	}

	let enum_name_str = name.to_string();

	let from_value_arms = deserialize_arms.clone();

	let expanded = quote! {
		impl McSerialize for #name {
			fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
				let id: #underlying_type = match self {
					#(#serialize_arms)*
				};
				id.mc_serialize(serializer)
			}
		}

		impl McDeserialize for #name {
			fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
				let id = <#underlying_type>::mc_deserialize(deserializer)?;
				match id {
					#(#deserialize_arms)*
					_ => Err(SerializingErr::OutOfBounds(format!("Invalid {} id: {}", #enum_name_str, id))),
				}
			}
		}

		impl #name {
			pub fn from_value(value: #underlying_type) -> SerializingResult<'static, Self> {
				match value {
					#(#from_value_arms)*
					_ => Err(SerializingErr::OutOfBounds(format!("Invalid {} id: {}", #enum_name_str, value))),
				}
			}
		}
	};

	TokenStream::from(expanded)
}

/// Derive the `McDefault` trait for a struct. This trait provides a default value for the struct,
/// which can be used for automated packet testing.
#[proc_macro_derive(McDefault)]
pub fn derive_mc_default(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	let mc_default_impl = match &input.data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(fields) => {
				let default_fields = fields.named.iter().map(|field| {
					let field_name = field.ident.as_ref().unwrap();
					quote! { #field_name: McDefault::mc_default() }
				});
				quote! {
					Self {
						#(#default_fields),*
					}
				}
			}
			Fields::Unnamed(fields) => {
				let default_fields = fields.unnamed.iter().map(|_field| {
					quote! { McDefault::mc_default() }
				});
				quote! {
					Self(
						#(#default_fields),*
					)
				}
			}
			Fields::Unit => quote! { Self },
		},
		Data::Enum(data_enum) => {
			let first_variant = data_enum.variants.iter().next().expect("Enum must have at least one variant");
			let variant_ident = &first_variant.ident;

			match &first_variant.fields {
				Fields::Named(fields) => {
					let default_fields = fields.named.iter().map(|field| {
						let field_name = &field.ident;
						quote! { #field_name: McDefault::mc_default() }
					});
					quote! {
						Self::#variant_ident {
							#(#default_fields),*
						}
					}
				}
				Fields::Unnamed(fields) => {
					let default_fields = fields.unnamed.iter().map(|_field| {
						quote! { McDefault::mc_default() }
					});
					quote! {
						Self::#variant_ident(
							#(#default_fields),*
						)
					}
				}
				Fields::Unit => {
					quote! {
						Self::#variant_ident
					}
				}
			}
		}
		Data::Union(_) => {
			panic!("#[derive(McDefault)] is not supported for unions");
		}
	};

	let expanded = quote! {
		impl McDefault for #name {
			fn mc_default() -> Self {
				#mc_default_impl
			}
		}
	};

	TokenStream::from(expanded)
}
