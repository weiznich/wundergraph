use proc_macro2::Span;
use quote;
use syn::spanned::Spanned;
use syn;

use meta::*;
use utils::*;
use diagnostic_shim::Diagnostic;

pub struct Field {
    pub ty: syn::Type,
    pub name: FieldName,
    pub span: Span,
    flags: MetaItem,
}

impl Field {
    pub fn from_struct_field(field: &syn::Field, index: usize) -> Self {
        let name = match field.ident {
            Some(mut x) => {
                // https://github.com/rust-lang/rust/issues/47983#issuecomment-362817105
                x.span = fix_span(x.span, Span::call_site());
                FieldName::Named(x)
            }
            None => FieldName::Unnamed(syn::Index {
                index: index as u32,
                // https://github.com/rust-lang/rust/issues/47312
                span: Span::call_site(),
            }),
        };
        let flags = MetaItem::with_name(&field.attrs, "wundergraph")
            .unwrap_or_else(|| MetaItem::empty("wundergraph"));
        let span = field.span();

        Self {
            ty: field.ty.clone(),
            name,
            flags,
            span,
        }
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.has_flag(flag)
    }

    pub fn foreign_key(&self) -> Result<syn::Path, Diagnostic> {
        self.flags.get_flag("foreign_key")
    }

    pub fn remote_table(&self) -> Result<syn::Type, Diagnostic> {
        self.flags.get_flag("remote_table")
    }

    pub fn filter(&self) -> Option<syn::Path> {
        let filter_name = if let Some(n) = inner_ty_arg(&self.ty, "HasMany", 0) {
            format!(
                "{}Filter<DB>",
                ty_name(inner_of_option_ty(n)).expect("Invalid type")
            )
        } else if let Some(n) = inner_ty_arg(&self.ty, "HasOne", 1) {
            format!(
                "{}Filter<DB>",
                ty_name(inner_of_option_ty(n)).expect("Invalid type")
            )
        } else {
            return None;
        };
        if let Ok(filter) = self.flags.nested_item("filter") {
            match filter.bool_value() {
                Ok(true) => syn::parse_str(&filter_name).ok(),
                Ok(false) => return None,
                Err(_) => self.flags.get_flag("filter").ok(),
            }
        } else {
            syn::parse_str(&filter_name).ok()
        }
    }

    pub fn is_nullable_reference(&self) -> bool {
        self.flags
            .nested_item("is_nullable_reference")
            .and_then(|m| m.bool_value())
            .unwrap_or(false)
    }
}

pub enum FieldName {
    Named(syn::Ident),
    Unnamed(syn::Index),
}

impl FieldName {
    #[allow(unused)]
    pub fn assign(&self, expr: &syn::Expr) -> syn::FieldValue {
        let span = self.span();
        // Parens are to work around https://github.com/rust-lang/rust/issues/47311
        let tokens = quote_spanned!(span=> #self: (#expr));
        parse_quote!(#tokens)
    }

    pub fn access(&self) -> quote::Tokens {
        let span = self.span();
        // Span of the dot is important due to
        // https://github.com/rust-lang/rust/issues/47312
        quote_spanned!(span=> .#self)
    }

    pub fn span(&self) -> Span {
        match *self {
            FieldName::Named(x) => x.span,
            FieldName::Unnamed(ref x) => x.span,
        }
    }
}

impl quote::ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut quote::Tokens) {
        match *self {
            FieldName::Named(x) => x.to_tokens(tokens),
            FieldName::Unnamed(ref x) => x.to_tokens(tokens),
        }
    }
}
