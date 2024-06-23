use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::DeriveInput;

use super::*;

/// Interpret the struct and do all the fun processing and error handling here
pub fn common_macro_processing<'a>(item: TokenStream) -> deluxe::Result<(StructMetadata<'a>, Vec<ParsedRegisterFieldAttribute>)> {
    let mut ast: DeriveInput = syn::parse2(item)?;
    
    let register_meta: RegisterStructAttributes = deluxe::extract_attributes(&mut ast)?;
    let ident = ast.ident.clone();
    let endian = Endian::parse(register_meta.endian.as_str()).ok_or_else(|| syn::Error::new_spanned(&ast.ident, "Must specify `endian = \"big\" | \"little\"`"))?;
    // let ast_generics: AstGenerics = ast.generics.clone().split_for_impl();
    // let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    
    // let where_clause = where_clause.map(|inner| inner.clone());
    // Parse the structs inner attributes
    let attrs = if let syn::Data::Struct(s) = &mut ast.data {
        let attrs: Result<Vec<(proc_macro2::Ident, RegisterFieldAttribute)>, syn::Error> = s.fields.iter_mut().map(| field | {
            // let field_ident = field.ident.clone().unwrap().to_token_stream();
            let ident = field.ident.clone().unwrap();
            let attr: RegisterFieldAttribute = deluxe::extract_attributes(field)?;

            Ok((ident, attr))
        }).collect();

        Ok(attrs?)
    } else {
        Err(syn::Error::new_spanned(&ast, "derive(Register) is only supported on type `struct`"))
    }?;

    let field_attrs: Result<Vec<ParsedRegisterFieldAttribute>, syn::Error> = attrs.iter().map(|(ident, attr)| {
        let bits = match (&attr.bit, &attr.bits) {
            (Some(bit), None) => { 
                let parsed: usize = bit.parse().map_err(|_| syn::Error::new_spanned(ident, "Unable to parse number"))?;
                
                Ok( BitRange::Bit(parsed) )
            },
            (None, Some(bits)) => {
                let parts: Vec<&str> = bits.split("..").collect();

                if parts.len() == 2 {
                    let parsed_start: usize = parts[0].trim().parse().map_err(|_| syn::Error::new_spanned(ident, "Unable to parse number"))?;
                    let parsed_end: usize = parts[1].trim().parse().map_err(|_| syn::Error::new_spanned(ident, "Unable to parse number"))?;

                    Ok( BitRange::Bits((parsed_start, parsed_end)))
                } else {
                    Err(syn::Error::new_spanned(ident, "Range must be in format `start..end`"))
                }                
            },
            _ => Err(syn::Error::new_spanned(ident, "Must have one of `register(bit = \"...\")` or `register(bits = \"...\")` set")),
        }?;

        Ok(ParsedRegisterFieldAttribute {
            bits,
            reset: attr.reset.clone(),
            ident: ident.clone(),
            endian,
        })
    }).collect();

    let ast_generics = ast.generics;

    let field_attrs = field_attrs?;
    let wrapped_meta: StructMetadata = (ident, register_meta.address, register_meta.length, format_ident!("{}", register_meta.word), endian, ast_generics, register_meta.write_fn, register_meta.read_fn);
    
    Ok((wrapped_meta, field_attrs))
}


#[derive(Clone, Copy)]
pub enum Endian {
    Big,
    Little,
}

impl Endian {
    fn parse(input: &str) -> Option<Self> {
        match input {
            "big" =>    Some(Endian::Big),
            "little" => Some(Endian::Little),
            _ => None
        }
    } 
}

#[derive(Clone, Copy)]
pub enum BitRange {
    Bit(usize),
    Bits((usize, usize))
}

impl BitRange {
    fn start(&self) -> usize {
        match *self {
            Self::Bit(start) => start,
            Self::Bits((start, _end)) => start,
        }
    }

    fn end(&self) -> usize {
        match *self {
            Self::Bit(start) => start,
            Self::Bits((_start, end)) => end,
        }
    }

    /// Applies `|i| => i % word_length_bits` to each element
    fn norm(&self, word_length_bits: usize) -> Self {
        match *self {
            Self::Bit(start) => Self::Bit(start % word_length_bits),
            Self::Bits((start, end)) => Self::Bits((start % word_length_bits, end % word_length_bits)),
        }
    }
}

/// Generate [`TokenStream`] for [`BitRange`]
pub fn unpack_bits_read(bits: BitRange) -> TokenStream {
    match bits {
        BitRange::Bit(bit_idx) => quote! { bit(#bit_idx) },
        BitRange::Bits((bits_start, bits_end)) => {
            quote! { bits(#bits_start..=#bits_end) }
        }
    }
}

/// Generate [`TokenStream`] for [`BitRange`]
pub fn unpack_bits_set(bits: BitRange, token: &Ident) -> TokenStream {
    match bits { 
        BitRange::Bit(bit_idx) => quote! { set_bit(#bit_idx, self.#token.clone().try_into()?) },
        BitRange::Bits((bits_start, bits_end)) => {
            quote! { set_bits(#bits_start..=#bits_end, self.#token.clone().try_into()?) }
        }
    }
}

pub fn relocate_bits(endian: Endian, bits: BitRange, word_length_bits: usize, length: usize) -> (usize, BitRange) {
    match endian {
        Endian::Little => {
            // Works so long as the bit index doesn't cross word boundaries
            // i.e. `5..10` won't work for `u8` since the boundary is at `8`
            let word_idx = bits.start() / word_length_bits;
            let bits = bits.norm(word_length_bits);

            (word_idx, bits)
        }
        Endian::Big => {
            let bits = bits.norm(word_length_bits);
            let word_idx = ((length * word_length_bits) - (bits.end() + 1)) / word_length_bits;

            (word_idx, bits)
        }
    }
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(register))]
struct RegisterStructAttributes {
    address: Lit,
    length: usize,
    #[deluxe(default = "u8".into())]
    word: String,
    // #[deluxe(default = "RO".to_string())]
    // rw: String
    #[deluxe(default = "little".into())] // Using little endian because its less numbers when parsing
    endian: String,
    #[deluxe(default = None)]
    write_fn: Option<TokenStream>,
    #[deluxe(default = None)]
    read_fn: Option<TokenStream>,
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(register))]
struct RegisterFieldAttribute {
    reset: TokenStream,
    #[deluxe(default = None)]
    bit: Option<String>,
    #[deluxe(default = None)]
    bits: Option<String>,
}

pub struct ParsedRegisterFieldAttribute {
    pub reset: TokenStream,
    pub bits: BitRange,
    pub ident: Ident,
    pub endian: Endian,
}

/// Metadata for the struct
/// ```no_run
/// let (struct_ident, address, length, word_type, endian, ( impl_generics, type_generics, where_clause ), write_fn, read_fn) = meta;
/// ```
// type StructMetadata<'a> = (Ident, Lit, usize, Ident, Endian, AstGenerics<'a, 'a, 'a>);
pub type StructMetadata<'a> = (Ident, Lit, usize, Ident, Endian, Generics, Option<TokenStream>, Option<TokenStream>);