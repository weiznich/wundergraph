#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_delete {
    ($registry: ident, $entity: ident, $fields: ident, $info: expr, $field_info: expr, $context: ty, ) => {
        __wundergraph_expand_delete!{$registry, $entity, $fields, $info, $field_info, $context, delete = true}
    };
    ($registry: ident, $entity: ident, $fields: ident, $info: expr, $field_info: expr, $context: ty, delete = false) => {};
    ($registry: ident, $entity: ident, $fields: ident, $info: expr, $field_info: expr, $context: ty, delete = true) => {
        __wundergraph_expand_delete!{$registry, $entity, $fields,
                                     &$crate::helper::primary_keys::PrimaryKeyInfo::new(
                                         <$entity as $crate::diesel::associations::HasTable>::table()
                                     ),
                                     $field_info,
                                     $context,
                                     delete = $crate::helper::primary_keys::PrimaryKeyArgument<
                                     'static,
                                     <$entity as $crate::diesel::associations::HasTable>::Table,
                                     $context,
                                     <&'static $entity as $crate::diesel::Identifiable>::Id>}
    };
    ($registry: ident, $entity: ident, $fields: ident, $info: expr, $field_info:expr, $context: ty, delete = $delete:ty) => {
        {
            let delete = $registry.arg::<$delete>(concat!("Delete", stringify!($entity)), $info);
            let delete = $registry.field::<Option<$entity>>(concat!("Delete", stringify!($entity)), $field_info)
                .argument(delete);
            $fields.push(delete);
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_handle_delete {
    ($conn: ty, $entity_name: ident, $executor: ident, $arguments: ident,) => {
        __wundergraph_expand_handle_delete!{$conn, $entity_name, $executor, $arguments, delete = true}
    };
    ($conn: ty, $entity: ident, $executor: ident, $arguments: ident, delete = true) => {
        __wundergraph_expand_handle_delete!{$conn, $entity, $executor, $arguments,
                                            delete =
                                            $crate::helper::primary_keys::PrimaryKeyArgument<
                                            'static,
                                            <$entity as $crate::diesel::associations::HasTable>::Table,
                                            Self::Context,
                                            <&'static $entity as $crate::diesel::Identifiable>::Id>
        }
    };
    ($conn: ty, $entity_name: ident, $executor: ident, $arguments: ident, delete = false) => {
        Err($crate::juniper::FieldError::new(
            "Unknown field:",
            $crate::juniper::Value::String(concat!("Delete", stringify!($entity_name)).to_owned()),
        ))
    };
    ($conn: ty, $entity_name: ident, $executor: ident, $arguments: ident, delete = $delete: ty) => {
        $crate::mutations::handle_delete::<
            <$conn as $crate::diesel::Connection>::Backend,
        $delete,
        $entity_name,
        Self::Context
            >(
                $executor,
                $arguments,
                concat!("Delete", stringify!($entity_name))
            )
    }
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "postgres")]
macro_rules! __wundergraph_expand_pg_mutation {
    (
        $mutation_name: ident {
            $($entity_name: ident(
                $(insert = $insert: ident,)*
                $(update = $update: ident,)*
                $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {
        __wundergraph_expand_mutation_graphql_type! {
            $crate::diesel::PgConnection,
            $mutation_name(context = $crate::diesel::r2d2::PooledConnection<$crate::diesel::r2d2::ConnectionManager<$crate::diesel::PgConnection>>) {
                $($entity_name(
                    $(insert = $insert,)*
                        $(update = $update,)*
                        $(delete = $($delete)+)*
                ),)*
            }
        }
    };
    (
        $mutation_name: ident(context = $($context: tt)::+<Conn>) {
            $($entity_name: ident(
                $(insert = $insert: ident,)*
                    $(update = $update: ident,)*
                    $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {
        __wundergraph_expand_mutation_graphql_type! {
            $crate::diesel::PgConnection,
            $mutation_name(context = $($context)::+<$crate::diesel::PgConnection>) {
                $($entity_name(
                    $(insert = $insert,)*
                        $(update = $update,)*
                        $(delete = $($delete)+)*
                ),)*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "sqlite")]
macro_rules! __wundergraph_expand_sqlite_mutation {
        (
        $mutation_name: ident {
            $($entity_name: ident(
                $(insert = $insert: ident,)*
                $(update = $update: ident,)*
                $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {
        __wundergraph_expand_mutation_graphql_type! {
            $crate::diesel::SqliteConnection,
            $mutation_name(context = $crate::diesel::r2d2::PooledConnection<$crate::diesel::ConnectionManager<$crate::diesel::SqliteConnection>>) {
                $($entity_name(
                    $(insert = $insert,)*
                        $(update = $update,)*
                        $(delete = $($delete)+)*
                ),)*
            }
        }
    };
    (
        $mutation_name: ident(context = $($context: tt)::+<Conn>) {
            $($entity_name: ident(
                $(insert = $insert: ident,)*
                    $(update = $update: ident,)*
                    $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {
        __wundergraph_expand_mutation_graphql_type! {
            $crate::diesel::SqliteConnection,
            $mutation_name(context = $($context)::+<$crate::diesel::SqliteConnection>) {
                $($entity_name(
                    $(insert = $insert,)*
                        $(update = $update,)*
                        $(delete = $($delete)+)*
                ),)*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "sqlite"))]
macro_rules! __wundergraph_expand_sqlite_mutation {
    (
        $mutation_name: ident $((context = $($context: tt)*))* {
            $($entity_name: ident (
                $(insert = $insert: ident,)*
                $(update = $update: ident,)*
                $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {}
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "postgres"))]
macro_rules! __wundergraph_expand_pg_mutation {
    (
        $mutation_name: ident $((context = $($context: tt)*))* {
            $($entity_name: ident (
                $(insert = $insert: ident,)*
                    $(update = $update: ident,)*
                    $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {}
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_mutation_graphql_type {
    (
        $conn: ty,
        $mutation_name: ident(context = $context: ty) {
            $($entity_name: ident(
                $(insert = $insert: ident,)*
                $(update = $update: ident,)*
                $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {
        impl $crate::juniper::GraphQLType for $mutation_name<$crate::diesel::r2d2::Pool<$crate::diesel::r2d2::ConnectionManager<$conn>>>
        {
            type Context = $context;
            type TypeInfo = ();

            fn name(_info: &Self::TypeInfo) -> Option<&str> {
                Some(stringify!($mutation_name))
            }

            #[allow(non_snake_case)]
            fn meta<'r>(
                info: &Self::TypeInfo,
                registry: &mut $crate::juniper::Registry<'r>
            ) -> $crate::juniper::meta::MetaType<'r> {
                let mut fields = Vec::new();
                $(
                    $(
                        let new = registry.arg::<$insert>(concat!("New", stringify!($entity_name)), info);
                        let new = registry.field::<Option<$entity_name>>(concat!("Create", stringify!($entity_name)), info)
                            .argument(new);
                        fields.push(new);
                        let new = registry.arg::<Vec<$insert>>(concat!("New", stringify!($entity_name), "s"), info);
                        let new = registry.field::<Vec<$entity_name>>(concat!("Create", stringify!($entity_name), "s"), info)
                            .argument(new);
                        fields.push(new);
                    )*
                )*
                    $(
                        $(
                            let update = registry.arg::<$update>(concat!("Update", stringify!($entity_name)), info);
                            let update = registry.field::<Option<$entity_name>>(concat!("Update", stringify!($entity_name)), info)
                                .argument(update);
                            fields.push(update);
                        )*
                    )*
                    $(
                        __wundergraph_expand_delete!{registry,
                                                     $entity_name,
                                                     fields,
                                                     info,
                                                     info,
                                                     $context,
                                                     $(delete = $($delete)+)*}
                    )*
                    let mutation = registry.build_object_type::<Self>(info, &fields);
                $crate::juniper::meta::MetaType::Object(mutation)
            }

            fn resolve_field(
                &self,
                _info: &Self::TypeInfo,
                field_name: &str,
                arguments: &$crate::juniper::Arguments,
                executor: &$crate::juniper::Executor<Self::Context>,
            ) -> $crate::juniper::ExecutionResult {
                match field_name {
                    $(
                        $(
                            concat!("Create", stringify!($entity_name)) => {
                                $crate::mutations::handle_insert::<
                                    <$conn as $crate::diesel::Connection>::Backend,
                                    $insert,
                                    $entity_name,
                                    Self::Context>
                                (
                                    executor,
                                    arguments,
                                    concat!("New", stringify!($entity_name))
                                )
                            }
                            concat!("Create", stringify!($entity_name), "s") => {
                                $crate::mutations::handle_batch_insert::<
                                    <$conn as $crate::diesel::Connection>::Backend,
                                    $insert,
                                    $entity_name,
                                    Self::Context>
                                (
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
                                    $crate::mutations::handle_update::<
                                        <$conn as $crate::diesel::Connection>::Backend,
                                        $update,
                                        $entity_name,
                                        Self::Context
                                        >(
                                            executor,
                                            arguments,
                                            concat!("Update", stringify!($entity_name))
                                        )
                                }
                            )*
                        )*
                        $(
                            concat!("Delete", stringify!($entity_name)) => {
                                __wundergraph_expand_handle_delete!{
                                    $conn, $entity_name, executor, arguments, $(delete = $($delete)+)*
                                }
                            }
                        )*
                        e => Err($crate::juniper::FieldError::new(
                            "Unknown field:",
                            $crate::juniper::Value::String(e.to_owned()),
                        )),
                    }
            }
        }
    }
}


#[macro_export]
macro_rules! wundergraph_mutation_object {
    (
        $mutation_name: ident $((context = $($context: tt)*))* {
            $($entity_name: ident (
                $(insert = $insert: ident,)*
                $(update = $update: ident,)*
                $(delete = $($delete: tt)+)*
            ),)*
        }
    ) => {
        #[derive(Debug)]
        pub struct $mutation_name<P>(::std::marker::PhantomData<P>);

        impl<P> Default for $mutation_name<P> {
            fn default() -> Self {
                $mutation_name(Default::default())
            }
        }
        __wundergraph_expand_pg_mutation!{
            $mutation_name $((context = $($context)*))* {
                $($entity_name(
                    $(insert = $insert,)*
                    $(update = $update,)*
                    $(delete = $($delete)+)*
                ),)*
            }
        }
        __wundergraph_expand_sqlite_mutation!{
                        $mutation_name $((context = $($context)*))* {
                $($entity_name(
                    $(insert = $insert,)*
                    $(update = $update,)*
                    $(delete = $($delete)+)*
                ),)*
            }
        }
    };
}
