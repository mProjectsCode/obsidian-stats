# Obsidian Stats

A stats website for the Obsidian Eco-System.

https://www.moritzjung.dev/obsidian-stats/

## Project Structure

The project is split into 4 parts.

1. The website located in `website`
2. A shared Rust library in `data-lib`
3. The data collection code in `data`
4. The data analysis code used for the website in `data-wasm`

All collected data is persisted in the `data/out` folder and tracked in Git, except for cloned plugin repos.

The project uses Makefiles that sometimes think targets are up to date. In that case you can add the `-B` flag to the make command.

### How to Build

Requirements:

1. [Bun](https://bun.com/)
2. [Latest Rust Compiler](https://rustup.rs/)
3. [wasm-pack](https://github.com/drager/wasm-pack)
4. Probably be on Linux, idk

To test and build the website locally you first need to build the `data-wasm` folder. 
For this you can run `make wasm` in the repo root.

After this you can navigate to the `website` folder and install dependencies with `bun i`.
Then you can run either the dev server with `bun run dev` or build the website via `bun run build`.

### How to Run Data Collection

Requirements:

1. [Latest Rust Compiler](https://rustup.rs/)
2. Probably Linux, idk
3. 10+ minutes of time

Depending on your goal, the first thing you might want to run is `make submodule-update`.

After that you can run the data collection via `make data`.

The data collection consists of multiple phases:

1. Theme Data
    1. Read theme data from the `obsidian-releases` repo
    2. Build and save theme data to `data/out/theme-data`
2. Plugin Data
    1. Read plugin data from the `obsidian-releases` repo, including download data
    2. Build plugin, download, and version data
    3. Save that data to `data/out/plugin-data`
3. Clone Plugin Repos to `data/out/plugin-repos`
4. Extract extra data from plugin repos to `data/out/plugin-repo-data`
5. Process licenses from `choosealicense.com` to `data/out/licenses.json`
6. Obsidian Releases Data
    1. Get release data from GitHub releases, process, and save to `data/out/releases-github-raw`. This data is incremental.
    2. Interpolate download data and save to `data/out/releases-github-interpolated`
    3. Get the Obsidian changelog, process and save to `data/out/releases-changelog`

Currently you select which steps to run by commenting out the steps that you don't want in `data/src/main.rs`.

## Credits

I want to thank the following people:

- Fevol for his work on parsing Obsidian release data
- joethei for answering all my questions about plugin and theme releases
- The Astro team for their work on Astro and Starlight
