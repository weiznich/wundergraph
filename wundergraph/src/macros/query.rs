#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_optional_argument {
    ($name: expr,
     $arg_ty: ty,
     $registry: ident,
     $entity: ident,
     $info: ident, true $(, $rest: expr)*) => {
        let arg = $registry.arg_with_default::<Option<$arg_ty>>($name, &None, &$info);
        $entity = $entity.argument(arg);
        __wundergraph_expand_optional_argument!($name, $arg_ty, $registry, $entity, $info $(, $rest )*)
    };
    ($name: expr,
     $arg_ty: ty,
     $registry: ident,
     $entity: ident,
     $info: ident, false $(, $rest: expr)*) => {
        __wundergraph_expand_optional_argument!($name, $arg_ty, $registry, $entity, $info $(, $rest )*)
    };
    ($name:expr, $arg_ty: ty, $registry: ident, $entity: ident, $info: ident) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_limit {
    ($registry: ident, $entity: ident, $info: ident, ) => {
        __wundergraph_expand_optional_argument!("limit", i32, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident, $(,$limit: tt)+) => {
        __wundergraph_expand_optional_argument!("limit", i32, $registry, $entity, $info $(,$limit)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_offset {
    ($registry: ident, $entity: ident, $info: ident, ) => {
        __wundergraph_expand_optional_argument!("offset", i32, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident, $(,$offset: tt)+) => {
        __wundergraph_expand_optional_argument!("offset", i32, $registry, $entity, $info $(,$offset)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_order {
    ($registry: ident, $entity: ident, $info: ident, ) => {
        __wundergraph_expand_optional_argument!("order", Vec<$crate::order::OrderBy>, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident, $(,$order: tt)+) => {
        __wundergraph_expand_optional_argument!("order", Vec<$crate::order::OrderBy>, $registry, $entity, $info $(,$order)*)
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "postgres")]
macro_rules! __wundergraph_expand_pg_loading_handler {
    (
        $query_name: ident {
            $($entity_name: ident(
                $graphql_struct: ident
                $(, filter = $filter_name: ident)*
                $(, limit = $limit: tt)*
                $(, offset = $offset: tt)*
                $(, order = $order: tt)*
            ),)*
        }
    ) => {
        __wundergraph_expand_graphql_type_for_query!{
            $crate::diesel::PgConnection,
            $query_name(context = $crate::diesel::r2d2::PooledConnection<$crate::diesel::r2d2::ConnectionManager<$crate::diesel::PgConnection>>) {
                $($entity_name(
                    $graphql_struct
                    $(, filter = $filter_name)*
                    $(, limit = $limit)*
                    $(, offset = $offset)*
                    $(, order = $order)*
                ),)*
            }
        }
    };
    (
        $query_name: ident(context = $($context:tt)::+<Conn>) {
            $($entity_name: ident(
                $graphql_struct: ident
                $(, filter = $filter_name: ident)*
                $(, limit = $limit: tt)*
                $(, offset = $offset: tt)*
                $(, order = $order: tt)*
            ),)*
        }
    ) => {
        wundergraph_query_object!{
            @expand_loading_handler $crate::diesel::PgConnection,
            $query_name(context = $($context)::+<$crate::diesel::PgConnection>) {
                $($entity_name(
                    $graphql_struct
                    $(, filter = $filter_name)*
                    $(, limit = $limit)*
                    $(, offset = $offset)*
                    $(, order = $order)*
                ),)*
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "sqlite")]
macro_rules! __wundergraph_expand_sqlite_loading_handler {
    (
        $query_name: ident {
            $($entity_name: ident(
                $graphql_struct: ident
                $(, filter = $filter_name: ident)*
                $(, limit = $limit: tt)*
                $(, offset = $offset: tt)*
                $(, order = $order: tt)*
            ),)*
        }
    ) => {
        __wundergraph_expand_graphql_type_for_query!{
            $crate::diesel::SqliteConnection,
            $query_name(context = $crate::diesel::r2d2::PooledConnection<$crate::diesel::r2d2::ConnectionManager<$crate::diesel::SqliteConnection>>) {
                $($entity_name(
                    $graphql_struct
                    $(, filter = $filter_name)*
                    $(, limit = $limit)*
                    $(, offset = $offset)*
                    $(, order = $order)*
                ),)*
            }
        }
    };
    (
        $query_name: ident(context = $($context:tt)::+<Conn>) {
            $($entity_name: ident(
                $graphql_struct: ident
                $(, filter = $filter_name: ident)*
                $(, limit = $limit: tt)*
                $(, offset = $offset: tt)*
                $(, order = $order: tt)*
            ),)*
        }
    ) => {
        __wundergraph_expand_graphql_type_for_query!{
            $crate::diesel::SqliteConnection,
            $query_name(context = $($context)::+<$crate::diesel::SqliteConnection>) {
                $($entity_name(
                    $graphql_struct
                    $(, filter = $filter_name)*
                    $(, limit = $limit)*
                    $(, offset = $offset)*
                    $(, order = $order)*
                ),)*
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "postgres"))]
macro_rules! __wundergraph_expand_pg_loading_handler {
    (
        $query_name:ident $((context = $($context:tt)*))* {
            $(
                $entity_name:ident(
                    $graphql_struct:ident
                    $(,filter = $filter_name:ident)*
                    $(,limit = $limit:tt)*
                    $(,offset = $offset:tt)*
                    $(,order = $order:tt)*
                ),
            )*
         }
    ) => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "sqlite"))]
macro_rules! __wundergraph_expand_sqlite_loading_handler {
    (
        $query_name:ident $((context = $($context:tt)*))* {
            $(
                $entity_name:ident(
                    $graphql_struct:ident
                    $(,filter = $filter_name:ident)*
                    $(,limit = $limit:tt)*
                    $(,offset = $offset:tt)*
                    $(,order = $order:tt)*
                ),
            )*
         }
    ) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_expand_graphql_type_for_query {
    ($conn:ty,
     $query_name: ident(context = $context: ty) {
         $($entity_name: ident(
             $graphql_struct: ident
                 $(, filter = $filter_name: ident)*
                 $(, limit = $limit: tt)*
                 $(, offset = $offset: tt)*
                 $(, order = $order: tt)*
         ),)*
     }
    )=> {
        impl $crate::juniper::GraphQLType for $query_name<$crate::diesel::r2d2::Pool<$crate::diesel::r2d2::ConnectionManager<$conn>>>
        {
            type Context = $context;
            type TypeInfo = ();

            fn name(_info: &Self::TypeInfo) -> Option<&str> {
                Some(stringify!($query_name))
            }

            #[allow(non_snake_case)]
            fn meta<'r>(
                info: &Self::TypeInfo,
                registry: &mut $crate::juniper::Registry<'r>
            ) -> $crate::juniper::meta::MetaType<'r> {
                let fields = &[
                    $(
                        {
                            let mut field = registry.field::<Vec<$graphql_struct>>(
                                concat!(stringify!($graphql_struct), "s"),
                                info
                            );

                            $(
                                let filter = registry.arg_with_default::<Option<
                                    $crate::filter::Filter<
                                    $filter_name,
                                <$graphql_struct as $crate::diesel::associations::HasTable>::Table>>
                                    >
                                    ("filter", &None, &Default::default());
                                field = field.argument(filter);
                            )*
                                __wundergraph_expand_limit!(registry, field, info, $(, $limit)*);
                            __wundergraph_expand_offset!(registry, field, info, $(, $offset)*);
                            __wundergraph_expand_order!(registry, field, info, $(, $order)*);
                            field
                        },
                        {
                            let key_info = $crate::helper::primary_keys::PrimaryKeyInfo::new(<$graphql_struct as $crate::diesel::associations::HasTable>::table());
                            let key = registry.arg::<
                                $crate::helper::primary_keys::PrimaryKeyArgument<
                                'static,
                            <$graphql_struct as $crate::diesel::associations::HasTable>::Table,
                            $context,
                            <&'static $graphql_struct as $crate::diesel::Identifiable>::Id
                                >
                                >("primaryKey", &key_info);
                            registry.field::<Option<$graphql_struct>>(
                                stringify!($graphql_struct),
                                info
                            ).argument(key)
                        },

                    )*
                ];
                let query = registry.build_object_type::<Self>(info, fields);
                $crate::juniper::meta::MetaType::Object(query)
            }

            fn resolve_field(
                &self,
                _info: &Self::TypeInfo,
                field_name: &str,
                _arguments: &$crate::juniper::Arguments,
                executor: &$crate::juniper::Executor<Self::Context>,
            ) -> $crate::juniper::ExecutionResult {
                match field_name {
                    $(
                        concat!(stringify!($graphql_struct), "s") => self.handle_filter::<$graphql_struct, Self::Context>(
                            executor,
                            executor.look_ahead(),
                        ),
                        stringify!($graphql_struct) => self.handle_by_key::<$graphql_struct, Self::Context>(
                            executor,
                            executor.look_ahead(),
                        ),
                    )*
                        e => Err($crate::juniper::FieldError::new(
                            "Unknown field:",
                            $crate::juniper::Value::String(e.to_owned()),
                        )),
                }
            }

            fn concrete_type_name(&self, _context: &Self::Context, _info: &Self::TypeInfo) -> String {
                String::from(stringify!($query_name))
            }
        }

        impl $query_name<$crate::diesel::r2d2::Pool<$crate::diesel::r2d2::ConnectionManager<$conn>>>
        {
            fn handle_filter<T, Ctx>(
                &self,
                e: &$crate::juniper::Executor<Ctx>,
                s: $crate::juniper::LookAheadSelection,
            ) -> $crate::juniper::ExecutionResult
            where
                T: $crate::LoadingHandler<<$conn as $crate::diesel::Connection>::Backend, Context = Ctx>
                + $crate::juniper::GraphQLType<TypeInfo = ()>,
                T::Table: $crate::diesel::associations::HasTable<Table = T::Table>,
                Ctx: $crate::WundergraphContext<<$conn as $crate::diesel::Connection>::Backend>,
            <T as $crate::juniper::GraphQLType>::Context: $crate::juniper::FromContext<Ctx>,
            {
                use $crate::diesel::QueryDsl;

                let ctx = e.context();
                let q = T::default_query().into_boxed();
                let items = T::load_items(&s, ctx, q)?;
                e.resolve_with_ctx(&(), &items)
            }

            fn handle_by_key<T, Ctx>(
                &self,
                e: &$crate::juniper::Executor<Ctx>,
                s: $crate::juniper::LookAheadSelection,
            ) -> $crate::juniper::ExecutionResult
            where
                T: $crate::LoadingHandler<<$conn as $crate::diesel::Connection>::Backend, Context = Ctx> + 'static
                + $crate::juniper::GraphQLType<TypeInfo = ()>,
                T::Table: $crate::diesel::associations::HasTable<Table = T::Table>,
                Ctx: $crate::WundergraphContext<<$conn as $crate::diesel::Connection>::Backend>,
            <T as $crate::juniper::GraphQLType>::Context: $crate::juniper::FromContext<Ctx>,
            &'static T: $crate::diesel::Identifiable,
            <&'static T as $crate::diesel::Identifiable>::Id: $crate::helper::primary_keys::UnRef<'static>,
                $crate::helper::primary_keys::PrimaryKeyArgument<
                'static,
                T::Table,
            Ctx,
            <&'static T as $crate::diesel::Identifiable>::Id,
            >: $crate::helper::FromLookAheadValue,
            <T::Table as $crate::diesel::Table>::PrimaryKey: $crate::diesel::EqAll<<<&'static T as $crate::diesel::Identifiable>::Id as $crate::helper::primary_keys::UnRef<'static>>::UnRefed>,
            <<T::Table as $crate::diesel::Table>::PrimaryKey as $crate::diesel::EqAll<<<&'static T as $crate::diesel::Identifiable>::Id as $crate::helper::primary_keys::UnRef<'static>>::UnRefed>>::Output: $crate::diesel::AppearsOnTable<T::Table> + $crate::diesel::expression::NonAggregate + $crate::diesel::query_builder::QueryFragment<<$conn as $crate::diesel::Connection>::Backend>,
            {
                use $crate::diesel::QueryDsl;

                let ctx = e.context();
                let q = T::default_query().into_boxed();
                let item = T::load_item(&s, ctx, q)?;
                e.resolve_with_ctx(&(), &item)
            }
        }
    }
}

#[macro_export]
macro_rules! wundergraph_query_object {
    (
        $query_name: ident $((context = $($context: tt)*))* {
            $($entity_name: ident(
                $graphql_struct: ident
                $(, filter = $filter_name: ident)*
                $(, limit = $limit: tt)*
                $(, offset = $offset: tt)*
                $(, order = $order: tt)*
            ),)*
        }
    ) => {
        #[derive(Debug)]
        pub struct $query_name<P>(::std::marker::PhantomData<P>);

        impl<P> Default for $query_name<P> {
            fn default() -> Self {
                $query_name(Default::default())
            }
        }
        __wundergraph_expand_pg_loading_handler!{
            $query_name $((context = $($context)*))* {
                $($entity_name(
                    $graphql_struct
                        $(, filter = $filter_name)*
                        $(, limit = $limit)*
                        $(, offset = $offset)*
                        $(, order = $order)*
                ),)*
            }
        }

        __wundergraph_expand_sqlite_loading_handler!{
            $query_name $((context = $($context)*))* {
                $($entity_name(
                    $graphql_struct
                        $(, filter = $filter_name)*
                        $(, limit = $limit)*
                        $(, offset = $offset)*
                        $(, order = $order)*
                ),)*
            }
        }
    };
}
