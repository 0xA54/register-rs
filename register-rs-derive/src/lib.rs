use std::ops::RangeBounds;

use deluxe::extract_attributes;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, ToTokens, format_ident};
use syn::{DeriveInput, Lit, RangeLimits, token::Paren, ImplGenerics, TypeGenerics, WhereClause, Generics};

use deluxe::ParseMetaItem;

/// You know when you shove all your mess in a cupboard when someone says their coming over?
/// Thats what this is...
mod background;
use background::*;

mod reciprocal_enum;
use reciprocal_enum::*;

/// [`ReadableRegister`] Implementation
fn readable_register_impl(meta: StructMetadata, field_attrs: Vec<ParsedRegisterFieldAttribute>) -> TokenStream {
    let (struct_ident, _address, length, word_type, endian, ast_generics, _, read_fn) = meta;
    let ( impl_generics, type_generics, where_clause ) = ast_generics.split_for_impl();

    let readable_register_attrs: Vec<TokenStream> = field_attrs.iter().map(|ParsedRegisterFieldAttribute { ident, bits, ..} | {
        let (word_idx, bits) = relocate_bits(endian, *bits, 8, length); // TODO: word_size
        let bit_token = unpack_bits_read(bits);
        
        quote!{
            #ident: buffer[#word_idx].#bit_token.try_into().unwrap() // TODO: Infaliable error type be problematic for throwing error upstream :(
        }
    }).collect();

    // Output
    if let Some(override_fn) = read_fn {
        quote! {
            impl #impl_generics ReadableRegister<#word_type> for #struct_ident #type_generics #where_clause {
                fn from_bytes(buffer: &[#word_type; Self::LENGTH]) -> RegisterResult<Self> {
                    #override_fn
                }
            }
        }
    } else {
        quote!{
            impl #impl_generics ReadableRegister<#word_type> for #struct_ident #type_generics #where_clause {
                fn from_bytes(buffer: &[#word_type; Self::LENGTH]) -> RegisterResult<Self> {
                    Ok(Self {
                        #(#readable_register_attrs),*
                    })
                }
            }
        }
    }
}

/// [`WriteableRegister`] Implementation
fn writeable_register_impl(meta: StructMetadata, field_attrs: Vec<ParsedRegisterFieldAttribute>) -> TokenStream {
    let (struct_ident, _address, length, word_type, endian, ast_generics, write_fn, read_fn) = meta;
    let ( impl_generics, type_generics, where_clause ) = ast_generics.split_for_impl();

    let writeable_register_attrs: Vec<TokenStream> = field_attrs.iter().map(| ParsedRegisterFieldAttribute { ident, bits, .. } | {
        let (word_idx, bits) = relocate_bits(endian, *bits, 8, length); // TODO: word_size
        let bit_token = unpack_bits_set(bits, &ident);
        
        quote! {
            buffer[#word_idx].#bit_token;
        }
    }).collect();

    // Output
    if let Some(override_fn) = read_fn {
        quote! {
            impl #impl_generics WriteableRegister<#word_type> for #struct_ident #type_generics #where_clause {
                fn into_bytes(&self) -> RegisterResult<[#word_type; Self::LENGTH]> {
                    #override_fn
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics WriteableRegister<#word_type> for #struct_ident #type_generics #where_clause {
                fn into_bytes(&self) -> RegisterResult<[#word_type; Self::LENGTH]> {
                    let mut buffer: [#word_type; Self::LENGTH] = [0; Self::LENGTH];

                    #(#writeable_register_attrs)*

                    Ok(buffer)
                }
            }
        }
    }
}

/// [`Register`] Implementation
fn register_impl(meta: StructMetadata, field_attrs: Vec<ParsedRegisterFieldAttribute>) -> TokenStream {
    let (struct_ident, address, length, word_type, _endian, ast_generics, _, _) = meta;
    let ( impl_generics, type_generics, where_clause ) = ast_generics.split_for_impl();

    let reset_values: Vec<TokenStream> = field_attrs.iter().map(| ParsedRegisterFieldAttribute { ident, reset, .. } | {
        quote!{
            #ident: #reset.try_into().unwrap() // These values were user provided, thats on you
        }
    }).collect();

    quote!{
        impl #impl_generics Register<#word_type> for #struct_ident #type_generics #where_clause {
            const ADDRESS: #word_type = #address;
            const LENGTH: usize = #length;

            fn reset_value() -> Self {
                Self {
                    #(#reset_values),*
                }
            }
        }
    }
}


type FnImpl = fn(StructMetadata, Vec<ParsedRegisterFieldAttribute>) -> TokenStream;
/// Little helper to take care of repetitious `to_compile_error`
fn wrapped_macro_processing(item: proc_macro::TokenStream, with: FnImpl) -> proc_macro::TokenStream {
    match common_macro_processing(item.into()) {
        Ok((meta, field_attrs)) => {
            with(meta, field_attrs).into()
        }
        Err(err) => err.to_compile_error().into()
    }
}

#[proc_macro_derive(Register, attributes(register))]
pub fn register_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    wrapped_macro_processing(item, register_impl)
}

#[proc_macro_derive(ReadableRegister, attributes(register))]
pub fn readable_register_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    wrapped_macro_processing(item, readable_register_impl)
}

#[proc_macro_derive(WriteableRegister, attributes(register))]
pub fn writeable_register_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    wrapped_macro_processing(item, writeable_register_impl)
}

#[proc_macro_derive(Valued, attributes(valued))]
pub fn reciprocal_enum_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match reciprocal_enum_impl(item.into(), false) {
        Ok(item) => item.into(),
        Err(err) => err.to_compile_error().into()
    }
}

#[proc_macro_derive(TryValued, attributes(valued))]
pub fn try_reciprocal_enum_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match reciprocal_enum_impl(item.into(), true) {
        Ok(item) => item.into(),
        Err(err) => err.to_compile_error().into()
    }
}