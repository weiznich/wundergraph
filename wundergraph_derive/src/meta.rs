use proc_macro2::{Ident, Span};
use quote::quote;
use syn::fold::Fold;
use syn::spanned::Spanned;

use crate::diagnostic_shim::*;
use crate::resolved_at_shim::*;
use crate::utils::*;

#[derive(Debug)]
pub struct MetaItem {
    meta: syn::Meta,
}

impl MetaItem {
    pub fn all_with_name(attrs: &[syn::Attribute], name: &str) -> Vec<Self> {
        attrs
            .iter()
            .filter_map(|attr| {
                attr.parse_meta()
                    .ok()
                    .map(|m| FixSpan(attr.pound_token.spans[0]).fold_meta(m))
            })
            .filter_map(|meta| {
                if meta.path().is_ident(name) {
                    Some(Self { meta })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_deprecated(attrs: &[syn::Attribute]) -> Option<String> {
        Self::with_name(attrs, "deprecated")
            .and_then(|d| d.nested_item("note").and_then(|n| n.str_value()).ok())
    }

    pub fn get_docs(attrs: &[syn::Attribute]) -> Option<String> {
        attrs
            .iter()
            .filter_map(|a| {
                let meta = a.parse_meta().expect("Failed to parse meta");
                if meta.path().is_ident("doc") {
                    if let syn::Meta::NameValue(value) = meta {
                        let s = match value.lit {
                            syn::Lit::Str(v) => v.value(),
                            _ => return None,
                        };
                        let s = s.replace("\"", "");
                        let mut s = s.trim();
                        if s.starts_with("///") {
                            s = &s[3..];
                        }
                        Some(s.trim().to_owned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .fold(None, |acc, s| {
                if let Some(acc) = acc {
                    Some(format!("{}\n{}", acc, s))
                } else {
                    Some(s)
                }
            })
    }

    pub fn with_name(attrs: &[syn::Attribute], name: &str) -> Option<Self> {
        Self::all_with_name(attrs, name).pop()
    }

    pub fn empty(name: &str) -> Self {
        Self {
            meta: syn::Meta::List(syn::MetaList {
                path: syn::Path::from(Ident::new(name, Span::call_site())),
                paren_token: syn::token::Paren::default(),
                nested: syn::punctuated::Punctuated::default(),
            }),
        }
    }

    pub fn nested_item(&self, name: &str) -> Result<Self, Diagnostic> {
        self.nested().and_then(|mut i| {
            i.find(|n| match n.meta {
                syn::Meta::NameValue(syn::MetaNameValue { path: ref p, .. })
                | syn::Meta::Path(ref p) => p.is_ident(name),
                syn::Meta::List(_) => false,
            })
            .ok_or_else(|| {
                self.span()
                    .error(format!("Missing required option {}", name))
            })
        })
    }

    pub fn expect_ident_value(&self) -> syn::Path {
        self.ident_value().unwrap_or_else(|e| {
            e.emit();
            self.name().clone()
        })
    }

    pub fn ident_value(&self) -> Result<syn::Path, Diagnostic> {
        let maybe_attr = self.nested().ok().and_then(|mut n| n.nth(0));
        let maybe_word = maybe_attr.as_ref().and_then(|m| m.path().ok());
        match maybe_word {
            Some(x) => {
                self.span()
                    .warning(format!(
                        "The form `{0}(value)` is deprecated. Use `{0} = \"value\"` instead",
                        self.name().get_ident().unwrap(),
                    ))
                    .emit();
                Ok(x)
            }
            _ => Ok(syn::Ident::new(
                &self.str_value()?,
                self.value_span().resolved_at(Span::call_site()),
            )
            .into()),
        }
    }

    pub fn path(&self) -> Result<syn::Path, Diagnostic> {
        if let syn::Meta::Path(ref x) = self.meta {
            Ok(x.clone())
        } else {
            let meta = &self.meta;
            Err(self.span().error(format!(
                "Expected `{}` found `{}`",
                self.name().get_ident().unwrap(),
                quote!(#meta)
            )))
        }
    }

    pub fn nested(&self) -> Result<Nested<'_>, Diagnostic> {
        use syn::Meta::*;

        match self.meta {
            List(ref list) => Ok(Nested(list.nested.iter())),
            _ => Err(self.span().error(format!(
                "`{0}` must be in the form `{0}(...)`",
                self.name().get_ident().unwrap()
            ))),
        }
    }

    pub fn name(&self) -> &syn::Path {
        self.meta.path()
    }

    pub fn str_value(&self) -> Result<String, Diagnostic> {
        self.lit_str_value().map(syn::LitStr::value)
    }

    pub fn lit_str_value(&self) -> Result<&syn::LitStr, Diagnostic> {
        use syn::Lit::*;

        match *self.lit_value()? {
            Str(ref s) => Ok(s),
            _ => Err(self.span().error(format!(
                "`{0}` must be in the form `{0} = \"value\"`",
                self.name().get_ident().unwrap()
            ))),
        }
    }

    fn lit_value(&self) -> Result<&syn::Lit, Diagnostic> {
        use syn::Meta::*;

        match self.meta {
            NameValue(ref name_value) => Ok(&name_value.lit),
            _ => Err(self.span().error(format!(
                "`{0}` must be in the form `{0} = \"value\"`",
                self.name().get_ident().unwrap()
            ))),
        }
    }

    #[allow(unused)]
    pub fn warn_if_other_options(&self, options: &[&str]) {
        let nested = match self.nested() {
            Ok(x) => x,
            Err(_) => return,
        };
        let unrecognized_options = nested
            .filter(|n| !options.contains(&(&n.name().get_ident().unwrap().to_string() as _)));
        for ignored in unrecognized_options {
            ignored
                .span()
                .warning(format!(
                    "Option {} has no effect",
                    ignored.name().get_ident().unwrap()
                ))
                .emit();
        }
    }

    pub fn value_span(&self) -> Span {
        use syn::Meta::*;

        match self.meta {
            Path(ref path) => path.span(),
            List(ref meta) => meta.nested.span(),
            NameValue(ref meta) => meta.lit.span(),
        }
    }

    pub fn span(&self) -> Span {
        self.meta.span()
    }

    pub fn get_flag<T>(&self, name: &str) -> Result<T, Diagnostic>
    where
        T: syn::parse::Parse,
    {
        self.nested_item(name)
            .and_then(|s| s.str_value())
            .and_then(|s| {
                syn::parse_str(&s)
                    .map_err(|_| self.value_span().error(String::from("Expected a path")))
            })
    }
}

pub struct Nested<'a>(syn::punctuated::Iter<'a, syn::NestedMeta>);

impl<'a> Iterator for Nested<'a> {
    type Item = MetaItem;

    fn next(&mut self) -> Option<Self::Item> {
        use syn::NestedMeta::*;

        match self.0.next() {
            Some(&Meta(ref item)) => Some(MetaItem { meta: item.clone() }),
            Some(_) => self.next(),
            None => None,
        }
    }
}

/// If the given span is affected by
/// <https://github.com/rust-lang/rust/issues/47941>,
/// returns the span of the pound token
struct FixSpan(Span);

impl Fold for FixSpan {
    fn fold_span(&mut self, span: Span) -> Span {
        fix_span(span, self.0)
    }
}
