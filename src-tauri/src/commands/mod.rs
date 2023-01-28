use std::{fs, path, process, thread, time, io::Write, ops::Index};

use crate::commands::logging::log;
use execute::Execute;
use rfd::FileDialog;

pub mod database;
pub mod logging;

#[tauri::command]
// This function is ran everytime a search query is made
pub fn handle_scraper(path: String, query: String) {
    let start_time = time::Instant::now();

    // Create a command object for the scraper chosen (The command is just its path)
    // Pass in its path, a query and the destination folder for the cache file as arguments
    let mut command = process::Command::new(path.clone());
    command.arg(query.clone());
    command.arg(crate::paths::get_pbp().join("queries"));

    log(2, &format!("Searching for \"{}\" with {}", query, path));

    // Run the scraper and tell us about its exit code
    if let Some(exit_code) = command.execute().unwrap() {
        if exit_code == 0 {
            log(2, "Scraper query completed successfully");
            let final_time = time::Instant::now() - start_time;
            log(2, &format!("Took {} second(s)", final_time.as_secs()))
        } else {
            log(0, "Scraper query failed successfully");
        }
    } else {
        log(2, "Scraper query interrupted");
    }
}

#[derive(serde::Deserialize)]
struct Preset {
    base_url: String,
    query_url: String,
    game_page_link: Route,
    download_link: Route,
}

#[derive(serde::Deserialize)]
struct Route {
    route: String,
    index: u8
}

#[derive(serde::Serialize)]
pub struct Response {
    response: Vec<Game>
}

#[derive(serde::Serialize, Debug)]
pub struct Game {
    title: String,
    urls: Vec<String>
}

#[tauri::command]
pub fn handle_bpe_scraper(path: String, query: String, index: u8) {
    println!("Starting BPE scraper search");
    let contents = fs::read(path).expect("Reading file failed");
    let mut deserialized: Preset =
        serde_json::from_slice(&contents).expect("JSON deserialization failed");
    deserialized.query_url = deserialized.query_url.replace("%BPE%", &query);

    let body = reqwest::blocking::get(&deserialized.query_url)
        .expect("GET request failed")
        .text()
        .expect("Failed to get response text");
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse(&deserialized.game_page_link.route)
        .expect("Failed to parse selectors");
    let selection: Vec<scraper::ElementRef> = document.select(&selector).collect();

    let mut games = vec![];
    let mut i = 0;

    for title in selection {
        if i < 5 {
            let link = title.value().attr("href").expect("Failed to get attribute");
            let title = title.inner_html();
            let mut urls = vec![];

            if let Some(download) = get_download(deserialized.base_url.to_string(), link.to_string(), &deserialized) {
                urls.push(download);
            }

            games.push(Game { title, urls });
            i += 1;
        }
    }
    println!("{:#?}", games);

    fn get_download(base_url: String, mut url: String, deserialized: &Preset) -> Option<String> {

        if !url.starts_with("http") || !url.starts_with("magnet") {
            url = format!("{}{}", base_url, url)
        }

        let body = reqwest::blocking::get(url)
            .expect("GET request failed")
            .text()
            .expect("Failed to get response text");
        let document = scraper::Html::parse_document(&body);
        let selector = scraper::Selector::parse(&deserialized.download_link.route)
            .expect("Failed to parse selectors");
        for download in document.select(&selector) {
            let link = download
                .value()
                .attr("href")
                .expect("Failed to get attribute");
            if link.starts_with("magnet") || link.starts_with("http") {
                return Some(link.to_string());
            }
        }

        None
    }

    println!("Ending BPE scraper search");
    let response = Response { response: games };
    let json = serde_json::to_vec_pretty(&response).expect("JSON serialization failed");
    let mut file = fs::File::create(crate::paths::get_pbp().join("queries").join("results.json")).expect("Failed to create file");
    file.write_all(&json).expect("Writing JSON to file failed");
}

#[tauri::command]
// This function is ran everytime the user clicks the "Install scraper" button on the Preferences page
pub fn install_scraper() {
    let file = match FileDialog::new()
        .add_filter("Executables", &["exe", "com", "cmd", "bat"])
        .set_directory("/")
        .pick_file()
    {
        Some(file) => file.display().to_string(),
        None => "None".to_string(),
    };

    // Copy the scraper from the location the user selected to the scrapers folder
    if file != "None" {
        let file = path::Path::new(&file);
        fs::copy(
            file,
            crate::paths::get_pbp()
                .join("scrapers")
                .join(file.file_name().unwrap()),
        )
        .expect("Installing scraper failed");
    }

    log(
        2,
        &format!(
            "Installed scraper with path {}",
            path::Path::new(&file).display()
        ),
    );
}

#[tauri::command]
// Opens a file dialog that prompts the user for an executable
pub fn file_dialog() -> String {
    log(2, "Executable file dialog opened");

    // Prompt the user to select a file from their computer as an input
    // For error handling, you can use if- and match statements
    match FileDialog::new()
        .add_filter("Executables", &["exe", "com", "cmd", "bat", "sh"])
        .set_directory("/")
        .pick_file()
    {
        // If the user picked a file, return the path to the frontend
        Some(file) => file.display().to_string(),
        // If the user just closed the window, without picking a file, return "None" to the frontend
        None => "None".to_string(),
    }
}

#[tauri::command]
// Opens a file dialog that prompts the user for an image
pub fn image_dialog() -> String {
    log(2, "Image file dialog opened");

    // Prompt the user to select a file from their computer as an input
    // For error handling, you can use if- and match statements
    match rfd::FileDialog::new()
        .add_filter(
            "Images",
            &["png", "jpg", "jpeg", "gif", "bmp", "ico", "webp"],
        )
        .set_directory("/")
        .pick_file()
    {
        // If the user picked a file, return the path to the frontend
        Some(file) => file.display().to_string(),
        // If the user just closed the window, without picking a file, return "None" to the frontend
        None => "None".to_string(),
    }
}

#[tauri::command]
// This function is ran everytime the user clicks "Run" on a library entry
pub fn run_game(path: String) {
    let start_time = time::Instant::now();
    let mut command = process::Command::new(path);

    thread::spawn(move || {
        if let Some(exit_code) = command.execute().unwrap() {
            if exit_code == 0 {
                log(2, "Game ran successfully");
                let final_time = time::Instant::now() - start_time;
                log(
                    2,
                    &format!("Game ran for {} second(s)", final_time.as_secs()),
                )
            } else {
                log(0, "Scraper query failed successfully");
            }
        };
    });
}
