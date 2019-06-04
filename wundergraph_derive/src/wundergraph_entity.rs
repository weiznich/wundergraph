use crate::diagnostic_shim::{Diagnostic, DiagnosticShim};
use crate::field::Field;
use crate::model::Model;
use crate::utils::{inner_of_option_ty, inner_ty_args, is_has_many, wrap_in_dummy_mod};
use proc_macro2::{Span, TokenStream};
use std::collections::HashMap;
use syn;

pub fn derive(item: &syn::DeriveInput) -> Result<TokenStream, Diagnostic> {
    let model = Model::from_item(item)?;
    let pg_loading_handler = if cfg!(feature = "postgres") {
        Some(derive_loading_handler(
            &model,
            item,
            &quote!(diesel::pg::Pg),
        )?)
    } else {
        None
    };

    let sqlite_loading_handler = if cfg!(feature = "sqlite") {
        Some(derive_loading_handler(
            &model,
            item,
            &quote!(diesel::sqlite::Sqlite),
        )?)
    } else {
        None
    };

    let pg_non_table_field_filter = if cfg!(feature = "postgres") {
        Some(derive_non_table_filter(
            &model,
            item,
            &quote!(diesel::pg::Pg),
        )?)
    } else {
        None
    };

    let sqlite_non_table_field_filter = if cfg!(feature = "sqlite") {
        Some(derive_non_table_filter(
            &model,
            item,
            &quote!(diesel::sqlite::Sqlite),
        )?)
    } else {
        None
    };

    let belongs_to = model
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
                    &model,
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
                    &model,
                    item,
                    parent_ty,
                    &key_ty,
                    f.sql_name(),
                    &quote!(diesel::sqlite::Sqlite),
                )?)
            } else {
                None
            };
            Ok(quote! {
                #pg
                #sqlite
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(wrap_in_dummy_mod(
        "wundergraph_entity",
        &model.name,
        &quote! {
            use wundergraph::diesel;
            use wundergraph::query_builder::selection::LoadingHandler;
            use wundergraph::graphql_type::WundergraphGraphqlMapper;

            #pg_loading_handler
            #sqlite_loading_handler
            #pg_non_table_field_filter
            #sqlite_non_table_field_filter

            #(#belongs_to)*
        },
    ))
}

fn derive_loading_handler(
    model: &Model,
    item: &syn::DeriveInput,
    backend: &TokenStream,
) -> Result<TokenStream, Diagnostic> {
    let struct_type = &model.name;
    let (_, ty_generics, _) = item.generics.split_for_impl();
    let table = model.table_type()?;
    let field_names = model.fields().iter().map(Field::graphql_name);
    let field_list = model.fields().iter().map(|f| &f.ty);
    let columns = model.fields().iter().filter_map(|f| {
        if is_has_many(&f.ty) {
            None
        } else {
            let column = f.sql_name();
            Some(quote!(#table::#column))
        }
    });
    let primary_keys = model.primary_key();
    assert!(!primary_keys.is_empty());
    let primary_key_index = model
        .primary_key()
        .iter()
        .map(|primary_key| {
            model
                .fields()
                .iter()
                .enumerate()
                .find(|(_, f)| *f.sql_name() == *primary_key)
                .map(|(i, _)| {
                    let index = syn::Ident::new(&format!("TupleIndex{}", i), Span::call_site());
                    quote!(wundergraph::helper::tuple::#index)
                })
                .ok_or_else(|| {
                    Span::call_site().error(
                        "No primary key found, use `#[primary_key(\"column\")]` to specify one",
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let primary_key_index = if primary_key_index.len() == 1 {
        primary_key_index[0].clone()
    } else {
        quote!((#(#primary_key_index,)*))
    };

    let description = model.fields().iter().enumerate().map(|(i, f)| {
        if let Some(ref d) = f.doc {
            quote!(#i => std::option::Option::Some(#d))
        } else {
            quote!(#i => std::option::Option::None)
        }
    });

    let deprecated = model.fields().iter().enumerate().map(|(i, f)| {
        if let Some(ref d) = f.deprecated {
            quote!(#i => std::option::Option::Some(std::option::Option::Some(#d)))
        } else {
            quote!(#i => std::option::Option::None)
        }
    });

    let type_description = model.docs.as_ref().map_or_else(
        || quote!(std::option::Option::None),
        |d| quote!(std::option::Option::Some(#d)),
    );

    let filter = model.filter_type().map_or_else(
        || {
            quote! {
                wundergraph::query_builder::selection::filter::filter_helper::FilterWrapper<Self, #backend, __Ctx>
            }
        },
        |p| quote!(#p),
    );

    let mut generics = item.generics.clone();
    generics
        .params
        .push(parse_quote!(__Ctx: wundergraph::context::WundergraphContext + 'static));
    {
        let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(<__Ctx as wundergraph::context::WundergraphContext>::Connection: wundergraph::diesel::Connection<Backend = #backend>));
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote! {

        impl #impl_generics WundergraphGraphqlMapper<#backend, __Ctx> for #struct_type #ty_generics
            #where_clause
        {
            type GraphQLType = wundergraph::graphql_type::GraphqlWrapper<#struct_type, #backend, __Ctx>;

            fn register_arguments<'r>(
                registry: &mut wundergraph::juniper::Registry<'r, wundergraph::scalar::WundergraphScalarValue>,
                field: wundergraph::juniper::meta::Field<'r, wundergraph::scalar::WundergraphScalarValue>
            ) -> wundergraph::juniper::meta::Field<'r, wundergraph::scalar::WundergraphScalarValue> {
                let arg = registry.arg_with_default::<
                    std::option::Option<wundergraph::query_builder::selection::filter::Filter<
                    <Self as LoadingHandler<#backend, __Ctx>>::Filter,
                <Self as wundergraph::diesel::associations::HasTable>::Table
                    >>
                    >(
                        "filter",
                        &std::option::Option::None,
                        &std::default::Default::default(),
                    );
                field.argument(arg)
            }
        }

        impl #impl_generics LoadingHandler<#backend, __Ctx> for #struct_type #ty_generics
            #where_clause
        {
            type Columns = (#(#columns,)*);
            type FieldList = (#(#field_list,)*);

            type PrimaryKeyIndex = #primary_key_index;
            type Filter = #filter;

            const FIELD_NAMES: &'static [&'static str] = &[#(stringify!(#field_names),)*];
            const TYPE_NAME: &'static str = stringify!(#struct_type);

            fn field_description(idx: usize) -> std::option::Option<&'static str> {
                match idx {
                    #(#description,)*
                    _ => std::option::Option::None,
                }
            }

            fn type_description() -> std::option::Option<&'static str> {
                #type_description
            }

            fn field_deprecation(idx: usize) -> std::option::Option<std::option::Option<&'static str>> {
                match idx {
                    #(#deprecated,)*
                    _ => std::option::Option::None,
                }
            }
        }
    })
}

fn derive_belongs_to(
    model: &Model,
    item: &syn::DeriveInput,
    other: &syn::Type,
    key_ty: &TokenStream,
    key_column: &syn::Ident,
    backend: &TokenStream,
) -> Result<TokenStream, Diagnostic> {
    let struct_type = &model.name;
    let table_name = model.table_type()?;
    let key_column = quote!(#table_name::#key_column);

    let (_, ty_generics, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics
        .params
        .push(parse_quote!(__Ctx: wundergraph::context::WundergraphContext + 'static));
    {
        let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(<__Ctx as wundergraph::context::WundergraphContext>::Connection: wundergraph::diesel::Connection<Backend = #backend>));
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl#impl_generics wundergraph::query_builder::selection::fields::associations::WundergraphBelongsTo<
            <#other as wundergraph::diesel::associations::HasTable>::Table,
            #backend,
            __Ctx,
            #key_column,
        > for #struct_type #ty_generics
            #where_clause
        {
            type Key = #key_ty;

            fn resolve(
                look_ahead: &wundergraph::juniper::LookAheadSelection<wundergraph::scalar::WundergraphScalarValue>,
                selection: std::option::Option<&[wundergraph::juniper::Selection<wundergraph::scalar::WundergraphScalarValue>]>,
                keys: &[std::option::Option<#key_ty>],
                executor: &wundergraph::juniper::Executor<__Ctx, wundergraph::scalar::WundergraphScalarValue>,
            ) -> std::result::Result<std::collections::HashMap<
                    std::option::Option<#key_ty>,
                    std::vec::Vec<juniper::Value<WundergraphScalarValue>>
                >, wundergraph::failure::Error> {
                    use wundergraph::diesel::{ExpressionMethods, RunQueryDsl, QueryDsl, NullableExpressionMethods};
                    use wundergraph::context::WundergraphContext;
                    use wundergraph::query_builder::selection::{LoadingHandler, BoxedQuery};
                    let conn = executor.context().get_connection();
                    let query = <_ as QueryDsl>::filter(
                        <BoxedQuery<Self, #backend, __Ctx> as QueryDsl>::select(
                           <Self as LoadingHandler<#backend, __Ctx>>::build_query(look_ahead)?,
                            (
                                #key_column::default().nullable(),
                                <Self as LoadingHandler<#backend, __Ctx>>::get_select(look_ahead)?,
                            )
                       ),
                        #key_column::default().nullable().eq_any(keys),
                    );
                    <Self as wundergraph::query_builder::selection::fields::associations::WundergraphBelongsTo<
                        <#other as wundergraph::diesel::associations::HasTable>::Table,
                    #backend,
                    __Ctx,
                    #key_column
                    >>::build_response(query.load(conn)?, look_ahead, selection, executor)
            }
        }
    })
}

fn derive_non_table_filter(
    model: &Model,
    item: &syn::DeriveInput,
    backend: &TokenStream,
) -> Result<TokenStream, Diagnostic> {
    let (_, ty_generics, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics
        .params
        .push(parse_quote!(__Ctx: wundergraph::context::WundergraphContext + 'static));
    {
        let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(<__Ctx as wundergraph::context::WundergraphContext>::Connection: wundergraph::diesel::Connection<Backend = #backend>));
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let table = model.table_type()?;
    let table = &quote!(#table::table);
    let struct_type = &model.name;
    let filter = &quote! {
        <wundergraph::query_builder::selection::filter::filter_helper::FilterConverter<
            #struct_type #ty_generics, #backend, __Ctx> as wundergraph::query_builder::selection::filter::filter_helper::CreateFilter>::Filter
    };

    Ok(quote! {
        impl#impl_generics wundergraph::query_builder::selection::filter::filter_helper::BuildFilterHelper<
            #backend,
            #filter,
            __Ctx,
            > for #table
            #where_clause
        {
            type Ret = Box<dyn wundergraph::diesel_ext::BoxableFilter<#table, #backend, SqlType = wundergraph::diesel::sql_types::Bool>>;
            const FIELD_COUNT: usize = <wundergraph::query_builder::selection::filter::filter_helper::FilterBuildHelper<
                #filter,
                #struct_type  #ty_generics,
                #backend,
                __Ctx> as wundergraph::query_builder::selection::filter::inner_filter::InnerFilter>::FIELD_COUNT;

            fn into_filter(
                f: #filter,
            ) -> std::option::Option<Self::Ret> {
                use wundergraph::query_builder::selection::filter::build_filter::BuildFilter;
                BuildFilter::<#backend>::into_filter(f)
            }

            fn from_inner_look_ahead(
                objs: &[(&str, wundergraph::juniper::LookAheadValue<wundergraph::scalar::WundergraphScalarValue>)]
            ) -> #filter {
                use wundergraph::query_builder::selection::filter::inner_filter::InnerFilter;
                wundergraph::query_builder::selection::filter::filter_helper::FilterBuildHelper::<#filter, #struct_type #ty_generics, #backend, __Ctx>::from_inner_look_ahead(objs).0
            }

            fn from_inner_input_value(
                obj: wundergraph::indexmap::IndexMap<&str, &wundergraph::juniper::InputValue<wundergraph::scalar::WundergraphScalarValue>>,
            ) -> std::option::Option<#filter> {
                use wundergraph::query_builder::selection::filter::inner_filter::InnerFilter;
                std::option::Option::Some(
                    wundergraph::query_builder::selection::filter::filter_helper::FilterBuildHelper::<#filter, #struct_type #ty_generics, #backend, __Ctx>::from_inner_input_value(obj)?.0
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
                use wundergraph::query_builder::selection::filter::inner_filter::InnerFilter;
                wundergraph::query_builder::selection::filter::filter_helper::FilterBuildHelper::<#filter, #struct_type #ty_generics, #backend, __Ctx>::register_fields(&Default::default(), registry)
            }
        }
    })
}
