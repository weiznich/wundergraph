use proc_macro2::{Ident, Span};
use syn;
use syn::fold::Fold;
use syn::spanned::Spanned;

use diagnostic_shim::*;
use resolved_at_shim::*;
use utils::*;

#[derive(Debug)]
pub struct MetaItem {
    meta: syn::Meta,
}

impl MetaItem {
    pub fn all_with_name(attrs: &[syn::Attribute], name: &str) -> Vec<Self> {
        attrs
            .iter()
            .filter_map(|attr| {
                attr.interpret_meta()
                    .map(|m| FixSpan(attr.pound_token.spans[0]).fold_meta(m))
            })
            .filter_map(|meta| {
                if meta.name() == name {
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
                let meta = a.parse_meta().unwrap();
                if meta.name() == "doc" {
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
                ident: Ident::new(name, Span::call_site()),
                paren_token: syn::token::Paren::default(),
                nested: syn::punctuated::Punctuated::default(),
            }),
        }
    }

    pub fn nested_item(&self, name: &str) -> Result<Self, Diagnostic> {
        self.nested().and_then(|mut i| {
            i.find(|n| n.name() == name).ok_or_else(|| {
                self.span()
                    .error(format!("Missing required option {}", name))
            })
        })
    }


    pub fn expect_ident_value(&self) -> syn::Ident {
        self.ident_value().unwrap_or_else(|e| {
            e.emit();
            self.name()
        })
    }

    pub fn ident_value(&self) -> Result<syn::Ident, Diagnostic> {
        let maybe_attr = self.nested().ok().and_then(|mut n| n.nth(0));
        let maybe_word = maybe_attr.as_ref().and_then(|m| m.word().ok());
        match maybe_word {
            Some(x) => {
                self.span()
                    .warning(format!(
                        "The form `{0}(value)` is deprecated. Use `{0} = \"value\"` instead",
                        self.name(),
                    ))
                    .emit();
                Ok(x.clone())
            }
            _ => Ok(syn::Ident::new(
                &self.str_value()?,
                self.value_span().resolved_at(Span::call_site()),
            )),
        }
    }

    pub fn word(&self) -> Result<syn::Ident, Diagnostic> {
        if let syn::Meta::Word(ref x) = self.meta {
            Ok(x.clone())
        } else {
            let meta = &self.meta;
            Err(self.span().error(format!(
                "Expected `{}` found `{}`",
                self.name(),
                quote!(#meta)
            )))
        }
    }

    pub fn nested(&self) -> Result<Nested, Diagnostic> {
        use syn::Meta::*;

        match self.meta {
            List(ref list) => Ok(Nested(list.nested.iter())),
            _ => Err(self
                .span()
                .error(format!("`{0}` must be in the form `{0}(...)`", self.name()))),
        }
    }

    pub fn name(&self) -> syn::Ident {
        self.meta.name()
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
                self.name()
            ))),
        }
    }

    fn lit_value(&self) -> Result<&syn::Lit, Diagnostic> {
        use syn::Meta::*;

        match self.meta {
            NameValue(ref name_value) => Ok(&name_value.lit),
            _ => Err(self.span().error(format!(
                "`{0}` must be in the form `{0} = \"value\"`",
                self.name()
            ))),
        }
    }

    #[allow(unused)]
    pub fn warn_if_other_options(&self, options: &[&str]) {
        let nested = match self.nested() {
            Ok(x) => x,
            Err(_) => return,
        };
        let unrecognized_options =
            nested.filter(|n| !options.contains(&(&n.name().to_string() as _)));
        for ignored in unrecognized_options {
            ignored
                .span()
                .warning(format!("Option {} has no effect", ignored.name()))
                .emit();
        }
    }

    pub fn value_span(&self) -> Span {
        use syn::Meta::*;

        match self.meta {
            Word(ref ident) => ident.span(),
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

#[cfg_attr(rustfmt, rustfmt_skip)] // https://github.com/rust-lang-nursery/rustfmt/issues/2392
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
