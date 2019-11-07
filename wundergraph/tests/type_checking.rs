use crate::helper::*;
use wundergraph_example::MyContext;

#[test]
fn test_type_decoration() {
    let (schema, pool) = get_example_schema();
    let ctx = MyContext::new(pool.get().unwrap());

    let res = execute_query(
        &schema,
        &ctx,
        r#"
{
  __type(name: "Hero") {
    name
    description
    fields {
      name
      description
      isDeprecated
      deprecationReason
    }
  }
}
"#,
    );

    assert!(res.is_ok());
    assert_json_snapshot!(
        res.as_json(), @r###"[
  {
    "__type": {
      "description": "A hero from Star Wars",
      "fields": [
        {
          "deprecationReason": null,
          "description": "Internal id of a hero",
          "isDeprecated": false,
          "name": "id"
        },
        {
          "deprecationReason": null,
          "description": "The name of a hero",
          "isDeprecated": false,
          "name": "heroName"
        },
        {
          "deprecationReason": null,
          "description": "Which species a hero belongs to",
          "isDeprecated": false,
          "name": "species"
        },
        {
          "deprecationReason": null,
          "description": "On which world a hero was born",
          "isDeprecated": false,
          "name": "home_world"
        },
        {
          "deprecationReason": null,
          "description": "Episodes a hero appears in",
          "isDeprecated": false,
          "name": "appears_in"
        },
        {
          "deprecationReason": null,
          "description": "List of friends of the current hero",
          "isDeprecated": false,
          "name": "friends"
        }
      ],
      "name": "Hero"
    }
  },
  []
]"###
    );
}
