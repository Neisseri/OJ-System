use actix_web::{get, middleware::Logger, post, web, App, HttpServer, Responder};
use env_logger;
use log;
use ranking_list::get_contests_ranklist;
use crate::config::get_config;
use crate::judge_task::post_job_api::post_jobs;
use crate::judge_list::get_jobs;
use crate::get_single_job::get_job_id;
use crate::put_jobs_id::put_jobs;
use crate::users_api::{post_users, get_users};

mod config;
mod judge_task;
mod error;
mod global;
mod response;
mod tool;
mod judge_list;
mod get_single_job;
mod put_jobs_id;
mod users_api;
mod ranking_list;
mod post_contest;

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
            .service(get_jobs)
            .service(get_job_id)
            .service(put_jobs)
            .service(post_users)
            .service(get_users)
            .service(get_contests_ranklist)
            .app_data(web::Data::new(config.clone()))

    })
    .bind(("127.0.0.1", 12345))?
    .run()
    .await
}
