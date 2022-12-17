use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
#[derive(Serialize, Deserialize, Debug)]
struct Panelist {
    name: String,
    description: String,
    prelude: String,
}
#[derive(Deserialize, Debug)]
struct OAIChoices {
    text: String,
    index: u8,
    logprobs: Option<u8>,
    finish_reason: String,
}
#[derive(Deserialize, Debug)]
struct OAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OAIChoices>,
}
#[derive(Serialize, Debug)]
struct OAIRequest {
    prompt: String,
    max_tokens: u32,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    let uri = "https://api.openai.com/v1/engines/text-davinci-001/completions";
    let preamble = "Answer the following question accurately, but find a funny way to shame republicans or praise democrats in your answer";

    let oai_token: String = env::var("OPENAI_API_KEY").unwrap();
    let auth_header_val = format!("Bearer {}", oai_token);
    let mut file = File::open("Panelists.json")?;
    let mut panelists_json = String::new();
    file.read_to_string(&mut panelists_json)?;
    let panelists: Vec<Panelist> = serde_json::from_str(&panelists_json)?;
    let panel_size = panelists.len().try_into().unwrap();

    println!("{esc}c", esc = 27 as char);
    println!(" Welcome to our Question and Answer Chat");
    println!(
        "Today we have {} distinguished panelists. They are",
        panelists.len()
    );
    let mut index = 1;
    for i in 0..(panel_size) as usize {
        println!(
            "{}: {}, {} ",
            i + 1,
            panelists[i].name,
            panelists[i].description
        );
    }
    println!(
        "Begin your question with a number or name to ask a specific panelist. Use Quit  or ^C to exit"
    );
    let mut rng = rand::thread_rng();

    let quit_str = "QUIT";

    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut user_text = String::new();

        stdin()
            .read_line(&mut user_text)
            .expect("Failed to read line");
        let first_word = &user_text.split(" ").next().unwrap();
        // println!("first word is {}", first_word);
        if first_word.to_uppercase().trim().eq(quit_str) {
            println!("Bye");
            break;
        }
        let mut people = panelists.iter();
        let index = match people.position(|p| {
            p.name
                .to_ascii_uppercase()
                .eq(&first_word.to_ascii_uppercase())
        }) {
            Some(n) => n, // A panelists name was detected
            None => match first_word.parse::<usize>() {
                // See if a number begins the question6 what
                Ok(n) => {
                    if n > 0 && n <= panel_size {
                        n - 1
                    } else {
                        // No panelist addressed so select one at random
                        rand::thread_rng().gen_range(0..panel_size) as usize
                    }
                }
                Err(_e) => rand::thread_rng().gen_range(0..panel_size.try_into().unwrap()),
            },
        };

        let panelist = &panelists[index as usize];
        println!("");
        let mut sp = Spinner::new(Spinners::SimpleDots, "\t\tOpen AI is thinking ...".into());
        let oai_request = OAIRequest {
            prompt: format!("{} {}", panelist.prelude, user_text),
            max_tokens: 500,
        };

        let body = Body::from(serde_json::to_vec(&oai_request)?);

        let req = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header("Authorization", &auth_header_val)
            .body(body)
            .unwrap();
        let res = client.request(req).await?;
        let body = hyper::body::aggregate(res).await?;
        let json: OAIResponse = serde_json::from_reader(body.reader())?;
        sp.stop();
        println!();
        println!("{}: {}", panelist.name, &json.choices[0].text[1..]);
    }
    Ok(())
}
