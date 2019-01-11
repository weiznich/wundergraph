use helper::*;
use wundergraph_example::MyContext;
use serde_json::Value;

#[test]
fn order_asc() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros {
        heroName
    }
}
",
    );
    assert!(res.is_ok());
    assert_eq!(json!([{"Heros": [
        {"heroName": "Luke Skywalker"},
        {"heroName": "Darth Vader"},
        {"heroName": "Han Solo"},
        {"heroName": "Leia Organa"},
        {"heroName": "Wilhuff Tarkin"}
    ]}, []]), res.as_json());
}
