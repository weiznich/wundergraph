use crate::diagnostic_shim::Diagnostic;
use crate::model::Model;
use crate::utils::wrap_in_dummy_mod;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let model = Model::from_item(item)?;

    let pg = if cfg!(feature = "postgres") {
        Some(derive_non_table_filter(
            &model,
            item,
            &quote!(diesel::pg::Pg),
        )?)
    } else {
        None
    };

    let sqlite = if cfg!(feature = "sqlite") {
        Some(derive_non_table_filter(
            &model,
            item,
            &quote!(diesel::sqlite::Sqlite),
        )?)
    } else {
        None
    };

    let mysql = if cfg!(feature = "mysql") {
        Some(derive_non_table_filter(
            &model,
            item,
            &quote!(diesel::mysql::Mysql),
        )?)
    } else {
        None
    };

    Ok(wrap_in_dummy_mod(
        "build_filter_helper",
        &model.name,
        &quote! {
            #pg
            #sqlite
            #mysql
        },
    ))
}

pub fn derive_non_table_filter(
    model: &Model,
    item: &syn::DeriveInput,
    backend: &TokenStream,
) -> Result<TokenStream, Diagnostic> {
    let (_, ty_generics, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics
        .params
        .push(parse_quote!(__Ctx: wundergraph::WundergraphContext + 'static));
    {
        let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(<__Ctx as wundergraph::WundergraphContext>::Connection: wundergraph::diesel::Connection<Backend = #backend>));
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let table = model.table_type()?;
    let table = &quote!(#table::table);
    let struct_type = &model.name;
    let filter = &quote! {
        <wundergraph::query_builder::selection::filter::FilterConverter<
            #struct_type #ty_generics, #backend, __Ctx> as wundergraph::query_builder::selection::filter::CreateFilter>::Filter
    };

    Ok(quote! {
        impl#impl_generics wundergraph::query_builder::selection::filter::BuildFilterHelper<
            #backend,
            #filter,
            __Ctx,
            > for #table
            #where_clause
        {
            type Ret = Box<dyn wundergraph::diesel_ext::BoxableFilter<#table, #backend, SqlType = wundergraph::diesel::sql_types::Bool>>;
            const FIELD_COUNT: usize = <wundergraph::query_builder::selection::filter::FilterBuildHelper<
                #filter,
                #struct_type  #ty_generics,
                #backend,
                __Ctx> as wundergraph::query_builder::selection::filter::InnerFilter>::FIELD_COUNT;

            fn into_filter(
                f: #filter,
            ) -> std::option::Option<Self::Ret> {
                use wundergraph::query_builder::selection::filter::BuildFilter;
                BuildFilter::<#backend>::into_filter(f)
            }

            fn from_inner_look_ahead(
                objs: &[(&str, wundergraph::juniper::LookAheadValue<wundergraph::scalar::WundergraphScalarValue>)]
            ) -> #filter {
                use wundergraph::query_builder::selection::filter::InnerFilter;
                wundergraph::query_builder::selection::filter::FilterBuildHelper::<#filter, #struct_type #ty_generics, #backend, __Ctx>::from_inner_look_ahead(objs).0
            }

            fn from_inner_input_value(
                obj: wundergraph::indexmap::IndexMap<&str, &wundergraph::juniper::InputValue<wundergraph::scalar::WundergraphScalarValue>>,
            ) -> std::option::Option<#filter> {
                use wundergraph::query_builder::selection::filter::InnerFilter;
                std::option::Option::Some(
                    wundergraph::query_builder::selection::filter::FilterBuildHelper::<#filter, #struct_type #ty_generics, #backend, __Ctx>::from_inner_input_value(obj)?.0
                )
            }

            fn to_inner_input_value(
                _f: &#filter,
                _v: &mut wundergraph::indexmap::IndexMap<&str, wundergraph::juniper::InputValue<wundergraph::scalar::WundergraphScalarValue>>
            ) {

            }

            fn register_fields<'__r>(
                _info: &wundergraph::juniper_ext::NameBuilder<()>,
                registry: &mut wundergraph::juniper::Registry<'__r, wundergraph::scalar::WundergraphScalarValue>
            ) -> std::vec::Vec<wundergraph::juniper::meta::Argument<'__r, wundergraph::scalar::WundergraphScalarValue>> {
                use wundergraph::query_builder::selection::filter::InnerFilter;
                wundergraph::query_builder::selection::filter::FilterBuildHelper::<#filter, #struct_type #ty_generics, #backend, __Ctx>::register_fields(&Default::default(), registry)
            }
        }
    })
}
