# MKV-Renamer

Rename [makeMKV](https://www.makemkv.com/) rips in a simpler way.

Please note this is meant for when you **legally** backup media you **own** for playback through a media server such as [Plex](https://www.plex.tv/) or [Jellyfin](https://jellyfin.org/) or similar.

## Why do we need this?

When you rip TV series from optical media, the file names are not that of the episode names of the series. They are usually a sequential numbering e such as: `DVD_TS_01.mkv`, `DVD_TS_02.mkv` etc. This numbering scheme is repeated for all discs of the media in the series. For example:


1. disc1 -> `DVD_TS_01.mkv`, `DVD_TS_02.mkv`
1. disc2 -> `DVD_TS_01.mkv`, `DVD_TS_02.mkv`
1. disc3 -> `DVD_TS_01.mkv`, `DVD_TS_02.mkv`
1. disc4 -> `DVD_TS_01.mkv`, `DVD_TS_02.mkv`

Now if you want something like Plex to index these episodes correctly and download the appropriate art and metadata you need to follow certain [conventions](https://support.plex.tv/articles/naming-and-organizing-your-tv-show-files/). In addition if you tag the series with [The TVDB](https://thetvdb.com/) or [IMDB](https://www.imdb.com/) series ids, the index process is more accurate. I've chosen to use the TVDB ids in this case.

The recommended format is:

```
<SERIES_NAME> {tvdb-<TVDB_ID>}/SEASON <SEASON_NUMBER>/S<SEASON_NUMBER>E<EPISODE_NUMBER> - <EPISODE_NUMBER NAME>
```

For example:

```
Band of Brothers {tvdb-74205}/Season 01/S01E01 - Currahee
```


Renaming these sequential rips is tedious and error-prone - specially if you have a lot of discs. mkv-renamer hopes to make that process easier.

## Usage

```
Rename TV series ripped from optical media

Usage: mkv-renamer --processing-dir <PROCESSING_DIR> <--url-metadata <url>|--file-metadata <file>>

Options:
  -u, --url-metadata <url>
          The url of TVDB season information. Example: https://thetvdb.com/series/thundercats/seasons/official/1
  -f, --file-metadata <file>
          The location of series metadata file. An example format can be found at: https://raw.githubusercontent.com/ssanj/mkv-renamer/main/sample.conf
  -p, --processing-dir <PROCESSING_DIR>
          The location of the processing directory (PD). Structure: PD/{Rips, Renames, Encodes}
  -h, --help
          Print help
  -V, --version
          Print version
```

You need to supplied a processing directory (see below), and some metadata about the series, either via a URL or a JSON file.

## Folder structure of processing directory

Your processing directory (PD) should have the following structure:

```
<PD>
  |- Rips
      |- disc1
      |- disc2
      |- disc3
      ..
      |- discn
  |- Renames
  |- Encodes
```

You can create this folder structure by running the following in your processing directory:

```
mkdir -p Rips/{disc1,disc2,disc3,disc4} Renames Encodes
```

Change the number of disc folders to suit your needs.

### Rips

The folder that contains all the disc subfolders. All rips will go into one of the disc**N** directories corresponding to the disc being ripped.

### Renames

Once you run mkv-renamer, the renamed files will be renamed/moved here from the `Rips` directory

### Encodes

The encodes directory is where the renamed files are encoded to. mkv-renamer will create a target folder of the format: `<SERIES_NAME> {tvdb-<TVDB_ID>}/SEASON <SEASON_NUMBER>`. When encoding the renamed files, choose this as the target folder.

## Metadata

The metadata for the series can be supplied either as a JSON file path or a URL to the TVDB series.

### Config file

You must specify the following `metadata` fields:

| Field | Value |
| ----- | ----- |
| name | The name of the series |
| tvdb_id | The TVDB ID of the series |
| season_number | The season number |
| episodes | The list of episodes |

For each `episode` the following fields are needed:

| Field | Value |
| ----- | ----- |
| number | The number of the episode in S00E00 format |
| name | The name of the episode |


An example config file:

```json
{
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
}
```

You can find [The TVDB]() id as follows:

1. Search for your TV series at [The TVDB](https://thetvdb.com/) [example](https://thetvdb.com/search?query=Strange%20new%20worlds)
1. Copy it from the search results

![Where to find The TVDB id](tvdb-id.png)

Example usage:

```
mkv-renamer -p /some/processing/directory -f /path/to/series.json
```

## URL to TVDB season

To find the correct URL to supply to mkv-renamer do the following:

1. Search for your TV series at [The TVDB](https://thetvdb.com/) [example](https://thetvdb.com/search?query=Strange%20new%20worlds)
1. Click through to the homepage for your series [example](https://thetvdb.com/series/star-trek-strange-new-worlds)
1. Click on `Seasons` to go to the Seasons homepage for your series [example](https://thetvdb.com/series/star-trek-strange-new-worlds#seasons)
1. Click on your specific season (use this link for mkv-renamer) [example](https://thetvdb.com/series/star-trek-strange-new-worlds/seasons/official/1)


Example usage:
```
mkv-renamer -p /some/processing/directory -u https://thetvdb.com/series/star-trek-strange-new-worlds/seasons/official/1
```

## Workflow

1. Rip each disc of your TV series into the corresponding `PD/Rips/disc<NUMBER>` folder.

   For example:
     - `disc1` rips will go into `PD/Rips/disc1`
     - `disc2` rips will go into `PD/Rips/disc2`
1. Use `mkv-renamer` to match the disc names to actual episode names.

   This will:
     1. Write the correctly named episode MKV files into your `PD/Renames` folder.
     1. Create a folder in the `PD/Encodes` folder with the following format: `<SERIES_NAME> {tvdb-<TVDB_ID>}/SEASON <SEASON_NUMBER>`
1. Use a tool like [Handbrake](https://handbrake.fr/) to encode your MKV to something smaller like mp4 and choose the above folder as the target: `PD/Encodes/<SERIES_NAME> {tvdb-<TVDB_ID>}/SEASON <SEASON_NUMBER>`.
1. Copy the folder and its encoded contents to your media server for indexing.


## Build and Install

- Checkout this project on GitHub
- [Install Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Create the executable with:

```
cargo build --release
```

You can then copy the `target/release/mkv-renamer` executable to your PATH.

I realise this is tedious for peeps that don't use [Rust](https://www.rust-lang.org/). If there's interest I'll build the executables through CI.
