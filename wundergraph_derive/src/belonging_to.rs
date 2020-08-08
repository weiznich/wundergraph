use crate::diagnostic_shim::Diagnostic;
use crate::model::Model;
use crate::utils::{inner_of_option_ty, inner_ty_args, wrap_in_dummy_mod};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::parse_quote;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let model = Model::from_item(item)?;
    let belonging_to = derive_belonging_to(&model, item)?;

    Ok(wrap_in_dummy_mod(
        "belonging_to",
        &model.name,
        &quote! {
            #(#belonging_to)*
        },
    ))
}

pub fn derive_belonging_to(
    model: &Model,
    item: &syn::DeriveInput,
) -> Result<Vec<TokenStream>, Diagnostic> {
    model
        .fields()
        .iter()
        .filter_map(|f| {
            if let Some(args) = inner_ty_args(inner_of_option_ty(&f.ty), "HasOne") {
                let key_ty = if let syn::GenericArgument::Type(ref ty) = args[0] {
                    quote!(#ty)
                } else {
                    panic!("No key type found");
                };
                let parent_ty = if let syn::GenericArgument::Type(ref ty) = args[1] {
                    ty
                } else {
                    panic!("No parent type found");
                };
                Some((parent_ty, (key_ty, f)))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>()
        .into_iter()
        .map(|(parent_ty, (key_ty, f))| {
            let pg = if cfg!(feature = "postgres") {
                Some(derive_belongs_to(
                    model,
                    item,
                    parent_ty,
                    &key_ty,
                    f.sql_name(),
                    &quote!(diesel::pg::Pg),
                )?)
            } else {
                None
            };
            let sqlite = if cfg!(feature = "sqlite") {
                Some(derive_belongs_to(
                    model,
                    item,
                    parent_ty,
                    &key_ty,
                    f.sql_name(),
                    &quote!(diesel::sqlite::Sqlite),
                )?)
            } else {
                None
            };
            let mysql = if cfg!(feature = "mysql") {
                Some(derive_belongs_to(
                    model,
                    item,
                    parent_ty,
                    &key_ty,
                    f.sql_name(),
                    &quote!(diesel::mysql::Mysql),
                )?)
            } else {
                None
            };
            Ok(quote! {
                #pg
                #sqlite
                #mysql
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn derive_belongs_to(
    model: &Model,
    item: &syn::DeriveInput,
    other: &syn::Type,
    key_ty: &TokenStream,
    key_column: &syn::Path,
    backend: &TokenStream,
) -> Result<TokenStream, Diagnostic> {
    let struct_type = &model.name;
    let table_name = model.table_type()?;
    let key_column = quote!(#table_name::#key_column);
    let debug = if cfg!(feature = "debug") {
        Some(quote!(wundergraph::log::debug!("{:?}", wundergraph::diesel::debug_query(&query));))
    } else {
        None
    };

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
    Ok(quote! {
        impl#impl_generics wundergraph::query_builder::selection::fields::WundergraphBelongsTo<
            <#other as wundergraph::diesel::associations::HasTable>::Table,
            #backend,
            __Ctx,
            #key_column,
        > for #struct_type #ty_generics
            #where_clause
        {
            type Key = #key_ty;

            fn resolve(
                global_args: &[wundergraph::juniper::LookAheadArgument<wundergraph::scalar::WundergraphScalarValue>],
                look_ahead: &wundergraph::juniper::LookAheadSelection<wundergraph::scalar::WundergraphScalarValue>,
                selection: std::option::Option<&[wundergraph::juniper::Selection<wundergraph::scalar::WundergraphScalarValue>]>,
                keys: &[std::option::Option<#key_ty>],
                executor: &wundergraph::juniper::Executor<__Ctx, wundergraph::scalar::WundergraphScalarValue>,
            ) -> wundergraph::error::Result<std::collections::HashMap<
                    std::option::Option<#key_ty>,
                    std::vec::Vec<juniper::Value<wundergraph::scalar::WundergraphScalarValue>>>>
            {
                    use wundergraph::diesel::{ExpressionMethods, RunQueryDsl, QueryDsl, NullableExpressionMethods};
                    use wundergraph::WundergraphContext;
                    use wundergraph::query_builder::selection::{LoadingHandler, BoxedQuery};
                    let conn = executor.context().get_connection();
                    let query = <_ as QueryDsl>::filter(
                        <BoxedQuery<Self, #backend, __Ctx> as QueryDsl>::select(
                           <Self as LoadingHandler<#backend, __Ctx>>::build_query(global_args, look_ahead)?,
                            (
                                #key_column::default().nullable(),
                                <Self as LoadingHandler<#backend, __Ctx>>::get_select(look_ahead)?,
                            )
                       ),
                        #key_column::default().nullable().eq_any(keys),
                    );
                    #debug
                    <Self as wundergraph::query_builder::selection::fields::WundergraphBelongsTo<
                        <#other as wundergraph::diesel::associations::HasTable>::Table,
                    #backend,
                    __Ctx,
                    #key_column
                    >>::build_response(query.load(conn)?, global_args, look_ahead, selection, executor)
            }
        }
    })
}
