use crate::{NovelInfo, Result};
use rusqlite::Connection;

pub fn connect_database() -> Result<Connection> {
    let conn = Connection::open("novel_info.db")?;
    conn.execute("CREATE TABLE IF NOT EXISTS novel_info (wenku8_id INTEGER PRIMARY KEY,name TEXT,auther TEXT,status TEXT,tag TEXT,summary TEXT,image_url TEXT,download_url TEXT);",
        (),
    )?;
    Ok(conn)
}

pub fn search_novel_status(id: i32, conn: &Connection) -> Result<String> {
    let result = conn.query_row(
        "SELECT status FROM novel_info WHERE wenku8_id = ?;",
        (id,),
        |row| row.get(0),
    )?;
    Ok(result)
}

pub fn insert_novel_info(novel_info: NovelInfo, conn: &Connection) -> Result<()> {
    let _ = conn.execute("INSERT INTO novel_info (wenku8_id, name, auther, status, tag, summary, image_url, download_url) VALUES(?,?,?,?,?,?,?,?);", (
        novel_info.wenku8_id,
        novel_info.name,
        novel_info.auther,
        novel_info.status,
        novel_info.tags.join(","),
        novel_info.summary,
        novel_info.image_link,
        novel_info.download_link,
    ))?;
    Ok(())
}

pub fn update_novel_info(novel_info: NovelInfo, conn: &Connection) -> Result<()> {
    let _ = conn.execute("UPDATE novel_info SET name=?, auther=?, status=?, tag=?, summary=?, image_url=?, download_url=? WHERE wenku8_id=?;", (
        novel_info.name,
        novel_info.auther,
        novel_info.status,
        novel_info.tags.join(","),
        novel_info.summary,
        novel_info.image_link,
        novel_info.download_link,
        novel_info.wenku8_id,
    ))?;
    Ok(())
}
