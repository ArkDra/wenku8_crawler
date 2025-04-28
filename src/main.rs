mod client;
mod database;

use rayon::prelude::*;
use client::{get_max_page, get_novel_info, get_novel_url_and_status};
use database::{insert_novel_info, search_novel_status, update_novel_info};
use std::sync::{Arc, Mutex};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const BASE_URL: &str = "https://www.wenku8.net";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";

#[derive(Debug)]
struct NovelInfo {
    wenku8_id: i32,
    name: String,
    auther: String,
    status: String,
    tags: Vec<String>,
    summary: String,
    image_link: String,
    download_link: String,
}

fn main() -> Result<()> {
    let client = client::build_client()?;
    let range = 1..=get_max_page(&client)?;
    let conn = Arc::new(Mutex::new(database::connect_database()?));

    range.into_par_iter().for_each(|i| {
        let _ = get_novel_url_and_status(
            &client,
            &format!("{}/modules/article/articlelist.php?page={}", BASE_URL, i),
        )
        .unwrap_or_default()
        .into_par_iter()
        .for_each(|e| {
            let conn_clone = Arc::clone(&conn);
            let id =
                e.0.split('/')
                    .last()
                    .and_then(|s| s.split('.').next())
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or_default();
            let new_status = e.1;
            let conn_clone_lock = conn_clone.lock().expect("Lock error");
            match search_novel_status(id, &conn_clone_lock) {
                Ok(status) => {
                    if status != new_status {
                        println!("ID：{} 更新状态：{} => {}", id, status, new_status);
                        if let Ok(novel_info) = get_novel_info(&client, &e.0) {
                            let _ = update_novel_info(novel_info, &conn_clone_lock);
                        };
                    }
                }
                Err(_) => {
                    println!("ID：{} 插入小说信息", id);
                    if let Ok(novel_info) = get_novel_info(&client, &e.0) {
                        let _ = insert_novel_info(novel_info, &conn_clone_lock);
                    }
                }
            }
        });
    });
    Ok(())
}
