use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SeriesMetaData {
  pub name: String,
  pub tvdb_id: String,
  pub season_number: String,
}


#[derive(Debug, Deserialize, PartialEq)]
pub struct EpisodesDefinition {
  pub metadata: SeriesMetaData,
  pub episodes: Vec<EpisodeDefinition>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EpisodeDefinition {
  pub number: String,
  pub name: String,
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_episode_deserialization() {
      let conf = r#"{
        "metadata": {
          "name":"Thundercats",
          "tvdb_id":"70355",
          "season_number":"1"
        },
        "episodes": [
          { "number":"S01E01", "name":"Exodus"},
          { "number":"S01E02", "name":"The Unholy Alliance"},
          { "number":"S01E03", "name":"Berbils"},
          { "number":"S01E04", "name":"The Slaves of Castle Plun-Darr"},
          { "number":"S01E05", "name":"Pumm-Ra"},
          { "number":"S01E06", "name":"The Terror of Hammerhand"}
        ]
      }"#;

      let expected_episodes =
        vec![
          EpisodeDefinition {
            number:"S01E01".to_string(),
            name:"Exodus".to_string()
          },
          EpisodeDefinition {
            number:"S01E02".to_string(),
            name:"The Unholy Alliance".to_string()
          },
          EpisodeDefinition {
            number:"S01E03".to_string(),
            name:"Berbils".to_string()
          },
          EpisodeDefinition {
            number:"S01E04".to_string(),
            name:"The Slaves of Castle Plun-Darr".to_string()
          },
          EpisodeDefinition {
            number:"S01E05".to_string(),
            name:"Pumm-Ra".to_string()
          },
          EpisodeDefinition {
            number:"S01E06".to_string(),
            name:"The Terror of Hammerhand".to_string()
          }
        ];

      let expected_episodes_definition =
        EpisodesDefinition {
          metadata: SeriesMetaData {
            name: "Thundercats".to_string(),
            tvdb_id: "70355".to_string(),
            season_number: "1".to_string()
          },
          episodes: expected_episodes
        };

      let episodes_definition: EpisodesDefinition = serde_json::from_str(conf).unwrap();
      assert_eq!(episodes_definition, expected_episodes_definition)
    }
}
