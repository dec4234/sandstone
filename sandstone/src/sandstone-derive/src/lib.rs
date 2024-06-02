use proc_macro::TokenStream;

use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};
use syn::__private::Span;

#[proc_macro_derive(McSerialize)]
pub fn derive_mc_serialize(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let fields = match &input.data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(fields) => fields.named.iter().map(|field| {
				let field_name = field.ident.as_ref().unwrap();
				quote! {
					self.#field_name.mc_serialize(serializer)?;
				}
			}).collect(),
			Fields::Unnamed(fields) => fields.unnamed.iter().enumerate().map(|(i, field)| {
				let field_name = Ident::new(&format!("__{}", i), Span::call_site());
				quote! {
					self.#field_name.mc_serialize(serializer)?;
				}
			}).collect(),
			Fields::Unit => vec![],
		},
		Data::Enum(_) => panic!("Enums are not supported"),
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

#[proc_macro_derive(McDeserialize)]
pub fn derive_mc_deserialize(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let fields = match &input.data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(fields) => fields.named.iter().map(|field| {
				let field_name = field.ident.as_ref().unwrap();
				let field_type = &field.ty;
				quote! {
					let #field_name = <#field_type>::mc_deserialize(deserializer)?;
				}
			}).collect(),
			Fields::Unnamed(fields) => fields.unnamed.iter().enumerate().map(|(i, field)| {
				let field_name = Ident::new(&format!("__{}", i), Span::call_site());
				let field_type = &field.ty;
				quote! {
					let #field_name = <#field_type>::mc_deserialize(deserializer)?;
				}
			}).collect(),
			Fields::Unit => vec![],
		},
		Data::Enum(_) => panic!("Enums are not supported"),
		Data::Union(_) => panic!("Unions are not supported"),
	};
	let expanded = quote! {
		impl McDeserialize for #name {
			fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
				#(#fields)*
				Ok(Self {
					#(
						#fields
					)*
				})
			}
		}
	};
	TokenStream::from(expanded)
}
