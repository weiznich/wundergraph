use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn limit() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(limit: 2) {
        heroName
    }
}
",
    );
    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
        {"heroName": "Luke Skywalker"},
        {"heroName": "Darth Vader"},
    ]}, []]),
        res.as_json()
    );
}

#[test]
fn offset() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(offset: 2) {
        heroName
    }
}
",
    );
    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
        {"heroName": "Han Solo"},
        {"heroName": "Leia Organa"},
        {"heroName": "Wilhuff Tarkin"}
    ]}, []]),
        res.as_json()
    );
}

#[test]
fn limit_offset() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(limit: 2, offset: 2) {
        heroName
    }
}
",
    );
    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
        {"heroName": "Han Solo"},
        {"heroName": "Leia Organa"},
    ]}, []]),
        res.as_json()
    );
}
