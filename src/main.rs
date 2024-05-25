use elasticsearch::{
    http:: transport::Transport,
    Elasticsearch, Error
};
use serde_json::json;
use myesrs::{MyDocument, search_doc};
use dotenv::dotenv;


#[tokio::main]
async fn main() -> Result<(), Error> {
    init();
    let es_url = std::env::var("ES_URL").expect("ES_URL must be set");
    let transport = Transport::single_node(es_url.as_str()).unwrap();
    let client = Elasticsearch::new(transport);
    let query = json!({
        "query":{
            "match_all":{}
        }
    });
    let res: Vec<MyDocument> = search_doc(&client,query).await?;
    print!("{:?}", res);

    Ok(())
}

fn init() {
    dotenv().ok();
}
