#[doc(hidden)]
#[macro_export]
macro_rules! __expand_limit {
    ($registry: ident, $entity: ident, $info: ident, meta = [] ) => {
        let arg = $registry.arg_with_default::<Option<i32>>("limit", &None, $info);
        $entity = $entity.argument(arg);
    };
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[wundergraph(offset = true $($stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_limit!($registry, $entity, $info, meta = [])
    };
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[wundergraph(limit = false $($stuff:tt)*)], $($rest:tt)*]
    ) => {};
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[wundergraph($stuff:tt $($other_stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_limit!(
            $registry, $entity, $info,
            meta = [#[wundergraph($($other_stuff)*)], $($rest)*]
        )
    };
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[$($stuff1:tt)*], $($rest:tt)*]
    ) => {
        $crate::__expand_limit!($registry, $entity, $info, meta = [$($rest)*])
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_offset {
    ($registry: ident, $entity: ident, $info: ident, meta = [] ) => {
        let arg = $registry.arg_with_default::<Option<i32>>("offset", &None, $info);
        $entity = $entity.argument(arg);
    };
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[wundergraph(offset = true $($stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_offset!($registry, $entity, $info, meta = [])
    };
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[wundergraph(offset = false $($stuff:tt)*)], $($rest:tt)*]
    ) => {};
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[wundergraph($stuff:tt $($other_stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_offset!(
            $registry, $entity, $info,
            meta = [#[wundergraph($($other_stuff)*)], $($rest)*]
        )
    };
    (
        $registry: ident, $entity: ident, $info: ident,
        meta = [#[$($stuff1:tt)*], $($rest:tt)*]
    ) => {
        $crate::__expand_offset!($registry, $entity, $info, meta = [$($rest)*])
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_order {
    ($registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident, meta = [] ) => {
        let arg = $registry.arg_with_default::<Option<
            Vec<$crate::query_builder::selection::order::OrderBy<$graphql_struct, <$conn as $crate::diesel::Connection>::Backend, Ctx>>
         >>("order", &None, &Default::default());
        $entity = $entity.argument(arg);
    };
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[wundergraph(order = true $($stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_order!($registry, $entity, $conn, $graphql_struct, meta = [])
    };
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[wundergraph(order = false $($stuff2:tt)*)], $($rest:tt)*]
    ) => {};
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[wundergraph($stuff:tt $($other_stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_order!(
            $registry, $entity, $conn, $graphql_struct,
            meta = [#[wundergraph($($other_stuff)*)], $($rest)*]
        )
    };
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[$($stuff1:tt)*], $($rest:tt)*]
    ) => {
        $crate::__expand_order!($registry, $entity, $conn, $graphql_struct, meta = [$($rest)*])
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand_filter {
    ($registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident, meta = []) => {
        let arg = $registry.arg_with_default::<Option<
            $crate::query_builder::selection::filter::Filter<<$graphql_struct as $crate::query_builder::selection::LoadingHandler<<$conn as $crate::diesel::Connection>::Backend, Ctx>>::Filter, <$graphql_struct as $crate::diesel::associations::HasTable>::Table>
            >>("filter", &None, &Default::default());
        $entity = $entity.argument(arg);
    };
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[wundegraph(filter = true $($stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_filter!($registry, $entity, $conn, $graphql_struct, meta = [])
    };
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[wundegraph(filter = false $($stuff:tt)*)], $($rest:tt)*]
    ) => {};
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[wundegraph($stuff:tt $($other_stuff:tt)*)], $($rest:tt)*]
    ) => {
        $crate::__expand_filter!(
            $registry, $entity, $conn, $graphql_struct,
            meta = [#[wundergraph($($other_stuff)*)], $($rest)*]
        )
    };
    (
        $registry: ident, $entity: ident, $conn: ty, $graphql_struct: ident,
        meta = [#[$($stuff:tt)*], $($rest:tt)*]
    ) => {
        $crate::__expand_filter!($registry, $entity, $conn, $graphql_struct, meta = [$($rest)*])
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
    ($field: ident, #[wundergraph($($stuff: tt)*)], $($rest:tt)*) => {
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
    ($graphql_struct: ident, #[wundergraph(graphql_name = $name: expr, $($stuff:tt)*)], $($rest:tt)*) => {
        $name
    };
    ($graphql_struct: ident, #[wundergraph($stuff:tt $($other_stuff:tt)*)], $($rest:tt)*) => {
        $crate::__expand_name!($graphql_struct, #[wundergraph($($other_stuff)*)], $($rest)*)
    };
    ($graphql_struct: ident, #[$($ignored:tt)*], $($rest:tt)*) => {
        $crate::__expand_name!($graphql_struct, $($rest)*)
    };
    ($graphql_struct: ident, ) => {
        concat!(stringify!($graphql_struct), "s")
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __expand_additional_arg {
    (
        [$arg: ident: $arg_ty: ty = $arg_default: expr]
        $registry: ident,
        $entity: ident,
        $info: expr
    ) => {
        let arg = $registry.arg_with_default::<$arg_ty>(stringify!($arg), &$arg_default, &$info);
        $entity = $entity.argument(arg);
    };
    (
        [$arg: ident: $arg_ty: ty]
        $registry: ident,
        $entity: ident,
        $info: expr
    ) => {
        let arg = $registry.arg::<$arg_ty>(stringify!($arg), &$info);
        $entity = $entity.argument(arg);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_graphql_obj_for_query {
    (
        query_name = {$($query_name:tt)*},
        structs = [$($graphql_struct: ident,)*],
        $(lt = $lt: tt,)?
        body = {
            $($inner: tt)*
        }
    ) => {
        $crate::paste::item!{
            impl<$($lt,)? Ctx, DB, $([<$graphql_struct _table>], [<$graphql_struct _id>],)*> $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>
                for $($query_name)*<$($lt,)? Ctx>
            where Ctx: $crate::WundergraphContext,
                  DB: $crate::diesel::backend::Backend + $crate::query_builder::selection::offset::ApplyOffset + 'static,
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
              > +  $crate::diesel::QuerySource
              + $crate::diesel::Table + $crate::diesel::associations::HasTable<Table = [<$graphql_struct _table>]> + 'static,)*
                $($graphql_struct: $crate::query_builder::selection::LoadingHandler<DB, Ctx> + $crate::diesel::associations::HasTable<Table = [<$graphql_struct _table>]>,)*
                $(Ctx: $crate::query_builder::selection::QueryModifier<$graphql_struct, DB>,)*
                $(<[<$graphql_struct _table>] as $crate::diesel::QuerySource>::FromClause: $crate::helper::NamedTable + $crate::diesel::query_builder::QueryFragment<DB>,)*
                $(<$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::Columns: $crate::query_builder::selection::order::BuildOrder<[<$graphql_struct _table>], DB>,)*
                $(<$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::Columns: $crate::query_builder::selection::select::BuildSelect<
                  [<$graphql_struct _table>],
                  DB,
                  $crate::query_builder::selection::SqlTypeOfPlaceholder<
                  <$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList,
                  DB,
                  <$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                  [<$graphql_struct _table>],
                  Ctx
                  >
                  >,)*
                $(<$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList: $crate::query_builder::selection::fields::WundergraphFieldList<
                  DB,
                  <$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                  [<$graphql_struct _table>],
                  Ctx,
                  >,)*
                $(<$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList:
                  $crate::graphql_type::WundergraphGraphqlHelper<$graphql_struct, DB, Ctx> +
                  $crate::query_builder::selection::order::WundergraphGraphqlOrderHelper<$graphql_struct, DB, Ctx> +
                  $crate::query_builder::selection::fields::FieldListExtractor,)*
                $(DB: $crate::diesel::sql_types::HasSqlType<$crate::query_builder::selection::SqlTypeOfPlaceholder<
                  <$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList,
                  DB,
                  <$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                  [<$graphql_struct _table>],
                  Ctx
                  >>,)*
                $(&'static $graphql_struct: $crate::diesel::Identifiable<Id = [<$graphql_struct _id>]>,)*
                $([<$graphql_struct _id>]: std::hash::Hash + std::cmp::Eq + $crate::helper::UnRef<'static>,)*
                $([<$graphql_struct _table>]::PrimaryKey: std::default::Default + $crate::helper::PrimaryKeyInputObject<
                  <[<$graphql_struct _id>] as $crate::helper::UnRef<'static>>::UnRefed, ()
                  >,)*
                $([<$graphql_struct _table>]::PrimaryKey: $crate::diesel::EqAll<<[<$graphql_struct _id>] as $crate::helper::UnRef<'static>>::UnRefed>,)*
                $(<[<$graphql_struct _table>]::PrimaryKey as $crate::diesel::EqAll<<[<$graphql_struct _id>] as $crate::helper::UnRef<'static>>::UnRefed>>::Output: $crate::diesel::AppearsOnTable<[<$graphql_struct _table>]> + $crate::diesel::query_builder::QueryFragment<DB> + $crate::diesel::expression::NonAggregate,)*
                $(<<$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::Filter as $crate::query_builder::selection::filter::BuildFilter<DB>>::Ret: $crate::diesel::AppearsOnTable<[<$graphql_struct _table>]>,)*
                $(<<$graphql_struct as $crate::query_builder::selection::LoadingHandler<DB, Ctx>>::FieldList as $crate::query_builder::selection::fields::FieldListExtractor>::Out:
                  $crate::graphql_type::WundergraphGraphqlHelper<$graphql_struct, DB, Ctx> +
                  $crate::query_builder::selection::order::WundergraphGraphqlOrderHelper<$graphql_struct, DB, Ctx>,
            )*
                $($crate::helper::PrimaryKeyArgument<'static, [<$graphql_struct _table>], (), <&'static $graphql_struct as $crate::diesel::Identifiable>::Id>: $crate::juniper_ext::FromLookAheadValue,)*
            {
                $($inner)*
            }
        }
    }
}

/// Macro to register the main query object
///
/// # Annotated example
/// ```
/// ##[macro_use]
/// # extern crate diesel;
/// # use wundergraph::WundergraphEntity;
/// # use wundergraph::query_builder::types::{HasOne, HasMany};
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
/// #          home_world -> Nullable<Integer>,
/// #     }
/// # }
/// #
/// # table! {
/// #     home_worlds {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// #[derive(WundergraphEntity, Identifiable)]
/// #[table_name = "species"]
/// pub struct Species {
///     id: i32,
///     name: String,
///     heros: HasMany<Hero, heros::species,>
/// }
///
/// #[derive(WundergraphEntity, Identifiable)]
/// #[table_name = "home_worlds"]
/// pub struct HomeWorld {
///     id: i32,
///     name: String,
///     heros: HasMany<Hero, heros::home_world>
/// }
///
/// #[derive(WundergraphEntity, Identifiable)]
/// #[table_name = "heros"]
/// pub struct Hero {
///     id: i32,
///     name: String,
///     species: HasOne<i32, Species>,
///     home_world: Option<HasOne<i32, HomeWorld>>,
/// }
///
/// wundergraph::query_object! {
///     // The main query object. The provided name
///     // maps directly to the generated struct which
///     // could be used then as juniper GraphQL struct
///
///     /// An optional doc comment describing the main query object
///     /// Rendered as GraphQL description
///     Query {
///         // Register a wundergraph GraphQL entity
///         //
///         // There are several optional attributes here:
///         // * Documentation comments, rendered as description
///         // * Deprecation attribute, rendered as deprecated notice
///         // * A name attribute, that renames the field to the given name
///
///         /// GraphQL description for the hero entity
///         #[deprecated(note = "Deprecated notice for hero")]
///         #[wundergraph(graphql_name = "TheHero")]
///         Hero,
///         // Additionally there are a few attributes to control the generated
///         // field:
///         // * `#[wundergraph(filter = true)]` Specifies if a filter
///         //   argument is generated for the current entity.
///         //   Possible Values: true, false
///         // * `#[wundergraph(limit = true)]` Specifies if a limit
///         //   argument is generated for the current entity.
///         //   Possible Values: true, false
///         // * `#[wundergraph(offset = true)]` Specifies if an offset clause
///         //   argument is generated for the current entity.
///         //   Possible Values: true, false
///         // * `#[wundergraph(order = true)]` Specifies if an order clause
///         //   argument is generated for the current entity.
///         //   Possible Values: true, false
///         //
///         // Default values for all options are true. As shown below it is
///         // possible to have multiple flags in one attribute.
///         //
///         #[wundergraph(filter = false)]
///         #[wundergraph(offset = true, order = false, limit = false,)]
///         Species,
///         // It is also possible to register additional arguments for entities.
///         // To use this arguments it is required to provide a manual
///         // implementation of LoadingHandler
///         HomeWorld(
///             // Define a required argument for the entity
///             additional_arg: String,
///             // Define a optional argument for the entity with a
///             // given default values
///             #[wundergraph(default = None)]
///             another_arg: Option<String>
///         ),
///     }
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! query_object {
    (
        $(#[doc = $glob_doc: expr])*
        $query_name: ident {
            $(
                $(#[$($meta: tt)*])*
                $graphql_struct: ident$((
                        $($(#[wundergraph(default = $arg_default: expr)])? $arg: ident : $arg_ty: ty $(,)?)*
                ))?$(,)?)*
        }
    ) => {

        #[derive(Debug)]
        $(#[doc = $glob_doc])*
        // Use Arc<Mutex<C>> here to force make this Sync
        pub struct $query_name<C>(::std::marker::PhantomData<std::sync::Arc<std::sync::Mutex<C>>>);

        impl<C> Default for $query_name<C> {
            fn default() -> Self {
                $query_name(::std::marker::PhantomData)
            }
        }


        $crate::paste::item!{
            $crate::__impl_graphql_obj_for_query! {
                query_name = {$query_name},
                structs = [$($graphql_struct,)*],
                body = {
                    type Context = Ctx;

                    type TypeInfo = ();

                    fn name(info: &Self::TypeInfo) -> ::std::option::Option<&str> {
                        <[<$query_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::name(info)
                    }

                    fn meta<'r>(
                        info: &Self::TypeInfo,
                        registry: &mut $crate::juniper::Registry<'r, $crate::scalar::WundergraphScalarValue>
                    ) -> $crate::juniper::meta::MetaType<'r, $crate::scalar::WundergraphScalarValue>
                    where
                        $crate::scalar::WundergraphScalarValue: 'r
                    {
                        <[<$query_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::meta(info, registry)
                    }

                    fn resolve_field(
                        &self,
                        info: &Self::TypeInfo,
                        field_name: &str,
                        arguments: &$crate::juniper::Arguments<$crate::scalar::WundergraphScalarValue>,
                        executor: &$crate::juniper::Executor<Self::Context, $crate::scalar::WundergraphScalarValue>,
                    ) -> $crate::juniper::ExecutionResult<$crate::scalar::WundergraphScalarValue> {
                        let wrapper = [<$query_name _wrapper>](
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
            pub struct [<$query_name _wrapper>]<'a, C>(
                // Use Arc<Mutex<C>> here to force make this Sync
                ::std::marker::PhantomData<std::sync::Arc<std::sync::Mutex<C>>>,
                &'a str,
                &'a $crate::juniper::Arguments<'a, $crate::scalar::WundergraphScalarValue>,
            );

            $crate::__impl_graphql_obj_for_query! {
                query_name = {[<$query_name _wrapper>]},
                structs = [$($graphql_struct,)*],
                lt = 'a,
                body = {
                    type Context = Ctx;

                    type TypeInfo = ();

                    fn name(info: &Self::TypeInfo) -> ::std::option::Option<&str> {
                        <[<$query_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::name(info)
                    }

                    fn meta<'r>(
                        info: &Self::TypeInfo,
                        registry: &mut $crate::juniper::Registry<'r, $crate::scalar::WundergraphScalarValue>
                    ) -> $crate::juniper::meta::MetaType<'r, $crate::scalar::WundergraphScalarValue>
                    where
                        $crate::scalar::WundergraphScalarValue: 'r
                    {
                        <[<$query_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::meta(info, registry)
                    }

                    fn resolve(
                        &self,
                        info: &Self::TypeInfo,
                        selection_set: ::std::option::Option<&[$crate::juniper::Selection<$crate::scalar::WundergraphScalarValue>]>,
                        executor: &$crate::juniper::Executor<Self::Context, $crate::scalar::WundergraphScalarValue>,
                    ) -> $crate::juniper::Value<$crate::scalar::WundergraphScalarValue> {
                        let inner = [<$query_name _inner>] (
                            ::std::marker::PhantomData,
                            selection_set
                        );
                        let r = <[<$query_name _inner>]<Ctx> as $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>>::resolve_field(
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

            #[derive(Debug)]
            #[doc(hidden)]
            /// An internal helper type
            pub struct [<$query_name _inner>]<'a, C>(
                // Use Arc<Mutex<C>> here to force make this Sync
                ::std::marker::PhantomData<std::sync::Arc<std::sync::Mutex<C>>>,
                ::std::option::Option<&'a [$crate::juniper::Selection<'a, $crate::scalar::WundergraphScalarValue>]>,
            );

            $crate::__impl_graphql_obj_for_query! {
                query_name = {[<$query_name _inner>]},
                structs = [$($graphql_struct,)*],
                lt = 'a,
                body = {
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
                                    $($(
                                        $crate::__expand_additional_arg!(
                                            [$arg: $arg_ty $(= $arg_default)*]
                                            registry,
                                            field,
                                            info
                                        );
                                    )*)*
                                    $crate::__expand_filter!(
                                        registry,
                                        field,
                                        <Ctx as $crate::WundergraphContext>::Connection,
                                        $graphql_struct,
                                        meta = [$(#[$($meta)*],)*]
                                    );
                                    $crate::__expand_limit!(registry, field, info, meta = [$(#[$($meta)*],)*]);
                                    $crate::__expand_offset!(registry, field, info, meta = [$(#[$($meta)*],)*]);
                                    $crate::__expand_order!(
                                        registry,
                                        field,
                                        <Ctx as $crate::WundergraphContext>::Connection,
                                        $graphql_struct, meta = [$(#[$($meta)*],)*]);
                                    field
                                },
                                {
                                    let key_info = $crate::helper::PrimaryKeyInfo::default();
                                    let key = registry.arg::<
                                        $crate::helper::PrimaryKeyArgument<
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
                        use $crate::query_builder::selection::LoadingHandler;
                        use $crate::WundergraphContext;
                        use $crate::juniper::LookAheadMethods;
                        match field_name {
                            $(
                                $crate::__expand_name!($graphql_struct, $(#[$($meta)*],)*) => {
                                    let look_ahead = executor.look_ahead();
                                    let q = $graphql_struct::build_query(look_ahead.arguments(), &look_ahead)?;
                                    let items = $graphql_struct::load(&look_ahead, self.1, executor, q)?;
                                    Ok($crate::juniper::Value::List(items))
                                },
                                stringify!($graphql_struct) => {
                                    let look_ahead = executor.look_ahead();
                                    let q = $graphql_struct::build_query(look_ahead.arguments(), &look_ahead)?;
                                    let item = $graphql_struct::load_by_primary_key(&look_ahead, self.1, executor, q)?;
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
        }
    };
}
