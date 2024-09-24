mod parser;
mod utils;
use parser::{a_html_response, all_pages, get_categories, get_page_ads, Ad, HtmlResponse};
use rayon::prelude::*;
use reqwest::Client;
// use std::sync::{Arc, Mutex};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use utils::{add_to_csv, parse_category};

pub const MAIN_URL: &str = "https://sportingshot.ru/";

pub const SALES: &str = "https://sportingshot.ru/sales/";

async fn get_html_from_urls(
    urls: Vec<String>,
) -> Result<Vec<HtmlResponse>, Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    let semaphore = Arc::new(Semaphore::new(10)); // Ограничение до 10 одновременных запросов
    let mut futures = Vec::new();

    for url in urls.into_iter() {
        let client = client.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        futures.push(tokio::spawn(async move {
            let _permit = permit; // Сохраняем разрешение в замыкании
            a_html_response(&client, url).await.ok()
        }));
    }

    let htmls = futures::future::join_all(futures)
        .await
        .into_par_iter()
        .filter_map(|html_string| html_string.expect("Ошибка какая-то"))
        .collect::<Vec<HtmlResponse>>();
    Ok(htmls)
}

async fn get_ads_category_urls(
    category_url: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    let html = a_html_response(&client, category_url.clone()).await?;

    let pages = all_pages(html.html, &category_url)?;

    let semaphore = Arc::new(Semaphore::new(10)); // Ограничение до 10 одновременных запросов
    let mut futures = Vec::new();

    for page in pages {
        let client = client.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        futures.push(tokio::spawn(async move {
            let _permit = permit; // Сохраняем разрешение в замыкании
            a_html_response(&client, page).await.ok()
        }));
    }
    let pages_html = futures::future::join_all(futures)
        .await
        .into_par_iter()
        .filter_map(|html_string| html_string.expect("Ошибка какая-то"))
        .collect::<Vec<HtmlResponse>>();

    let adds_urls: Vec<String> = pages_html
        .into_par_iter()
        .filter_map(|html| match get_page_ads(html.html) {
            Ok(urls) => Some(urls),

            Err(e) => {
                println!("Ошибка парсинга селектора: {}", e);
                None
            }
        })
        .flatten()
        .collect();

    Ok(adds_urls)
}

async fn _test_cat(category: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ads_urls = get_ads_category_urls(category.to_owned()).await?;
    let ads_html = get_html_from_urls(ads_urls).await?;

    let mut futures = Vec::new();

    let client = Client::builder().build()?;
    let semaphore = Arc::new(Semaphore::new(10));

    for ad in ads_html {
        let client = client.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        futures.push(tokio::spawn(async move {
            let _permit = permit;
            Ad::from(&client, ad).await
        }));
    }

    let ads = futures::future::join_all(futures).await;

    for ad in ads.into_iter().flatten() {
        add_to_csv(ad, &format!("{}.csv", parse_category(category)))?;
    }

    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    let sales_response = a_html_response(&client, SALES.to_owned()).await?;
    let categories = get_categories(sales_response.html)?;

    for category in categories.into_iter() {
        let ads_urls = get_ads_category_urls(category.to_owned()).await?;
        let ads_html = get_html_from_urls(ads_urls).await?;

        let mut futures = Vec::new();

        let client = Client::builder().build()?;
        let semaphore = Arc::new(Semaphore::new(10));

        for ad in ads_html {
            let client = client.clone();
            let permit = semaphore.clone().acquire_owned().await?;
            futures.push(tokio::spawn(async move {
                let _permit = permit;
                Ad::from(&client, ad).await
            }));
        }

        let ads = futures::future::join_all(futures).await;

        for ad in ads.into_iter().flatten() {
            add_to_csv(ad, &format!("{}.csv", parse_category(&category)))?;
        }
    }

    Ok(())
}

async fn _test_ad(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    let ad_html = a_html_response(&client, url.to_owned()).await?;

    let ad = Ad::from(&client, ad_html).await;
    println!("{ad:#?}");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let client = Client::builder().build()?;
    // let sales_response = a_html_response(&client, SALES.to_owned()).await?;
    // let categories = get_categories(sales_response.html)?;
    // println!("{categories:#?}");
    // let category = "https://sportingshot.ru/sales/oruzhie/";
    // _test_cat(category).await?;
    // _test_ad().await?;
    run().await?;
    // let ad_url = "https://sportingshot.ru/sales/oruzhie/1996-prodam_pompovoe_ruzhyo_germanica_h_wragf_(fabarm_sdass_wood)";
    // _test_ad(ad_url).await?;
    Ok(())
}
