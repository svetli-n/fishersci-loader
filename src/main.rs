use std::{io, fmt};
use std::fs;
use std::io::{Read, BufReader, BufRead, stdout, Write};
use std::fs::File;
use curl::easy::Easy;
use std::collections::HashMap;
use std::ops::Index;
use std::collections::hash_map::RandomState;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::document::Document;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_json::map::Values;
use fantoccini::{ClientBuilder, Locator};
use tokio;
use meilisearch_sdk::search::Selectors;


#[derive(Debug)]
enum Currency {
    USD,
    EUR,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self);
        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {

    //TODO
    // download pdf catalog
    // call convert to txt

    // let path = std::env::args().nth(1).expect("File name missing.");
    // let mut file = fs::File::open(path)?;
    // let urls = get_urls(file)?;
    // println!("{:?}", urls);

    // RUN docker run --rm --name geckodriver -p 4444:4444 instrumentisto/geckodriver
    let mut c = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await.expect("failed to connect to WebDriver");
    let ids = vec!["BP1758-100", "BP2687100", "PLGD22M", "AA4322422", "AC41957-1000"];
    // let ids = vec!["BP1758-100"];
    for id in ids {
        let mut spec = HashMap::new();
        let url = format!("https://www.fishersci.com/us/en/catalog/search/products?keyword={}", id);
        c.goto(url.as_str()).await?;
        let url = c.current_url().await?;
        let mut spec_table = c.find(Locator::Css(".spec_table")).await?;
        for mut row in  spec_table.find_all(Locator::Css("tr")).await? {
            let mut cols = row.find_all(Locator::Css("td")).await?;
                if cols.len() == 2 {
                    let key = cols[0].text().await?;
                    let value = cols[1].text().await?;
                    if !key.is_empty() && !value.is_empty() {
                        // println!("{}: {}", key, value);
                        spec.insert(key, value);
                    }
                }
        }
        let mut price = c.find(Locator::Css(".qa_single_price")).await?;
        let price = price.find(Locator::Css("b")).await?.text().await?;
        spec.insert("id".to_string(), id.to_string());
        spec.insert("price".to_string(), price.to_string().replace("$", ""));
        spec.insert("currency".to_string(), Currency::USD.to_string());
        spec.insert("supplier".to_string(), "fishersci".to_string());
        // println!("{:?}", spec);
        insert_to_db(spec).await;
    }

    c.close().await;

    Ok(())
}

// docker run -it --rm -p 7700:7700 getmeili/meilisearch:latest ./meilisearch
async fn insert_to_db(spec: HashMap<String, String>) {
    let host_port = "http://localhost:7700";
    let c = Client::new(host_port, "");
    let index_name = "chemicals";
    let chemicals = c.get_or_create(index_name).await.unwrap();

    let client = reqwest::Client::new();
    let res = client.post(format!("{}/indexes/{}/documents", host_port, index_name))
        .json(&[spec])
        .send().await.unwrap();
    println!("{:?}", res.status());
    // println!("{:?}", chemicals.search().with_query("max").execute::<HashMap<String, String>>().await.unwrap().hits);
}


fn get_urls(mut file: File) -> io::Result<Vec<String>> {
    let mut lines = BufReader::new(file).lines();
    let mut inside = false;
    let mut cnt = 0;
    let mut urls = Vec::new();
    for line in lines.by_ref() {
        let current = line?;
        if current.contains("Quantity Packaging Cat. No.") {
            inside = true;
            continue;
        }
        if inside {
            if current.is_empty() {
                cnt += 1;
                if cnt == 2 {
                    inside = false;
                    cnt = 0;
                }
            } else {
                let id = current.split_whitespace().last();
                match id {
                    Some(val) => urls.push(format!("https://www.fishersci.com/us/en/catalog/search/products?keyword={}", val)),
                    _ => panic!("bad value: {}", current),
                }
            }
        }
    }
    Ok(urls)
}
