#[macro_export]
macro_rules! wundergraph_mutation_object {
    (
        $mutation_name: ident {
            $($entity_name: ident(
                $(insert = $insert: ident)*
                    $(, update = $update: ident)*
                    $(, delete = $delete: ident)*
            ),)*
        }
    ) => {
        wundergraph_mutation_object!{
            $mutation_name(context = $crate::diesel::r2d2::PooledConnection<$crate::diesel::r2d2::ConnectionManager<Conn>>){
                $($entity_name(
                    $(insert = $insert)*
                    $(, update = $update)*
                    $(, delete = $delete)*
                ),)*
            }
        }
    };
    (
        $mutation_name: ident(context = $context: ty) {
            $($entity_name: ident(
                $(insert = $insert: ident)*
                $(, update = $update: ident)*
                $(, delete = $delete: ident)*
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

        impl<Conn> $crate::juniper::GraphQLType for $mutation_name<$crate::diesel::r2d2::Pool<$crate::diesel::r2d2::ConnectionManager<Conn>>>
        where Conn: $crate::diesel::Connection<TransactionManager = $crate::diesel::connection::AnsiTransactionManager> + 'static,
              Conn::Backend: $crate::diesel::backend::UsesAnsiSavepointSyntax + 'static + Clone,
              <Conn::Backend as $crate::diesel::backend::Backend>::QueryBuilder: Default,

        $(
            $(
                $insert: $crate::mutations::HandleInsert<
                    Conn::Backend,
                    $entity_name,
                    $context,
                >,
            )*
        )*
        $(
            $(
                $delete: $crate::mutations::HandleDelete<
                    Conn::Backend,
                    $entity_name,
                    $context,
                >,
              )*
        )*
        $(
            $(
                $update: $crate::mutations::HandleUpdate<
                    Conn::Backend,
                    $entity_name,
                    $context,
                >,
            )*
        )*
        $(
            $entity_name: $crate::LoadingHandler<Conn::Backend, Context = $context>,
         )*
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
                    $(
                        let delete = registry.arg::<$delete>(concat!("Delete", stringify!($entity_name)), info);
                        let delete = registry.field::<Option<$entity_name>>(concat!("Delete", stringify!($entity_name)), info)
                            .argument(delete);
                        fields.push(delete);
                    )*
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
                                $crate::mutations::handle_insert::<Conn::Backend, $insert, _, _>(
                                    executor,
                                    arguments,
                                    concat!("New", stringify!($entity_name))
                                )
                            }
                            concat!("Create", stringify!($entity_name), "s") => {
                                $crate::mutations::handle_batch_insert::<Conn::Backend, $insert, _, _>(
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
                                    Conn::Backend,
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
                                    Conn::Backend,
                                    $delete,
                                    $entity_name,
                                    Self::Context
                                >(
                                    executor,
                                    arguments,
                                    concat!("Delete", stringify!($entity_name))
                                )
                            }
                        )*
                    )*
                    e => Err($crate::juniper::FieldError::new(
                        "Unknown field:",
                        $crate::juniper::Value::String(e.to_owned()),
                    )),
                }
            }
        }
    };
}
