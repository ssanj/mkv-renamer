use crate::models::{EpisodesDefinition, EpisodeDefinition, SeriesMetaData};
use scraper::{Html, Selector};

/// This whole class is pretty loose with the error handling
/// If the HTML structure has changed, then we simply fail
/// because this class will have to be rewritten to handle the new format
pub fn get_series_metadata(html: &str) -> EpisodesDefinition {
  let document = Html::parse_document(html);
  let title_selector = Selector::parse("title").unwrap();
  let row_selector = Selector::parse("tbody tr").unwrap();
  let column_selector = Selector::parse("td").unwrap();
  let anchor_selector = Selector::parse("a").unwrap();
  let tvid_selector = Selector::parse(r#"div[class="btn-group"]"#).unwrap();

  let title =
    document
      .select(&title_selector)
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
      .select(&tvid_selector)
      .collect::<Vec<_>>()
      .first()
      .and_then(|e|{
        e
          .value()
          .attr("data-permission")
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
      .unwrap()
      .to_string();

  let episodes: Vec<EpisodeDefinition> =
    document
      .select(&row_selector)
      .map(|row_fragment|{
        let required_columns =
          row_fragment
              .select(&column_selector)
              .enumerate()
              .filter_map(|(index, column)|{
                if index == 0 { // number
                  Option::Some(column.inner_html())
                } else if index == 1 { // name, is within an <a href="">NAME</a>
                  column
                    .select(&anchor_selector)
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

        let number = required_columns.first().expect("Could not get number from html").to_owned();
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
        metadata,
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
            .take_while(|c| c.ne(&'E')) // Given: S01E02, take everything up to the E: S01
            .skip_while(|c| c.is_alphabetic()) // Drop the S: 01
            .collect()
        })
        .unwrap();

      format!("{}", season_number.parse::<u8>().unwrap()) // 01 -> 1
}
