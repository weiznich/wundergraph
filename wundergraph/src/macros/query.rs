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

#[macro_export]
macro_rules! __wundergraph_expand_limit {
    ($registry: ident, $entity: ident, $info: ident, ) => {
        __wundergraph_expand_optional_argument!("limit", i32, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident, $(,$limit: tt)+) => {
        __wundergraph_expand_optional_argument!("limit", i32, $registry, $entity, $info $(,$limit)*)
    };
}

#[macro_export]
macro_rules! __wundergraph_expand_offset {
    ($registry: ident, $entity: ident, $info: ident, ) => {
        __wundergraph_expand_optional_argument!("offset", i32, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident, $(,$offset: tt)+) => {
        __wundergraph_expand_optional_argument!("offset", i32, $registry, $entity, $info $(,$offset)*)
    };
}

#[macro_export]
macro_rules! __wundergraph_expand_order {
    ($registry: ident, $entity: ident, $info: ident, ) => {
        __wundergraph_expand_optional_argument!("order", Vec<$crate::order::OrderBy>, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident, $(,$order: tt)+) => {
        __wundergraph_expand_optional_argument!("order", Vec<$crate::order::OrderBy>, $registry, $entity, $info $(,$order)*)
    };
}

#[macro_export]
macro_rules! wundergraph_query_object {
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
        #[derive(Debug)]
        pub struct $query_name<P>(::std::marker::PhantomData<P>);

        impl<P> Default for $query_name<P> {
            fn default() -> Self {
                $query_name(Default::default())
            }
        }

        impl<Conn> $crate::juniper::GraphQLType for $query_name<$crate::diesel::r2d2::Pool<$crate::diesel::r2d2::ConnectionManager<Conn>>>
        where
            Conn: $crate::diesel::Connection + 'static,
            Conn::Backend: Clone,
            <Conn::Backend as Backend>::QueryBuilder: Default,
            String: $crate::diesel::deserialize::FromSql<$crate::diesel::sql_types::Text, Conn::Backend>,
            i32: $crate::diesel::deserialize::FromSql<$crate::diesel::sql_types::Integer, Conn::Backend>,
            bool: $crate::diesel::deserialize::FromSql<$crate::diesel::sql_types::Bool, Conn::Backend>,
            f64: $crate::diesel::deserialize::FromSql<$crate::diesel::sql_types::Double, Conn::Backend>,
            i16: $crate::diesel::deserialize::FromSql<$crate::diesel::sql_types::SmallInt, Conn::Backend>,
        {
            type Context = $crate::diesel::r2d2::PooledConnection<$crate::diesel::r2d2::ConnectionManager<Conn>>;
            type TypeInfo = ();

            fn name(_info: &Self::TypeInfo) -> Option<&str> {
                Some(stringify!($query_name))
            }

            #[allow(non_snake_case)]
            fn meta<'r>(
                info: &Self::TypeInfo,
                registry: &mut $crate::juniper::Registry<'r>
            ) -> $crate::juniper::meta::MetaType<'r> {
                $(
                    let mut $graphql_struct = registry.field::<Vec<$graphql_struct>>(
                        stringify!($graphql_struct),
                        info
                    );

                    $(
                        let filter = registry.arg_with_default::<Option<
                            $crate::filter::Filter<
                                   $filter_name,
                                   <$graphql_struct as $crate::diesel::associations::HasTable>::Table>>
                            >
                            ("filter", &None, &Default::default());
                        $graphql_struct = $graphql_struct.argument(filter);
                    )*
                    __wundergraph_expand_limit!(registry, $graphql_struct, info, $(, $limit)*);
                    __wundergraph_expand_offset!(registry, $graphql_struct, info, $(, $offset)*);
                    __wundergraph_expand_order!(registry, $graphql_struct, info, $(, $order)*);

                )*
                let query = registry.build_object_type::<Self>(info, &[$($graphql_struct,)*]);
                $crate::juniper::meta::MetaType::Object(query)
            }

            fn resolve_field(
                &self,
                _info: &Self::TypeInfo,
                field_name: &str,
                _arguments: &$crate::juniper::Arguments,
                executor: &$crate::juniper::Executor<Self::Context>,
            ) -> $crate::juniper::ExecutionResult {
                use $crate::diesel::associations::HasTable;
                use $crate::diesel::QueryDsl;
                match field_name {
                    $(
                        stringify!($graphql_struct) => self.handle::<$graphql_struct>(
                            executor,
                            executor.look_ahead(),
                            <$graphql_struct as HasTable>::table().into_boxed(),
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

        impl<Conn> $query_name<$crate::diesel::r2d2::Pool<$crate::diesel::r2d2::ConnectionManager<Conn>>>
        where
            Conn: $crate::diesel::Connection + 'static,
            <Conn::Backend as $crate::diesel::backend::Backend>::QueryBuilder: Default,
        {
            fn handle<T>(
                &self,
                e: &$crate::juniper::Executor<
                    $crate::diesel::r2d2::PooledConnection<
                    $crate::diesel::r2d2::ConnectionManager<Conn>>>,
                s: $crate::juniper::LookAheadSelection,
                sel: $crate::diesel::query_builder::BoxedSelectStatement<T::SqlType, T::Table, Conn::Backend>,
            ) -> $crate::juniper::ExecutionResult
            where
                T: $crate::LoadingHandler<Conn> + $crate::juniper::GraphQLType<TypeInfo = (), Context = ()>,
                T::Table: $crate::diesel::associations::HasTable<Table = T::Table>,
            {
                let conn = e.context();
                let items = T::load_item(&s, conn, sel)?;
                e.resolve_with_ctx(&(), &items)
            }
        }
    }
}
