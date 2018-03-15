use quote;
use syn;
use diagnostic_shim::Diagnostic;
use utils::{inner_of_option_ty, inner_ty_arg, is_has_many, is_has_one, is_option_ty,
            wrap_in_dummy_mod};
use model::Model;

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

fn derive_loading_handler(
    model: &Model,
    item: &syn::DeriveInput,
) -> Result<quote::Tokens, Diagnostic> {
    let item_name = item.ident;
    let table = model.table_type()?;

    let (_, ty_generics, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    generics.params.push(parse_quote!(__C));
    {
        // TODO: improve this
        // maybe try to remove the explicit Backend bound and
        // replace it with with the next level of bounds?
        let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(__C: Connection<Backend = DB> + 'static));
        where_clause
            .predicates
            .push(parse_quote!(DB: Backend + Clone + 'static));
        where_clause
            .predicates
            .push(parse_quote!(Self: Queryable<<#table::table as AsQuery>::SqlType, __C::Backend>));
        where_clause
            .predicates
            .push(parse_quote!(DB::QueryBuilder: Default));

        // TODO: add more types
        let supported_types = [
            (quote!(i32), quote!(Integer)),
            (quote!(String), quote!(Text)),
            (quote!(bool), quote!(Bool)),
            (quote!(f64), quote!(Double)),
            (quote!(i16), quote!(SmallInt)),
        ];
        for &(ref rust_ty, ref diesel_ty) in &supported_types {
            where_clause
                .predicates
                .push(parse_quote!(#rust_ty: FromSql<#diesel_ty, DB>));
        }
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let filter = if let Some(filter) = model.filter_type() {
        Some(quote!{
           if let Some(f) = select.argument("filter") {
               source = <self::wundergraph::filter::Filter<#filter, DB, #table::table> as
                   self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(f.value())
                   .ok_or(Error::CouldNotBuildFilterArgument)?
                   .apply_filter(source);
           }
        })
    } else {
        None
    };

    let limit = if model.should_have_limit() {
        Some(quote!{
            if let Some(l) = select.argument("limit") {
                source = source.limit(<i32 as self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(l.value())
                                      .ok_or(Error::CouldNotBuildFilterArgument)?
                                      as i64);
            }
        })
    } else {
        None
    };

    let offset = if model.should_have_offset() {
        Some(quote!{
            if let Some(o) = select.argument("offset") {
                source = source.offset(<i32 as self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(o.value())
                                       .ok_or(Error::CouldNotBuildFilterArgument)?
                                       as i64);
            }
        })
    } else {
        None
    };

    let order = if model.should_have_order() {
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
            None
        } else {
            Some(quote!{
                if let Some(o) = select.argument("order") {
                    let order: Vec<_> = <Vec<self::wundergraph::order::OrderBy> as self::wundergraph::helper::FromLookAheadValue>::from_look_ahead(o.value())
                        .ok_or(Error::CouldNotBuildFilterArgument)?;
                    for o in order {
                        match (&o.column as &str, o.direction) {
                            #(#fields)*
                            (s, _) => {
                                return Err(Error::UnknownDatabaseField(s.to_owned()));
                            }
                        }
                    }
                }
            })
        }
    } else {
        None
    };
    let field_count = model
        .fields()
        .iter()
        .filter(|f| !f.has_flag("skip"))
        .count();

    let has_many = model.fields().iter().filter_map(|f| {
        if f.has_flag("skip") || !is_has_many(&f.ty) {
            None
        } else {
            let field_name = &f.name;
            let parent_ty = inner_ty_arg(&f.ty, "HasMany", 0);
            let field_access = f.name.access();
            let inner = quote!{
                let p = <#parent_ty as LoadingHandler<_>>::load_item(
                    select,
                    conn,
                    <#parent_ty as diesel::BelongingToDsl<_>>::belonging_to(&ret).into_boxed())?;
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
    });

    let has_one = model
        .fields()
        .iter()
        .filter_map(|f| {
            if f.has_flag("skip") || !is_has_one(&f.ty) {
                None
            } else {
                let field_name = &f.name;
                let child_ty = inner_ty_arg(&f.ty, "HasOne", 1).expect("Is HasOne, so this exists");
                let child_ty = inner_of_option_ty(child_ty);
                let id_ty = inner_ty_arg(&f.ty, "HasOne", 0).expect("Is HasOne, so this exists");
                let field_access = f.name.access();
                let table = match f.remote_table() {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let collect_ids = if is_option_ty(id_ty) {
                    quote!{
                        let ids = ret
                            .iter()
                            .filter_map(|i| *i#field_access.expect_id("Id is there"))
                            .collect::<Vec<_>>();
                    }
                } else {
                    quote!{
                        let ids = ret
                            .iter()
                            .map(|i| *i#field_access.expect_id("Id is there"))
                            .collect::<Vec<_>>();
                    }
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
                    let items = <#child_ty as LoadingHandler<_>>::load_item(
                        select,
                        conn,
                        #table::table.filter(<_ as diesel::ExpressionMethods>::eq_any(#table::id, ids)).into_boxed()
                    )?.into_iter()
                        .map(|c| (c.id, c))
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
            }
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;

    Ok(quote!{
        use self::wundergraph::error::Error;
        use self::wundergraph::LoadingHandler;
        use self::wundergraph::diesel::query_builder::{AsQuery, BoxedSelectStatement};
        use self::wundergraph::diesel::{self, Connection, Queryable, QueryDsl};
        use self::wundergraph::diesel::RunQueryDsl;
        use self::wundergraph::diesel::backend::Backend;
        use self::wundergraph::diesel::sql_types::{Bool, Text, Integer, Double, SmallInt};
        use self::wundergraph::juniper::LookAheadSelection;

        #[allow(unused_mut)]
        impl#impl_generics LoadingHandler<__C> for #item_name #ty_generics
            #where_clause
        {
            type Table = #table::table;
            type SqlType = <#table::table as AsQuery>::SqlType;

            fn load_item<'a>(
                select: &LookAheadSelection,
                conn: &__C,
                mut source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, __C::Backend>,
            ) -> Result<Vec<Self>, Error> {
                #filter

                #limit
                #offset

                #order
                println!("{}", diesel::debug_query(&source));
                let mut ret: Vec<Self> = source.load(conn)?;

                #(#has_many)*
                #(#has_one)*

                Ok(ret)
            }
        }
    })
}

fn derive_graphql_object(
    model: &Model,
    item: &syn::DeriveInput,
) -> Result<quote::Tokens, Diagnostic> {
    let item_name = item.ident;
    let (impl_generics, ty_generics, _) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    {
        // TODO: improve this
        // maybe try to remove the explicit Backend bound and
        // replace it with with the next level of bounds?
        let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(DB: Backend + 'static));
    }
    let (_, _, where_clause) = generics.split_for_impl();

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
            use self::wundergraph::juniper::{GraphQLType, Registry, Arguments, Executor,
                                             ExecutionResult, Selection, Value};
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
                            let table = match f.remote_table() {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            };
                            Some(Ok(quote!{
                                let filter = registry.arg_with_default::<Option<
                                    self::wundergraph::filter::Filter<
                                    #filter, DB, #table::table>>>(
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
