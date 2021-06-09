use std::io;
use std::fs;
use std::io::{Read, BufReader, BufRead, stdout, Write};
use std::fs::File;
use curl::easy::Easy;
use std::collections::HashMap;
use std::ops::Index;
use std::collections::hash_map::RandomState;
use meilisearch_sdk::client::Client;
use futures::executor::block_on;
use meilisearch_sdk::document::Document;
use std::fmt::Display;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_json::map::Values;


fn main() -> io::Result<()> {

    //TODO
    // download pdf catalog
    // call convert to txt

    // let path = std::env::args().nth(1).expect("File name missing.");
    // let mut file = fs::File::open(path)?;
    // let urls = get_urls(file)?;
    // println!("{:?}", urls);

    let ids = vec!["BP1758-100", "BP2687100", "PLGD22M", "AA4322422", "AC41957-1000"];
    for id in ids {
        let url = format!("https://www.fishersci.com/us/en/catalog/search/products?keyword={}", id);
        let html = fetch_url(&url);
        let mut spec = get_spec(html);
        spec.insert("id".to_string(), id.to_string());
        insert_to_db(spec);
    }
    Ok(())
}

// docker run -it --rm -p 7700:7700 getmeili/meilisearch:latest ./meilisearch
fn insert_to_db(spec: HashMap<String, String>) {
    block_on(async move {
        let host_port = "http://localhost:7700";
        let c = Client::new(host_port, "");
        let index_name = "chemicals";
        let chemicals = c.get_or_create(index_name).await.unwrap();

        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("{}/indexes/{}/documents", host_port, index_name))
            .json(&[spec])
            .send().unwrap();
        println!("{:?}", res.status());
        println!("{:?}", chemicals.search().with_query("100").execute::<HashMap<String, String>>().await.unwrap().hits);
    })
}

//TODO use https://docs.rs/fantoccini/0.17.3/fantoccini/ instead to get js
//TODO get sku and price
fn get_spec(st: String) -> HashMap<String, String> {
    let mut vals = Vec::new();
    let mut lines = st.split("\n").filter(|line| !line.is_empty());
    while let Some(line) = lines.next() {
        //TODO doesnt work, its in JavaScript
        // <b data-pirce-uom="EA" data-stockromm-sku="BP1758100" data-uw-rm-sr="">$85.00</b>
        if line.contains("data-stockromm-sku") {
            if let Some(mut start) = line.find("data-stockromm-sku") {
                start += "data-stockromm-sku".len();
                if let Some(end) = line[start..].find(" ") {
                    let sku = &line[start + 2..end];
                    vals.push("sku".to_string());
                    vals.push(sku.to_string());
                }
            }
        }
        if line.contains("spec_table") {
            lines.next();
            while let Some(inner) = lines.next() {
                if inner.contains("/table") {
                    break;
                }
                if inner.contains("td") {
                    if let Some(start) = inner.find(">") {
                        if let Some(end) = inner[start..].find("<") {
                            let val = inner[start + 1..start + end].to_string();
                            if !val.is_empty() {
                                vals.push(val);
                            }
                        }
                    }
                }
            }
            break;
        }
    }
    let mut dict = HashMap::new();
    for i in (1..vals.len()).step_by(2) {
        dict.insert(vals[i - 1].to_string(), vals[i].to_string());
    }
    dict
}

fn fetch_url(url: &str) -> String {
    let mut st = String::new();
    {
        let mut easy = Easy::new();
        easy.follow_location(true).unwrap();
        easy.url(url).unwrap();
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            st.push_str(std::str::from_utf8(data).unwrap());
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    st

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
