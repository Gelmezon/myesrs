
use std::str::FromStr;

use elasticsearch::{
    http:: transport::Transport,
    Elasticsearch, Error
};
use dotenv::dotenv;
use myesrs::{read_from_file,parse_log_line,read_date_from_file_name,insert_doc};



#[tokio::main]
async fn main() -> Result<(), Error> {
    init();
    let es_url = std::env::var("ES_URL").expect("ES_URL must be set");
    let transport = Transport::single_node(es_url.as_str()).unwrap();
    let client = Elasticsearch::new(transport);
    let file_name = "samle.2024-05-22.log";
    let file_path = String::from("./") + file_name;
    let (date, _loglevel) = read_date_from_file_name(file_name);    
    let content =  read_from_file(file_path.as_str()).await?;
    //content用\n 换行符切割字符串
    let content = content.split("\n");
    let mut log_str = String::new();
    let mut row_num = 0;
    let mut log_data = Vec::new();
    for line in content {
        //如果line是\t开头
        if line.starts_with("\t") {
            //去掉\t
            log_str.push_str(line.trim_start_matches("\t"));
            log_str.push_str(" \n ");
            continue;
        }
        //如果log_str不为空
        log_str = String::from_str(line).unwrap();
        let log = parse_log_line(&log_str,row_num,&date);
        match log {
            Some(log) => {
                log_data.push(log);
            },
            None => {
                println!("No match");
            }
        }
        row_num += 1;
    }

    println!("{:?}", log_data);
    for data in log_data {
        let res = insert_doc(&client,&data).await?;
        println!("{:?}", res);
    }
    //如果log_str不为空
    // if !log_str.is_empty() {
    //     println!("{:?}", log_str);
    //     let log = parse_log_line(&log_str,row_num);
    //     match log {
    //         Some(log) => {
    //             println!("{:?}", log);
    //         },
    //         None => {
    //             println!("No match");
    //         }
    //     }
    // }

    Ok(())
}

fn init() {
    dotenv().ok();
}
