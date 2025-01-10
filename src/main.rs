use  actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

// Implementing function to code completion
#[post("/complete")]
async fn handle_complete(request: String) -> impl Responder {
    
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    HttpServer::new(|| {
        App::new()
           .service(hello)
           .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
