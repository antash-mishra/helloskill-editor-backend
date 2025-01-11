use  actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, };
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use log::info;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")] // Ensures JSON uses camelCase strings for enum variants
struct CursorPosition {
    column: i32,
    line_number: i32,
}


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")] 
struct EditorState {
    completion_mode: CompletionMode,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")] // Ensures JSON uses lowercase strings for enum variants
enum CompletionMode {
    Insert,
    Complete,
    Continue,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")] 
struct CompletionModeMetadata {
    cursor_position: CursorPosition,
    editor_state: EditorState,
    language: String,
    text_after_cursor: String,
    text_before_cursor: String,  
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestBody {
    completion_metadata: CompletionModeMetadata
}

fn generate_prompt(metadata: RequestBody) -> String {
    format!(
        "Please complete the following {} code:\n\n{}<cursor>\n{}\n\nUse modern {} practices and hooks where appropriate. Please provide only the completed part of the code without additional comments or explanations.",
        metadata.completion_metadata.language,
        metadata.completion_metadata.text_before_cursor,
        metadata.completion_metadata.text_after_cursor,
        metadata.completion_metadata.language
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
    // let body: String = request.parse().unwrap();
    info!("body: {:?}", body.completion_metadata.text_after_cursor);
    io::stdout().flush().unwrap();

    let prompt : &str = "Please complete the following {metadata['language']} code:
{metadata['textBeforeCursor']}
<cursor>
{metadata['textAfterCursor']}

Use modern {metadata['language']} practices and hooks where appropriate. Please provide only the completed part of the
code without additional comments or explanations."

    HttpResponse::Ok().json(body.into_inner())
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
