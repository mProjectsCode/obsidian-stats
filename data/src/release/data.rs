use chrono::DateTime;
use crate::{
    constants::{RELEASE_STATS_URL, RELEASE_INFO_URL},
    release::{GithubReleaseEntry, GithubAssetInfo, ObsidianReleaseInfo, ObsidianPlatform},
};
use data_lib::version::Version;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

fn get_release_lists() -> Vec<GithubAssetInfo> {
    let mut current_link = Some(RELEASE_STATS_URL.to_string());
    let mut release_lists: Vec<GithubReleaseEntry> = vec![];
    while let Some(api_link) = current_link {
        let response = reqwest::blocking::get(api_link)
            .expect("Failed to fetch release stats");
        if response.status().is_success() {
            let link_header = response.headers().get(reqwest::header::LINK);
            // TODO: 🪲 🦗 cannot move out of `response` because it is borrowed [E0505]  🐛 🐞
            let json: Vec<GithubReleaseEntry> = response
                .json()
                .expect("Failed to parse release stats JSON");
            release_lists.extend(json);
            if let Some(link_header) = link_header &&
                let Ok(link_str) = link_header.to_str() &&
                let Some(next_link) = link_str.split(',').find_map(|s| {
                    if s.contains("rel=\"next\"") {
                        s.split(';').next().map(|s| s.trim_matches('<').trim_matches('>'))
                    } else {
                        None
                    }
                }) {
                current_link = Some(next_link.to_string());
            } else {
                current_link = None;
            }
        } else {
            eprintln!("Failed to fetch release stats: {}", response.status());
            break;
        }
    }

    let mut release_info = vec![];
    for release in release_lists {
        for asset in &release.assets {
            release_info.push(GithubAssetInfo {
                version: Version::parse(&release.tag_name).expect("Failed to parse version"),
                date: release.published_at,
                asset: asset.name.clone(),
                downloads: asset.download_count,
                size: asset.size,
            });
        }
    }

    release_info
}

// TODO: Maybe this works, maybe not.
// fn get_release_info() -> Vec<ObsidianReleaseInfo> {
//     let response = reqwest::blocking::get(RELEASE_INFO_URL)
//         .expect("Failed to fetch release info");
//     if response.status().is_success() {
//         let mut reader = Reader::from_reader(response.bytes().expect("Failed to read response").as_ref());
//         let mut buf = Vec::new();
//         let mut release_info: Vec<ObsidianReleaseInfo> = vec![];
//         loop {
//             match reader.read_event_into(&mut buf) {
//                 Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
//
//                 Ok(Event::Eof) => break vec![],
//
//                 Ok(Event::Start(e)) => {
//                     match e.name().as_ref() {
//                         b"entry" => {
//                             let mut id = None;
//                             let mut version = None;
//                             let mut date = None;
//                             let mut info = String::new();
//                             let mut insider = false;
//                             let mut major_release = false;
//                             let mut platform = None;
//
//                             loop {
//                                 match reader.read_event_into(&mut buf) {
//                                     Ok(Event::End(_)) => break,
//                                     Ok(Event::Start(inner_e)) => {
//                                         match inner_e.name().as_ref() {
//                                             b"id" => {
//                                                 if let Ok(Event::Text(text)) = reader.read_event_into(&mut buf) {
//                                                     id = Some(text.decode().unwrap().into_owned()[30..].to_string());
//                                                     platform = if info.contains("desktop") {
//                                                         Some(ObsidianPlatform::Desktop)
//                                                     } else if info.contains("mobile") {
//                                                         Some(ObsidianPlatform::Mobile)
//                                                     } else if info.contains("publish") {
//                                                         Some(ObsidianPlatform::Publish)
//                                                     } else {
//                                                         None
//                                                     };
//                                                     if let Some(pos) = id.as_ref().and_then(|s| s.rfind("v")) {
//                                                         version = Some(Version::parse(&id.as_ref().unwrap()[pos + 1..]));
//                                                     }
//                                                 }
//                                             }
//                                             b"title" => {
//                                                 if let Ok(Event::Text(text)) = reader.read_event_into(&mut buf) {
//                                                     insider = text.decode().unwrap().contains("Early access");
//                                                     // major_release IFF title only contains X.Y (and not X.Y.Z)
//                                                 }
//                                             }
//                                             b"updated" => {
//                                                 if let Ok(Event::Text(text)) = reader.read_event_into(&mut buf) {
//                                                     date = DateTime::parse_from_rfc3339(&text.decode().unwrap())
//                                                         .ok().map(|dt| dt.with_timezone(&chrono::Utc));
//                                                 }
//                                             }
//                                             b"content" => {
//                                                 if let Ok(Event::Text(text)) = reader.read_event_into(&mut buf) {
//                                                     info = text.decode().unwrap().into_owned();
//                                                 }
//                                             }
//                                             _ => ()
//                                         }
//                                     }
//                                     _ => ()
//                                 }
//                             }
//
//                             if let (Some(version), Some(date)) = (version, date) {
//                                 release_info.push(ObsidianReleaseInfo {
//                                     version: version.expect("Failed to parse version"),
//                                     platform: ObsidianPlatform::Desktop, // Assuming desktop for now
//                                     insider,
//                                     date,
//                                     info,
//                                     major_release,
//                                 });
//                             }
//                         }
//                         _ => ()
//                     }
//                 }
//
//                 _ => ()
//             }
//         }
//
//         release_info
//     } else {
//         eprintln!("Failed to fetch release info: {}", response.status());
//         return vec![];
//     }
// }


pub fn build_release_stats() -> Result<(), Box<dyn std::error::Error>> {
    let time = std::time::Instant::now();
    let mut time2 = std::time::Instant::now();

    let release_lists = get_release_lists();

    // println!("Get release lists: {:#?}", time2.elapsed());
    // time2 = std::time::Instant::now();
    //
    // let mut release_data = build_release_data(&release_lists);
    //
    // println!("Build release Data {:#?}", time2.elapsed());
    // time2 = std::time::Instant::now();
    //
    // empty_dir(Path::new(RELEASE_DATA_PATH))?;
    //
    // write_in_chunks(Path::new(RELEASE_DATA_PATH), &release_data, 50)?;

    println!("Filtered and write release data: {:#?}", time2.elapsed());

    println!("Release stats built in {:#?}", time.elapsed());


    Ok(())
}
