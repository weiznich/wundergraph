#[macro_export]
macro_rules! __wundergraph_delete_key_names {
    ($enity: ident,($key1: tt, $key2: tt, $key3: tt, $key4: tt, $key5: tt)) => {
        (
            concat!("Delete", stringify!($entity), "_id_1"),
            concat!("Delete", stringify!($entity), "_id_2"),
            concat!("Delete", stringify!($entity), "_id_3"),
            concat!("Delete", stringify!($entity), "_id_4"),
            concat!("Delete", stringify!($entity), "_id_5"),
        )
    };
    ($entity: ident,($key1: tt, $key2: tt, $key3: tt, $key4: tt)) => {
        (
            concat!("Delete", stringify!($entity), "_id_1"),
            concat!("Delete", stringify!($entity), "_id_2"),
            concat!("Delete", stringify!($entity), "_id_3"),
            concat!("Delete", stringify!($entity), "_id_4"),
        )
    };
    ($entity: ident,($key1: tt, $key2: tt, $key3: tt,)) => {
        (
            concat!("Delete", stringify!($entity), "_id_1"),
            concat!("Delete", stringify!($entity), "_id_2"),
            concat!("Delete", stringify!($entity), "_id_3"),
        )
    };
    ($entity: ident,($key1: tt, $key2: tt)) => {
        (
            concat!("Delete", stringify!($entity), "_id_1"),
            concat!("Delete", stringify!($entity), "_id_2"),
        )
    };
    ($entity: ident, $key1: tt) => {
        concat!("Delete", stringify!($entity), "_id")
    };
}

#[macro_export]
macro_rules! __wundergraph_mutation_expand_delete {
    ($entity: ident, $executor: ident, $arguments: ident, $primary_key: tt,true,) => {
        handle_delete::<Conn, $entity, _, _, _>(
            $executor,
            $arguments,
            __wundergraph_delete_key_names!($entity, $primary_key),
        )
    };
    ($entity: ident, $executor: ident, $arguments: ident, $primary_key: tt,false,) => {
        Err(FieldError::new(
            "Unknown field:",
            Value::String(concat!("Delete", stringify!($entity)).to_string()),
        ))
    };
    ($entity: ident, $executor: ident, $arguments: ident, $primary_key: tt,) => {
        __wundergraph_mutation_expand_delete!(
            $entity,
            $executor,
            $arguments,
            $primary_key,
            true,
        )
    };
}

#[macro_export]
macro_rules! __wundergraph_mutation_register_delete_args{
    ($entity: ident, $registry: ident, $info: ident, $delete: ident,
     ($key1: tt, $key2:tt, $key3: tt, $key4: tt, $key5: tt)) => {
        let name = concat!("Delete", stringify!($entity), "_id_1");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name = concat!("Delete", stringify!($entity), "_id_2");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name=concat!("Delete", stringify!($entity), "_id_3");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name = concat!("Delete", stringify!($entity), "_id_4");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name = concat!("Delete", stringify!($entity), "_id_5");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
    };
    ($entity: ident, $registry: ident, $info: ident, $delete: ident,
     ($key1: tt, $key2:tt, $key3: tt, $key4: tt) ) => {
        let name = concat!("Delete", stringify!($entity), "_id_1");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name = concat!("Delete", stringify!($entity), "_id_2");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name=concat!("Delete", stringify!($entity), "_id_3");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name = concat!("Delete", stringify!($entity), "_id_4");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
    };
    ($entity: ident, $registry: ident, $info: ident, $delete: ident,
     ($key1: tt, $key2:tt, $key3: tt,) ) => {
        let name = concat!("Delete", stringify!($entity), "_id_1");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name = concat!("Delete", stringify!($entity), "_id_2");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name=concat!("Delete", stringify!($entity), "_id_3");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
    };
    ($entity: ident, $registry: ident, $info: ident, $delete: ident,
     ($key1: tt, $key2:tt) ) => {
        let name = concat!("Delete", stringify!($entity), "_id_1");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
        let name = concat!("Delete", stringify!($entity), "_id_2");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
    };
    ($entity: ident, $registry: ident, $info: ident, $delete: ident,
     $key1: tt) => {
        let name = concat!("Delete", stringify!($entity), "_id_1");
        let arg = $registry.arg::<$key1>(name, $info);
        $delete = $delete.argument(arg);
    };
}

#[macro_export]
macro_rules! __wundergraph_mutation_register_delete {
    ($fields: ident, $registry: ident, $info: ident,
     $entity_name: ident, $primary_key: tt, true, ) => {
        let mut delete = $registry.field::<Option<$entity_name>>(concat!("Delete", stringify!($entity_name)), $info);
        __wundergraph_mutation_register_delete_args!($entity_name, $registry, $info, delete, $primary_key);
        $fields.push(delete);
    };
    ($fields: ident, $registry: ident, $info: ident, $entity_name: ident, $primary_key: tt,, false, )=> {};
    ($fields: ident, $registry: ident, $info: ident, $entity_name: ident, $primary_key: tt, )=> {
        __wundergraph_mutation_register_delete!($fields, $registry, $info, $entity_name, $primary_key, true, )
    }

}

#[macro_export]
macro_rules! wundergraph_mutation_object {
    (
        $mutation_name: ident {
            $($entity_name: ident(
                key = $primary_key: tt,
                table = $table: ty
                $(, insert = $insert: ident)*
                $(, update = $update: ident)*
                $(, delete = $delete: tt)*
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
                Conn::Backend: $crate::mutations::HandleInsert<
                    Conn,
                    $insert,
                    $table,
                    $entity_name,
                    $primary_key,
                >,
            )*
        )*
        $(
            Conn::Backend: $crate::mutations::HandleDelete<
                Conn,
                $table,
                $primary_key,
                $entity_name
            >,
        )*
        $(
            $(
                Conn::Backend: $crate::mutations::HandleUpdate<
                    Conn,
                    $update,
                    $entity_name
                    >,
            )*
        )*
        $(
            $entity_name: $crate::LoadingHandler<Conn, SqlType = <$table as $crate::diesel::query_builder::AsQuery>::SqlType, Table = $table>,
        )*
        {
            type Context = $crate::diesel::r2d2::PooledConnection<$crate::diesel::r2d2::ConnectionManager<Conn>>;
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
                    __wundergraph_mutation_register_delete!(fields, registry, info, $entity_name, $primary_key, $($delete, )*);
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
                use $crate::mutations::{handle_insert, handle_batch_insert, handle_delete, HandleUpdate};
                match field_name {
                    $(
                        $(
                            concat!("Create", stringify!($entity_name)) => {
                                handle_insert::<Conn, $insert, _, _, _>(
                                    executor,
                                    arguments,
                                    concat!("New", stringify!($entity_name))
                                )
                            }
                            concat!("Create", stringify!($entity_name), "s") => {
                                handle_batch_insert::<Conn, $insert, _, _, _>(
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
                                if let Some(update) = arguments.get::<$update>(
                                    concat!("Update", stringify!($entity_name))
                                ) {
                                    Conn::Backend::handle_update(executor, &update)
                                } else {
                                    Err($crate::juniper::FieldError::new(
                                        concat!("Missing argument Update",
                                                stringify!($entity_name)),
                                        $crate::juniper::Value::Null))
                                }
                            }
                        )*
                    )*
                    $(
                        concat!("Delete", stringify!($entity_name)) => {
                            __wundergraph_mutation_expand_delete!($entity_name, executor, arguments, $primary_key, $($delete, )*)
                        }
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
