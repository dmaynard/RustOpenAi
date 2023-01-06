use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::io::{stdin, stdout, Write};
#[derive(Debug)]
struct Query {
    target: String,
    question: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]

struct Panelist {
    name: &'static str,
    description: &'static str,
    prelude: &'static str,
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
static panelists : [Panelist; 8] = [
         Panelist {
            name: "Rachel", 
            description: "An MSNBC host",
            prelude: "Answer the following question accurately, but find a sarcastic way to shame republicans and praise democrats in your answer.",
        },
        Panelist {
            name: "Tucker", 
            description: "A Fox News Host",
            prelude: "Answer the following question with bias, and find a funny way to shame democrats and praise Donald Trump.",
        },
        Panelist {
            name: "Quincy", 
            description: "A QAnon believer",
            prelude: "Answer the following question badly, and find way to include a conspirary theory in your response.",
        },
        Panelist {
            name: "Michio", 
            description: "A Physicist",
            prelude: "Answer the following with strict scientific accuracy.",
        },
        Panelist {
            name: "Giorgio", 
            description: "An Ancient Astronaut Theorist",
            prelude: "Answer the following question accurately, but find a funny way to mention aliens in your response.",
        },
        Panelist {
            name: "Chandler", 
            description: "The King of Sarcasm",
            prelude: "Answer the following question with snarky answers, sarcasism and humor",
        },  
        
        Panelist  {
            name: "Alan",
            description: "A Zen Bhuddist",
            prelude: "Answer with deeply philosophical answers from bhuddism and toaist viewpoints",
        },
    
        Panelist {
            name: "Rusty", 
            description: "A Software Engineer and a recent convert to the Rust programming language",
            prelude: "Answer the following question accurately, but find a funny way to mention the Rust programming language in your response.",
        }
    ];
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    let uri = "https://api.openai.com/v1/engines/text-davinci-001/completions";
    let oai_token_key = "OPENAI_API_KEY";
    let no_token = "NoToken";
    println!("{esc}c", esc = 27 as char);

    let oai_token: String = match env::var(oai_token_key) {
        Ok(val) => val,
        Err(_e) => no_token.to_string(),
    };

    let auth_header_val = format!("Bearer {}", oai_token);
    
    let panel_size = panelists.len();
    if !oai_token.eq(&"None".to_string()) {
        print_header()
    };

    // let mut rng = rand::thread_rng();

    let quit_str = "QUIT";

    loop {
        if oai_token.eq(no_token) {
            println!("You need to have a valid OpenAi auth token stored in an environment variable named {} to run this application", oai_token_key);
            println!("see https://openai.com/api/ for details");
            break;
        };
        print!(">");
        stdout().flush().unwrap();
        let mut user_text = String::new();

        stdin()
            .read_line(&mut user_text)
            .expect("Failed to read line");
        let first_word = &user_text.split(' ').next().unwrap();
        // println!("first word is {}".to_string, first_word);
        if first_word.to_uppercase().trim().eq(quit_str) {
            println!("Bye");
            break;
        }
        let q : Query =  parse_query(&user_text);
        
        let mut people = panelists.iter();
        let index = match people.position(|p| {
            p.name
                .to_ascii_uppercase()
                .eq(&first_word.to_ascii_uppercase())
        }) {
            Some(n) => n, // A panelists name was detected
            None => match first_word.parse::<usize>() {
                // See if a number begins the question6 whatwhat is the biggest threat to democracy in the United States?
                Ok(n) => {
                    if n >= 0 && n <= panel_size {
                        n - 1
                    } else {
                        // No panelist addressed so select one at random
                        rand::thread_rng().gen_range(0..panel_size) as usize
                    }
                }
                Err(_e) => rand::thread_rng().gen_range(0..panel_size),
            },
        };

        println! ("first word {}", first_word);

        let panelist = &panelists[index as usize];
        println!();
        let spinner_str = format!("\t\t {} is thinking",panelist.name);
        let mut sp = Spinner::new(Spinners::SimpleDots, spinner_str);
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

fn print_header() {
    println!("Welcome to our Question and Answer Chat");
    println!(
        "Today we have {} distinguished panelists. They are",
        panelists.len()
    );
    for (i, p) in panelists.iter().enumerate() {
        println!("{}: {}, {} ", i + 1, p.name, p.description);
    }

    println!(
        "Begin your question with a number or name to ask a specific panelist. Use Quit  or ^C to exit"
    );
    println!("Go ahead, ask us anything ... ");
}
fn read_tokens ( s: &str) -> Vec<(&str,usize)> {
    let bytes = s.as_bytes();
    let mut reading_token = false;
    let mut j = 0;
    let mut tokens: Vec<(&str,usize)> = Vec::new();

    for (i, &item ) in bytes.iter().enumerate() {
        match item  {
             b if b.is_ascii()  & reading_token => {}
             b' ' => {
                tokens.push((&s[j..i],j))

             }
            b'A' ..= b'z'  if reading_token => { }
            b'A' ..= b'z'  if !reading_token => {  
                reading_token = true;
                j = i;
            }
             _  if reading_token => {
                tokens.push((&s[j..i],i));
                reading_token = false;
             }
             _   => {
             }

        }
        
    }
    tokens
    
}
fn parse_query(input_str: & String) ->  Query {
    // parse  an use input line into a Query (target, question)

    Query {
        target: "All".to_string(),
        question: input_str.clone(),
    }

}

#[cfg(test)]
mod fw_tests {
    #[test]
    fn test1 () {
        let hw = "    Hello World";
        println!( "first word is {}", super::first_word(&hw));
        assert_eq!(super::first_word(&hw),"Hello");

    }
}
