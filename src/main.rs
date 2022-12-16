use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
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
    let preamble = "Answer the folloing question accurately, but find a funny way to shame republicans or praise democrats in your answer";

    let oai_token: String = env::var("OPENAI_API_KEY").unwrap();
    let auth_header_val = format!("Bearer {}", oai_token);
    let mut file = File::open("Panelists.json")?;
    let mut panelists_json = String::new();
    file.read_to_string(&mut panelists_json)?;
    let panelists: Vec<Panelist> = serde_json::from_str(&panelists_json)?;

    for p in panelists {
        println!("{:#?}", p);
    }

    // println!("{esc}c", esc = 27 as char);
    let quit_str = "Q";

    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut user_text = String::new();

        stdin()
            .read_line(&mut user_text)
            .expect("Failed to read line");

        if user_text.to_uppercase().trim().eq(quit_str) {
            println!("Bye");
            break;
        }
        println!("");
        let mut sp = Spinner::new(Spinners::SimpleDots, "\t\tOpen AI is thinking ...".into());
        let oai_request = OAIRequest {
            prompt: format!("{} {}", preamble, user_text),
            max_tokens: 500,
        };

        let body = Body::from(serde_json::to_vec(&oai_request)?);

        let req = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header("Authorization", &auth_header_val)
            .body(body)
            .unwrap();
        // println!(" making request");
        // println!(" OAI header val {}", &auth_header_val);
        let res = client.request(req).await?;
        // println!(" request made res = {}", res.status());
        let body = hyper::body::aggregate(res).await?;

        let json: OAIResponse = serde_json::from_reader(body.reader())?;
        sp.stop();
        // println!("Json {:?}", json);
        println!("{}", json.choices[0].text);
    }
    Ok(())
}
