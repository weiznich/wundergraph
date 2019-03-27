use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn order_asc() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(order: [{column: heroName, direction: ASC}]) {
        heroName
    }
}
",
    );
    println!("{:?}", res);
    assert!(res.is_ok());
    assert_eq!(json!([{"Heros": [
        {"heroName": "Darth Vader"},
        {"heroName": "Han Solo"},
        {"heroName": "Leia Organa"},
        {"heroName": "Luke Skywalker"},
        {"heroName": "Wilhuff Tarkin"}
    ]}, []]), res.as_json());
}


#[test]
fn order_desc() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros(order: [{column: heroName, direction: DESC}]) {
        heroName
    }
}
",
    );
    println!("{:?}", res);
    assert!(res.is_ok());
    assert_eq!(json!([{"Heros": [
        {"heroName": "Wilhuff Tarkin"},
        {"heroName": "Luke Skywalker"},
        {"heroName": "Leia Organa"},
        {"heroName": "Han Solo"},
        {"heroName": "Darth Vader"},
    ]}, []]), res.as_json());
}
