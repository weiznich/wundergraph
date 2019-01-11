use helper::*;
use serde_json::Value;
use wundergraph_example::MyContext;

#[test]
fn simple_query_single_field() {
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
    assert_eq!(
        json!([{"Heros": [
        {"heroName": "Luke Skywalker"},
        {"heroName": "Darth Vader"},
        {"heroName": "Han Solo"},
        {"heroName": "Leia Organa"},
        {"heroName": "Wilhuff Tarkin"}
    ]}, []]),
        res.as_json()
    );
}

#[test]
fn simple_query_multiple_field() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros {
        id
        heroName
    }
}
",
    );
    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
        {"id": 1, "heroName": "Luke Skywalker"},
        {"id": 2, "heroName": "Darth Vader"},
        {"id": 3, "heroName": "Han Solo"},
        {"id": 4, "heroName": "Leia Organa"},
        {"id": 5, "heroName": "Wilhuff Tarkin"}
    ]}, []]),
        res.as_json()
    );
}

#[test]
fn simple_query_nested() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        "
{
    Heros {
        heroName
        home_world {
             name
        }
    }
}
",
    );
    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
        {"heroName": "Luke Skywalker", "home_world": {"name": "Tatooine"}},
        {"heroName": "Darth Vader", "home_world": {"name": "Tatooine"}},
        {"heroName": "Han Solo", "home_world": Value::Null},
        {"heroName": "Leia Organa", "home_world": {"name": "Alderaan"}},
        {"heroName": "Wilhuff Tarkin", "home_world": Value::Null}
    ]}, []]),
        res.as_json()
    );
}
