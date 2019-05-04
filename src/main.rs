use pipers::Pipe;
use regex::Regex;
use rss::{Channel, Item};
use serde::Deserialize;
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

#[derive(Deserialize, Debug)]
struct TorrentEntry {
    name: String,
    directory: String,
}

fn get_rss_feed(url: &str) -> Channel {
    Channel::from_url(url).unwrap()
}

fn match_item_title(item: &Item, title: &str) -> bool {
    match item.title() {
        Some(t) => t.contains(title),
        None => false,
    }
}

fn match_items(items: Vec<Item>, title: &str) -> Result<Item, &'static str> {
    let matched: Vec<Item> = items
        .into_iter()
        .filter(|item| match_item_title(item, title))
        .collect();

    match matched.len() {
        0 => Err("No matched items"),
        1 => Ok(matched[0].clone()),
        _ => Err("Too many matches"),
    }
}

fn start_torrent(torrent_name: &str) {
    let grep_command = format!("grep {}", torrent_name);
    let out = Pipe::new("transmission-remote -l")
        .then(&grep_command)
        .then("awk '{ print $1 }'")
        .finally()
        .expect("Commands did not pipe")
        .wait_with_output()
        .expect("failed to wait on child");

    Command::new("transmission-remote")
        .args(&["-t", &String::from_utf8(out.stdout).unwrap(), "--start"])
        .status()
        .expect("Failed to start torrent");
}

fn link_directory(torrent_name: &str, directory: &str) {
    Command::new("ln")
        .args(&[
            "-l",
            &format!("{}/{}", directory, torrent_name),
            "$HOME/Downloads",
        ])
        .status()
        .expect("Failed to link directory");
}

fn extract_torrent_name(item: &Item) -> String {
    let link = item.link().unwrap().to_string();
    let re = Regex::new(r".*/(.*).torrent$").unwrap();
    let cap = re
        .captures(&link)
        .expect("Unable to capture output")
        .get(1)
        .expect("Unable to extract torrent name");
    String::from(cap.as_str())
}

fn start_torrents(torrent_entries: Vec<TorrentEntry>, items: Vec<Item>) {
    for entry in torrent_entries.into_iter() {
        match match_items(items.to_vec(), &entry.name) {
            Ok(item) => {
                let name = extract_torrent_name(&item);
                println!(
                    "Matched against \"{}\" and got name \"{}\"",
                    entry.name, name
                );
                link_directory(&name, &entry.directory);
                start_torrent(&name);
            }
            Err(e) => println!("Error \"{}\": {}", entry.name, e),
        }
    }
}

fn get_torrent_and_directory(file: &str) -> Result<Vec<TorrentEntry>, std::io::Error> {
    let mut contents = String::new();
    File::open(file)?.read_to_string(&mut contents)?;

    let entries: Vec<TorrentEntry> = serde_json::from_str(&contents)?;

    Ok(entries)
}

fn main() {
    let url = args().nth(1).unwrap();
    let torrent_entries = get_torrent_and_directory("torrents.json");
    match torrent_entries {
        Ok(torrent_entries) => {
            let stuff = get_rss_feed(&url);

            start_torrents(torrent_entries, stuff.items().to_vec());
        }
        Err(e) => println!("Error getting torrents and directory {:?}", e),
    }
}
