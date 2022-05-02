use std::ops::Deref;
use std::sync::{Mutex};
use actix_web::{get, post, web, App, HttpServer, HttpResponse, middleware};
use actix_web::web::{Data, Json};
use serde::{Serialize, Deserialize};

// This struct represents state
struct AppState {
    names: Mutex<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct NameRequest {
    pub name: String,
}

#[derive(Serialize, Debug)]
struct NameResponse {
    pub message: String,
}

#[get("/hello/{name}")]
async fn greet(name_path: web::Path<String>, app_state: Data<AppState>) -> HttpResponse {
    let names = app_state.names.lock().unwrap();
    let n = &name_path.into_inner();
    if !names.contains(&n) {
        return HttpResponse::BadRequest().body("Name not in the list");
    }
    HttpResponse::Ok().body(format!("Hello {n}!"))
}

#[get("/names")]
async fn get_names(app_state: Data<AppState>) -> HttpResponse {
    let names = app_state.names.lock().unwrap();
    HttpResponse::Ok().json(names.deref())
}

#[post("/names")]
async fn name(name_request: Json<NameRequest>, app_state: Data<AppState>) -> HttpResponse {
    let mut names = app_state.names.lock().unwrap();
    let n = name_request.into_inner();
    if names.contains(&n.name) {
        return HttpResponse::Conflict().json(NameResponse { message: String::from("Already contains name") });
    };
    names.push(n.name);
    HttpResponse::Created()
        .json(NameResponse { message: String::from(format!("Successfully created.")) })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // Create some global state prior to building the server
    let app_state = Data::new(AppState {
        names: Mutex::new(vec!["Divit"].into_iter().map(|s| s.to_owned()).collect()),
    });
    // move is necessary to give closure below ownership of names
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone()) // add shared state
            // enable logger
            .wrap(middleware::Logger::default())
            // register simple handler
            .service(greet)
            .service(name)
            .service(get_names)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}