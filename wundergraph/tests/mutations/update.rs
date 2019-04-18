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
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "hair_color": "blond",
        "heroName": "Luke Skywalker",
        "id": 1
      },
      {
        "hair_color": null,
        "heroName": "Darth Vader",
        "id": 2
      },
      {
        "hair_color": null,
        "heroName": "Han Solo",
        "id": 3
      },
      {
        "hair_color": null,
        "heroName": "Leia Organa",
        "id": 4
      },
      {
        "hair_color": null,
        "heroName": "Wilhuff Tarkin",
        "id": 5
      }
    ]
  },
  []
]"###
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
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "UpdateHero": {
      "hair_color": "dark",
      "heroName": "Leia Organa"
    }
  },
  []
]"###
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
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "hair_color": "blond",
        "heroName": "Luke Skywalker",
        "id": 1
      },
      {
        "hair_color": null,
        "heroName": "Darth Vader",
        "id": 2
      },
      {
        "hair_color": null,
        "heroName": "Han Solo",
        "id": 3
      },
      {
        "hair_color": null,
        "heroName": "Wilhuff Tarkin",
        "id": 5
      },
      {
        "hair_color": "dark",
        "heroName": "Leia Organa",
        "id": 4
      }
    ]
  },
  []
]"###
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
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "hair_color": "blond",
        "heroName": "Luke Skywalker",
        "id": 1
      },
      {
        "hair_color": null,
        "heroName": "Darth Vader",
        "id": 2
      },
      {
        "hair_color": null,
        "heroName": "Han Solo",
        "id": 3
      },
      {
        "hair_color": null,
        "heroName": "Leia Organa",
        "id": 4
      },
      {
        "hair_color": null,
        "heroName": "Wilhuff Tarkin",
        "id": 5
      }
    ]
  },
  []
]"###
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
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "UpdateHero": null
  },
  []
]"###
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
    assert_json_snapshot_matches!(
        res.as_json(), @r###"[
  {
    "Heros": [
      {
        "hair_color": "blond",
        "heroName": "Luke Skywalker",
        "id": 1
      },
      {
        "hair_color": null,
        "heroName": "Darth Vader",
        "id": 2
      },
      {
        "hair_color": null,
        "heroName": "Han Solo",
        "id": 3
      },
      {
        "hair_color": null,
        "heroName": "Leia Organa",
        "id": 4
      },
      {
        "hair_color": null,
        "heroName": "Wilhuff Tarkin",
        "id": 5
      }
    ]
  },
  []
]"###
    );
}
