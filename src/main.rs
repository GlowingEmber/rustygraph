use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct PageResponse {
    parse: Parse,
}

#[derive(Deserialize, Debug)]
struct Parse {
    title: String,
}

#[derive(Deserialize, Debug)]
struct PageLinksResponse {
    #[serde(rename = "continue")]
    continued: Option<Continued>,
    query: Query,
}

#[derive(Deserialize, Debug)]
struct Continued {
    plcontinue: String,
}

#[derive(Deserialize, Debug)]
struct Query {
    pages: std::collections::HashMap<String, Page>,
}

#[derive(Deserialize, Debug)]
struct Page {
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

    // let mut url = format!(
    //         "https://en.wikipedia.org/w/api.php?action=query&page={}&format=json",
    //         from_string
    // );

    let from_json: PageResponse = client
        .get(format!(
            "https://en.wikipedia.org/w/api.php?action=parse&page={}&format=json",
            from_string
        ))
        .header("User-Agent", "rustypath/1.0 (ethan.e.hopkins@gmail.com)")
        .send()
        .await?
        .json()
        .await?;

    let from_string = from_json.parse.title;

    // let mut url = format!(
    //     "https://en.wikipedia.org/w/api.php?action=query&page={}&format=json",
    //     to_string
    // );

    let to_json: PageResponse = client
        .get(format!(
            "https://en.wikipedia.org/w/api.php?action=parse&page={}&format=json",
            to_string
        ))
        .header("User-Agent", "rustypath/1.0 (ethan.e.hopkins@gmail.com)")
        .send()
        .await?
        .json()
        .await?;

    let to_string = to_json.parse.title;

    print!("Finding a path from {} to {}...", from_string, to_string);
    io::stdout().flush().unwrap();

    let mut visited: HashMap<String, String> = HashMap::new();
    visited.insert(from_string.clone(), "".to_string());
    let mut frontier: Vec<String> = Vec::new();
    frontier.push(from_string.clone());

    fn path_list(mut child: String, visited: &HashMap<String, String>) -> Vec<String> {
        let mut path: Vec<String> = Vec::new();
        let mut parent: String = visited.get(&child).cloned().unwrap_or_default();
        while !child.is_empty() {
            path.push(child.clone());
            child = parent;
            parent = visited.get(&child).cloned().unwrap_or_default();
        }
        path.reverse();
        path
    }

    'outer: while frontier.len() > 0 {
        let page_string = frontier.pop().expect("Frontier was empty");
        let mut url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&titles={}&prop=links&pllimit=max&plnamespace=0&format=json",
            page_string
        );
        loop {
            let response: PageLinksResponse = client
                .get(&url)
                .header("User-Agent", "rustypath/1.0 (ethan.e.hopkins@gmail.com)")
                .send()
                .await?
                .json()
                .await?;
            for (_, page) in &response.query.pages {
                if let Some(links) = &page.links {
                    for link in links {
                        // println!("{}", link.title.clone());
                        if link.title.clone() == to_string {
                            visited.insert(link.title.clone(), page_string.to_string());
                            println!("{:?}", path_list(link.title.clone(), &visited));
                            break 'outer;
                        }
                        if !visited.contains_key(&link.title) {
                            visited.insert(link.title.clone(), page_string.to_string());
                            frontier.push(link.title.clone());
                        }
                    }
                }
            }
            if let Some(c) = &response.continued {
                url = format!(
                    "https://en.wikipedia.org/w/api.php?action=query&titles={}&prop=links&pllimit=max&plnamespace=0&format=json&plcontinue={}",
                    page_string, c.plcontinue,
                );
            } else {
                break;
            }
        }
    }

    Ok(())
}
