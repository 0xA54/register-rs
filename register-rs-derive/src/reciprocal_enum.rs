use proc_macro2::{TokenStream, Ident, Span};
use quote::{quote, format_ident};
use syn::{DeriveInput, Lit};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(valued))]
struct ReciprocalEnum {
    #[deluxe(default = quote!{u8})]
    r#type: TokenStream,
    #[deluxe(default = None)]
    default: Option<TokenStream>
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(valued))]
// struct ReciprocalField {
//     value: Lit
// }
struct ReciprocalField(TokenStream);

pub fn reciprocal_enum_try_impl(item: TokenStream) -> deluxe::Result<TokenStream> {



    todo!()
}

pub fn reciprocal_enum_impl(item: TokenStream, try_implementation: bool) -> deluxe::Result<TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;
    let meta: ReciprocalEnum = deluxe::extract_attributes(&mut ast)?;
    let ident = ast.ident.clone();

    let values = if let syn::Data::Enum(e) = &mut ast.data {
        let values: Result<Vec<Option<(Ident, TokenStream)>>, syn::Error> = e.variants.iter_mut().map(|variant| {
            let ident = variant.ident.clone();
            let attr: ReciprocalField = deluxe::extract_attributes(variant)?;

            if attr.0.to_string() == "None" {
                // How to skip
                Ok(None)
            } else {
                Ok(Some((ident, attr.0)))
            }
        }).collect();

        Ok(values?)
    } else {
        Err(syn::Error::new_spanned(&ast, "derive(Reciprocal) is only supported on type `enum`"))
    }?;

    let d_type = meta.r#type; //format_ident!("{}", meta.r#type);
    let (
        default_from, 
        default_into) = if let Some(invalid_value) = meta.default {
        (
            quote!{ _ => #invalid_value },
            quote!{}, 
            //quote!{ _ => #invalid_value.into() }
        )
    } else { // No default, assume all branches were taken care of
        (
            quote!{}, 
            quote!{}
        )
    };

    match try_implementation {
        true => {
            let from_matches: Vec<TokenStream> = values.iter().filter(|&p| p.is_some()).map(|a | {
                let (ident, value) = a.as_ref().unwrap();
        
                quote! {
                    #value => Ok(Self::#ident),
                }
            }).collect();
        
            let into_matches: Vec<TokenStream> = values.iter().filter(|&p| p.is_some()).map(|a| {
                let (ident, value) = a.as_ref().unwrap();
                quote! {
                    Self::#ident => Ok(#value),
                }
            }).collect();

            let try_from_impl = quote! {
                impl TryFrom<#d_type> for #ident {
                    type Error = RegisterError;
        
                    fn try_from(value: #d_type) -> Result<Self, Self::Error> {
                        match value {
                            #(#from_matches)*
                            _ => Err(RegisterError::ConversionError)
                        }
                    }
                }
            };

            let try_to_impl = quote! {
                impl TryInto<#d_type> for #ident {
                    type Error = RegisterError;
        
                    fn try_into(self) -> Result<#d_type, Self::Error> {
                        match self {
                            #(#into_matches)*
                            _ => Err(RegisterError::InvalidConfiguration)
                        }
                    }
                }
            };

            Ok(quote! {
                #try_from_impl
                #try_to_impl
            })
        },
        false => {
            let from_matches: Vec<TokenStream> = values.iter().filter(|&p| p.is_some()).map(|a | {
                let (ident, value) = a.as_ref().unwrap();
        
                quote! {
                    #value => Self::#ident,
                }
            }).collect();
        
            let into_matches: Vec<TokenStream> = values.iter().filter(|&p| p.is_some()).map(|a| {
                let (ident, value) = a.as_ref().unwrap();
                quote! {
                    Self::#ident => #value,
                }
            }).collect();

            let from_impl = quote! {
                impl From<#d_type> for #ident {
                    fn from(value: #d_type) -> Self {
                        match value {
                            #(#from_matches)*
                            #default_from
                        }
                    }
                }
            };

            let to_impl = quote! {
                impl Into<#d_type> for #ident {
                    fn into(self) -> #d_type {
                        match self {
                            #(#into_matches)*
                            #default_into
                        }
                    }
                }
            };

            Ok(quote!{ 
                #from_impl
                #to_impl
             })  
        }
    }
}