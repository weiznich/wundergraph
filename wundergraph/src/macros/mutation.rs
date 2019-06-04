#[doc(hidden)]
#[macro_export]
macro_rules! __expand_register_delete {
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident,) => {
        $crate::__expand_register_delete!($entity_name, $registry, $fields, $info, true)
    };
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, true) => {
        $crate::__expand_register_delete!(
            $entity_name, $registry, $fields, $info,
            $crate::helper::primary_keys::PrimaryKeyArgument<
                'static,
                 <$entity_name as $crate::diesel::associations::HasTable>::Table,
                 Ctx,
                 <&'static $entity_name as $crate::diesel::Identifiable>::Id
            >
        )
    };
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, false) => {};
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, $($delete:tt)*) => {{
        let delete = $registry.arg::<$($delete)*>(
            concat!("Delete", stringify!($entity_name)),
            &std::default::Default::default(),
        );
        let delete = $registry.field::<Option<$crate::query_builder::mutations::DeletedCount>>(
            concat!("Delete", stringify!($entity_name)),
            $info
        ).argument(delete);
        $fields.push(delete);
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_resolve_delete {
    ($entity_name: ident, $executor: ident, $arguments: ident, ) => {
        $crate::__expand_resolve_delete!($entity_name, $executor, $arguments, true)
    };
    ($entity_name: ident, $executor: ident, $arguments: ident, true) => {
        $crate::__expand_resolve_delete!(
            $entity_name, $executor, $arguments,
            $crate::helper::primary_keys::PrimaryKeyArgument<
                'static,
                 <$entity_name as $crate::diesel::associations::HasTable>::Table,
                 Ctx,
                 <&'static $entity_name as $crate::diesel::Identifiable>::Id
            >
        )
    };
    ($entity_name: ident, $executor: ident, $arguments: ident, false) => {
        Err($crate::juniper::FieldError::new(
            "Unknown field:",
            $crate::juniper::Value::scalar(concat!("Delete", stringify!($entity_name))),
        ))
    };
    ($entity_name: ident, $executor: ident, $arguments: ident, $($delete:tt)*) => {
       $crate::query_builder::mutations::handle_delete::<
           DB,
       $($delete)*,
       $entity_name,
       Self::Context,
               >($executor, $arguments, concat!("Delete", stringify!($entity_name)))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __build_mutation_trait_bounds {
    (
        mutation_name = {$($mutation_name:tt)*},
        structs = [$($entity_name: ident(
            $(insert = $insert: ident,)?
            $(update = $update: ident,)?
            $(delete = $($delete:tt)*)?
        ),)*],
        $(lt = $lt: tt,)?
        body = {
            $($inner: tt)*
        }
    ) => {
        $crate::paste::item! {
            $crate::__build_mutation_trait_bounds! {
                input = {
                    $($entity_name($(delete = {$($delete)*},)? table = {[<$entity_name _table>]},),)*
                },
                original = [
                    mutation_name = {$($mutation_name)*},
                    structs = [$($entity_name(
                        $(insert = $insert,)?
                            $(update = $update,)?
                    ),)*],
                    $(lt = $lt,)?
                        body = {
                            $($inner)*
                        }
                ],
                additional_bound = [],
            }
        }
    };
    (
        input = {
            $entity_name: ident (table = {$($table:tt)*},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(delete = {true}, table = {$($table)*},),
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*],
        }
    };
    (
        input = {
            $entity_name: ident (delete = {true}, table = {$($table:tt)*},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(
                    delete = {$crate::helper::primary_keys::PrimaryKeyArgument<
                        'static,
                        <$entity_name as $crate::diesel::associations::HasTable>::Table,
                        Ctx,
                        <&'static $entity_name as $crate::diesel::Identifiable>::Id
                    >},
                    table = {$($table)*},
                ),
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*],
        }
    };
    (
        input = {
            $entity_name: ident (delete = {false}, table = {$($table:tt)*},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*],
        }
    };
    (
        input = {
            $entity_name: ident (delete = {$($delete:tt)*}, table = {$($table:tt)*},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*
                {
                    $($table)*: $crate::query_builder::mutations::HandleDelete<$entity_name, $($delete)*, DB, Ctx>
                },
            ],
        }
    };
    (
        input = {},
        original = [
            mutation_name = {$($mutation_name:tt)*},
           structs = [

                $($entity_name: ident(
                   $(insert = $insert: ident,)?
                   $(update = $update: ident,)?
                ),)*
           ],
           $(lt = $lt: tt,)?
            body = {
                $($inner: tt)*
            }
        ],
        additional_bound = [$({$($bounds:tt)*},)*],
    ) => {
        $crate::__impl_graphql_obj_for_mutation! {
            mutation_name = {$($mutation_name)*},
            structs = [
                $($entity_name($(insert = $insert,)? $(update = $update,)?),)*
            ],
            additional_bound = [$({$($bounds)*},)*],
            $(lt = $lt,)?
            body = {
                $($inner)*
            }
        }
    };
}


#[doc(hidden)]
#[macro_export]
macro_rules! __impl_graphql_obj_for_mutation {
    (
        mutation_name = {$($mutation_name:tt)*},
        structs = [$($entity_name: ident(
            $(insert = $insert: ident,)?
            $(update = $update: ident,)?
            $(delete = ($($delete:tt)*))?
        ),)*],
        $(lt = $lt: tt,)?
        body = {
            $($inner: tt)*
        }
    ) => {
        $crate::__build_mutation_trait_bounds! {
            mutation_name = {$($mutation_name)*},
            structs = [
                $($entity_name($(insert = $insert,)? $(update = $update,)? $(delete = $($delete)*)?),)*
            ],
            $(lt = $lt,)?
            body = {
                $($inner)*
            }
        }
    };
    (
        mutation_name = {$($mutation_name:tt)*},
        structs = [$($entity_name: ident(
            $(insert = $insert: ident,)?
            $(update = $update: ident,)?
        ),)*],
        additional_bound = [$({$($bounds:tt)*},)*],
        $(lt = $lt: tt,)?
        body = {
            $($inner: tt)*
        }
    ) => {
        $crate::paste::item! {
            impl<$($lt,)? Ctx, DB, $([<$entity_name _table>],)* $([<$entity_name _id>],)*> $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>
                for $($mutation_name)*<$($lt,)? Ctx>
            where Ctx: $crate::context::WundergraphContext,
                  DB: $crate::diesel::backend::Backend + $crate::query_builder::selection::offset::ApplyOffset + 'static,
                  DB::QueryBuilder: std::default::Default,
                  Ctx::Connection: $crate::diesel::Connection<Backend = DB>,
                  $($entity_name: $crate::query_builder::selection::LoadingHandler<DB, Ctx> + $crate::diesel::associations::HasTable<Table = [<$entity_name _table>]>,)*
                  $([<$entity_name _table>]: $crate::diesel::Table + 'static +
                      $crate::diesel::QuerySource<FromClause = $crate::diesel::query_builder::nodes::Identifier<'static>> +  $crate::diesel::Table + $crate::diesel::associations::HasTable<Table = [<$entity_name _table>]>,)*
                  $([<$entity_name _table>]::FromClause: $crate::diesel::query_builder::QueryFragment<DB>,)*
                  $(<$entity_name as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::Columns: $crate::query_builder::selection::order::BuildOrder<[<$entity_name _table>], DB>,)*
                  $(<$entity_name as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::Columns: $crate::query_builder::selection::select::BuildSelect<
                      [<$entity_name _table>],
                      DB,
                      $crate::query_builder::selection::SqlTypeOfPlaceholder<
                      <$entity_name as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList,
                      DB,
                      <$entity_name as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                      [<$entity_name _table>],
                      Ctx,
                      >
                  >,)*
                  $(<$entity_name as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList: $crate::query_builder::selection::fields::WundergraphFieldList<
                      DB,
                      <$entity_name as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                      [<$entity_name _table>],
                      Ctx
                  >,)*
                  $(<$entity_name as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList:
                      $crate::graphql_type::WundergraphGraphqlHelper<$entity_name, DB, Ctx> +
                    $crate::query_builder::selection::fields::FieldListExtractor,)*
                  $(&'static $entity_name: $crate::diesel::Identifiable<Id = [<$entity_name _id>]>,)*
                  $([<$entity_name _id>]: std::hash::Hash + std::cmp::Eq + $crate::helper::primary_keys::UnRef<'static>,)*
                  $([<$entity_name _table>]::PrimaryKey: $crate::helper::primary_keys::PrimaryKeyInputObject<
                    <[<$entity_name _id>] as $crate::helper::primary_keys::UnRef<'static>>::UnRefed, ()
                  >,)*
                  $($([<$entity_name _table>]: $crate::query_builder::mutations::HandleInsert<$entity_name, $insert, DB, Ctx>,)*)*
                  $($([<$entity_name _table>]: $crate::query_builder::mutations::HandleBatchInsert<$entity_name, $insert, DB, Ctx>,)*)*
                  $($([<$entity_name _table>]: $crate::query_builder::mutations::HandleUpdate<$entity_name, $update, DB, Ctx>,)*)*
                  $($($bounds)*,)*

            {
                $($inner)*
            }
        }
    }
}

#[macro_export]
macro_rules! mutation_object {
    (
        $(#[doc = $glob_doc: expr])*
        $mutation_name: ident {
            $($entity_name: ident (
                $(insert = $insert: ident,)?
                $(update = $update: ident,)?
                $(delete = $($delete: tt)*)?
                $(,)?
            ),)*
        }
    ) => {
        // Use Arc<Mutex<C>> here to force make this Sync
        #[derive(Debug)]
        $(#[doc = $glob_doc])*
        pub struct $mutation_name<C>(::std::marker::PhantomData<std::sync::Arc<std::sync::Mutex<C>>>);


        impl<P> Default for $mutation_name<P> {
            fn default() -> Self {
                $mutation_name(::std::marker::PhantomData)
            }
        }

        $crate::paste::item! {
            $crate::__impl_graphql_obj_for_mutation! {
                mutation_name = {$mutation_name},
                structs = [$($entity_name(
                    $(insert = $insert,)?
                    $(update = $update,)?
                    $(delete = ($($delete)*))?
                ),)*],
                body = {
                    type Context = Ctx;

                    type TypeInfo = ();

                    fn name(info: &Self::TypeInfo) -> ::std::option::Option<&str> {
                        <[<$mutation_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::name(info)
                    }

                    fn meta<'r>(
                        info: &Self::TypeInfo,
                        registry: &mut $crate::juniper::Registry<'r, $crate::scalar::WundergraphScalarValue>
                    ) -> $crate::juniper::meta::MetaType<'r, $crate::scalar::WundergraphScalarValue>
                    where
                        $crate::scalar::WundergraphScalarValue: 'r
                    {
                        <[<$mutation_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::meta(info, registry)
                    }

                    fn resolve_field(
                        &self,
                        info: &Self::TypeInfo,
                        field_name: &str,
                        arguments: &$crate::juniper::Arguments<$crate::scalar::WundergraphScalarValue>,
                        executor: &$crate::juniper::Executor<Self::Context, $crate::scalar::WundergraphScalarValue>,
                    ) -> $crate::juniper::ExecutionResult<$crate::scalar::WundergraphScalarValue> {
                        let wrapper = [<$mutation_name _wrapper>](
                            ::std::marker::PhantomData,
                            field_name,
                            arguments,
                        );
                        executor.resolve(info, &wrapper)
                    }
                }
            }

            #[derive(Debug)]
            #[doc(hidden)]
            /// An internal helper type
            pub struct [<$mutation_name _wrapper>]<'a, C>(
                // Use Arc<Mutex<C>> here to force make this Sync
                ::std::marker::PhantomData<std::sync::Arc<std::sync::Mutex<C>>>,
                &'a str,
                &'a $crate::juniper::Arguments<'a, $crate::scalar::WundergraphScalarValue>,
            );

            $crate::__impl_graphql_obj_for_mutation! {
                mutation_name = {[<$mutation_name _wrapper>]},
                structs = [$($entity_name(
                    $(insert = $insert,)?
                    $(update = $update,)?
                    $(delete = ($($delete)*))?
                ),)*],
                lt = 'a,
                body = {
                    type Context = Ctx;

                    type TypeInfo = ();

                    fn name(info: &Self::TypeInfo) -> ::std::option::Option<&str> {
                        <[<$mutation_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::name(info)
                    }

                    fn meta<'r>(
                        info: &Self::TypeInfo,
                        registry: &mut $crate::juniper::Registry<'r, $crate::scalar::WundergraphScalarValue>
                    ) -> $crate::juniper::meta::MetaType<'r, $crate::scalar::WundergraphScalarValue>
                    where
                        $crate::scalar::WundergraphScalarValue: 'r
                    {
                        <[<$mutation_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::meta(info, registry)
                    }

                    fn resolve(
                        &self,
                        info: &Self::TypeInfo,
                        selection_set: ::std::option::Option<&[$crate::juniper::Selection<$crate::scalar::WundergraphScalarValue>]>,
                        executor: &$crate::juniper::Executor<Self::Context, $crate::scalar::WundergraphScalarValue>,
                    ) -> $crate::juniper::Value<$crate::scalar::WundergraphScalarValue> {
                        let inner = [<$mutation_name _inner>] (
                            ::std::marker::PhantomData,
                            selection_set
                        );
                        let r = <[<$mutation_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::resolve_field(
                            &inner,
                            info,
                            self.1,
                            self.2,
                            executor
                        );
                        match r {
                            ::std::result::Result::Ok(v) => v,
                            ::std::result::Result::Err(e) => {
                                executor.push_error(e);
                                $crate::juniper::Value::null()
                            }
                        }
                    }
                }
            }


            #[doc(hidden)]
            #[derive(Debug)]
            /// An internal helper type
            pub struct [<$mutation_name _inner>]<'a, C>(
                // Use Arc<Mutex<C>> here to force make this Sync
                ::std::marker::PhantomData<std::sync::Arc<std::sync::Mutex<C>>>,
                ::std::option::Option<&'a [$crate::juniper::Selection<'a, $crate::scalar::WundergraphScalarValue>]>,
            );

            $crate::__impl_graphql_obj_for_mutation! {
                mutation_name = {[<$mutation_name _inner>]},
                structs = [$($entity_name(
                    $(insert = $insert,)?
                    $(update = $update,)?
                    $(delete = ($($delete)*))?
                ),)*],
                lt = 'a,
                body = {
                    type Context = Ctx;
                    type TypeInfo = ();

                    fn name(_info: &Self::TypeInfo) -> Option<&str> {
                        Some(stringify!($mutation_name))
                    }

                    #[allow(non_snake_case)]
                    fn meta<'r>(
                        info: &Self::TypeInfo,
                        registry: &mut $crate::juniper::Registry<'r, $crate::scalar::WundergraphScalarValue>
                    ) -> $crate::juniper::meta::MetaType<'r, $crate::scalar::WundergraphScalarValue>
                    where $crate::scalar::WundergraphScalarValue: 'r
                    {
                        let mut fields = Vec::new();
                        $(
                            $(
                                let new = registry.arg::<$insert>(concat!("New", stringify!($entity_name)), info);
                                let new = registry.field::<Option<$crate::graphql_type::GraphqlWrapper<$entity_name, DB, Ctx>>>(
                                    concat!("Create", stringify!($entity_name)),
                                    info
                                ).argument(new);
                                fields.push(new);
                                let new = registry.arg::<Vec<$insert>>(concat!("New", stringify!($entity_name), "s"), info);
                                let new = registry.field::<Vec<$crate::graphql_type::GraphqlWrapper<$entity_name, DB, Ctx>>>(
                                    concat!("Create", stringify!($entity_name), "s"),
                                    info
                                )
                                    .argument(new);
                                fields.push(new);
                            )*
                        )*
                            $(
                                $(
                                    let update = registry.arg::<$update>(concat!("Update", stringify!($entity_name)), info);
                                    let update = registry.field::<Option<$crate::graphql_type::GraphqlWrapper<$entity_name, DB, Ctx>>>(
                                        concat!("Update", stringify!($entity_name)),
                                        info
                                    ).argument(update);
                                    fields.push(update);
                                )*
                            )*
                            $(
                                $crate::__expand_register_delete!($entity_name, registry, fields, info, $($($delete)*)?);
                            )*
                            let mut mutation = registry.build_object_type::<Self>(info, &fields);
                        mutation = mutation.description(concat!($($glob_doc, "\n",)* ""));
                        $crate::juniper::meta::MetaType::Object(mutation)
                    }

                    fn resolve_field(
                        &self,
                        _info: &Self::TypeInfo,
                        field_name: &str,
                        arguments: &$crate::juniper::Arguments<$crate::scalar::WundergraphScalarValue>,
                        executor: &$crate::juniper::Executor<Self::Context, $crate::scalar::WundergraphScalarValue>,
                    ) -> $crate::juniper::ExecutionResult<$crate::scalar::WundergraphScalarValue> {
                        match field_name {
                            $(
                                $(
                                    concat!("Create", stringify!($entity_name)) => {
                                        $crate::query_builder::mutations::handle_insert::<
                                            DB,
                                        $insert,
                                        $entity_name,
                                        Self::Context>
                                            (
                                                self.1,
                                                executor,
                                                arguments,
                                                concat!("New", stringify!($entity_name))
                                            )
                                    }
                                    concat!("Create", stringify!($entity_name), "s") => {
                                        $crate::query_builder::mutations::handle_batch_insert::<
                                            DB,
                                        $insert,
                                        $entity_name,
                                        Self::Context>
                                            (
                                                self.1,
                                                executor,
                                                arguments,
                                                concat!("New", stringify!($entity_name), "s")
                                            )
                                    }
                                )*
                            )*
                                $(
                                    $(
                                        concat!("Update", stringify!($entity_name)) => {
                                            $crate::query_builder::mutations::handle_update::<
                                                DB,
                                            $update,
                                            $entity_name,
                                            Self::Context
                                                >(
                                                    self.1,
                                                    executor,
                                                    arguments,
                                                    concat!("Update", stringify!($entity_name))
                                                )
                                        }
                                    )*
                                )*
                                $(
                                    concat!("Delete", stringify!($entity_name)) => {
                                        $crate::__expand_resolve_delete!($entity_name, executor, arguments, $($($delete)*)?)
                                    }
                                 )*
                                e => Err($crate::juniper::FieldError::new(
                                    "Unknown field:",
                                    $crate::juniper::Value::scalar(e),
                                )),
                        }
                    }
                }
            }
        }
    };
}
