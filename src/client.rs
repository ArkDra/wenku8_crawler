use crate::{BASE_URL, NovelInfo, Result, USER_AGENT};
use reqwest::blocking::Client;
use reqwest::header;
use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use toml::Table;

macro_rules! parse_element {
    ($selector:expr) => {
        Selector::parse($selector)?
    };
}

fn import_account() -> Result<String> {
    let mut toml_file = String::new();
    File::open("cookies.toml")?.read_to_string(&mut toml_file)?;
    return Ok(toml_file);
}

fn get_cookies() -> Result<String> {
    let account = Table::from_str(import_account()?.as_str())?;
    let cookies = format!(
        "jieqiUserInfo={}; PHPSESSID={}",
        account["jieqiUserInfo"]
            .as_str()
            .ok_or("Failed to parse jieqiUserInfo")?,
        account["PHPSESSID"]
            .as_str()
            .ok_or("Failed to parse PHPSESSID")?,    
    );
    Ok(cookies)
}

pub fn build_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, USER_AGENT.parse()?);
    headers.insert(header::COOKIE, get_cookies()?.parse()?);
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(60))
        .use_native_tls()
        .build()?;
    return Ok(client);
}
pub fn get_response(client: &Client, url: &str) -> Result<Html> {
    thread::sleep(Duration::from_millis(100));
    let response = client.get(url).send()?.text_with_charset("gb18030")?;
    Ok(Html::parse_document(&response))
}

pub fn get_max_page(client: &Client) -> Result<i32> {
    let max_page = get_response(
        client,
        &format!("{}/modules/article/articlelist.php", BASE_URL),
    )?
    .select(&parse_element!(".last"))
    .next()
    .ok_or("Failed to get max page")?
    .text()
    .collect::<Vec<_>>()[0]
        .parse::<i32>()?;
    return Ok(max_page);
}

pub fn get_novel_url_and_status(client: &Client, page_url: &str) -> Result<Vec<(String, String)>> {
    let html = get_response(client, &page_url)?;
    let novel_urls: Vec<String> = html
        .select(&parse_element!("tr > td > div > div:nth-child(1) > a"))
        .into_iter()
        .filter_map(|novel_url| {
            novel_url
                .value()
                .attr("href")
                .map(|href| format!("{}{}", BASE_URL, href))
        })
        .collect();
    let novel_statuses: Vec<String> = html
        .select(&parse_element!(
            "tr > td > div > div:nth-child(2) > p:nth-child(3)"
        ))
        .into_iter()
        .map(|novel_status| {
            let text = novel_status.text().collect::<String>();
            if text.contains("已完结") {
                "已完结".to_string()
            } else if text.contains("连载中") {
                "连载中".to_string()
            } else {
                "未知".to_string()
            }
        })
        .collect();
    let novel_urls_and_statuses: Vec<(String, String)> = novel_urls
        .iter()
        .zip(novel_statuses.iter())
        .map(|(url, status)| (url.to_owned(), status.to_owned()))
        .collect();

    return Ok(novel_urls_and_statuses);
}

pub fn get_novel_info(client: &Client, novel_url: &str) -> Result<NovelInfo> {
    let doc = get_response(client, novel_url)?;
    let wenku8_id = novel_url.split("/").collect::<Vec<_>>()[4]
        .split(".")
        .collect::<Vec<_>>()[0]
        .parse::<i32>()?;
    let name = doc
        .select(&parse_element!("span > b"))
        .next()
        .ok_or("Failed to find novel name")?
        .text()
        .collect::<Vec<_>>()[0]
        .to_string();
    let auther = doc
        .select(&parse_element!("tbody > tr:nth-child(2) > td:nth-child(2)"))
        .next()
        .ok_or("Failed to find novel auther")?
        .text()
        .collect::<Vec<_>>()[0]
        .to_string();
    let auther = auther.split("：").collect::<Vec<_>>()[1].to_string();
    let status = doc
        .select(&parse_element!("tbody > tr:nth-child(2) > td:nth-child(3)"))
        .next()
        .ok_or("Failed to find novel status")?
        .text()
        .collect::<Vec<_>>()[0]
        .to_string();
    let status = status.split("：").collect::<Vec<_>>()[1].to_string();
    let tags = doc
        .select(&parse_element!(
            "tbody > tr > td:nth-child(2) > span:nth-child(1) > b"
        ))
        .next()
        .ok_or("Failed to find novel tags")?
        .text()
        .collect::<Vec<_>>()[0]
        .to_string();
    let tags = tags.split("：").collect::<Vec<_>>()[1]
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let summary = doc
        .select(&parse_element!(
            "tbody > tr > td:nth-child(2) > span:last-of-type"
        ))
        .next()
        .ok_or("Failed to find novel summary")?
        .text()
        .collect::<Vec<_>>()
        .iter()
        .map(|s| s.trim())
        .collect();
    let image_link = doc
        .select(&parse_element!("tbody > tr > td:nth-child(1) > img"))
        .next()
        .ok_or("Failed to find novel image link")?
        .value()
        .attr("src")
        .ok_or("Failed to find novel image link")?
        .to_string();
    let download_link = format!("https://dl1.wenku8.com/down.php?type=utf8&id={wenku8_id}");

    let novel_info = NovelInfo {
        wenku8_id,
        name,
        auther,
        status,
        tags,
        summary,
        image_link,
        download_link,
    };
    return Ok(novel_info);
}
