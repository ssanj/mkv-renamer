use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MovieDefinition {
  name: String,
  tvdb_id: String
}

impl MovieDefinition {
  pub fn new(name: String, tvdb_id: String) -> Self {
    Self {
      name,
      tvdb_id
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn tvdb_id(&self) -> &str {
    &self.tvdb_id
  }
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
