use diagnostic_shim::{Diagnostic, DiagnosticShim};
use model::Model;
use proc_macro2::{Span, TokenStream};
use syn;
use utils::{inner_of_option_ty, inner_ty_args, is_has_many, wrap_in_dummy_mod};

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
                let pg = if cfg!(feature = "postgres") {
                    Some(derive_belongs_to(
                        &model,
                        item,
                        parent_ty,
                        &key_ty,
                        f.sql_name(),
                        &quote!(diesel::pg::Pg),
                    ))
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
                    ))
                } else {
                    None
                };
                match (pg, sqlite) {
                    (None, None) => panic!("One feature needs to be enabled"),
                    (Some(Ok(pg)), Some(Ok(sqlite))) => Some(Ok(quote! {#pg #sqlite})),
                    (None, Some(Ok(sqlite))) => Some(Ok(sqlite)),
                    (Some(Ok(pg)), None) => Some(Ok(pg)),
                    (Some(Err(e)), _) | (_, Some(Err(e))) => Some(Err(e)),
                }
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(wrap_in_dummy_mod(
        "wundergraph_entity",
        &model.name,
        &quote! {
            use wundergraph::diesel;
            use wundergraph::LoadingHandler;
            use wundergraph::graphql_type::WundergraphGraphqlMapper;

            #pg_loading_handler
            #sqlite_loading_handler

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
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let table = model.table_type()?;
    let field_names = model.fields().iter().map(|f| f.graphql_name());
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
                    quote!(wundergraph::query_helper::tuple::#index)
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

    Ok(quote! {

            impl #impl_generics WundergraphGraphqlMapper<#backend> for #struct_type #ty_generics
                #where_clause
            {
                type GraphQLType = wundergraph::graphql_type::GraphqlWrapper<#struct_type, #backend>;
            }

            impl #impl_generics LoadingHandler<#backend> for #struct_type #ty_generics
                #where_clause
            {
                type Columns = (#(#columns,)*);
                type FieldList = (#(#field_list,)*);

                type PrimaryKeyIndex = #primary_key_index;

                type Filter =
                    wundergraph::filter::filter_helper::FilterWrapper<
                    <wundergraph::filter::filter_helper::FilterConverter<
                              <Self::FieldList as wundergraph::query_helper::placeholder::FieldListExtractor>::Out,
                             Self::Columns,
                             Self,
                             #backend
                > as wundergraph::filter::filter_helper::CreateFilter>::Filter,
                Self,
                #backend
                >;
                const FIELD_NAMES: &'static [&'static str] = &[#(stringify!(#field_names),)*];
                const TYPE_NAME: &'static str = stringify!(#struct_type);
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

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    Ok(quote! {
        impl#impl_generics wundergraph::query_helper::placeholder::WundergraphBelongsTo<
            <#other as wundergraph::diesel::associations::HasTable>::Table,
            #backend
        > for #struct_type #ty_generics
            #where_clause
        {
            type ForeignKeyColumn = #key_column;
            type Key = #key_ty;

            fn resolve(
                selection: &wundergraph::juniper::LookAheadSelection<wundergraph::scalar::WundergraphScalarValue>,
                keys: &[std::option::Option<#key_ty>],
                conn: &impl wundergraph::diesel::Connection<Backend = #backend>,
            ) -> std::result::Result<std::collections::HashMap<
                    std::option::Option<#key_ty>,
                    std::vec::Vec<juniper::Value<WundergraphScalarValue>>
                >, wundergraph::failure::Error> {
                use wundergraph::diesel::{ExpressionMethods, RunQueryDsl, QueryDsl, NullableExpressionMethods};

                    let query = <_ as QueryDsl>::filter(
                        <_ as QueryDsl>::select(
                            Self::build_query(selection)?,
                            (
                                Self::ForeignKeyColumn::default().nullable(),
                                Self::get_select(selection)?,
                            ),
                        ),
                        Self::ForeignKeyColumn::default().nullable().eq_any(keys),
                    );
                    <Self as wundergraph::query_helper::placeholder::WundergraphBelongsTo<
                        <#other as wundergraph::diesel::associations::HasTable>::Table,
                        #backend
                    >>::build_response(query.load(conn)?, selection, conn)
            }
        }
    })
}
