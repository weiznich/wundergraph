use diagnostic_shim::Diagnostic;
use model::Model;
use quote;
use syn;
use utils::{inner_of_option_ty, inner_ty_arg, is_has_many, is_has_one, is_option_ty,
            wrap_in_dummy_mod};

pub fn derive(item: &syn::DeriveInput) -> Result<quote::Tokens, Diagnostic> {
    let model = Model::from_item(item)?;
    let graphql_type = derive_graphql_object(&model, item)?;
    let loading_handler = derive_loading_handler(&model, item)?;

    let dummy_mod = model.dummy_mod_name("wundergraph_entity");
    Ok(wrap_in_dummy_mod(
        dummy_mod,
        &quote!{
            #graphql_type
            #loading_handler
        },
    ))
}

fn apply_filter(model: &Model) -> Option<quote::Tokens> {
    if let Some(filter) = model.filter_type() {
        Some(quote!{
           if let Some(f) = select.argument("filter") {
               source = <self::wundergraph::filter::Filter<#filter, <Self as diesel::associations::HasTable>::Table> as
                   self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(f.value())
                   .ok_or(WundergraphError::CouldNotBuildFilterArgument)?
                   .apply_filter(source);
           }
        })
    } else {
        None
    }
}

fn apply_limit(model: &Model) -> Option<quote::Tokens> {
    if model.should_have_limit() {
        Some(quote!{
            if let Some(l) = select.argument("limit") {
                source = source.limit(<i32 as self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(l.value())
                                      .ok_or(WundergraphError::CouldNotBuildFilterArgument)?
                                      as i64);
            }
        })
    } else {
        None
    }
}

fn apply_offset(model: &Model) -> Option<quote::Tokens> {
    if model.should_have_offset() {
        Some(quote!{
            if let Some(o) = select.argument("offset") {
                source = source.offset(<i32 as self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(o.value())
                                       .ok_or(WundergraphError::CouldNotBuildFilterArgument)?
                                       as i64);
            }
        })
    } else {
        None
    }
}

fn apply_order(model: &Model) -> Result<Option<quote::Tokens>, Diagnostic> {
    if model.should_have_order() {
        let table = model.table_type()?;
        let fields = model
            .fields()
            .iter()
            .filter_map(|f| {
                if f.has_flag("skip") || is_has_one(&f.ty) || is_has_many(&f.ty) {
                    None
                } else {
                    let field_name = &f.name;
                    Some(quote!{
                        (stringify!(#field_name), self::wundergraph::order::Order::Desc) => {
                            source = source.then_order_by(diesel::ExpressionMethods::desc(#table::#field_name));
                        }
                        (stringify!(#field_name), self::wundergraph::order::Order::Asc) => {
                            source = source.then_order_by(diesel::ExpressionMethods::asc(#table::#field_name));
                        }
                    })
                }
            })
            .collect::<Vec<_>>();
        if fields.is_empty() {
            Ok(None)
        } else {
            Ok(Some(quote!{
                if let Some(o) = select.argument("order") {
                    let order: Vec<_> = <Vec<self::wundergraph::order::OrderBy> as self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(o.value())
                        .ok_or(WundergraphError::CouldNotBuildFilterArgument)?;
                    for o in order {
                        match (&o.column as &str, o.direction) {
                            #(#fields)*
                            (s, _) => {
                                return Err(failure::Error::from(WundergraphError::UnknownDatabaseField {
                                    name:s.to_owned()
                                }));
                            }
                        }
                    }
                }
            }))
        }
    } else {
        Ok(None)
    }
}

fn handle_has_many(model: &Model, field_count: usize) -> Vec<quote::Tokens> {
    model
        .fields()
        .iter()
        .filter_map(|f| {
            if f.has_flag("skip") || !is_has_many(&f.ty) {
                None
            } else {
                let field_name = &f.name;
                let parent_ty = inner_ty_arg(&f.ty, "HasMany", 0);
                let field_access = f.name.access();
                let inner = quote! {
                    let query = <#parent_ty as LoadingHandler<_>>::default_query().into_boxed();
                    let p = {
                        let ids = ret.iter().map(diesel::Identifiable::id).collect::<Vec<_>>();
                        let eq = diesel::expression_methods::ExpressionMethods::eq_any(
                            <#parent_ty as diesel::associations::BelongsTo<Self>>::foreign_key_column(),
                            &ids
                        );
                        let query = diesel::query_dsl::methods::FilterDsl::filter(
                            query,
                            eq
                        );
                        <#parent_ty as LoadingHandler<_>>::load_items(
                            select,
                            ctx,
                            query)?
                    };
                    let p = <_ as diesel::GroupedBy<_>>::grouped_by(p, &ret);
                    for (c, p) in ret.iter_mut().zip(p.into_iter()) {
                        c#field_access = self::wundergraph::query_helper::HasMany::Items(p);
                    }
                };
                if field_count > 1 {
                    Some(quote!{
                        if let Some(select) =
                            <_ as self::wundergraph::juniper::LookAheadMethods>::select_child(
                                select,
                                stringify!(#field_name),
                            ) {
                            #inner
                        }
                    })
                } else {
                    Some(inner)
                }
            }
        })
        .collect()
}

fn handle_has_one(model: &Model, field_count: usize) -> Result<Vec<quote::Tokens>, Diagnostic> {
    model
        .fields()
        .iter()
        .filter_map(|f| {
            if f.has_flag("skip") {
                None
            } else if let Some(child_ty) = inner_ty_arg(&f.ty, "HasOne", 1) {
                let field_name = &f.name;
                let child_ty = inner_of_option_ty(child_ty);
                let id_ty = inner_ty_arg(&f.ty, "HasOne", 0).expect("Is HasOne, so this exists");
                let field_access = f.name.access();
                let table = f.remote_table()
                    .map(|t| quote!(#t::table))
                    .unwrap_or_else(|_| {
                        let remote_type = inner_of_option_ty(
                            inner_ty_arg(&f.ty, "HasOne", 1).expect("It's HasOne"),
                        );
                        quote!{
                            <#remote_type as diesel::associations::HasTable>::Table
                        }
                    });
                let map_fn = if is_option_ty(id_ty) {
                    quote!(filter_map)
                } else {
                    quote!(map)
                };
                let collect_ids = quote!{
                    let ids = ret
                        .iter()
                        .#map_fn(|i| *i#field_access.expect_id("Id is there"))
                        .collect::<Vec<_>>();
                };
                let lookup_and_assign = if is_option_ty(id_ty) {
                    quote!{
                        if let Some(id) = id {
                            if let Some(c) = items.get(&id).cloned() {
                                i#field_access = self::wundergraph::query_helper::HasOne::Item(Some(c));
                            }
                        } else {
                            i#field_access = self::wundergraph::query_helper::HasOne::Item(None);
                        }
                    }
                } else {
                    quote!{
                        if let Some(c) = items.get(&id).cloned() {
                            i#field_access = self::wundergraph::query_helper::HasOne::Item(c);
                        }
                    }
                };
                let inner = quote!{
                    #collect_ids
                    let items = <#child_ty as LoadingHandler<_>>::load_items(
                        select,
                        ctx,
                        <#child_ty as LoadingHandler<_>>::default_query()
                            .filter(<_ as diesel::ExpressionMethods>::eq_any(
                                <_ as diesel::Table>::primary_key(&<#table as diesel::associations::HasTable>::table()),
                                ids)).into_boxed()
                    )?.into_iter()
                        .map(|c| (*<_ as diesel::Identifiable>::id(&c), c))
                        .collect::<self::std::collections::HashMap<_, _>>();
                    for i in &mut ret {
                        let id = *i#field_access.expect_id("Id is there");
                        #lookup_and_assign
                    }
                };
                if field_count > 1 {
                    Some(Ok(quote!{
                        if let Some(select) = <_ as self::wundergraph::juniper::LookAheadMethods>::select_child(
                            select,
                            stringify!(#field_name)
                        ) {
                            #inner
                        }
                    }))
                } else {
                    Some(Ok(inner))
                }
            } else {
                None
            }
        })
        .collect()
}

fn impl_loading_handler(
    item: &syn::DeriveInput,
    backend: &quote::Tokens,
    filter: Option<&quote::Tokens>,
    limit: Option<&quote::Tokens>,
    offset: Option<&quote::Tokens>,
    order: Option<&quote::Tokens>,
    remote_fields: &[quote::Tokens],
    context: syn::Path,
    query_modifier: &syn::Path,
    select: Option<&quote::Tokens>,
) -> quote::Tokens {
    let item_name = item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let (query, query_ty, sql_ty) = if let Some(select) = select {
        let query =
            quote!(<Self::Table as diesel::associations::HasTable>::table().select(#select));
        let query_ty = quote!(
            diesel::dsl::Select<Self::Table, #select>
        );
        let sql_ty = quote!(diesel::dsl::SqlTypeOf<#select>);
        (query, query_ty, sql_ty)
    } else {
        let query = quote!(<Self::Table as diesel::associations::HasTable>::table());
        let query_ty = quote!(Self::Table);
        let sql_ty = quote!(<<Self as diesel::associations::HasTable>::Table as diesel::query_builder::AsQuery>::SqlType);
        (query, query_ty, sql_ty)
    };
    quote!{
        #[allow(unused_mut)]
        impl#impl_generics LoadingHandler<#backend> for #item_name #ty_generics
            #where_clause
        {
            type Query = #query_ty;
            type SqlType = #sql_ty;
            type QueryModifier = #query_modifier;
            type Context = #context;

            fn load_items<'a>(
                select: &LookAheadSelection,
                ctx: &Self::Context,
                mut source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, #backend>,
            ) -> Result<Vec<Self>, failure::Error>
            {
                use wundergraph::juniper::LookAheadMethods;

                let modifier = <Self::QueryModifier as BuildQueryModifier<Self>>::from_ctx(ctx)?;
                let conn = ctx.get_connection();
                #filter

                #limit
                #offset

                #order
                source = modifier.modify_query(source, select)?;
                println!("{}", diesel::debug_query(&source));
                let mut ret: Vec<Self> = source.load(conn)?;

                #(#remote_fields)*

                Ok(ret)
            }

            fn default_query() -> Self::Query {
                #query
            }
        }
    }
}

fn derive_loading_handler(
    model: &Model,
    item: &syn::DeriveInput,
) -> Result<quote::Tokens, Diagnostic> {
    //    let item_name = item.ident;

    let field_count = model
        .fields()
        .iter()
        .filter(|f| !f.has_flag("skip"))
        .count();

    let filter = apply_filter(model);
    let limit = apply_limit(model);
    let offset = apply_offset(model);
    let order = apply_order(model)?;
    let has_many = handle_has_many(model, field_count);
    let has_one = handle_has_one(model, field_count)?;
    let mut remote_fields = has_many;
    remote_fields.extend(has_one);
    let query_modifier = model.query_modifier_type();
    let select = model.select();
    let table = model.table_type()?;
    let select = if select.is_empty() {
        None
    } else {
        let select = select.into_iter().map(|s| quote!{#table::#s});
        Some(quote!((#(#select,)*)))
    };

    let pg = if cfg!(feature = "postgres") {
        let context = model.context_type(parse_quote!(diesel::PgConnection))?;
        Some(impl_loading_handler(
            item,
            &quote!(diesel::pg::Pg),
            filter.as_ref(),
            limit.as_ref(),
            offset.as_ref(),
            order.as_ref(),
            &remote_fields,
            context,
            &query_modifier,
            select.as_ref(),
        ))
    } else {
        None
    };

    let sqlite = if cfg!(feature = "sqlite") {
        let context = model.context_type(parse_quote!(diesel::SqliteConnection))?;
        Some(impl_loading_handler(
            item,
            &quote!(diesel::sqlite::Sqlite),
            filter.as_ref(),
            limit.as_ref(),
            offset.as_ref(),
            order.as_ref(),
            &remote_fields,
            context,
            &query_modifier,
            select.as_ref(),
        ))
    } else {
        None
    };

    Ok(quote!{
        use self::wundergraph::error::WundergraphError;
        use self::wundergraph::LoadingHandler;
        use self::wundergraph::diesel::{RunQueryDsl, QueryDsl, self};

        #pg
        #sqlite
    })
}

fn derive_graphql_object(
    model: &Model,
    item: &syn::DeriveInput,
) -> Result<quote::Tokens, Diagnostic> {
    let item_name = item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let field_count = model
        .fields()
        .iter()
        .filter(|f| !f.has_flag("skip"))
        .count();
    if field_count == 1 {
        let field = model
            .fields()
            .iter()
            .find(|f| !f.has_flag("skip"))
            .expect("This exists because we have at least one field");

        let ty = &field.ty;
        let field_access = field.name.access();
        Ok(quote!{
            use self::wundergraph::juniper::{GraphQLType, Registry, Arguments, Executor, ExecutionResult, Selection, Value};
            use self::wundergraph::juniper::meta::MetaType;

            impl #impl_generics GraphQLType for #item_name #ty_generics
                #where_clause
            {
                type Context = ();
                type TypeInfo = ();

                fn name(info: &Self::TypeInfo) -> Option<&str> {
                    <#ty as GraphQLType>::name(info)
                }

                fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
                    <#ty as GraphQLType>::meta(info, registry)
                }

                fn resolve_field(
                    &self,
                    info: &Self::TypeInfo,
                    field_name: &str,
                    arguments: &Arguments,
                    executor: &Executor<Self::Context>,
                ) -> ExecutionResult {
                    <#ty as GraphQLType>::resolve_field(&self#field_access,
                                                        info,
                                                        field_name,
                                                        arguments,
                                                        executor)
                }

                fn resolve_into_type(
                    &self,
                    info: &Self::TypeInfo,
                    type_name: &str,
                    selection_set: Option<&[Selection]>,
                    executor: &Executor<Self::Context>,
                ) -> ExecutionResult {
                    <#ty as GraphQLType>::resolve_into_type(&self#field_access,
                                                            info,
                                                            type_name,
                                                            selection_set,
                                                            executor)
                }

                fn concrete_type_name(&self, context: &Self::Context, info: &Self::TypeInfo) -> String {
                    <#ty as GraphQLType>::concrete_type_name(&self#field_access,
                                                             context,
                                                             info)
                }

                fn resolve(
                    &self,
                    info: &Self::TypeInfo,
                    selection_set: Option<&[Selection]>,
                    executor: &Executor<Self::Context>,
                ) -> Value {
                    <#ty as GraphQLType>::resolve(&self#field_access, info, selection_set, executor)
                }
            }
        })
    } else {
        let register_fields = model
            .fields()
            .iter()
            .filter_map(|f| {
                if f.has_flag("skip") {
                    None
                } else {
                    let field_name = &f.name;
                    let field_ty = &f.ty;
                    let field = quote!{
                        let #field_name = registry.field::<#field_ty>(stringify!(#field_name), info);
                    };

                    if let Some(filter) = f.filter() {
                        if is_has_many(&f.ty) {
                            let table = f.remote_table().map(|t| quote!(#t::table)).unwrap_or_else(
                                |_| {
                                    let remote_type =
                                        inner_ty_arg(&f.ty, "HasMany", 0).expect("It is HasMany");
                                    quote!(<<#remote_type as diesel::associations::BelongsTo<#item_name>>::ForeignKeyColumn as diesel::Column>::Table)
                                },
                            );
                            Some(Ok(quote!{
                                let filter = registry.arg_with_default::<Option<
                                    self::wundergraph::filter::Filter<
                                    #filter,  #table>>>(
                                        "filter",
                                        &None,
                                        &Default::default(),
                                    );
                                #field
                                let #field_name = #field_name.argument(filter)
                            }))
                        } else {
                            Some(Ok(field))
                        }
                    } else {
                        Some(Ok(field))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let fields = model.fields().iter().filter_map(|f| {
            if f.has_flag("skip") {
                None
            } else {
                let field_name = &f.name;
                Some(quote!(#field_name))
            }
        });

        let resolve_field = model.fields().iter().filter_map(|f| {
            if f.has_flag("skip") {
                None
            } else {
                let field_access = f.name.access();
                let field_name = &f.name;
                Some(quote!(stringify!(#field_name) => executor.resolve(info, &self#field_access)))
            }
        });

        Ok(quote! {
            use self::wundergraph::juniper::{GraphQLType, Registry, Arguments,
                                             Executor, ExecutionResult, FieldError, Value};
            use self::wundergraph::juniper::meta::MetaType;

            impl #impl_generics GraphQLType for #item_name #ty_generics
                #where_clause
            {
                type Context = ();
                type TypeInfo = ();

                fn name(_info: &Self::TypeInfo) -> Option<&str> {
                    Some(stringify!(#item_name))
                }

                fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
                    #(#register_fields;)*

                    let ty = registry.build_object_type::<Self>(
                        info,
                        &[#(#fields,)*]
                    );
                    MetaType::Object(ty)
                }

                fn resolve_field(
                    &self,
                    info: &Self::TypeInfo,
                    field_name: &str,
                    _arguments: &Arguments,
                    executor: &Executor<Self::Context>,
                ) -> ExecutionResult {
                    match field_name {
                        #(#resolve_field,)*

                        e => Err(FieldError::new(
                            "Unknown field:",
                            Value::String(e.to_owned()),
                        )),
                    }
                }

                fn concrete_type_name(&self, _context: &Self::Context, _info: &Self::TypeInfo) -> String {
                    String::from(stringify!(#item_name))
                }
            }
        })
    }
}
