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
    assert_eq!(
        json!([{
            "__type": {
                "name": "Hero",
                "description": "A hero from Star Wars",
                "fields": [
                    {
                        "name": "id",
                        "description": "Internal id of a hero",
                        "isDeprecated": false,
                        "deprecationReason": null
                    },
                    {
                        "name": "heroName",
                        "description": "The name of a hero",
                        "isDeprecated": false,
                        "deprecationReason": null
                    },
                    {
                        "name": "species",
                        "description": "Which species a hero belongs to",
                        "isDeprecated": false,
                        "deprecationReason": null
                    },
                    {
                        "name": "home_world",
                        "description": "On which world a hero was born",
                        "isDeprecated": false,
                        "deprecationReason": null
                    },
                    {
                        "name": "friends",
                        "description": "List of friends of the current hero",
                        "isDeprecated": false,
                        "deprecationReason": null
                    }
                ]
            }
        }, []]),
        res.as_json()
    );
}
