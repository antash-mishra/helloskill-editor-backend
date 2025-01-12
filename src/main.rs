use std::io::{self, Write};
use dotenv::dotenv;
use log::info;
use  actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, };
use actix_cors::Cors;
use openai::{chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}, Credentials, OpenAiError};
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // Ensures JSON uses camelCase strings for enum variants
struct CursorPosition {
    column: i32,
    line_number: i32,
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] 
struct EditorState {
    completion_mode: CompletionMode,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")] // Ensures JSON uses lowercase strings for enum variants
enum CompletionMode {
    Insert,
    Complete,
    Continue,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] 
struct CompletionModeMetadata {
    cursor_position: CursorPosition,
    editor_state: EditorState,
    language: String,
    text_after_cursor: String,
    text_before_cursor: String,  
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RequestBody {
    completion_metadata: CompletionModeMetadata
}

fn generate_user_prompt(metadata: CompletionModeMetadata) -> String {
    format!(
        "Please complete the following {} code:\n\n{}<cursor>\n{}\n\nUse modern {} practices and hooks where appropriate. Please provide only the completed part of the code without additional comments or explanations.",
        metadata.language,
        metadata.text_before_cursor,
        metadata.text_after_cursor,
        metadata.language
    )
}

#[get("/")]
async fn hello() -> impl Responder {
    print!("HI");   
    HttpResponse::Ok().body("Hello, world!")
}

// Implementing function to code completion
#[post("/complete")]
async fn handle_complete(body: web::Json<RequestBody>) -> impl Responder {
    dotenv().ok();
    
    // let body: String = request.parse().unwrap();
    info!("body: {:?}", body.completion_metadata.text_after_cursor);
    io::stdout().flush().unwrap();

    let system_prompt = format!(
        "You are an expert code completion assistant.\n\n**Context**\nLanguage: {}.",
        body.completion_metadata.language,
    );

    let user_prompt: String = generate_user_prompt(body.completion_metadata.clone());

    let messages: Vec<ChatCompletionMessage> = vec![
        ChatCompletionMessage{
            role: ChatCompletionMessageRole::System,
            content: Some(system_prompt),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: vec![],
        },
        ChatCompletionMessage{
            role: ChatCompletionMessageRole::User,
            content: Some(user_prompt),
            name: None,
            function_call:None,
            tool_call_id: None,
            tool_calls: vec![],
        },
    ];

    let credentials = Credentials::from_env();


    println!("Credentials {:?}", credentials);
    

    let chat_completion = ChatCompletion::builder("gpt-4o-mini", messages.clone())
        .credentials(credentials.clone())
        .create()
        .await
        .unwrap();
    
    let returned_message = chat_completion.choices.first().unwrap().message.clone();
    // println!("Credentials {:#?}", credentials);
    
    println!(
        "{:#?} {:#?}: {}",
        returned_message,
        returned_message.role,
        returned_message.content.clone().unwrap().trim()
    );

    HttpResponse::Ok().json(json!({
        "completion": returned_message.content.clone().unwrap().trim(),
    }))
    
}


async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Initialize logger
    println!("Hello, world!");
    
    HttpServer::new(|| {
        let cors = Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

        App::new()
           .wrap(cors)
           .service(hello)
           .route("/hey", web::get().to(manual_hello))
           .service(handle_complete)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
