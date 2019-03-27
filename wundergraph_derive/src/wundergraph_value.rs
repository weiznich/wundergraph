use diagnostic_shim::*;
use meta::MetaItem;
use proc_macro2::{Span, TokenStream};
use syn;
use syn::spanned::Spanned;
use utils::wrap_in_dummy_mod;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let filter_value = filter_value(item);
    let nameable = nameable(item);
    let look_ahead = from_look_ahead(item)?;
    let wundergraph_value = wundergraph_value(item)?;
    let as_filter = as_column_filter(item);

    Ok(wrap_in_dummy_mod(
        "wundergraph_value",
        &item.ident,
        &quote! {
            use wundergraph::filter::filter_value::FilterValue;
            use wundergraph::helper::FromLookAheadValue;
            use wundergraph::juniper::{self, LookAheadValue};
            use wundergraph::scalar::WundergraphScalarValue;
            use wundergraph::helper::Nameable;
            use wundergraph::filter::filter_helper::AsColumnFilter;
            use wundergraph::filter::FilterOption;
            use wundergraph::query_helper::placeholder::{WundergraphValue, PlaceHolder};
            use wundergraph::diesel::sql_types::Nullable;


            #filter_value
            #nameable
            #look_ahead
            #wundergraph_value
            #as_filter
        },
    ))
}

fn as_column_filter(item: &syn::DeriveInput) -> TokenStream {
    let item_name = &item.ident;
    let (_, ty_generics, where_clause) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics.params.push(parse_quote!(__C));
    generics.params.push(parse_quote!(__DB));
    generics.params.push(parse_quote!(__Ctx));
    let (impl_generics, _, _) = generics.split_for_impl();

    quote! {
        impl #impl_generics AsColumnFilter<__C, __DB, __Ctx> for #item_name #ty_generics
            #where_clause
        {
            type Filter = FilterOption<Self, __C>;
        }
    }
}

fn wundergraph_value(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let sql_type = MetaItem::with_name(&item.attrs, "sql_type")
        .map(|m| m.expect_ident_value())
        .ok_or_else(|| {
            item.span()
                .error(format!("Missing required option `sql_type`",))
        })?;
    let item_name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics WundergraphValue for #item_name #ty_generics
            #where_clause
        {
            type PlaceHolder = PlaceHolder<Self>;
            type SqlType = Nullable<#sql_type>;
        }
    })
}

fn from_look_ahead(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let item_name = &item.ident;
    let field_list = enum_fields(item)?.map(|(name, f)| {
        let variant = &f.ident;
        quote! {
            #name => Some(#item_name::#variant)
        }
    });

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics FromLookAheadValue for #item_name #ty_generics
            #where_clause
        {
            fn from_look_ahead(v: &LookAheadValue<WundergraphScalarValue>) -> Option<Self> {
                if let LookAheadValue::Enum(ref e) = *v {
                    match *e {
                        #(#field_list,)*
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    })
}

pub(crate) fn nameable(item: &syn::DeriveInput) -> TokenStream {
    let item_name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    quote! {
        impl #impl_generics Nameable for #item_name #ty_generics
            #where_clause
        {
            fn name() -> String {
                String::from(stringify!(#item_name))
            }
        }
    }
}

fn filter_value(item: &syn::DeriveInput) -> TokenStream {
    let item_name = &item.ident;
    let (_, ty_generics, where_clause) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics.params.push(parse_quote!(__C));
    let (impl_generics, _, _) = generics.split_for_impl();
    quote!(
        impl #impl_generics FilterValue<__C> for #item_name #ty_generics
            #where_clause
        {
            type RawValue = #item_name #ty_generics;
            type AdditionalFilter = ();
        }
    )
}

pub(crate) fn to_upper_snake_case(s: &str) -> String {
    let mut last_lower = false;
    let mut upper = String::new();
    for c in s.chars() {
        if c == '_' {
            last_lower = false;
        } else if c.is_lowercase() {
            last_lower = true;
        } else if c.is_uppercase() {
            if last_lower {
                upper.push('_');
            }
            last_lower = false;
        }

        for u in c.to_uppercase() {
            upper.push(u);
        }
    }
    upper
}

fn enum_fields(
    item: &syn::DeriveInput,
) -> Result<impl Iterator<Item = (String, &syn::Variant)>, Diagnostic> {
    let e = match item.data {
        syn::Data::Enum(ref e) => e,
        _ => return Err(Span::call_site().error("This derive can only be used on enums")),
    };
    Ok(e.variants.iter().map(|f| {
        let name = MetaItem::with_name(&f.attrs, "graphql")
            .and_then(|g| g.nested_item("name").ok())
            .and_then(|n| n.str_value().ok())
            .unwrap_or_else(|| to_upper_snake_case(&f.ident.to_string()));
        (name, f)
    }))
}
