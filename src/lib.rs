use elasticsearch::{
    http::response::Response,
    Elasticsearch, Error, IndexParts, SearchParts,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

pub trait EsEntity {
    fn index() -> String;
    fn id(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyDocument {
    key: String,
    size: i32,
    content: String,
}

impl EsEntity for MyDocument {
    fn index() -> String {
        String::from("my_index")
    }
    fn id(&self) -> &str {
        &self.key
    }

}

impl MyDocument {
    pub fn new(key: &str, size: i32, content: &str) -> MyDocument {
        MyDocument {
            key: key.to_string(),
            size: size,
            content: content.to_string(),
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
