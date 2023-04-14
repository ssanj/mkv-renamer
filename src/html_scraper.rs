use crate::models::{EpisodesDefinition, EpisodeDefinition, SeriesMetaData};
use scraper::{Html, Selector};

pub fn get_series_metadata(html: &str) -> EpisodesDefinition {
  let document = Html::parse_document(&html);
  let titleSelector = Selector::parse("title").unwrap();
  let rowSelector = Selector::parse("tbody tr").unwrap();
  let columnSelector = Selector::parse("td").unwrap();
  let anchorSelector = Selector::parse("a").unwrap();
  let tvidSelector = Selector::parse(r#"div[class="btn-group"]"#).unwrap();

  let title =
    document
      .select(&titleSelector)
      .collect::<Vec<_>>()
      .first()
      .and_then(|e|
        e
          .inner_html()
          .as_str()
          .split('-')
          .collect::<Vec<_>>()
          .first()
          .map(|s| s.trim().to_string())
      )
      .unwrap();

  let tvid =
    document
      .select(&tvidSelector)
      .collect::<Vec<_>>()
      .first()
      .and_then(|e|{
        e
          .value()
          .attr("data-permission")
          .clone()
      })
      .unwrap();

    // expected format: series-TVDBID-artwork
  let splits: Vec<_> =
    tvid
      .split('-')
      .collect();

  let tvdb_id =
    splits
      .get(1) // get the second element
      .clone()
      .unwrap()
      .to_string();

  let episodes: Vec<EpisodeDefinition> =
    document
      .select(&rowSelector)
      .map(|row_fragment|{
        let required_columns =
          row_fragment
              .select(&columnSelector)
              .enumerate()
              .filter_map(|(index, column)|{
                if index == (0 as usize) { // number
                  Option::Some(column.inner_html())
                } else if index == (1 as usize) { // name, is within an <a href="">NAME</a>
                  column
                    .select(&anchorSelector)
                    .map(|anchor|{
                       anchor.inner_html()
                    })
                    .collect::<Vec<_>>()
                    .first()
                    .cloned()
                } else {
                  None
                }
              })
              .map(|column|{
                column.as_str().trim().to_owned()
              })
              .collect::<Vec<_>>();

        let number = required_columns.get(0).expect("Could not get number from html").to_owned();
        let name = required_columns.get(1).expect("Could not get name from html").to_owned();

        EpisodeDefinition {
          number,
          name
        }
      }).
      collect();

    let season_number = get_season_number(&episodes);

    let metadata =
      SeriesMetaData {
        name: title,
        tvdb_id,
        season_number
      };

    EpisodesDefinition {
        metadata: metadata,
        episodes,
    }
}

fn get_season_number(episodes: &[EpisodeDefinition]) -> String {
  let season_number: String =
      episodes
        .first()
        .map(|e| {
          e
            .number
            .chars()
            .into_iter()
            .take_while(|c| c.ne(&'E')) // Given: S01E02, take everything up to the E: S01
            .skip_while(|c| c.is_alphabetic()) // Drop the S: 01
            .collect()
        })
        .unwrap();

      format!("{}", season_number.parse::<u8>().unwrap())
}
