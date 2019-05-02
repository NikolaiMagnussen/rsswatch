use regex::Regex;
use pipers::Pipe;
use std::env::args;
use rss::{Channel, Item};
use std::process::Command;

fn get_rss_feed(url: &str) -> Channel {
    Channel::from_url(url).unwrap()
}

fn match_items(items: Vec<Item>, title: &str) -> Vec<Item> {
    items.into_iter().filter(|item| item.title().unwrap().contains(title)).collect()
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
        .args(&["-l", &format!("{}/{}", directory, torrent_name), "$HOME/Downloads"])
        .status()
        .expect("Failed to link directory");
}

fn extract_torrent_name(item: &Item) -> String {
    let link = item.link().unwrap().to_string();
    let re = Regex::new(r".*/(.*).torrent$").unwrap();
    let cap = re.captures(&link)
        .expect("Unable to capture output")
        .get(1)
        .expect("Unable to extract torrent name");
    String::from(cap.as_str())
}

fn start_torrents(items: Vec<Item>, directory: &str) {
    for item in items.into_iter() {
        let name = extract_torrent_name(&item);
        println!("\nGot name \"{}\"", name);
        //link_directory(&name, directory);
        //start_torrent(&name);
    }
}

fn main() {
    let url = args().nth(1).unwrap();
    let search = "Bryggen";

    let stuff = get_rss_feed(&url);
    let a = match_items(stuff.items().to_vec(), search);

    start_torrents(a, "/");
}
