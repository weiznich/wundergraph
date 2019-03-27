#[macro_export]
macro_rules! mutation_object {
    (
        $(#[doc = $glob_doc: expr])*
        $mutation_name: ident {
            $($entity_name: ident (
                $(insert = $insert: ident,)?
                $(update = $update: ident,)?
                $(,)?
            ),)*
        }
    ) => {
        $crate::mutation_object!{
            $(#[doc = $glob_doc])*
            $mutation_name {
                $($entity_name($(insert = $insert,)* $(update = $update,)* delete = true),)*
            }
        }
    };
    (
        $(#[doc = $glob_doc: expr])*
        $mutation_name: ident {
            $($entity_name: ident (
                $(insert = $insert: ident,)?
                $(update = $update: ident,)?
                delete = false
                $(,)?
            ),)*
        }
    ) => {
        $crate::__impl_mutation_object! {
            $(#[doc = $glob_doc])*
            $mutation_name {
                $($entity_name($(insert = $insert,)* $(update = $update,)*),)*
            }
        }
    };
    (
        $(#[doc = $glob_doc: expr])*
        $mutation_name: ident {
            $($entity_name: ident (
                $(insert = $insert: ident,)?
                $(update = $update: ident,)?
                delete = true
                $(,)?
            ),)*
        }
    ) => {
        $crate::__impl_mutation_object! {
            $(#[doc = $glob_doc])*
            $mutation_name {
                $($entity_name($(insert = $insert,)* $(update = $update,)*
                               delete = ($crate::helper::primary_keys::PrimaryKeyArgument<
                               'static,
                               <$entity_name as $crate::diesel::associations::HasTable>::Table,
                               Ctx,
                               <&'static $entity_name as $crate::diesel::Identifiable>::Id
                >)),)*
            }
        }
    };
    (
        $(#[doc = $glob_doc: expr])*
        $mutation_name: ident {
            $($entity_name: ident (
                $(insert = $insert: ident,)?
                $(update = $update: ident,)?
                delete = $delete: ty
                $(,)?
            ),)*
        }
    ) => {
        $crate::__impl_mutation_object! {
            $(#[doc = $glob_doc])*
            $mutation_name {
                $($entity_name($(insert = $insert,)* $(update = $update,)* delete = ($delete)),)*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_mutation_object {
    (
        $(#[doc = $glob_doc: expr])*
        $mutation_name: ident {
            $($entity_name: ident (
                $(insert = $insert: ident,)?
                $(update = $update: ident,)?
                $(delete = ($($delete:tt)*))?
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
            impl<Ctx, DB, $([<$entity_name _table>],)* $([<$entity_name _id>],)*> $crate::juniper::GraphQLType<$crate::scalar::WundergraphScalarValue>
                for $mutation_name<Ctx>
            where Ctx: $crate::WundergraphContext,
                  DB: $crate::diesel::backend::Backend + 'static,
                  DB::QueryBuilder: std::default::Default,
                  Ctx::Connection: $crate::diesel::Connection<Backend = DB>,
                  $($entity_name: $crate::LoadingHandler<DB, Ctx> + $crate::diesel::associations::HasTable<Table = [<$entity_name _table>]>,)*
                  $([<$entity_name _table>]: $crate::diesel::Table + 'static +
                      $crate::diesel::QuerySource<FromClause = $crate::diesel::query_builder::nodes::Identifier<'static>>,)*
                  $([<$entity_name _table>]::FromClause: $crate::diesel::query_builder::QueryFragment<DB>,)*
                  $(<$entity_name as $crate::LoadingHandler<DB, Ctx>>::Columns: $crate::query_helper::order::BuildOrder<[<$entity_name _table>], DB>,)*
                  $(<$entity_name as $crate::LoadingHandler<DB, Ctx>>::Columns: $crate::query_helper::select::BuildSelect<
                      [<$entity_name _table>],
                      DB,
                      $crate::query_helper::placeholder::SqlTypeOfPlaceholder<
                      <$entity_name as $crate::LoadingHandler<DB, Ctx>>::FieldList,
                      DB,
                      <$entity_name as $crate::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                      [<$entity_name _table>],
                      Ctx,
                      >
                  >,)*
                  $(<$entity_name as $crate::LoadingHandler<DB, Ctx>>::FieldList: $crate::query_helper::placeholder::WundergraphFieldList<
                      DB,
                      <$entity_name as $crate::LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
                      [<$entity_name _table>],
                      Ctx
                  >,)*
                  $(<$entity_name as $crate::LoadingHandler<DB, Ctx>>::FieldList:
                      $crate::graphql_type::WundergraphGraphqlHelper<$entity_name, DB, Ctx> +
                    $crate::query_helper::placeholder::FieldListExtractor,)*
                  $(&'static $entity_name: $crate::diesel::Identifiable<Id = [<$entity_name _id>]>,)*
                  $([<$entity_name _id>]: std::hash::Hash + std::cmp::Eq + $crate::helper::primary_keys::UnRef<'static>,)*
                  $([<$entity_name _table>]::PrimaryKey: $crate::helper::primary_keys::PrimaryKeyInputObject<
                    <[<$entity_name _id>] as $crate::helper::primary_keys::UnRef<'static>>::UnRefed, ()
                  >,)*
                  $($([<$entity_name _table>]: $crate::mutations::HandleInsert<$entity_name, $insert, DB, Ctx>,)*)*
                  $($([<$entity_name _table>]: $crate::mutations::HandleBatchInsert<$entity_name, $insert, DB, Ctx>,)*)*
                  $($([<$entity_name _table>]: $crate::mutations::HandleUpdate<$entity_name, $update, DB, Ctx>,)*)*
                  $($([<$entity_name _table>]: $crate::mutations::HandleDelete<$entity_name, $($delete)*, DB, Ctx>,)*)*
            {
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
                             $(
                                 let delete = registry.arg::<$($delete)*>(
                                     concat!("Delete", stringify!($entity_name)),
                                     &$crate::helper::primary_keys::PrimaryKeyInfo::new(
                                         &<$entity_name as $crate::diesel::associations::HasTable>::table(),
                                     )
                                 );
                                 let delete = registry.field::<Option<$crate::mutations::DeletedCount>>(
                                     concat!("Delete", stringify!($entity_name)),
                                     info
                                 ).argument(delete);
                                 fields.push(delete);
                             )*
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
                                    $crate::mutations::handle_insert::<
                                        DB,
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
                                        DB,
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
                                        DB,
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
                            $(
                                concat!("Delete", stringify!($entity_name)) => {
                                    $crate::mutations::handle_delete::<
                                        DB,
                                        $($delete)*,
                                        $entity_name,
                                        Self::Context,
                                    >(executor, arguments, concat!("Delete", stringify!($entity_name)))
                                }
                            )*
                        )*
                        e => Err($crate::juniper::FieldError::new(
                            "Unknown field:",
                            $crate::juniper::Value::scalar(e),
                        )),
                    }
                }
            }
        }
    };
}
