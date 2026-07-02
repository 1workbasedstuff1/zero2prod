use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, BufReader};

use reqwest;
use zero2prod::configuration::get_configuration;

#[tokio::main]
async fn main() {
    let database = get_configuration().expect("Couldnt parse settings");
    let port = format!("http://127.0.0.1:{}", database.application.port);

    println!("please enter name");

    // NOTE use tokio async io
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut name = String::new();
    // read in, not blocking anymore
    reader
        .read_line(&mut name)
        .await
        .expect("Failed reading line");

    let url_name = name.replace(" ", "%20");

    println!("please enter email address");
    let mut email = String::new();
    reader
        .read_line(&mut email)
        .await
        .expect("Failed reading line");
    let url_email = email.replace("@", "%40");

    let url_form = format!("name={}&email={}", url_name, url_email);

    let html_form_input = "name=li%20dicky&email=lid%40gmail.com";

    // send HTTP POST to subscriptions
    let sub_response = reqwest::Client::new()
        .post(&format!("{}/subscriptions", port))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(url_form)
        .send()
        .await
        .expect("Failed to execute request");
}
