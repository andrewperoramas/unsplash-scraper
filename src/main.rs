use clap::Parser;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Debug)]
struct Counter {
    counter: u32,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(long = "url", short = 'u', default_value = "https://api.unsplash.com/photos/")]
    url: String,

    #[clap(long = "page", short = 'p', default_value_t = 1)]
    page: u32,

    #[clap(long = "scrape_count", default_value_t = 100)]  // Number of scrapes to perform
    scrape_count: u32,

    #[clap(long = "per_page", short = 'P', default_value_t = 30)]
    per_page: u32,

    #[clap(long = "interval", short = 'i', default_value_t = 240000)]
    interval: u64,

    #[clap(long = "access_key", short = 'k')]
    access_key: String,

    #[clap(long = "hosts", short = 'H', default_value = "http://localhost:8000")]
    hosts: String,
}

#[derive(Serialize, Debug)]
struct Payload {
    payload: String,
}

fn create_headers(access_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("My Rusty Baby boy"));
    headers.insert("Authorization", HeaderValue::from_str(&format!("Client-ID {}", access_key)).unwrap());
    headers
}

async fn fetch_scrape_url(args: &Cli) -> Result<String, Error> {
    let current_page = fetch_current_page(&args.hosts).await?;

    let full_url = format!("{}?page={}&per_page={}", args.url, current_page, args.per_page);
    println!("Fetching: {}", full_url);

    let client = Client::new();
    let headers = create_headers(&args.access_key);

    let response = client
        .get(&full_url)
        .headers(headers)
        .send()
        .await?;

    if !response.status().is_success() {
        eprintln!("Failed to fetch API: Status code {}", response.status());
        return Err(response.error_for_status().unwrap_err());
    }

    let response_text = response.text().await?;
    save_scraped_url(&args.hosts, response_text.clone(), &args.access_key).await?;
    increment_counter(&args.hosts).await?;

    Ok(response_text)
}

async fn fetch_current_page(host: &str) -> Result<u32, Error> {
    let get_current_page_url = format!("{}/api/unsplash_page", host);
    let client = Client::new();
    let response = client.get(&get_current_page_url).send().await?.json::<Counter>().await?;
    Ok(response.counter)
}

async fn increment_counter(host: &str) -> Result<(), Error> {
    let post_url = format!("{}/api/unsplash_page/increment", host);
    let client = Client::new();

    client
        .post(&post_url)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    Ok(())
}

async fn save_scraped_url(host: &str, payload: String, access_key: &str) -> Result<(), Error> {
    let client = Client::new();
    let headers = create_headers(access_key);

    let pay = Payload { payload };

    let scrape_url = format!("{}/api/unsplash_page/scrape", host);

    let response = client
        .post(&scrape_url)
        .headers(headers)
        .json(&pay)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    for _ in 0..args.scrape_count {
        match fetch_scrape_url(&args).await {
            Ok(_) => println!("Scraping successful!"),
            Err(e) => eprintln!("Error during scraping: {}", e),
        }

        println!("next scrape in {}", args.interval);
        sleep(Duration::from_millis(args.interval)).await;
    }

    println!("Done scraping.");
}
