use super::WundergraphFieldList;
use crate::error::Result;
use crate::query_builder::selection::offset::ApplyOffset;
use crate::query_builder::selection::query_resolver::get_sub_field;
use crate::query_builder::selection::LoadingHandler;
use crate::query_builder::types::HasMany;
use crate::scalar::WundergraphScalarValue;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::QueryFragment;
use diesel::{QuerySource, SelectableExpression};
use juniper::{Executor, LookAheadMethods, Selection};
use std::collections::HashMap;
use std::hash::Hash;

pub trait AssociationsLookup<K> {
    fn new() -> Self;

    fn insert(
        &mut self,
        key: Option<K>,
        len: usize,
        values: Vec<juniper::Value<WundergraphScalarValue>>,
    );

    fn get(
        &self,
        key: &Option<K>,
    ) -> Option<Vec<(usize, Vec<juniper::Value<WundergraphScalarValue>>)>>;
}

impl<K> AssociationsLookup<K>
    for HashMap<Option<K>, Vec<(usize, Vec<juniper::Value<WundergraphScalarValue>>)>>
where
    K: Eq + Hash,
{
    fn new() -> Self {
        HashMap::new()
    }

    fn insert(
        &mut self,
        key: Option<K>,
        len: usize,
        values: Vec<juniper::Value<WundergraphScalarValue>>,
    ) {
        self.entry(key).or_insert_with(Vec::new).push((len, values));
    }

    fn get(
        &self,
        key: &Option<K>,
    ) -> Option<Vec<(usize, Vec<juniper::Value<WundergraphScalarValue>>)>> {
        self.get(key).cloned()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct AssociationsReturn<'a, K, C> {
    keys: Vec<Option<K>>,
    fields: Vec<&'a str>,
    values: C,
}

impl<'a, K, C> AssociationsReturn<'a, K, C>
where
    C: AssociationsLookup<K>,
{
    fn empty() -> Self {
        Self {
            keys: Vec::new(),
            fields: Vec::new(),
            values: C::new(),
        }
    }

    fn init(&mut self, get_keys: &impl Fn() -> Vec<Option<K>>) {
        if self.keys.is_empty() {
            self.keys = get_keys()
        }
    }

    fn push_field<T, O, DB, Ctx>(
        &mut self,
        field: &'static str,
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'a, WundergraphScalarValue>,
        selection: Option<&'a [Selection<'a, WundergraphScalarValue>]>,
        executor: &'a Executor<'a, Ctx, WundergraphScalarValue>,
    ) -> Result<()>
    where
        DB: Backend,
        T: WundergraphResolveAssociation<K, O, DB, Ctx>,
    {
        let (name, alias, loc, selection) = get_sub_field(field, selection);
        let executor = executor.field_sub_executor(alias, name, loc, selection);

        let values = T::resolve(global_args, look_ahead, selection, &self.keys, &executor)?;

        let len = self.fields.len();
        self.fields.push(alias);

        for (k, v) in values {
            self.values.insert(k, len, v);
        }
        Ok(())
    }

    pub(crate) fn merge_with_object_list(
        self,
        objs: Vec<juniper::Object<WundergraphScalarValue>>,
    ) -> Vec<juniper::Value<WundergraphScalarValue>> {
        let Self {
            values,
            keys,
            fields,
        } = self;
        if keys.is_empty() {
            objs.into_iter().map(juniper::Value::object).collect()
        } else {
            objs.into_iter()
                .zip(keys.into_iter())
                .map(|(mut obj, key)| {
                    let values = values.get(&key);
                    if let Some(values) = values {
                        for (idx, field_name) in fields.iter().enumerate() {
                            let vals = values
                                .iter()
                                .filter_map(
                                    |(field_idx, val)| {
                                        if idx == *field_idx {
                                            Some(val)
                                        } else {
                                            None
                                        }
                                    },
                                )
                                .cloned()
                                .flatten()
                                .collect::<Vec<_>>();

                            obj.add_field(field_name.to_owned(), juniper::Value::List(vals));
                        }
                    } else {
                        for f in &fields {
                            obj.add_field(f.to_owned(), juniper::Value::List(Vec::new()));
                        }
                    }
                    obj
                })
                .map(juniper::Value::object)
                .collect()
        }
    }
}

#[doc(hidden)]
pub trait WundergraphResolveAssociations<K, Other, DB, Ctx>
where
    DB: Backend,
{
    type Container: AssociationsLookup<K>;

    fn resolve<'a>(
        global_args: &'a [juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &'a juniper::LookAheadSelection<'a, WundergraphScalarValue>,
        selection: Option<&'a [Selection<'a, WundergraphScalarValue>]>,
        get_name: impl Fn(usize) -> &'static str,
        get_keys: impl Fn() -> Vec<Option<K>>,
        executor: &'a Executor<'a, Ctx, WundergraphScalarValue>,
    ) -> Result<AssociationsReturn<'a, K, Self::Container>>;
}

impl<K, Other, DB, Ctx> WundergraphResolveAssociations<K, Other, DB, Ctx> for ()
where
    K: Eq + Hash,
    DB: Backend,
{
    type Container = HashMap<Option<K>, Vec<(usize, Vec<juniper::Value<WundergraphScalarValue>>)>>;

    fn resolve<'a>(
        _global_args: &'a [juniper::LookAheadArgument<WundergraphScalarValue>],
        _look_ahead: &'a juniper::LookAheadSelection<'a, WundergraphScalarValue>,
        _selection: Option<&'a [Selection<'a, WundergraphScalarValue>]>,
        _get_name: impl Fn(usize) -> &'static str,
        _get_keys: impl Fn() -> Vec<Option<K>>,
        _executor: &'a Executor<'a, Ctx, WundergraphScalarValue>,
    ) -> Result<AssociationsReturn<'a, K, Self::Container>> {
        Ok(AssociationsReturn::empty())
    }
}

#[doc(hidden)]
pub trait WundergraphResolveAssociation<K, Other, DB: Backend, Ctx> {
    type Container: AssociationsLookup<K>;

    fn resolve(
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        primary_keys: &[Option<K>],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<K>, Vec<juniper::Value<WundergraphScalarValue>>>>;
}

/// A helper trait used to resolve a association given by a `HasOne` marker type
///
/// **This traits needs to implemented for concrete types, because otherwise rustc
///  is not able to proof that certain traits are require implemented because of
///  potential circular dependencies**
///
/// # Type parameters:
/// * `Self`: Type implementing `LoadingHandler`
/// * `Other`: Table of type referenced by field
/// * `DB`: Backend type from diesel, so one of `Pg` or `Sqlite`
/// * `Ctx`: The used wundergraph context type
/// * `FK`: Foreign key referencing the remote table
///
/// # Deriving
/// An implementation of this trait is automatically generated by
/// [`#[derive(WundergraphEntity)]`](../derive.WundergraphEntity.html)
///  or [`#[derive(WundergraphBelongsTo)]`](derive.WundergraphBelongsTo.html)
///  for each `HasOne` field
///
/// # Manual implementation
///
/// Manually implementing this trait should only be the last resort if none of
/// the provided derives generate compatible code. Below an expanded version of
/// the generated implelmentation is shown.
///
/// ```
/// # #[macro_use] extern crate diesel;
/// # use wundergraph::helper::TupleIndex0;
/// # use wundergraph::query_builder::selection::LoadingHandler;
/// # use wundergraph::WundergraphEntity;
/// #
/// use wundergraph::query_builder::types::HasOne;
/// use wundergraph::query_builder::selection::fields::WundergraphBelongsTo;
/// use wundergraph::WundergraphContext;
/// use wundergraph::scalar::WundergraphScalarValue;
/// use wundergraph::error::Result;
/// use juniper::{LookAheadSelection, LookAheadArgument, Selection, Executor};
/// # #[cfg(feature = "postgres")]
/// use diesel::pg::Pg;
/// use diesel::prelude::*;
/// use std::collections::HashMap;
///
/// table! {
///     heros {
///         id -> Integer,
///         name -> Text,
///         species -> Integer,
///     }
/// }
///
/// table! {
///     species {
///         id -> Integer,
///         name -> Text,
///     }
/// }
///
/// #[derive(Identifiable)]
/// struct Hero {
///   id: i32,
///   name: String,
///   species: HasOne<i32, Species>,
/// }
///
/// # #[derive(WundergraphEntity)]
/// #[derive(Identifiable)]
/// #[table_name = "species"]
/// struct Species {
///     id: i32,
///     name: String,
/// }
///
/// # #[cfg(feature = "postgres")]
/// # impl<Ctx> LoadingHandler<Pg, Ctx> for Hero
/// # where
/// #    Ctx: WundergraphContext + 'static,
/// #    <Ctx as WundergraphContext>::Connection: Connection<Backend = Pg>,
/// # {
/// #    type Columns = (heros::id, heros::name, heros::species);
/// #    type FieldList = (i32, String, HasOne<i32, Species>);
/// #    type PrimaryKeyIndex = TupleIndex0;
/// #    type Filter = ();
/// #
/// #    const FIELD_NAMES: &'static [&'static str] = &["id", "name", "species"];
/// #    const TYPE_NAME: &'static str = "Hero";
/// # }
///
/// # #[cfg(feature = "postgres")]
/// impl<Ctx> WundergraphBelongsTo<species::table, Pg, Ctx, heros::species> for Hero
/// where
///     Ctx: WundergraphContext + 'static,
///     <Ctx as WundergraphContext>::Connection: Connection<Backend = Pg>,
/// {
///    type Key = i32;
///
///    fn resolve(
///        global_args: &[LookAheadArgument<WundergraphScalarValue>],
///        look_ahead: &LookAheadSelection<'_, WundergraphScalarValue>,
///        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
///        keys: &[Option<Self::Key>],
///        executor: &Executor<'_, Ctx, WundergraphScalarValue>
///    ) -> Result<HashMap<Option<Self::Key>, Vec<juniper::Value<WundergraphScalarValue>>>>
///    {
///        let conn = executor.context().get_connection();
///        let query = <Self as LoadingHandler<Pg, Ctx>>::build_query(global_args, look_ahead)?
///            .select((
///                heros::species.nullable(),
///                <Self as LoadingHandler<Pg, Ctx>>::get_select(look_ahead)?
///             ))
///            .filter(heros::species.nullable().eq_any(keys));
///        <Self as WundergraphBelongsTo<species::table, Pg, Ctx, heros::species>>::build_response(
///            query.load(conn)?,
///            global_args,
///            look_ahead,
///            selection,
///            executor
///        )
///    }
/// }
/// # fn main() {}
/// ```
pub trait WundergraphBelongsTo<Other, DB, Ctx, FK>: LoadingHandler<DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    Self::Table: 'static,
    <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    FK: Default + NonAggregate + SelectableExpression<Self::Table> + QueryFragment<DB>,
{
    /// Foreign key type
    type Key: Eq + Hash;

    /// Actual function called to resolve the association.
    ///
    /// See the documentation of the trait for details how to implement this
    /// function
    fn resolve(
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        keys: &[Option<Self::Key>],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<Self::Key>, Vec<juniper::Value<WundergraphScalarValue>>>>;

    /// Common part of the implementation that could be implemented in a
    /// generic way. Call this as soon as you have all required data
    fn build_response(
        res: Vec<(
            Option<Self::Key>,
            <Self::FieldList as WundergraphFieldList<
                DB,
                Self::PrimaryKeyIndex,
                Self::Table,
                Ctx,
            >>::PlaceHolder,
        )>,
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<Self::Key>, Vec<juniper::Value<WundergraphScalarValue>>>> {
        let (keys, vals): (Vec<_>, Vec<_>) = res.into_iter().unzip();
        let vals = <<Self as LoadingHandler<DB, Ctx>>::FieldList as WundergraphFieldList<
            DB,
            <Self as LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
            <Self as HasTable>::Table,
            Ctx,
        >>::resolve(
            vals,
            global_args,
            look_ahead,
            selection,
            <Self as LoadingHandler<DB, Ctx>>::FIELD_NAMES,
            executor,
        )?;
        Ok(keys
            .into_iter()
            .zip(vals.into_iter())
            .fold(HashMap::new(), |mut m, (k, v)| {
                (*m.entry(k).or_insert_with(Vec::new)).push(v);
                m
            }))
    }
}

impl<T, K, Other, DB, Ctx, FK> WundergraphResolveAssociation<K, Other, DB, Ctx> for HasMany<T, FK>
where
    DB: Backend + ApplyOffset + 'static,
    FK: Default + NonAggregate + QueryFragment<DB> + SelectableExpression<T::Table>,
    T: WundergraphBelongsTo<Other, DB, Ctx, FK, Key = K>,
    K: Eq + Hash,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Container = HashMap<Option<K>, Vec<(usize, Vec<juniper::Value<WundergraphScalarValue>>)>>;

    fn resolve(
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        primary_keys: &[Option<K>],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<K>, Vec<juniper::Value<WundergraphScalarValue>>>> {
        T::resolve(global_args, look_ahead, selection, primary_keys, executor)
    }
}

macro_rules! wundergraph_impl_resolve_association {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<Key, Back, Other, Ctx, $($T,)* Container> WundergraphResolveAssociations<Key, Other, Back, Ctx> for ($($T,)*)
            where Back: Backend,
                  Container: AssociationsLookup<Key>,
                $($T: WundergraphResolveAssociation<Key, Other, Back, Ctx, Container = Container>,)*

            {
                type Container = Container;

                fn resolve<'a>(
                    global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
                    look_ahead: &'a juniper::LookAheadSelection<'a, WundergraphScalarValue>,
                    selection: Option<&'a [Selection<'a, WundergraphScalarValue>]>,
                    get_name: impl Fn(usize) -> &'static str,
                    get_keys: impl Fn() -> Vec<Option<Key>>,
                    executor: &'a Executor<'a, Ctx, WundergraphScalarValue>,
                ) -> Result<AssociationsReturn<'a, Key, Container>>
                {
                    let mut ret = AssociationsReturn::empty();
                    $(
                        if let Some(look_ahead) = look_ahead.select_child(get_name($idx)) {
                            ret.init(&get_keys);
                            ret.push_field::<$T, Other, Back, Ctx>(
                                get_name($idx),
                                global_args,
                                look_ahead,
                                selection,
                                executor
                            )?;
                        }
                    )*
                    Ok(ret)
                }
            }
        )*
    }
}

__diesel_for_each_tuple!(wundergraph_impl_resolve_association);
