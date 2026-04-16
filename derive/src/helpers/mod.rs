use proc_macro2::TokenStream;
use quote::__private::ext::RepToTokensExt;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Error, GenericArgument, Lit, Meta, MetaList, PathArguments, Token, Type};

pub(crate) fn parse_comma_separated_str_array(
    tokens: ParseStream,
    ident: &str,
) -> Result<Vec<String>, Error> {
    let list = tokens.parse_terminated(Lit::parse, Token![,])?;
    let mut res = vec![];
    for x in list.iter() {
        if let Lit::Str(x) = x {
            res.push(x.value());
        } else {
            return Err(Error::new_spanned(x, format!("unrecognized {}", ident)));
        }
    }
    Ok(res)
}

pub(crate) fn parse_tokens_comma_separated(
    tokens: TokenStream,
    ident: &str,
) -> Result<Vec<String>, Error> {
    (|x: ParseStream| parse_comma_separated_str_array(x, ident)).parse2(tokens)
}

pub(crate) fn parse_meta_str_array(meta: &MetaList, ident: &str) -> Result<Vec<String>, Error> {
    parse_tokens_comma_separated(
        meta.tokens
            .next()
            .ok_or(Error::new_spanned(meta, "Empty lists are not allowed."))?
            .clone(),
        ident,
    )
}

pub(crate) fn parse_metalist(list: &MetaList) -> Result<Punctuated<Meta, Comma>, Error> {
    (|x: ParseStream| x.parse_terminated(Meta::parse, Token![,])).parse2(
        list.tokens
            .next()
            .ok_or(Error::new_spanned(list, "Empty lists are not allowed."))?
            .clone(),
    )
}

pub(crate) fn is_option(ty: &Type) -> bool {
    if let Type::Path(ty) = ty {
        ty.path
            .segments
            .last()
            .map(|x| x.ident == "Option")
            .unwrap_or_default()
    } else {
        false
    }
}

pub(crate) fn inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(ty) = ty {
        ty.path.segments.last().and_then(|x| {
            if let PathArguments::AngleBracketed(arg) = &x.arguments {
                if let GenericArgument::Type(ty) = arg.args.first().unwrap() {
                    Some(ty)
                } else {
                    None
                }
            } else {
                None
            }
        })
    } else {
        None
    }
}

pub(crate) fn require_option(ty: &Type) -> Result<&Type, Error> {
    inner_type(ty)
        .filter(|_| is_option(ty))
        .ok_or(Error::new_spanned(
            ty,
            "The field must be of type `Option`.",
        ))
}
