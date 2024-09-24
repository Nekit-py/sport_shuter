use chrono::NaiveDate;
use reqwest::{header, Client};
use scraper::{Html, Selector};

use crate::utils::{extract_number, generate_pages, parse_date};
use crate::MAIN_URL;

enum Contact {
    Phone,
    Email,
}

#[derive(Clone)]
pub struct HtmlResponse {
    pub url: String,
    pub html: String,
}

impl HtmlResponse {
    pub fn new(url: String, html: String) -> Self {
        Self { url, html }
    }
}

#[derive(Debug, Default)]
pub struct Ad {
    seller: Option<String>,
    seller_phone: Option<String>,
    email: Option<String>,
    city: Option<String>,
    announce_date: Option<NaiveDate>,
    price: Option<u32>,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
}

impl Ad {
    pub async fn from(client: &Client, html_r: HtmlResponse) -> Ad {
        let mut ad = Ad::default();

        if let Some(ad_id) = extract_number(&html_r.url) {
            if let Ok(Some(phone)) = get_contact(client, Contact::Phone, &ad_id).await {
                ad.seller_phone = Some(
                    phone
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>(),
                );
            }
            if let Ok(email) = get_contact(client, Contact::Email, &ad_id).await {
                ad.email = email
            }
        }

        ad.url = Some(html_r.url);
        ad.get_announce_date(&html_r.html);
        ad.get_city(&html_r.html);
        ad.get_title(&html_r.html);
        ad.get_description(&html_r.html);
        ad.get_price(&html_r.html);
        ad.get_seller(&html_r.html);
        ad
    }

    pub fn as_record(self) -> [String; 9] {
        [
            self.seller.unwrap_or_default(),
            self.seller_phone.unwrap_or_default(),
            self.email.unwrap_or_default(),
            self.price.map(|p| p.to_string()).unwrap_or_default(),
            self.city.unwrap_or_default(),
            self.title.unwrap_or_default(),
            self.description.unwrap_or_default(),
            self.announce_date
                .map(|d| d.to_string())
                .unwrap_or_default(),
            self.url.unwrap_or_default(),
        ]
    }

    fn get_announce_date(&mut self, html: &str) {
        let html = Html::parse_document(html);
        if let Ok(date_selector) =
            Selector::parse("#body > section > div > article > div > div > span")
        {
            if let Some(date) = html.select(&date_selector).next() {
                let str_date: String = date.text().collect();
                self.announce_date = parse_date(str_date.trim());
            }
        }
    }

    fn get_city(&mut self, html: &str) {
        let html = Html::parse_document(html);
        if let Ok(city_selector) =
            Selector::parse("#body > section > div > article > div > div > span:nth-child(2)")
        {
            if let Some(city) = html.select(&city_selector).next() {
                let city: String = city.text().collect();
                self.city = Some(city.trim().to_string());
            }
        }
    }

    fn get_seller(&mut self, html: &str) {
        let html = Html::parse_document(html);
        if let Ok(seller_selector) = Selector::parse(
            "#body > section > div > article > div > div > dl.ads_page_inf.first > dd",
        ) {
            if let Some(seller) = html.select(&seller_selector).next() {
                let seller: String = seller.text().collect();
                self.seller = Some(seller.trim().to_string());
            }
        }
    }

    fn get_price(&mut self, html: &str) {
        let html = Html::parse_document(html);
        if let Ok(price_selector) =
            Selector::parse("#body > section > div > article > div > div > div.ads_page_pr")
        {
            if let Some(price) = html.select(&price_selector).next() {
                let str_price: String = price
                    .text()
                    .flat_map(|s| s.chars().filter(|c| c.is_ascii_digit()))
                    .collect();
                if let Ok(digit_price) = str_price.parse::<u32>() {
                    self.price = Some(digit_price);
                }
            }
        }
    }

    fn get_title(&mut self, html: &str) {
        let html = Html::parse_document(html);
        if let Ok(title_selector) =
            Selector::parse("#body > section > div > article > div > div > h1")
        {
            if let Some(title) = html.select(&title_selector).next() {
                let title: String = title.text().collect();
                self.title = Some(title.trim().to_string());
            }
        }
    }

    fn get_description(&mut self, html: &str) {
        let html = Html::parse_document(html);
        if let Ok(description_selector) =
            Selector::parse("#body > section > div > article > div > div > div.ads_page_c")
        {
            if let Some(description) = html.select(&description_selector).next() {
                let description: String = description.text().collect();
                self.description = Some(description.trim().to_string());
            }
        }
    }

    pub fn show_phone(&self) {
        println!("{:?}", self.seller_phone);
    }

    pub fn show_email(&self) {
        println!("{:?}", self.email);
    }
}

async fn get_contact(
    client: &Client,
    contact: Contact,
    id: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let body_str = match contact {
        Contact::Phone => format!("ajax_take=phone&docid={}", id),
        Contact::Email => format!("ajax_take=email&docid={}", id),
    };
    let mut headers = header::HeaderMap::new();
    headers.insert("accept", "text/html, */*; q=0.01".parse().unwrap());
    headers.insert("accept-language", "ru,en;q=0.9".parse().unwrap());
    headers.insert(
        "content-type",
        "application/x-www-form-urlencoded; charset=UTF-8"
            .parse()
            .unwrap(),
    );
    headers.insert("origin", "https://sportingshot.ru".parse().unwrap());
    headers.insert("priority", "u=1, i".parse().unwrap());
    headers.insert("sec-ch-ua", "\"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"YaBrowser\";v=\"24.7\", \"Yowser\";v=\"2.5\"".parse().unwrap());
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
    headers.insert("sec-fetch-dest", "empty".parse().unwrap());
    headers.insert("sec-fetch-mode", "cors".parse().unwrap());
    headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
    headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 YaBrowser/24.7.0.0 Safari/537.36".parse().unwrap());
    headers.insert("x-requested-with", "XMLHttpRequest".parse().unwrap());

    let res = client
        .post("https://sportingshot.ru/kontaktyi_obyavleniya")
        .headers(headers)
        .body(body_str)
        .send()
        .await?;
    let text_resp = res.text().await?;
    Ok(Some(text_resp))
}

pub async fn a_html_response(
    client: &Client,
    url: String,
) -> Result<HtmlResponse, Box<dyn std::error::Error>> {
    let response = client
        .get(url.as_str())
        .header(header::USER_AGENT, "Mozilla/5.0")
        .send()
        .await?;

    let status_code = response.status();
    if status_code.is_client_error() || status_code.is_server_error() {
        eprintln!("Неверный статус-код: {}", status_code);
    }

    let body = response.text().await?;
    let document = Html::parse_document(&body);
    println!("{}", url.as_str());
    Ok(HtmlResponse::new(url, document.root_element().html()))
}

pub fn get_categories(body: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let selector = Selector::parse("#pdopage > div > div > a")?;
    let html = Html::parse_document(&body);
    let categories: Vec<String> = html
        .select(&selector)
        .filter_map(|link| {
            link.value()
                .attr("href")
                .map(|href| format!("{}{}", MAIN_URL, href))
        })
        .collect();
    Ok(categories)
}

// https://sportingshot.ru/sales/oruzhie/?page=2
pub fn all_pages(html: String, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let html = Html::parse_document(html.as_str());
    let selector = Selector::parse("#pager > ul > li > a")?;
    if let Some(last_page_element) = html.select(&selector).last() {
        if let Some(last_page_text) = last_page_element.text().next() {
            if let Ok(last_page_num) = last_page_text.trim().parse::<usize>() {
                return Ok(generate_pages(url, last_page_num));
            }
        }
    }
    // Err(Box::new(std::io::Error::new(
    //     std::io::ErrorKind::Other,
    //     "Не удалось получить список страниц",
    // )))
    // Тут бывает, что страница всего одна, она и главная, поэтому ее и возвращаем
    Ok(vec![url.to_owned()])
}

pub fn get_page_ads(body: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let selector = Selector::parse("#pdopage > article > div > div > div > div.ads_page_t > a")?;
    let html = Html::parse_document(&body);
    let ads_urls: Vec<String> = html
        .select(&selector)
        .filter_map(|link| {
            link.value()
                .attr("href")
                .map(|href| format!("{}{}", MAIN_URL, href))
        })
        .collect();
    Ok(ads_urls)
}
