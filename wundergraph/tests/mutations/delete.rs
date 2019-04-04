use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn delete_existing() {
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

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation DeleteHero {
  DeleteHero(DeleteHero: {id: 5}) {
    count
  }
}
"#
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"DeleteHero": {
            "count": 1
        }}, []]),
        res.as_json()
    );

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
        ]}, []]),
        res.as_json()
    );
}

#[test]
fn delete_non_existing() {
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

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation DeleteHero {
  DeleteHero(DeleteHero: {id: 42}) {
    count
  }
}
"#
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"DeleteHero": {
            "count": 0
        }}, []]),
        res.as_json()
    );

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
