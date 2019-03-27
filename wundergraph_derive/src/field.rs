use proc_macro2::{Span, TokenStream};
use quote;
use syn;
use syn::spanned::Spanned;

use diagnostic_shim::{Diagnostic, DiagnosticShim};
use meta::*;
use utils::*;

#[derive(Debug)]
pub struct Field {
    pub ty: syn::Type,
    rust_name: FieldName,
    graphql_name: syn::Ident,
    sql_name: syn::Ident,
    pub span: Span,
    pub doc: Option<String>,
    pub deprecated: Option<String>,
    flags: MetaItem,
}

impl Field {
    pub fn from_struct_field(field: &syn::Field, index: usize) -> Result<Self, Diagnostic> {
        let rust_name = match field.ident {
            Some(ref o) => {
                let mut x = o.clone();
                // https://github.com/rust-lang/rust/issues/47983#issuecomment-362817105
                x.set_span(fix_span(o.span(), Span::call_site()));
                FieldName::Named(x)
            }
            None => FieldName::Unnamed(syn::Index {
                index: index as u32,
                // https://github.com/rust-lang/rust/issues/47312
                span: Span::call_site(),
            }),
        };
        let span = field.span();
        let doc = MetaItem::get_docs(&field.attrs);
        let deprecated = MetaItem::get_deprecated(&field.attrs);
        let flags = MetaItem::with_name(&field.attrs, "wundergraph")
            .unwrap_or_else(|| MetaItem::empty("wundergraph"));

        let sql_name = MetaItem::with_name(&field.attrs, "column_name")
            .ok_or_else(|| span.error("No `#[column_name = \"name\"]` annotation found"))
            .and_then(|i| i.ident_value())
            .or_else(|_| {
                match rust_name {
                FieldName::Named(ref x) => Ok(x.clone()),
                FieldName::Unnamed(_) => Err(span.error("Tuple struct fields needed to be annotated with `#[column_name = \"sql_name\"]")),
            }
            })?;
        let graphql_name = flags
            .nested_item("graphql_name")
            .and_then(|i| i.ident_value())
            .or_else(|_| {
                match rust_name {
                FieldName::Named(ref x) => Ok(x.clone()),
                FieldName::Unnamed(_) => Err(span.error("Tuple struct fields needed to be annotated with `#[wundergraph(graphql_name = \"sql_name\")]")),
            }
            })?;

        Ok(Self {
            ty: field.ty.clone(),
            rust_name,
            graphql_name,
            sql_name,
            flags,
            span,
            doc,
            deprecated,
        })
    }
    pub fn rust_name(&self) -> &FieldName {
        &self.rust_name
    }

    pub fn graphql_name(&self) -> &syn::Ident {
        &self.graphql_name
    }

    pub fn sql_name(&self) -> &syn::Ident {
        &self.sql_name
    }
}

#[derive(Debug)]
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

    pub fn access(&self) -> TokenStream {
        let span = self.span();
        // Span of the dot is important due to
        // https://github.com/rust-lang/rust/issues/47312
        quote_spanned!(span=> .#self)
    }

    pub fn span(&self) -> Span {
        match *self {
            FieldName::Named(ref x) => x.span(),
            FieldName::Unnamed(ref x) => x.span(),
        }
    }
}

impl quote::ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            FieldName::Named(ref x) => x.to_tokens(tokens),
            FieldName::Unnamed(ref x) => x.to_tokens(tokens),
        }
    }
}
