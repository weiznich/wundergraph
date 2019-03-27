#[doc(hidden)]
#[macro_export]
macro_rules! __expand_optional_argument {
    ($name: expr,
     $arg_ty: ty,
     $registry: ident,
     $entity: ident,
     $info: expr, true $(, $rest: expr)*) => {
        let arg = $registry.arg_with_default::<Option<$arg_ty>>($name, &None, &$info);
        $entity = $entity.argument(arg);
        $crate::__expand_optional_argument!($name, $arg_ty, $registry, $entity, $info $(, $rest )*)
    };
    ($name: expr,
     $arg_ty: ty,
     $registry: ident,
     $entity: ident,
     $info: expr, false $(, $rest: expr)*) => {
        $crate::__expand_optional_argument!($name, $arg_ty, $registry, $entity, $info $(, $rest )*)
    };
    ($name:expr, $arg_ty: ty, $registry: ident, $entity: ident, $info: expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_limit {
    ($registry: ident, $entity: ident, $info: ident ) => {
        $crate::__expand_optional_argument!("limit", i32, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident $(,$limit: tt)+) => {
        $crate::__expand_optional_argument!("limit", i32, $registry, $entity, $info $(,$limit)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_offset {
    ($registry: ident, $entity: ident, $info: ident ) => {
        $crate::__expand_optional_argument!("offset", i32, $registry, $entity, $info, true)
    };
    ($registry: ident, $entity: ident, $info: ident $(,$offset: tt)+) => {
        $crate::__expand_optional_argument!("offset", i32, $registry, $entity, $info $(,$offset)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_order {
    ($registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident ) => {
        $crate::__expand_optional_argument!("order",
                                                Vec<$crate::order::OrderBy<$graphql_struct, <$conn as $crate::diesel::Connection>::Backend, Ctx>>,
                                                $registry, $entity, &Default::default(), true)
    };
    ($registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident $(,$order: tt)+) => {
        $crate::__expand_optional_argument!("order",
                                                Vec<$crate::order::OrderBy<$graphql_struct, <$conn as $crate::diesel::Connection>::Backend, Ctx>>,
                                                $registry, $entity, &Default::default() $(,$order)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_filter {
    ($registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident ) => {
        $crate::__expand_optional_argument!("filter",
                                            $crate::filter::Filter<<$graphql_struct as $crate::LoadingHandler<<$conn as $crate::diesel::Connection>::Backend, Ctx>>::Filter, <$graphql_struct as $crate::diesel::associations::HasTable>::Table>,
                                            $registry, $entity, &Default::default(), true)
    };
    ($registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident $(, $filter: tt)+) => {
        $crate::__expand_optional_argument!("filter",
                                            $crate::filter::Filter<<$graphql_struct as $crate::LoadingHandler<<$conn as $crate::diesel::Connection>::Backend, Ctx>>::Filter, <$graphql_struct as $crate::diesel::associations::HasTable>::Table>,
                                            $registry, $entity, &Default::default() $(,$filter)*)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __expand_meta {
    ($field: ident, #[doc = $doc:expr], $($rest:tt)*) => {
        $field = $field.push_docstring(&[$doc]);
        $crate::__expand_meta!($field, $($rest)*);
    };
    ($field: ident, #[deprecated(note = $deprecated: expr)], $($rest:tt)*) => {
        $field = $field.deprecated(Some($deprecated));
        $crate::__expand_meta!($field, $($rest)*);
    };
    ($field: ident, #[wundergraph(graphql_name = $name: expr)], $($rest:tt)*) => {
        $crate::__expand_meta!($field, $($rest)*);
    };
    ($field: ident,) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __expand_name {
    ($graphql_struct: ident, #[wundergraph(graphql_name = $name: expr)], $($rest:tt)*) => {
        $name
    };
    ($graphql_struct: ident, #[$($ignored:tt)*], $($rest:tt)*) => {
        $crate::__expand_name!($graphql_struct, $($rest)*)
    };
    ($graphql_struct: ident, ) => {
        concat!(stringify!($graphql_struct), "s")
    };
}

#[macro_export]
macro_rules! query_object {
    (
        $(#[doc = $glob_doc: expr])*
        $query_name: ident {
            $(
                $(#[$($meta: tt)*])*
                $graphql_struct: ident$((
                        $( filter = $filter: tt)?
                        $(, limit = $limit: tt)?
                        $(, offset = $offset: tt)?
                        $(, order = $order: tt)?
                        $(,)?
                ))?,)*
        }
    ) => {

        // Use Arc<Mutex<C>> here to force make this Sync
        #[derive(Debug)]
        $(#[doc = $glob_doc])*
        pub struct $query_name<C>(::std::marker::PhantomData<std::sync::Arc<std::sync::Mutex<C>>>);

        impl<C> Default for $query_name<C> {
            fn default() -> Self {
                $query_name(::std::marker::PhantomData)
            }
        }

        $crate::paste::item!{
            impl<Ctx, DB, $([<$graphql_struct _table>], [<$graphql_struct _id>],)*> $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>
                for $query_name<Ctx>
            where Ctx: $crate::WundergraphContext,
                  DB: $crate::diesel::backend::Backend + 'static,
                  DB::QueryBuilder: std::default::Default,
                  Ctx::Connection: $crate::diesel::Connection<Backend = DB>,
            $([<$graphql_struct _table>]: $crate::diesel::Table + $crate::diesel::query_dsl::methods::BoxedDsl<
              'static,
              DB,
              Output = $crate::diesel::query_builder::BoxedSelectStatement<
              'static,
              $crate::diesel::dsl::SqlTypeOf<<[<$graphql_struct _table>] as $crate::diesel::Table>::AllColumns>,
              [<$graphql_struct _table>],
              DB
              >,
              > +  $crate::diesel::QuerySource<FromClause = $crate::diesel::query_builder::nodes::Identifier<'static>> + 'static,)*
                $($graphql_struct: $crate::LoadingHandler<DB, Ctx> + $crate::diesel::associations::HasTable<Table = [<$graphql_struct _table>]>,)*
                $(Ctx: $crate::QueryModifier<$graphql_struct, DB>,)*
                $(<[<$graphql_struct _table>] as $crate::diesel::QuerySource>::FromClause: $crate::diesel::query_builder::QueryFragment<DB>,)*
                $(<$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::Columns: $crate::query_helper::order::BuildOrder<[<$graphql_struct _table>], DB>,)*
                $(<$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::Columns: $crate::query_helper::select::BuildSelect<
                  [<$graphql_struct _table>],
                  DB,
                  $crate::query_helper::placeholder::SqlTypeOfPlaceholder<
                  <$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::FieldList,
                  DB,
                  <$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                  [<$graphql_struct _table>],
                  Ctx
                  >
                  >,)*
                $(<$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::FieldList: $crate::query_helper::placeholder::WundergraphFieldList<
                  DB,
                  <$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                  [<$graphql_struct _table>],
                  Ctx,
                  >,)*
                $(<$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::FieldList:
                  $crate::graphql_type::WundergraphGraphqlHelper<$graphql_struct, DB, Ctx> +
                  $crate::query_helper::placeholder::FieldListExtractor,)*
                $(DB: $crate::diesel::sql_types::HasSqlType<$crate::query_helper::placeholder::SqlTypeOfPlaceholder<
                  <$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::FieldList,
                  DB,
                  <$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                  [<$graphql_struct _table>],
                  Ctx
                  >>,)*
                $(&'static $graphql_struct: $crate::diesel::Identifiable<Id = [<$graphql_struct _id>]>,)*
                $([<$graphql_struct _id>]: std::hash::Hash + std::cmp::Eq + $crate::helper::primary_keys::UnRef<'static>,)*
                $([<$graphql_struct _table>]::PrimaryKey: $crate::helper::primary_keys::PrimaryKeyInputObject<
                  <[<$graphql_struct _id>] as $crate::helper::primary_keys::UnRef<'static>>::UnRefed, ()
                  >,)*
                $([<$graphql_struct _table>]::PrimaryKey: $crate::diesel::EqAll<<[<$graphql_struct _id>] as $crate::helper::primary_keys::UnRef<'static>>::UnRefed>,)*
                $(<[<$graphql_struct _table>]::PrimaryKey as $crate::diesel::EqAll<<[<$graphql_struct _id>] as $crate::helper::primary_keys::UnRef<'static>>::UnRefed>>::Output: $crate::diesel::AppearsOnTable<[<$graphql_struct _table>]> + $crate::diesel::query_builder::QueryFragment<DB> + $crate::diesel::expression::NonAggregate,)*
                $(<<$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::Filter as $crate::filter::build_filter::BuildFilter<DB>>::Ret: $crate::diesel::AppearsOnTable<[<$graphql_struct _table>]>,)*
                $(<<$graphql_struct as $crate::LoadingHandler<DB, Ctx>>::FieldList as $crate::query_helper::placeholder::FieldListExtractor>::Out: $crate::graphql_type::WundergraphGraphqlHelper<$graphql_struct, DB, Ctx>,)*
                $($crate::helper::primary_keys::PrimaryKeyArgument<'static, [<$graphql_struct _table>], (), <&'static $graphql_struct as $crate::diesel::Identifiable>::Id>: $crate::helper::FromLookAheadValue,)*
            {
                type Context = Ctx;
                type TypeInfo = ();

                fn name(_info: &Self::TypeInfo) -> Option<&str> {
                    Some(stringify!($query_name))
                }

                #[allow(non_snake_case)]
                fn meta<'r>(
                    info: &Self::TypeInfo,
                    registry: &mut $crate::juniper::Registry<'r, $crate::scalar::WundergraphScalarValue>
                ) -> $crate::juniper::meta::MetaType<'r, $crate::scalar::WundergraphScalarValue>
                where $crate::scalar::WundergraphScalarValue: 'r
                {
                    let fields = &[
                        $(
                            {
                                let mut field = registry.field::<Vec<$crate::graphql_type::GraphqlWrapper<
                                    $graphql_struct,
                                <<Ctx as $crate::WundergraphContext>::Connection as $crate::diesel::Connection>::Backend, Ctx>
                                    >>(
                                        $crate::__expand_name!($graphql_struct, $(#[$($meta)*],)*),
                                        info
                                    );
                                $crate::__expand_meta!(field, $(#[$($meta)*],)*);
                                $crate::__expand_filter!(
                                    registry,
                                    field,
                                    <Ctx as $crate::WundergraphContext>::Connection,
                                    $graphql_struct
                                        $($(, $filter)?)?
                                );
                                $crate::__expand_limit!(registry, field, info $($(, $limit)?)?);
                                $crate::__expand_offset!(registry, field, info $($(, $offset)?)?);
                                $crate::__expand_order!(
                                    registry,
                                    field,
                                    <Ctx as $crate::WundergraphContext>::Connection,
                                    $graphql_struct $($(, $order)?)?);
                                field
                            },
                            {
                                let key_info = $crate::helper::primary_keys::PrimaryKeyInfo::new(&<$graphql_struct as $crate::diesel::associations::HasTable>::table());
                                let key = registry.arg::<
                                    $crate::helper::primary_keys::PrimaryKeyArgument<
                                    'static,
                                <$graphql_struct as $crate::diesel::associations::HasTable>::Table,
                                Ctx,
                                <&'static $graphql_struct as $crate::diesel::Identifiable>::Id
                                    >
                                    >("primaryKey", &key_info);
                                registry.field::<Option<$crate::graphql_type::GraphqlWrapper<$graphql_struct, <<Ctx as $crate::WundergraphContext>::Connection as $crate::diesel::Connection>::Backend, Ctx>>>(
                                    stringify!($graphql_struct),
                                    info
                                ).argument(key)
                            }
                            ,

                        )*
                    ];
                    let mut obj = registry.build_object_type::<Self>(info, fields);
                    obj = obj.description(concat!($($glob_doc, "\n", )* ""));
                    obj.into_meta()
                }

                fn resolve_field(
                    &self,
                    _info: &Self::TypeInfo,
                    field_name: &str,
                    _arguments: &$crate::juniper::Arguments<$crate::scalar::WundergraphScalarValue>,
                    executor: &$crate::juniper::Executor<Self::Context, $crate::scalar::WundergraphScalarValue>,
                ) -> $crate::juniper::ExecutionResult<$crate::scalar::WundergraphScalarValue> {
                    use $crate::LoadingHandler;
                    use $crate::WundergraphContext;
                    match field_name {
                        $(
                            $crate::__expand_name!($graphql_struct, $(#[$($meta)*],)*) => {
                                let look_ahead = executor.look_ahead();
                                let q = $graphql_struct::build_query(&look_ahead)?;
                                let items = $graphql_struct::load(&look_ahead, executor, q)?;
                                Ok($crate::juniper::Value::List(items))
                            },
                            stringify!($graphql_struct) => {
                                let ctx = executor.context();
                                let look_ahead = executor.look_ahead();
                                let q = $graphql_struct::build_query(&look_ahead)?;
                                let item = $graphql_struct::load_by_primary_key(&look_ahead, executor, q)?;
                                Ok(item.unwrap_or($crate::juniper::Value::Null))
                            }
                        )*
                            e => Err($crate::juniper::FieldError::new(
                                "Unknown field:",
                                $crate::juniper::Value::scalar(e),
                            )),
                    }
                }

                fn concrete_type_name(&self, _context: &Self::Context, _info: &Self::TypeInfo) -> String {
                    String::from(stringify!($query_name))
                }
            }
        }
    };
}
