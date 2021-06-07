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


fn main() -> io::Result<()> {

    //TODO
    // download pdf catalog
    // call convert to txt

    // let path = std::env::args().nth(1).expect("File name missing.");
    // let mut file = fs::File::open(path)?;
    // let urls = get_urls(file)?;
    // println!("{:?}", urls);

    let id = "BP1758-500";
    let url = format!("https://www.fishersci.com/us/en/catalog/search/products?keyword={}", id);
    let html = fetch_url(&url);
    let mut spec = get_spec(html);
    spec.insert("id".to_string(), id.to_string());
    spec.insert("id".to_string(), id.to_string());
    insert_to_db(spec);
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Doc {
    id: String,
    ph: String,
    grade: String,
    packaging: String,
    identification: String,
    filtered_through: String,
    color: String,
    lead_pb: String,
    arsenic_as: String,
    iron_fe: String,
    physical_form: String,
    concentration: String,
    chemical_name_or_material: String,
    quantity: String,
    copper_cu: String,
    calcium_ca: String,
    protease: String,
    magnesium_mg: String,
    zinc_zn: String,
    dnase:  String,
}

impl Doc {
    fn from(spec: HashMap<String, String>) -> Self {
        Doc {
            id: spec.get("id").unwrap().to_owned(),
            ph: spec.get("pH").unwrap().to_owned(),
            grade: spec.get("Grade").unwrap().to_owned(),
            packaging: spec.get("Packaging").unwrap().to_owned(),
            identification: spec.get("Identification").unwrap().to_owned(),
            filtered_through: spec.get("Filtered Through").unwrap().to_owned(),
            color: spec.get("Color").unwrap().to_owned(),
            lead_pb: spec.get("Lead (Pb)").unwrap().to_owned(),
            arsenic_as: spec.get("Arsenic (As)").unwrap().to_owned(),
            iron_fe: spec.get("Iron (Fe)").unwrap().to_owned(),
            physical_form: spec.get("Physical Form").unwrap().to_owned(),
            concentration: spec.get("Concentration").unwrap().to_owned(),
            chemical_name_or_material: spec.get("Chemical Name or Material").unwrap().to_owned(),
            quantity: spec.get("Quantity").unwrap().to_owned(),
            copper_cu: spec.get("Copper (Cu)").unwrap().to_owned(),
            calcium_ca: spec.get("Calcium (Ca)").unwrap().to_owned(),
            protease: spec.get("Protease").unwrap().to_owned(),
            magnesium_mg: spec.get("Magnesium (Mg)").unwrap().to_owned(),
            zinc_zn: spec.get("Zinc (Zn)").unwrap().to_owned(),
            dnase: spec.get("DNase").unwrap().to_owned(),
        }
    }
}

impl Document for Doc {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
       &self.id
    }
}

// docker run -it --rm -p 7700:7700 getmeili/meilisearch:latest ./meilisearch
fn insert_to_db(spec: HashMap<String, String>) {
    block_on(async move {
        let client = Client::new("http://localhost:7700", "");
        let chemicals = client.get_or_create("chemicals").await.unwrap();
        let doc = Doc::from(spec);
        let res = chemicals.add_documents(&[doc], Some("id")).await.unwrap();
        println!("{:?}", chemicals.search().with_query("max").execute::<Doc>().await.unwrap().hits);
    })
}

//TODO use https://docs.rs/fantoccini/0.17.3/fantoccini/ instead to get js
//TODO get sku and price
fn get_spec(st: String) -> HashMap<String, String, RandomState> {
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
