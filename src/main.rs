use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::collections::HashSet;
use std::env;
use std::f32::consts::E;
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
static PANELISTS : [Panelist; 8] = [
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
            description: "A Zen Buddist",
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
    if !oai_token.eq(&"None".to_string()) {
        print_header()
    };
    let quit_str = "QUIT";

    loop {
        let mut responders: HashSet<&str> = HashSet::with_capacity(PANELISTS.len());
        let mut user_text = String::new();
        if oai_token.eq(no_token) {
            println!("You need to have a valid OpenAi auth token stored in an environment variable named {} to run this application", oai_token_key);
            println!("see https://openai.com/api/ for details");
            break;
        };
        print!(">");
        stdout().flush().unwrap();
        // let mut user_text = String::new();

        stdin()
            .read_line(&mut user_text)
            .expect("Failed to read line");
        if user_text.trim().to_ascii_uppercase() == quit_str {
            println!("Bye");
            break;
        }

        let question_start = read_tokens(&mut responders, &user_text);
        println!("Question: {}", &user_text[question_start..]);
        for person in responders.iter() {
            let panelist = PANELISTS
                .iter()
                .find(|&x| x.name.to_ascii_lowercase() == person.to_ascii_lowercase())
                .expect("Missing Panelist");
            println!();
            let spinner_str = format!("\t\t {} is thinking", panelist.name);
            let mut sp = Spinner::new(Spinners::SimpleDots, spinner_str);
            let oai_request = OAIRequest {
                prompt: format!("{} {}", panelist.prelude, &user_text[question_start..]),
                max_tokens: 1000,
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
    }
    Ok(())
}

fn print_header() {
    println!("Welcome to our Question and Answer Chat");
    println!(
        "Today we have {} distinguished panelists. They are",
        PANELISTS.len()
    );
    for (i, p) in PANELISTS.iter().enumerate() {
        println!("{}: {}, {} ", i + 1, p.name, p.description);
    }
    println!(
        "Begin your question with numbers or names to ask a specific panelist. Use Quit  or ^C to exit"
    );
    println!("Go ahead, ask us anything ... ");
}
enum CharClass {
    Digit,
    Alphabetic,
    Other,
}
fn char_class(c: u8) -> CharClass {
    match c {
        b'A'..=b'Z' | b'a'..=b'z' => CharClass::Alphabetic,
        b'0'..=b'9' => CharClass::Digit,
        _ => CharClass::Other,
    }
}
fn read_tokens<'a>(which: &mut HashSet<&'a str>, s: &'a str) -> usize {
    let bytes = s.as_bytes();
    let mut reading_name = false;
    let mut reading_number: bool = false;
    let mut j = 0; // start index of the current token
    let mut imax = 0; // where the list of people to ask ends, and thereal question begins.
    let all_str = "ALL";
    which.drain();
    if s.starts_with("Quit") || s.starts_with("QUIT") || s.starts_with("QUIT") {
        return 0;
    };

    'outer: for (i, &item) in bytes.iter().enumerate() {
        match char_class(item) {
            CharClass::Digit if reading_number => {}
            CharClass::Digit if !reading_number && !reading_name => {
                j = i;
                reading_number = true;
            }
            CharClass::Alphabetic if reading_name => {}
            CharClass::Alphabetic if !reading_name && !reading_number => {
                j = i;
                reading_name = true;
            }
            CharClass::Other => {
                if reading_name {
                    if s[j..i].to_uppercase().trim().eq(all_str) {
                        for p in PANELISTS.iter() {
                            which.insert(p.name);
                            imax = i;
                        }
                    }
                    match PANELISTS
                        .iter()
                        .find(|&x| s[j..i].to_ascii_lowercase() == (x.name.to_ascii_lowercase()))
                    {
                        Some(p) => {
                            which.insert(p.name);
                            imax = i;
                        }
                        None => {
                            if which.is_empty() {
                                // No panelist specified. Pick 1 or 2 at random
                                which.insert(
                                    PANELISTS
                                        [rand::thread_rng().gen_range(0..PANELISTS.len()) as usize]
                                        .name,
                                );
                                which.insert(
                                    PANELISTS
                                        [rand::thread_rng().gen_range(0..PANELISTS.len()) as usize]
                                        .name,
                                );
                            }
                            break 'outer;
                        }
                    }
                    reading_name = false;
                };
                if reading_number {
                    match is_panelist_number(&s[j..i]) {
                        Some(n) => {
                            which.insert(PANELISTS[n].name);
                            imax = i;
                            reading_number = false;
                        }
                        None => break,
                    }

                    // println!("found number {} j {} i {} ",&s[j..i],j,i);
                    reading_number = false;
                }
                j = i;
            }

            _ => {}
        }
    }
    imax
}

fn is_panelist_number(s: &str) -> Option<usize> {
    match &s.parse::<usize>() {
        Ok(n) => {
            if n > &0 && n <= &(PANELISTS.len()) {
                Some(n - 1)
            } else {
                None
            }
        }
        Err(_e) => None,
    }
}

#[cfg(test)]
mod fw_tests {
    use crate::{is_panelist_number, read_tokens, PANELISTS};
    use std::collections::HashSet;

    // #[test]
    // fn test1 () {
    //     let hw = "    Hello World";
    //     println!( "first word is {}", super::first_word(&hw));
    //     assert_eq!(super::first_word(&hw),"Hello");

    // }
    #[test]
    fn test2() {
        let mut a: HashSet<&str> = vec!["Rachel", "Tucker", "Rusty"].into_iter().collect();
        let test_inputs = vec![
            "Rachel 1 3 alan what does it all mean",
            "rachel tucker 5 This is the question",
            "Quit",
            " all what do you tink of global warming?",
            "Why is the sky blue?",
        ];
        //  println!("test_tokens {} ", test_tokens);
        let mut who: HashSet<&str> = HashSet::with_capacity(PANELISTS.len());

        let mut question_start = 0;
        for input in test_inputs {
            println!("input:{input}");
            question_start = read_tokens(&mut who, input);
            println!(" Question: {}", &input[question_start..]);
            for x in who.iter() {
                println!("{x}");
            }
        }
        let r = PANELISTS
            .iter()
            .find(|&x| x.name.to_ascii_lowercase() == "Rachel".to_ascii_lowercase())
            .unwrap();
        println!(" found panelist {:?}", r);

        question_start = read_tokens(
            &mut who,
            " tucker 8 rachel who won the 2020 presidential election?",
        );
        assert!(a == who);
        who.drain();

        question_start = read_tokens(&mut who, " 7 who won the 2020 presidential election?");
        assert!(who == vec!["Alan"].into_iter().collect());
        assert_eq!(is_panelist_number("9"), None);
        assert_eq!(is_panelist_number("8"), Some(7));
        assert_eq!(is_panelist_number("1"), Some(0));
        assert_eq!(is_panelist_number("2"), Some(1));
    }
}
