mod parser;
mod utils;
use parser::{a_html_response, all_pages, get_categories, get_page_ads, Ad, HtmlResponse};
use rayon::prelude::*;
use reqwest::Client;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    // let client = Arc::new(Client::builder().build())?;
    let sales_response = a_html_response(&client, SALES.to_owned()).await?;
    let categories = get_categories(sales_response.html)?;
    // println!("{categories:#?}");
    //
    // let cat = "https://sportingshot.ru/sales/oruzhie/";
    // let cat_response = a_html_response(&client, cat.to_owned()).await?;
    // let pages = all_pages(cat_response.html, cat)?;
    // println!("{pages:#?}");

    // let page = "https://sportingshot.ru/sales/oruzhie/?page=19";
    // let page_resp = a_html_response(&client, page.to_owned()).await?;
    // let ads_urls = get_page_ads(page_resp.html)?;
    // println!("{ads_urls:#?}");

    // let ad_url = "https://sportingshot.ru/sales/oruzhie/1927_ruzhe_izh_43m_gorizontalka,_kalibr_16";
    // let ad_body = a_html_response(&client, ad_url.to_owned()).await?;
    // let ad = Ad::from(&client, ad_body).await;
    // println!("{:#?}", ad);

    for category in categories.into_iter() {
        // let semaphore = Arc::new(Semaphore::new(10)); // Limit to 10 concurrent tasks

        let ads_urls = get_ads_category_urls(category.to_owned()).await?;
        let ads_html = get_html_from_urls(ads_urls).await?;

        let mut futures = Vec::new();

        for ad in ads_html {
            let client = Client::builder().build()?;
            // let permit = semaphore.clone().acquire_owned().await?;
            futures.push(async move { Ad::from(&client, ad).await });
            // futures.push(tokio::spawn(async move {
            //     let _permit = permit;
            //     Ad::from(&client, ad).await
            // }));
        }

        let ads = futures::future::join_all(futures).await;

        for ad in ads.into_iter() {
            add_to_csv(ad, &format!("{}.csv", parse_category(&category)))?;
        }
        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
