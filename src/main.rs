use std::{fmt::format, io::{self, Write}};
use dotenv::dotenv;
use log::info;
use  actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, };
use actix_cors::Cors;
use openai::{chat::{ChatCompletion, AnthropicChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}, Credentials, OpenAiError};
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
struct HintRequestBody {
    question: String,
    language: String,
    code: String,
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

fn generate_code_hint_user_prompt(metadata: HintRequestBody) -> String {
    format!("Here's the coding question I am working on:\n<coding_question>\n{}\n</coding_question>\n\nNow, here's the code I have written so far:\n<user_code>\n{}\n</user_code>",
        metadata.question,
        metadata.code,
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


#[post("/suggestion")]
async fn handle_suggestion(body: web::Json<HintRequestBody>) -> impl Responder {
    dotenv().ok();
    
    // let body: String = request.parse().unwrap();
    info!("body: {:?}", body.language);
    io::stdout().flush().unwrap();

    let system_prompt = format!(
        "You are an experienced and patient coding tutor. Your task is to help a student with their coding question and provide guidance on their code. You will analyze the code they've written, offer hints to help them solve the problem, and check for any errors. Remember to be encouraging and supportive throughout the process.\n\nPlease follow these steps to assist the student:\n\n1. Carefully read and understand both the coding question and the student's code.\n\n2. Analyze the code:\n   - Identify what parts of the problem the student has successfully addressed.\n   - Determine where the student might be stuck or what concepts they might be struggling with.\n   - Look for any logical errors or misunderstandings in their approach.\n\n3. Provide hints:\n   - If the student is stuck, offer a hint that guides them towards the next step without giving away the full solution.\n   - If they're on the right track but missing something, provide a gentle nudge in the right direction.\n   - If they've misunderstood a concept, explain it briefly and provide a small example if necessary.\n\n4. Check for errors:\n   - Identify any syntax errors in the code.\n   - Look for logical errors or edge cases the student might have missed.\n   - If the code is complete, test it mentally with different inputs to ensure it works as expected.\n\n5. Format your response as follows:\n\n   <hints>\n   Offer 1-3 hints to guide the student, depending on how much help they need.\n   </hints>\n\n   <errors>\n   List any syntax or logical errors you've identified. If there are no errors, state that the code looks good so far.\n   </errors>\n\nRemember to maintain a supportive and patient tone throughout your response. Your goal is to guide the student towards solving the problem on their own, not to solve it for them.\n\nIf the student has provided a complete solution, focus on error checking and optimization suggestions rather than hints for solving the problem.\n\nBegin your analysis now, and provide your response using the format specified above and output shoudl be user friendly and add some infographics."
    );

    let user_prompt: String = generate_code_hint_user_prompt(body.clone());

    let messages: Vec<ChatCompletionMessage> = vec![
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
    
    let chat_completion = AnthropicChatCompletion::builder("claude-3-5-sonnet-20241022", &system_prompt,messages.clone())
        .credentials(credentials.clone())
        .create()
        .await
        .unwrap();
    
    println!("Chat COmpletion: {:?}", chat_completion);

    let returned_message = chat_completion.content.first().unwrap().clone();
    // println!("Credentials {:#?}", credentials);
    
    println!(
        "{:#?}: {}",
        returned_message.typ,
        returned_message.text.clone().trim()
    );

    HttpResponse::Ok().json(json!({
        "completion": returned_message.text.clone().trim(),
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
           .service(handle_suggestion)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
