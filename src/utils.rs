use crate::parser::Ad;
use chrono::NaiveDate;
use std::{env, fs};

use regex::Regex;

#[allow(dead_code)]
pub fn add_to_csv(ad: Ad, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;
    let file_path = current_exe.parent().unwrap().join(file_name);

    let file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(ad.as_record())?;

    wtr.flush()?;
    Ok(())
}

#[inline]
pub fn generate_pages(url: &str, last_page_num: usize) -> Vec<String> {
    (1..=last_page_num)
        .map(|page_num| format!("{}?page={}", url, page_num))
        .collect()
}

pub fn extract_number(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('/').collect();
    let suffix = parts[5];
    if let Some(pos) = suffix.find('_') {
        return Some(suffix[..pos].to_string());
    }
    None
}

pub fn parse_date(date_str: &str) -> Option<NaiveDate> {
    let regex = Regex::new(r"(\d+) (\w+) (\d+)").unwrap();
    let captures = regex.captures(date_str)?;

    let day = captures[1].parse::<u32>().ok()?;
    let month = match &captures[2] {
        "января" => 1,
        "февраля" => 2,
        "марта" => 3,
        "апреля" => 4,
        "мая" => 5,
        "июня" => 6,
        "июля" => 7,
        "августа" => 8,
        "сентября" => 9,
        "октября" => 10,
        "ноября" => 11,
        "декабря" => 12,
        _ => return None,
    };
    let year = captures[3].parse::<i32>().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)
}

pub fn parse_category(url: &str) -> String {
    let splitted_parts = url.split("/").collect::<Vec<&str>>();
    splitted_parts
        .get(splitted_parts.len() - 2)
        .map(|&s| s.to_string())
        .unwrap_or_else(|| "undefined".to_string())
}
