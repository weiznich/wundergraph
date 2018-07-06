use helper::FromLookAheadValue;
use juniper::LookAheadValue;

#[derive(Debug, GraphQLEnum, Copy, Clone, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl FromLookAheadValue for Order {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Enum(e) = *v {
            match e {
                "ASC" => Some(Order::Asc),
                "DESC" => Some(Order::Desc),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug, GraphQLInputObject)]
pub struct OrderBy {
    pub column: String,
    pub direction: Order,
}

impl FromLookAheadValue for OrderBy {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let column = obj.iter()
                .find(|o| o.0 == "column")
                .and_then(|o| String::from_look_ahead(&o.1));
            let column = match column {
                Some(column) => column,
                None => return None,
            };
            let direction = obj.iter()
                .find(|o| o.0 == "direction")
                .and_then(|o| Order::from_look_ahead(&o.1));
            let direction = match direction {
                Some(direction) => direction,
                None => return None,
            };
            Some(Self { column, direction })
        } else {
            None
        }
    }
}
