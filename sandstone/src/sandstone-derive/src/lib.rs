use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::__private::Span;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Fields, GenericArgument, Ident, PathArguments, Type,
};

/// Derive the `McSerialize` trait for a struct. This implies that all fields of the struct also
/// implement `McSerialize`.
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

            for (index, variant) in enu.variants.iter().enumerate() {
                let variant_name = &variant.ident;
                let index_lit = index as u8;

                let pattern = match &variant.fields {
                    Fields::Named(fields) => {
                        let names: Vec<_> = fields
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();
                        quote! { #name::#variant_name { #(#names),* } }
                    }
                    Fields::Unnamed(fields) => {
                        let vars: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| Ident::new(&format!("f{}", i), Span::call_site()))
                            .collect();
                        quote! { #name::#variant_name(#(#vars),*) }
                    }
                    Fields::Unit => {
                        quote! { #name::#variant_name }
                    }
                };

                let serialize_fields = match &variant.fields {
                    Fields::Named(fields) => {
                        let names: Vec<_> = fields
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();
                        quote! { #(#names.mc_serialize(serializer)?;)* }
                    }
                    Fields::Unnamed(fields) => {
                        let vars: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| Ident::new(&format!("f{}", i), Span::call_site()))
                            .collect();
                        quote! { #(#vars.mc_serialize(serializer)?;)* }
                    }
                    Fields::Unit => quote! {},
                };

                match_arms.push(quote! {
                    #pattern => {
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
								if let Some(segment) = segments.last() { // Check the last segment instead of the first
									if segment.ident == "Option" {
										match &segment.arguments {
											PathArguments::AngleBracketed(args) => {
												if let Some(GenericArgument::Type(ty)) = args.args.first() {
													ty
												} else {
													panic!("Option must have an inner type for field {}", field_name);
												}
											}
											_ => panic!("Option must have angle bracketed arguments for field {}", field_name),
										}
									} else {
										panic!("deserialize_if can only be applied to Option fields, but field {} is {}", field_name, segment.ident);
									}
								} else {
									panic!("Invalid type path for field {}", field_name);
								}
							}
							_ => panic!("deserialize_if can only be applied to Option fields with a type path for field {} and field type {}", field_name, current_ty.to_token_stream()),
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
