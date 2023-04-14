use crate::models::{EpisodesDefinition, EpisodeDefinition, SeriesMetaData};
use scraper::{Html, Selector};

pub fn get_series_metadata(html: &str) -> EpisodesDefinition {
  let document = Html::parse_document(&html);
  let titleSelector = Selector::parse("title").unwrap();
  let rowSelector = Selector::parse("tbody tr").unwrap();
  let columnSelector = Selector::parse("td").unwrap();
  let anchorSelector = Selector::parse("a").unwrap();

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

  let episodes =
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

    let metadata =
      SeriesMetaData {
        name: title,
        tvdb_id: "xyz".to_string(), //TODO: Ask for TVDB Id from user
        season_number: "whatever".to_string(), // TODO: Try and glean this information from the html
      };

    EpisodesDefinition {
        metadata: metadata,
        episodes,
    }
}
