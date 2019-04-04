use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn create_one() {
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

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation NewHero {
  CreateHero(NewHero: {name: "Obi-Wan Kenobi", species: 1}) {
    heroName
    species {
      name
    }
  }
}
"#
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"CreateHero": {
            "heroName": "Obi-Wan Kenobi",
            "species": {
                "name": "Human"
            }
        }}, []]),
        res.as_json()
    );

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
            {"heroName": "Wilhuff Tarkin"},
            {"heroName": "Obi-Wan Kenobi"},
        ]}, []]),
        res.as_json()
    );
}


#[test]
fn create_multiple() {
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

    let res = execute_query(
        &schema,
        &ctx,
        r#"
mutation NewHeros {
  CreateHeros(NewHeros: [{name: "Obi-Wan Kenobi", species: 1}, {name: "R2-D2", species: 2}]) {
    heroName
    species {
      name
    }
  }
}
"#
    );

    assert!(res.is_ok());
    assert_eq!(
        json!([{"CreateHeros": [
            {
                "heroName": "Obi-Wan Kenobi",
                "species": {
                    "name": "Human"
                }
            },
            {
                "heroName": "R2-D2",
                "species": {
                    "name": "Robot"
                }
            }
        ]}, []]),
        res.as_json()
    );

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
            {"heroName": "Wilhuff Tarkin"},
            {"heroName": "Obi-Wan Kenobi"},
            {"heroName": "R2-D2"},
        ]}, []]),
        res.as_json()
    );
}
