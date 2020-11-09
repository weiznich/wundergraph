#[doc(hidden)]
#[macro_export]
macro_rules! __expand_register_delete {
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident,) => {
        $crate::__expand_register_delete!($entity_name, $registry, $fields, $info, true)
    };
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, true) => {
        $crate::__expand_register_delete!(
            $entity_name, $registry, $fields, $info,
            $crate::helper::PrimaryKeyArgument<
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
            $crate::helper::PrimaryKeyArgument<
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
           _,
       $($delete)*,
       $entity_name,
       Self::Context,
               >($executor, $arguments, concat!("Delete", stringify!($entity_name)))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_register_insert {
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident,) => {
        $crate::__expand_register_insert!($entity_name, $registry, $fields, $info, false)
    };
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, false) => {};
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, $($insert:tt)*) => {{
        let new = $registry.arg::<$($insert)*>(concat!("New", stringify!($entity_name)), $info);
        let new = $registry
            .field::<Option<<$entity_name as $crate::graphql_type::WundergraphGraphqlMapper<_, Ctx>>::GraphQLType>>(
                concat!("Create", stringify!($entity_name)),
                $info,
            )
            .argument(new);
        $fields.push(new);
        let new =
            $registry.arg::<Vec<$($insert)*>>(concat!("New", stringify!($entity_name), "s"), $info);
        let new = $registry
            .field::<Vec<<$entity_name as $crate::graphql_type::WundergraphGraphqlMapper<_, Ctx>>::GraphQLType>>(
                concat!("Create", stringify!($entity_name), "s"),
                $info,
            )
            .argument(new);
        $fields.push(new);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_resolve_insert {
    (
        $tpe: expr,
        $entity_name: ident,
        $executor: ident,
        $arguments: ident,
        $selection: expr,
    ) => {
        $crate::__expand_resolve_insert!(
            $tpe,
            $entity_name,
            $executor,
            $arguments,
            $selection,
            false
        )
    };
    (
        $tpe: expr,
        $entity_name: ident,
        $executor: ident,
        $arguments: ident,
        $selection: expr,
        false
    ) => {
        Err($crate::juniper::FieldError::new(
            "Unknown field:",
            $crate::juniper::Value::scalar($tpe),
        ))
    };
    (
        $tpe: expr,
        $entity_name: ident,
        $executor: ident,
        $arguments: ident,
        $selection: expr,
        $($insert:tt)*
    ) => {
        if $tpe == concat!("Create", stringify!($entity_name)) {
            $crate::query_builder::mutations::handle_insert::<
                _,
                $($insert)*,
                $entity_name,
                Self::Context,
            >(
                $selection,
                $executor,
                $arguments,
                concat!("New", stringify!($entity_name)),
            )
        } else {
            $crate::query_builder::mutations::handle_batch_insert::<
                _,
                $($insert)*,
                $entity_name,
                Self::Context,
            >(
                $selection,
                $executor,
                $arguments,
                concat!("New", stringify!($entity_name), "s"),
            )
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_register_update {
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident,) => {
        $crate::__expand_register_update!($entity_name, $registry, $fields, $info, false)
    };
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, false) => {};
    ($entity_name: ident, $registry: ident, $fields: ident, $info: ident, $($update:tt)*) => {{
        let update = $registry.arg::<$($update)*>(concat!("Update", stringify!($entity_name)), $info);
        let update = $registry
            .field::<Option<<$entity_name as $crate::graphql_type::WundergraphGraphqlMapper<_, Ctx>>::GraphQLType>>(
                concat!("Update", stringify!($entity_name)),
                $info,
            )
            .argument(update);
        $fields.push(update);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_resolve_update {
    (
        $entity_name: ident,
        $executor: ident,
        $arguments: ident,
        $selection: expr,
    ) => {
        $crate::__expand_resolve_update!($entity_name, $executor, $arguments, $selection, false)
    };
    (
        $entity_name: ident,
        $executor: ident,
        $arguments: ident,
        $selection: expr,
        false
    ) => {
        Err($crate::juniper::FieldError::new(
            "Unknown field:",
            $crate::juniper::Value::scalar(concat!("Update", stringify!($entity_name))),
        ))
    };
    (
        $entity_name: ident,
        $executor: ident,
        $arguments: ident,
        $selection: expr,
        $($update:tt)*
    ) => {
        $crate::query_builder::mutations::handle_update::<_, $($update)*, $entity_name, Self::Context>(
            $selection,
            $executor,
            $arguments,
            concat!("Update", stringify!($entity_name)),
        )
    };
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
        },
        db = [$db: ty],
    ) => {
        $crate::paste::item! {
            $crate::__build_mutation_trait_bounds! {
                input = {
                    $($entity_name(
                        table = {[<$entity_name _table>]},
                        $(insert = $insert,)?
                        $(update = $update,)?
                        $(delete = {$($delete)*},)?
                    ),)*
                },
                original = [
                    mutation_name = {$($mutation_name)*},
                    structs = [$($entity_name,)*],
                    $(lt = $lt,)?
                        body = {
                            $($inner)*
                        }
                ],
                additional_bound = [],
                db = [$db],
            }
        }
    };
    (
        input = {
            $entity_name: ident (
                table = {$($table:tt)*},
                insert = false,
                $($other:tt)*
            ),
            $($rest:tt)*
        },
        original = [ $($orig: tt)* ],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db:ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(table = {$($table)*}, $($other)*),
                $($rest)*
            },
            original = [ $($orig)*],
            additional_bound = [$({$($bounds)*},)*],
            db = [$db],
        }
    };
    (
        input = {
            $entity_name: ident (
                table = {$($table:tt)*},
                insert = $insert: ident,
                $($other:tt)*
            ),
            $($rest:tt)*
        },
        original = [ $($orig: tt)* ],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db: ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(table = {$($table)*}, $($other)*),
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [
                $({$($bounds)*},)*
                {
                    $($table)*: $crate::query_builder::mutations::HandleInsert<$entity_name, $insert, $db, Ctx>
                },
                {
                    $($table)*: $crate::query_builder::mutations::HandleBatchInsert<$entity_name, $insert, $db, Ctx>
                },
            ],
            db = [$db],
        }
    };
    (
        input = {
            $entity_name: ident (
                table = {$($table:tt)*},
                update = false,
                $($other:tt)*
            ),
            $($rest:tt)*
        },
        original = [ $($orig: tt)* ],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db: ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(table = {$($table)*}, $($other)*),
                $($rest)*
            },
            original = [ $($orig)*],
            additional_bound = [$({$($bounds)*},)*],
            db = [$db],
        }
    };
    (
        input = {
            $entity_name: ident (
                table = {$($table: tt)*},
                update = $update: ident,
                $($other:tt)*
            ),
            $($rest:tt)*
        },
        original = [ $($orig: tt)* ],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db: ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(table = {$($table)*}, $($other)*),
                $($rest)*
            },
            original = [ $($orig)*],
            additional_bound = [
                $({$($bounds)*},)*
                {
                    $($table)*: $crate::query_builder::mutations::HandleUpdate<$entity_name, $update, $db, Ctx>
                },
            ],
            db = [$db],
        }
    };
    (
        input = {
            $entity_name: ident (table = {$($table:tt)*},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db: ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(table = {$($table)*}, delete = {true},),
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*],
            db = [$db],
        }
    };
    (
        input = {
            $entity_name: ident (table = {$($table:tt)*}, delete = {true},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db:ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $entity_name(
                    table = {$($table)*},
                    delete = {$crate::helper::PrimaryKeyArgument<
                        'static,
                        <$entity_name as $crate::diesel::associations::HasTable>::Table,
                        Ctx,
                        <&'static $entity_name as $crate::diesel::Identifiable>::Id
                    >},
                ),
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*],
            db = [$db],
        }
    };
    (
        input = {
            $entity_name: ident (table = {$($table:tt)*}, delete = {false},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db: ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*],
            db = [$db],
        }
    };
    (
        input = {
            $entity_name: ident (table = {$($table:tt)*}, delete = {$($delete:tt)*},),
            $($rest:tt)*
        },
        original = [ $($orig: tt)*],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db: ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            input = {
                $($rest)*
            },
            original = [$($orig)*],
            additional_bound = [$({$($bounds)*},)*
                {
                    $($table)*: $crate::query_builder::mutations::HandleDelete<$entity_name, $($delete)*, $db, Ctx>
                },
            ],
            db = [$db],
        }
    };
    (
        input = {},
        original = [
            mutation_name = {$($mutation_name:tt)*},
            structs = [
                $($entity_name: ident,)*
            ],
            $(lt = $lt: tt,)?
            body = {
                $($inner: tt)*
            }
        ],
        additional_bound = [$({$($bounds:tt)*},)*],
        db = [$db: ty],
    ) => {
        $crate::__impl_graphql_obj_for_mutation! {
            mutation_name = {$($mutation_name)*},
            structs = [
                $($entity_name,)*
            ],
            additional_bound = [$({$($bounds)*},)*],
            $(lt = $lt,)?
            body = {
                $($inner)*
            },
            db = [$db],
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "postgres")]
macro_rules! __impl_graphql_obj_for_mutation_and_db {
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
        $crate::__impl_graphql_obj_for_mutation! {
            mutation_name =  {$($mutation_name)*},
            structs = [$($entity_name (
                $(insert = $insert,)?
                $(update = $update,)?
                $(delete = $($delete)*)?
            ),)*],
            $(lt = $lt,)?
            body = {
                $($inner)*
            },
            db = [$crate::diesel::pg::Pg],
        }
    }
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "sqlite")]
macro_rules! __impl_graphql_obj_for_mutation_and_db {
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
        $crate::__impl_graphql_obj_for_mutation! {
            mutation_name =  {$($mutation_name)*},
            structs = [$($entity_name (
                $(insert = $insert,)?
                $(update = $update,)?
                $(delete = $($delete)*)?
            ),)*],
            $(lt = $lt,)?
            body = {
                $($inner)*
            },
            db = [$crate::diesel::sqlite::Sqlite],
        }
    }
}

#[macro_export]
#[doc(hidden)]
#[cfg(all(feature = "postgres", feature = "sqlite"))]
macro_rules! __impl_graphql_obj_for_mutation_and_db {
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
        $crate::__impl_graphql_obj_for_mutation! {
            mutation_name =  {$($mutation_name)*},
            structs = [$($entity_name (
                $(insert = $insert,)?
                $(update = $update,)?
                $(delete = $($delete)*)?
            ),)*],
            $(lt = $lt,)?
            body = {
                $($inner)*
            },
            db = [$crate::diesel::pg::Pg],
        }

        $crate::__impl_graphql_obj_for_mutation! {
            mutation_name =  {$($mutation_name)*},
            structs = [$($entity_name (
                $(insert = $insert,)?
                $(update = $update,)?
                $(delete = $($delete)*)?
            ),)*],
            $(lt = $lt,)?
            body = {
                $($inner)*
            },
            db = [$crate::diesel::sqlite::Sqlite],
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_graphql_obj_for_mutation {
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
        $crate::__impl_graphql_obj_for_mutation_and_db! {
           mutation_name =  {$($mutation_name)*},
            structs = [$($entity_name (
                $(insert = $insert,)?
                $(update = $update,)?
                $(delete = $($delete)*)?
            ),)*],
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
            $(delete = $($delete:tt)*)?
        ),)*],
        $(lt = $lt: tt,)?
        body = {
            $($inner: tt)*
        },
        db = [$db: ty],
    ) => {
        $crate::__build_mutation_trait_bounds! {
            mutation_name = {$($mutation_name)*},
            structs = [
                $($entity_name($(insert = $insert,)? $(update = $update,)? $(delete = $($delete)*)?),)*
            ],
            $(lt = $lt,)?
            body = {
                $($inner)*
            },
            db = [$db],
        }
    };
    (
        mutation_name = {$($mutation_name:tt)*},
        structs = [$($entity_name: ident,)*],
        additional_bound = [$({$($bounds:tt)*},)*],
        $(lt = $lt: tt,)?
        body = {
            $($inner: tt)*
        },
        db = [$db: ty],
    ) => {
        $crate::paste::item! {
            impl<$($lt,)? Ctx, $([<$entity_name _table>],)*> $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>
                for $($mutation_name)*<$($lt,)? Ctx>
            where Ctx: $crate::WundergraphContext + 'static,
                  Ctx::Connection: $crate::diesel::Connection<Backend = $db>,
                  $($entity_name: $crate::diesel::associations::HasTable<Table = [<$entity_name _table>]>,)*
                  $($entity_name: $crate::graphql_type::WundergraphGraphqlMapper<$db, Ctx>, )*
                  $(<$entity_name as $crate::graphql_type::WundergraphGraphqlMapper<$db, Ctx>>::GraphQLType: $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue, TypeInfo = (), Context = ()>,)*
                $([<$entity_name _table>]: $crate::diesel::Table +
                  'static +
                  $crate::diesel::QuerySource +
                  $crate::diesel::associations::HasTable<Table = [<$entity_name _table>]>,)*
                $([<$entity_name _table>]::FromClause: $crate::helper::NamedTable + $crate::diesel::query_builder::QueryFragment<$db>,)*
                $([<$entity_name _table>]::AllColumns: $crate::diesel::SelectableExpression<[<$entity_name _table>]>,)*
                  $($($bounds)*,)*

            {
                $($inner)*
            }
        }
    }
}

/// Macro to register the main mutation object
///
/// # Annotated example
/// ```
/// ##[macro_use]
/// # extern crate diesel;
/// # use wundergraph::WundergraphEntity;
/// # use wundergraph::query_builder::types::HasOne;
/// # use juniper::GraphQLInputObject;
/// #
/// # table! {
/// #     species {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # table! {
/// #     heros {
/// #          id -> Integer,
/// #          name -> Text,
/// #          species -> Integer,
/// #     }
/// # }
/// #
/// # #[derive(WundergraphEntity, Identifiable)]
/// # #[table_name = "species"]
/// # pub struct Species {
/// #     id: i32,
/// #     name: String,
/// # }
/// #
/// #[derive(WundergraphEntity, Identifiable)]
/// #[table_name = "heros"]
/// pub struct Hero {
///     id: i32,
///     name: String,
///     species: HasOne<i32, Species>,
/// }
///
/// #[derive(Insertable, GraphQLInputObject)]
/// #[table_name = "heros"]
/// pub struct NewHero {
///     name: String,
///     species: i32,
/// }
///
/// #[derive(AsChangeset, Identifiable, GraphQLInputObject)]
/// #[table_name = "heros"]
/// pub struct HeroChangeset {
///     id: i32,
///     name: String,
/// }
///
/// wundergraph::mutation_object! {
///     // The main mutation object. The provided name
///     // maps directly to the generated struct which
///     // could be used then as juniper GraphQL struct
///
///     /// An optional doc comment describing the main mutation object
///     /// Rendered as GraphQL description
///     Mutation {
///         // Register mutations for a wundergraph GraphQL entity
///         //
///         // Each field has a set of optional arguments:
///         //  * insert: Specifies the used insert handler.
///         //    Possible values: a struct implementing
///         //    HandleInsert and HandleBatchInsert
///         //    If not set or set to false no insert mutation is
///         //    generated for the current entity
///         //  * update: Specifies the used update handler.
///         //    Possible values: a struct implementing
///         //    HandleUpdate.
///         //    If not set or set to false no update mutation is
///         //    generated for the current entity
///         //  * delete: Specifies the used delete handler.
///         //    Possible values: true, false or a struct implementing
///         //    HandleDelete.
///         //    If not set or set to fals no delete mutation is generatet,
///         //    if set to true a default delete mutation based on the
///         //    primary keys is generated
///         //
///         // At least on of the arguments in required
///         Hero(insert = NewHero, update = HeroChangeset, delete = true),
///         Species(insert = false, update = false, delete = true)
///     }
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! mutation_object {
    (
        $(#[doc = $glob_doc: expr])*
        $mutation_name: ident {
            $($entity_name: ident (
                $(insert = $insert: ident)?
                $($(,)? update = $update: ident)?
                $($(,)? delete = $delete: ident)?
                $(,)?
            )$(,)?)*
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
                    $(delete = $delete)?
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
                    $(delete = $delete)?
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
                    $(delete = $delete)?
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
                            $crate::__expand_register_insert!(
                                $entity_name,
                                registry,
                                fields,
                                info,
                                $($insert)?
                            );
                        )*
                        $(
                            $crate::__expand_register_update!(
                                $entity_name,
                                registry,
                                fields,
                                info,
                                $($update)?
                            );
                        )*
                        $(
                            $crate::__expand_register_delete!(
                                $entity_name,
                                registry,
                                fields,
                                info,
                                $($delete)?
                            );
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
                                c @ concat!("Create", stringify!($entity_name)) |
                                c @ concat!("Create", stringify!($entity_name), "s") => {
                                    $crate::__expand_resolve_insert!(
                                        c,
                                        $entity_name,
                                        executor,
                                        arguments,
                                        self.1,
                                        $($insert)?
                                    )
                                }
                            )*
                            $(
                                concat!("Update", stringify!($entity_name)) => {
                                    $crate::__expand_resolve_update!(
                                        $entity_name,
                                        executor,
                                        arguments,
                                        self.1,
                                        $($update)*
                                    )
                                }
                            )*
                            $(
                                concat!("Delete", stringify!($entity_name)) => {
                                    $crate::__expand_resolve_delete!(
                                        $entity_name,
                                        executor,
                                        arguments,
                                        $($delete)?
                                    )
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
