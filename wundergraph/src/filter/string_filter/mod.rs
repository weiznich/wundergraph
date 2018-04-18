use filter::build_filter::BuildFilter;
use filter::inner_filter::InnerFilter;
use filter::transformator::Transformator;

use diesel::backend::Backend;
use diesel::sql_types::Bool;
use diesel::{BoxableExpression, Column};

use juniper::meta::Argument;
use juniper::{FromInputValue, InputValue, LookAheadValue, Registry, ToInputValue};

use ordermap::OrderMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};

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
    Like<C>: BuildFilter<DB, Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

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

    fn from_inner_input_value(obj: OrderMap<&str, &InputValue>) -> Option<Self> {
        let like = obj.get("like")
            .map(|v| Option::from_input_value(*v))
            .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
        let like = match like {
            Some(like) => Like::new(like),
            None => return None,
        };
        Some(Self { like })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue)]) -> Self {
        let like = obj.iter()
            .find(|o| o.0 == "like")
            .and_then(|o| String::from_look_ahead(&o.1));
        Self {
            like: Like::new(like),
        }
    }

    fn to_inner_input_value(&self, map: &mut OrderMap<&str, InputValue>) {
        map.insert("is_null", self.like.to_input_value());
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>> {
        let like = registry.arg_with_default::<Option<String>>("like", &None, &Default::default());
        vec![like]
    }
}
