use filter::build_filter::BuildFilter;
use filter::inner_filter::InnerFilter;
use filter::transformator::Transformator;

use diesel::backend::Backend;
use diesel::sql_types::Bool;
use diesel::Column;
use diesel_ext::BoxableFilter;

use juniper::meta::Argument;
use juniper::{FromInputValue, InputValue, LookAheadValue, Registry, ToInputValue};

use indexmap::IndexMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};
use scalar::WundergraphScalarValue;

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
    Like<C>: BuildFilter<DB, Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>>,
{
    type Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        self.like.into_filter(t)
    }
}

impl<C> InnerFilter for StringFilter<C> {
    type Context = ();

    const FIELD_COUNT: usize = 1;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let like = obj
            .get("like")
            .map(|v| Option::from_input_value(*v))
            .unwrap_or_else(|| {
                let v: &InputValue<WundergraphScalarValue> = &InputValue::Null;
                Option::from_input_value(v)
            });
        let like = match like {
            Some(like) => Like::new(like),
            None => return None,
        };
        Some(Self { like })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<WundergraphScalarValue>)]) -> Self {
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
