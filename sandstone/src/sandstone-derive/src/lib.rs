use proc_macro::TokenStream;

use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};
use syn::__private::Span;
use syn::spanned::Spanned;

/// Derive the `McSerialize` trait for a struct. This implies that all fields of the struct also
/// implement `McSerialize`.
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
			Fields::Unnamed(fields) => fields.unnamed.iter().enumerate().map(|(i, _field)| {
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

#[proc_macro_derive(McDeserialize, attributes(mc))]
pub fn derive_mc_deserialize(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	let mut init_stmts = Vec::new();
	let mut field_names = Vec::new();

	match &input.data {
		Data::Struct(data) => match &data.fields {
			Fields::Named(fields) => {
				for field in &fields.named {
					let field_name = field.ident.as_ref().unwrap();
					let field_type = &field.ty;

					init_stmts.push(quote! {
						let #field_name = <#field_type>::mc_deserialize(deserializer)?;
					});

					field_names.push(quote! { #field_name });
				}
			}
			Fields::Unnamed(fields) => {
				for (i, field) in fields.unnamed.iter().enumerate() {
					let field_ident = Ident::new(&format!("__{}", i), field.span());
					let field_type = &field.ty;

					init_stmts.push(quote! {
						let #field_ident = <#field_type>::mc_deserialize(deserializer)?;
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
