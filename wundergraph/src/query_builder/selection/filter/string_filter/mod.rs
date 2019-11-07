use crate::diesel_ext::BoxableFilter;
use crate::juniper_ext::{FromLookAheadValue, NameBuilder, Nameable};
use crate::query_builder::selection::filter::build_filter::BuildFilter;
use crate::query_builder::selection::filter::inner_filter::InnerFilter;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::sql_types::Bool;
use diesel::Column;
use indexmap::IndexMap;
use juniper::meta::Argument;
use juniper::{FromInputValue, InputValue, LookAheadValue, Registry, ToInputValue};

mod like;
use self::like::Like;

#[derive(Debug)]
pub struct StringFilter<C> {
    like: Like<C>,
}

impl<C> Clone for StringFilter<C> {
    fn clone(&self) -> Self {
        Self {
            like: self.like.clone(),
        }
    }
}

impl<C> Nameable for StringFilter<C> {
    fn name() -> String {
        String::new()
    }
}

impl<C, DB> BuildFilter<DB> for StringFilter<C>
where
    DB: Backend,
    C: Column,
    Like<C>: BuildFilter<DB, Ret = Box<dyn BoxableFilter<C::Table, DB, SqlType = Bool>>>,
{
    type Ret = Box<dyn BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        self.like.into_filter()
    }
}

impl<C> InnerFilter for StringFilter<C> {
    type Context = ();

    const FIELD_COUNT: usize = 1;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let like = Like::new(obj.get("like").map_or_else(
            || {
                let v: &InputValue<WundergraphScalarValue> = &InputValue::Null;
                Option::from_input_value(v)
            },
            |v| Option::from_input_value(*v),
        )?);
        Some(Self { like })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {
        let like = obj
            .iter()
            .find(|o| o.0 == "like")
            .and_then(|o| String::from_look_ahead(&o.1));
        Self {
            like: Like::new(like),
        }
    }

    fn to_inner_input_value(&self, map: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        map.insert("like", self.like.to_input_value());
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        let like = registry.arg_with_default::<Option<String>>("like", &None, &Default::default());
        vec![like]
    }
}
