use crate::helpers::require_option;
use crate::registry::RegistryNode;
use fallible_iterator::{FallibleIterator, IteratorExt};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use std::collections::BTreeSet;
use syn::{parse_macro_input, DeriveInput, Error};

mod helpers;
mod registry;

fn registry_derive_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let struct_ident = &input.ident;
    let (struct_attributes, field_attributes, category_relationships) = registry::parser(&input)?;
    let setter_getter_functions = field_attributes.values().map(|x| {
        let field_type = &x.ty;
        let inner_type = require_option(field_type)?;
        let field_ident = &x.ident;
        let setter_name = &Ident::new(&format!("set_{}", x.get_field_name()), Span::call_site());
        let getter_name = &Ident::new(&format!("get_{}", x.get_field_name()), Span::call_site());
        let getter = if x.attrs.default.is_some() {
            let default = x.attrs.default.as_ref().unwrap();
            quote::quote! {pub fn #getter_name(&self) -> #inner_type { self.#field_ident.clone().unwrap_or_else(|| #default) }}
        } else {
            quote::quote! {pub fn #getter_name(&self) -> #field_type { self.#field_ident.clone() }}
        };

        Ok::<_, Error>(quote::quote! {
            pub fn #setter_name(&mut self, value: #field_type) { self.#field_ident = value; }
            #getter
        })
    }).transpose_into_fallible().collect::<Vec<_>>()?;

    let get_item_type_match_pattern = field_attributes
        .values()
        .map(|x| {
            let inner_type = require_option(&x.ty)?;
            let name = x.get_field_name();
            Ok::<_, Error>(quote::quote! {
                #name => std::any::TypeId::of::<#inner_type>(),
            })
        })
        .transpose_into_fallible()
        .collect::<Vec<_>>()?;

    let field_name1 = field_attributes.values().map(|x| x.get_field_name());
    let field_name2 = field_name1.clone();
    let setting_keys = field_name1.clone();
    let categories = struct_attributes.categories.iter();
    let setter_field_name = field_name1
        .clone()
        .map(|y| Ident::new(&format!("set_{}", y), Span::call_site()));
    let field_ident1 = field_attributes.iter().map(|(_x, y)| y.ident.clone());
    let field_ident2 = field_ident1.clone();

    let empty_btree_set = BTreeSet::new();
    let root_nodes = category_relationships
        .get(&None)
        .unwrap_or(&empty_btree_set)
        .iter()
        .map(|root_nodes| match root_nodes {
            RegistryNode::Category(x) => {
                quote::quote! {easy_settings::RegistryNode::Category(#x) }
            }
            RegistryNode::SettingItem(x) => {
                quote::quote! {easy_settings::RegistryNode::SettingItem(#x)}
            }
        });
    let children_nodes = category_relationships.keys().filter_map(|x| {
        x.as_ref().map(|category| {
            let child_nodes = category_relationships
                .get(&Some(category.clone()))
                .unwrap_or(&empty_btree_set);
            let category_nodes = child_nodes.iter().filter(|x| x.is_category()).map(|x| x.string_ref());
            let setting_item_nodes = child_nodes.iter().filter(|x| x.is_setting_item()).map(|x| x.string_ref());
            quote::quote! {
                    #category => &[ #(easy_settings::RegistryNode::Category(#category_nodes),)* #(easy_settings::RegistryNode::SettingItem(#setting_item_nodes),)* ]
                }
        })
    });

    Ok(quote::quote! {
        impl #struct_ident {
            #(#setter_getter_functions)*
        }

        impl std::default::Default for #struct_ident {
            fn default() -> Self {
                Self {
                    #(#field_ident1: std::option::Option::None),*
                }
            }
        }

        impl easy_settings::Registry for #struct_ident {
            fn set(&mut self, key: &str, value: easy_settings::SettingValue) {
                match key {
                    #(#field_name1 => self.#setter_field_name(value.into()),)*
                    &_ => {}
                }
            }

            fn get(&self, key: &str) -> std::option::Option<easy_settings::SettingValue> {
                std::option::Option::Some(match key {
                    #(#field_name2 => easy_settings::SettingValue::from(self.#field_ident2.as_ref()),)*
                    &_ => return std::option::Option::None,
                })
            }

            fn get_item_type(key: &str) -> std::option::Option<std::any::TypeId> {
                Some(match key {
                    #(#get_item_type_match_pattern)*
                    &_ => return std::option::Option::None,
                })
            }

            fn keys() -> &'static [&'static str] {
                &[
                    #(#setting_keys),*
                ]
            }

            fn categories() -> &'static [&'static str] {
                &[
                    #(#categories),*
                ]
            }

            fn child_nodes(parent_node: std::option::Option<&str>) -> &'static [easy_settings::RegistryNode] {
                match parent_node {
                    std::option::Option::None => &[
                        #(#root_nodes),*
                    ],
                    std::option::Option::Some(x) => match x {
                        #(#children_nodes,)*
                        &_ => &[],
                    },
                }
            }
        }
        }.into())
}

#[proc_macro_derive(Registry, attributes(easy_settings))]
pub fn registry_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    registry_derive_impl(input).unwrap_or_else(|e| e.to_compile_error().into())
}
