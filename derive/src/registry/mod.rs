use crate::helpers::{parse_meta_str_array, parse_metalist, parse_tokens_comma_separated};
use fallible_iterator::FallibleIterator;
use fallible_iterator::IteratorExt;
use proc_macro2::Ident;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use syn::{Data, Error, Expr, ExprLit, Lit, Meta, Type};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RegistryNode {
    Category(String),
    SettingItem(String),
}

impl RegistryNode {
    pub fn is_category(&self) -> bool {
        match self {
            RegistryNode::Category(_) => true,
            _ => false,
        }
    }

    pub fn is_setting_item(&self) -> bool {
        match self {
            RegistryNode::SettingItem(_) => true,
            _ => false,
        }
    }

    pub fn string_ref(&self) -> &String {
        match self {
            RegistryNode::Category(x) => x,
            RegistryNode::SettingItem(x) => x,
        }
    }
}

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct CategoryRelationship {
    pub parents: Vec<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Default)]
pub struct StructAttribute {
    pub categories: Vec<String>,
    pub rel: Vec<CategoryRelationship>,
}

impl StructAttribute {
    fn parse(input: &syn::DeriveInput) -> Result<StructAttribute, Error> {
        Ok(input
            .attrs
            .iter()
            .filter(|x| x.path().is_ident("easy_settings"))
            .map(|x| {
                let mut categories: Vec<String> = vec![];
                let mut relationships: Vec<CategoryRelationship> = vec![];
                let meta_list = parse_metalist(x.meta.require_list()?)?;
                for meta in meta_list {
                    if meta.path().is_ident("categories") {
                        categories.extend(parse_meta_str_array(meta.require_list()?, "categories")?)
                    } else if meta.path().is_ident("rel") {
                        let mut v: CategoryRelationship = CategoryRelationship::default();
                        let rel_list = parse_metalist(meta.require_list()?)?;
                        for rel in rel_list {
                            if rel.path().is_ident("parents") {
                                v.parents.extend(parse_tokens_comma_separated(
                                    rel.require_list()?.tokens.clone(),
                                    "parents",
                                )?);
                            } else if rel.path().is_ident("children") {
                                v.children.extend(parse_tokens_comma_separated(
                                    rel.require_list()?.tokens.clone(),
                                    "children",
                                )?);
                            } else {
                                return Err(Error::new_spanned(rel, "unrecognized rel"));
                            }
                        }
                        v.parents.sort();
                        v.children.sort();
                        relationships.push(v);
                    } else {
                        return Err(Error::new_spanned(meta, "unrecognized easy_settings"));
                    }
                }
                Ok::<StructAttribute, Error>(StructAttribute {
                    categories,
                    rel: relationships,
                })
            })
            .transpose_into_fallible()
            .collect::<Vec<StructAttribute>>()?
            .into_iter()
            .reduce(|x, y| StructAttribute {
                categories: [x.categories, y.categories].concat(),
                rel: [x.rel, y.rel].concat(),
            })
            .map(|mut x| {
                x.categories.sort();
                x.rel.sort();
                x
            })
            .unwrap_or_default())
    }
}

#[derive(Debug, Default)]
pub struct FieldAttribute {
    pub default: Option<Expr>,
    pub name: Option<String>,
    pub categories: Vec<String>,
}

#[derive(Debug)]
pub struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub attrs: FieldAttribute,
}

impl Field {
    pub fn get_field_name(&self) -> String {
        self.attrs
            .name
            .clone()
            .unwrap_or_else(|| self.ident.to_string())
    }

    fn append_from_meta(&mut self, meta: &Meta) -> Result<(), Error> {
        match meta {
            Meta::NameValue(x) => {
                if x.path.is_ident("name") {
                    if self.attrs.name.is_some() {
                        return Err(Error::new_spanned(x, "The `name` attribute is duplicated."));
                    }
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }) = x.value.clone()
                    {
                        self.attrs.name = Some(str.value());
                    } else {
                        return Err(Error::new_spanned(
                            x,
                            "The `name` attribute must be a string literal.",
                        ));
                    }
                } else if x.path.is_ident("default") {
                    if self.attrs.default.is_some() {
                        return Err(Error::new_spanned(
                            x,
                            "The `default` attribute is duplicated.",
                        ));
                    }
                    self.attrs.default = Some(x.value.clone());
                } else {
                    return Err(Error::new_spanned(
                        x,
                        format!("unrecognized {}", x.path.require_ident()?),
                    ));
                }
            }
            Meta::List(x) => {
                if x.path.is_ident("categories") {
                    self.attrs
                        .categories
                        .extend(parse_meta_str_array(x, "categories")?);
                } else {
                    return Err(Error::new_spanned(
                        x,
                        format!("unrecognized {}", x.path.require_ident()?),
                    ));
                }
            }
            _ => {
                return Err(Error::new_spanned(
                    meta,
                    format!("unrecognized {}", meta.path().require_ident()?),
                ));
            }
        }
        Ok(())
    }

    fn parse(input: &syn::DataStruct) -> Result<BTreeMap<String, Field>, Error> {
        let mut result = BTreeMap::new();

        for field in input.fields.iter() {
            let ident = match field.ident.as_ref() {
                None => return Err(Error::new_spanned(field, "missing ident.")),
                Some(x) => x,
            };

            let mut res = Field {
                ident: ident.clone(),
                ty: field.ty.clone(),
                attrs: Default::default(),
            };
            for attr in field
                .attrs
                .clone()
                .into_iter()
                .filter(|x| x.path().is_ident("easy_settings"))
            {
                let meta = parse_metalist(attr.meta.require_list()?)?;
                for x in &meta {
                    res.append_from_meta(x)?;
                }
            }
            res.attrs.categories.sort();
            result.insert(res.get_field_name(), res);
        }

        Ok(result)
    }
}

fn aggregate_category_nodes(
    struct_attributes: &StructAttribute,
    field_attributes: &BTreeMap<String, Field>,
) -> BTreeMap<Option<String>, BTreeSet<RegistryNode>> {
    let category_relationships: Vec<(String, Vec<RegistryNode>)> = struct_attributes
        .categories
        .iter()
        .map(|cate| {
            (
                cate.clone(),
                struct_attributes
                    .rel
                    .iter()
                    .filter(|x| x.parents.contains(cate))
                    .map(|x| x.children.clone().into_iter().map(RegistryNode::Category))
                    .flatten()
                    .chain(
                        field_attributes
                            .values()
                            .filter(|x| x.attrs.categories.contains(cate))
                            .map(|field| RegistryNode::SettingItem(field.get_field_name())),
                    )
                    .collect(),
            )
        })
        .collect();

    let not_root_node: HashSet<RegistryNode> = category_relationships
        .iter()
        .map(|(_x, y)| y.clone())
        .flatten()
        .collect();

    let mut category_relationships_map: BTreeMap<Option<String>, BTreeSet<RegistryNode>> =
        BTreeMap::new();
    category_relationships.into_iter().for_each(|(k, v)| {
        let k = Some(k);
        match category_relationships_map.get_mut(&k) {
            None => {
                category_relationships_map.insert(k, BTreeSet::from_iter(v));
            }
            Some(x) => x.extend(v),
        }
    });

    category_relationships_map.insert(
        None,
        struct_attributes
            .categories
            .iter()
            .map(|x| RegistryNode::Category(x.clone()))
            .filter(|x| !not_root_node.contains(x))
            .chain(
                field_attributes
                    .values()
                    .map(|x| RegistryNode::SettingItem(x.get_field_name()))
                    .filter(|x| !not_root_node.contains(x)),
            )
            .collect(),
    );
    category_relationships_map
}

pub fn parser(
    input: &syn::DeriveInput,
) -> Result<
    (
        StructAttribute,
        BTreeMap<String, Field>,
        BTreeMap<Option<String>, BTreeSet<RegistryNode>>,
    ),
    Error,
> {
    match &input.data {
        Data::Struct(d) => {
            let struct_attributes = StructAttribute::parse(&input)?;
            let field_attributes = Field::parse(d)?;
            let category_relationships_map =
                aggregate_category_nodes(&struct_attributes, &field_attributes);

            Ok((
                struct_attributes,
                field_attributes,
                category_relationships_map,
            ))
        }
        _ => Err(Error::new_spanned(
            input,
            "`Registry` only supported struct.",
        )),
    }
}
