use std::io;
use std::fs;
use std::io::{Read, BufReader, BufRead};
use std::fs::File;

fn main() -> io::Result<()> {

    //TODO
    // download pds catalog
    // call convert to txt

    let path = std::env::args().nth(1).expect("File name missing.");
    let mut file = fs::File::open(path)?;
    let urls = getUrls(file)?;
    println!("{:?}", urls);

    //TODO
    // fetch url
    // parser source
    // insert to db

    Ok(())
}

fn getUrls(mut file: File) -> io::Result<Vec<String>> {
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
