use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn update_existing() {
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
        hair_color
    }
}
",
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
            {"id": 1, "heroName": "Luke Skywalker", "hair_color": "blond"},
            {"id": 2, "heroName": "Darth Vader", "hair_color": null},
            {"id": 3, "heroName": "Han Solo", "hair_color": null},
            {"id": 4, "heroName": "Leia Organa", "hair_color": null},
            {"id": 5, "heroName": "Wilhuff Tarkin", "hair_color": null}
        ]}, []]),
        res.as_json()
    );

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation updateHero {
  UpdateHero(UpdateHero: {id: 4, hairColor: "dark"}) {
    heroName
    hair_color
  }
}
"#
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"UpdateHero": {
            "heroName": "Leia Organa",
            "hair_color": "dark"
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
        hair_color
    }
}
",
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
            {"id": 1, "heroName": "Luke Skywalker", "hair_color": "blond"},
            {"id": 2, "heroName": "Darth Vader", "hair_color": null},
            {"id": 3, "heroName": "Han Solo", "hair_color": null},
            {"id": 4, "heroName": "Leia Organa", "hair_color": "dark"},
            {"id": 5, "heroName": "Wilhuff Tarkin", "hair_color": null}
        ]}, []]),
        res.as_json()
    );
}


#[test]
fn update_non_existing() {
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
        hair_color
    }
}
",
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
            {"id": 1, "heroName": "Luke Skywalker", "hair_color": "blond"},
            {"id": 2, "heroName": "Darth Vader", "hair_color": null},
            {"id": 3, "heroName": "Han Solo", "hair_color": null},
            {"id": 4, "heroName": "Leia Organa", "hair_color": null},
            {"id": 5, "heroName": "Wilhuff Tarkin", "hair_color": null}
        ]}, []]),
        res.as_json()
    );

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation updateHero {
  UpdateHero(UpdateHero: {id: 42, hairColor: "dark"}) {
    heroName
    hair_color
  }
}
"#
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"UpdateHero": null}, []]),
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
        hair_color
    }
}
",
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"Heros": [
            {"id": 1, "heroName": "Luke Skywalker", "hair_color": "blond"},
            {"id": 2, "heroName": "Darth Vader", "hair_color": null},
            {"id": 3, "heroName": "Han Solo", "hair_color": null},
            {"id": 4, "heroName": "Leia Organa", "hair_color": null},
            {"id": 5, "heroName": "Wilhuff Tarkin", "hair_color": null}
        ]}, []]),
        res.as_json()
    );
}
