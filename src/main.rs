use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::io;

#[derive(Deserialize, Debug)]
struct APIResponse {
    query: Query,
}

#[derive(Deserialize, Debug)]
struct Query {
    pages: std::collections::HashMap<String, Page>,
}

#[derive(Deserialize, Debug)]
struct Page {
    title: String,
    links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
struct Link {
    #[serde(skip)]
    _namespace: i32,
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("From");
    let mut from_string = String::new();
    io::stdin()
        .read_line(&mut from_string)
        .expect("Failed to read line");
    println!("To");
    let mut to_string = String::new();
    io::stdin()
        .read_line(&mut to_string)
        .expect("Failed to read line");

    let client = Client::new();

    let mut visited: HashSet<String> = HashSet::new();
    let mut frontier: Vec<String> = Vec::new();
    frontier.push(from_string.clone());

    'outer: while frontier.len() > 0 {
        let page_string = frontier.pop().expect("Frontier was empty");
        let url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&titles={}&prop=links&pllimit=500&plnamespace=0&format=json",
            page_string
        );
        let response: APIResponse = client
            .get(url)
            .header("User-Agent", "rustypath/1.0 (ethan.e.hopkins@gmail.com)")
            .send()
            .await?
            .json()
            .await?;
        for (_, page) in response.query.pages {
            if let Some(links) = page.links {
                for link in links {
                    println!("{}", link.title.clone());
                    if link.title.clone() == "Surname" {
                        println!("Found!");
                        break 'outer;
                    }
                    if !visited.contains(&link.title) {
                        frontier.push(link.title.clone());
                    }
                    visited.insert(link.title);
                }
            }
        }
    }
    
    Ok(())
}
