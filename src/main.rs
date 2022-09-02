use actix_web::{get, middleware::Logger, post, web, App, HttpServer, Responder};
use env_logger;
use log;
use crate::config::get_config;
use crate::judge_task::post_job_api::post_jobs;

mod config;
mod judge_task;
mod error;
mod global;
mod response;
mod tool;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    log::info!(target: "greet_handler", "Greeting {}", name);
    format!("Hello {name}!")
}

// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = get_config();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(greet)
            // DO NOT REMOVE: used in automatic testing
            .service(exit)
            .service(post_jobs)
            .app_data(web::Data::new(config.clone()))

    })
    .bind(("127.0.0.1", 12345))?
    .run()
    .await
}
