use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct MovieDefinition {
  name: String,
  tvdb_id: String
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_movie_deserialization() {
      let conf = r#"
        {
          "name":"Star Wars: The Rise of Skywalker",
          "tvdb_id":"12879"
        }"#;

      let expected_movie_definition =
        MovieDefinition {
          name: "Star Wars: The Rise of Skywalker".to_owned(),
          tvdb_id: "12879".to_owned()
        };

      let movie_definition: MovieDefinition = serde_json::from_str(conf).unwrap();
      assert_eq!(movie_definition, expected_movie_definition)
    }
}
