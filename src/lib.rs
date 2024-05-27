use elasticsearch::{
    http::response::Response,
    Elasticsearch, Error, IndexParts, SearchParts,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use regex::Regex;
use lazy_static::lazy_static;
use std::{collections::HashMap, str::FromStr};


lazy_static! {
    static ref LOG_PATTERN: Regex = Regex::new(
        r"(?x)
        ^(?P<timestamp>\d{2}:\d{2}:\d{2}\.\d{3})\s+
        \[(?P<thread>[^\]]+)\]\s+
        (?P<loglevel>\w+)\s+
        (?P<class>[^\s]+)\s+-\s+
        \[(?P<context>[^\]]+)\]\s+-\s+
        (?P<message>.*?)
        (\n\s*at\s+(?P<stack>.+))?
        $"
    ).unwrap();

    static ref FILE_NAME_PATTERN: Regex = Regex::new(
        r"(?x)
        ^(?P<loglevel>\w+)\.
        (?P<date>\d{4}-\d{2}-\d{2})\.log$
        "
    ).unwrap();
}

//文件名为 info.2024-03-27.log  error.2024-03-27.log
pub fn read_date_from_file_name(file_name: &str) -> (String, String) {
    let captures = FILE_NAME_PATTERN.captures(file_name).unwrap();
    (captures["date"].to_string(), captures["loglevel"].to_string())
}


pub async fn read_from_file(file_path: &str) -> Result<String, Error> {
    let content = tokio::fs::read_to_string(file_path).await?;
    Ok(content)
}


pub fn parse_log_line(line: &String,row: u32,date:&String) -> Option<MyLogRecord> {
    if let Some(captures) = LOG_PATTERN.captures(line.as_str()) {
        let mut log_data = HashMap::new();
        log_data.insert("timestamp".to_string(), captures["timestamp"].to_string());
        log_data.insert("thread".to_string(), captures["thread"].to_string());
        log_data.insert("loglevel".to_string(), captures["loglevel"].to_string());
        log_data.insert("class".to_string(), captures["class"].to_string());
        log_data.insert("context".to_string(), captures["context"].to_string());
        log_data.insert("message".to_string(), captures["message"].to_string());
        
        if let Some(stack) = captures.name("stack") {
            log_data.insert("stack".to_string(), stack.as_str().to_string());
        }
        let key = format!("{}_{}", &log_data["timestamp"],row);
        let mut stack = String::from_str("").unwrap();

        if let Some(s) = log_data.get("stack") {
            stack = s.to_string();
        }

        let timestamp = format!("{} {}",date,&log_data["timestamp"]);

        let record = MyLogRecord::new(
            key,
            timestamp,
            &log_data["thread"],
            &log_data["loglevel"],
            &log_data["class"],
            &log_data["context"],
            &log_data["message"],
            stack,
        );
        

        Some(record)
    } else {
        None
    }
}




pub trait EsEntity {
    fn index() -> String;
    fn id(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyLogRecord {
    key: String,
    timestamp: String,
    thread: String,
    loglevel: String,
    class: String,
    context: String,
    message: String,
    stack: String,
}

impl EsEntity for MyLogRecord {
    fn index() -> String {
        String::from("my_log_index")
    }
    fn id(&self) -> &str {
        &self.key
    }

}

impl MyLogRecord {
    pub fn new(key: String, timestamp: String, thread: &str, loglevel: &str, class: &str, context: &str, message: &str, stack: String) -> Self {
        MyLogRecord {
            key: key,
            timestamp: timestamp.to_string(),
            thread: thread.to_string(),
            loglevel: loglevel.to_string(),
            class: class.to_string(),
            context: context.to_string(),
            message: message.to_string(),
            stack: stack
        }
    }
}

pub async fn insert_doc<T: EsEntity + Serialize>(
    client: &Elasticsearch,
    doc: &T,
) -> Result<Response, Error> {
    let body = serde_json::to_value(doc).unwrap();
    let res = client
        .index(IndexParts::IndexId(T::index().as_str(), doc.id()))
        .body(body)
        .send()
        .await?;
    Ok(res)
}

pub async fn search_doc<R>(client: &Elasticsearch,query:Value) -> Result<Vec<R>, Error>
where
    R: EsEntity + Serialize + DeserializeOwned,
{

    let response = client
        .search(SearchParts::Index(&[R::index().as_str()]))
        .body(query)
        .send()
        .await?;
    let response_body = response.json::<Value>().await?;
    //取出查询结果
    let hits = response_body["hits"]["hits"].as_array().unwrap();
    let mut result = Vec::new();
    for hit in hits {
        let source = hit["_source"].clone();
        let doc: R = serde_json::from_value(source).unwrap();
        result.push(doc);
    }
    Ok(result)
}
