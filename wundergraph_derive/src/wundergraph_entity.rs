use crate::build_filter_helper::derive_non_table_filter;
use crate::diagnostic_shim::{Diagnostic, DiagnosticShim};
use crate::field::Field;
use crate::model::Model;
use crate::utils::{is_has_many, wrap_in_dummy_mod};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

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

    let belongs_to = crate::belonging_to::derive_belonging_to(&model, item)?;

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
                .find(|(_, f)| f.sql_name() == primary_key)
                .map(|(i, _)| {
                    let index = syn::Ident::new(&format!("TupleIndex{}", i), Span::call_site());
                    quote!(wundergraph::helper::#index)
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
                wundergraph::query_builder::selection::filter::FilterWrapper<Self, #backend, __Ctx>
            }
        },
        |p| quote!(#p),
    );

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

            fn type_info() -> () {
                ()
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
            const TYPE_DESCRIPTION: std::option::Option<&'static str> = #type_description;

            fn field_description(idx: usize) -> std::option::Option<&'static str> {
                match idx {
                    #(#description,)*
                    _ => std::option::Option::None,
                }
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
