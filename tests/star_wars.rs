extern crate graphql;
extern crate log;
extern crate env_logger;

use graphql::types::definition::*;
use graphql::GraphQL;

use self::env_logger::*;

use std::collections::HashMap;

/*
 * Using our shorthand to describe type systems, the type system for our
 * Star Wars example is:
 *
 * enum Episode { NEWHOPE, EMPIRE, JEDI }
 *
 * interface Character {
 *   id: String!
 *   name: String
 *   friends: [Character]
 *   appearsIn: [Episode]
 * }
 *
 * type Human : Character {
 *   id: String!
 *   name: String
 *   friends: [Character]
 *   appearsIn: [Episode]
 *   homePlanet: String
 * }
 *
 * type Droid : Character {
 *   id: String!
 *   name: String
 *   friends: [Character]
 *   appearsIn: [Episode]
 *   primaryFunction: String
 * }
 *
 * type Query {
 *   hero(episode: Episode): Character
 *   human(id: String!): Human
 *   droid(id: String!): Droid
 * }
 *
 */

fn setup_schema() -> Schema {
    let mut episode_enum_values = HashMap::new();
    episode_enum_values.insert(String::from("NEWHOPE"), EnumValue {
        value: 4,
        description: String::from("Released in 1977.")
    });

    let episode_enum = Enum {
        name: String::from("Episode"),
        description: String::from("One of the films of the Star Wars trilogy."),
        values: episode_enum_values
    };

    Schema {
        query: Object {
            name: String::from("Query")
        }
    }
}

#[test]
fn basic_query() {
    let schema = setup_schema();
    let query = "
query HeroNameQuery {
    hero {
        name
    }
}
".to_string();
    let expected = String::from("");
    let result = GraphQL::query(schema, query);
    assert_eq!(result, expected);
}
