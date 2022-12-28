use actix_web::{web, App, HttpServer};
use sqlx::{mysql::{MySqlPool}, Pool, MySql};
use actix_web::middleware::Logger;
use env_logger::Env;
use std::fs;

mod controllers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = setup_db().await;
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(pool.clone()))
            .service(controllers::activities::activity_index)
            .service(controllers::activities::activity_find)
            .service(controllers::activities::activity_create)
            .service(controllers::activities::activity_delete)
            .service(controllers::activities::activity_edit)
            .service(controllers::todos::todo_index)
            .service(controllers::todos::todo_find)
            .service(controllers::todos::todo_create)
            .service(controllers::todos::todo_edit)
            .service(controllers::todos::todo_delete)
    })
    .bind(("127.0.0.1", 3030))?
    .run()
    .await
}

async fn setup_db() -> Pool<MySql> {
    let host = std::env::var("MYSQL_HOST").expect("No host");
    let user = std::env::var("MYSQL_USER").expect("No user");
    let pass = std::env::var("MYSQL_PASSWORD").expect("No pass");
    let db = std::env::var("MYSQL_DBNAME").expect("No dbname");

    let activities_query = fs::read_to_string("/usr/local/bin/schema/activities.sql")
        .expect("File Not Found");
    let todos_query = fs::read_to_string("/usr/local/bin/schema/todos.sql")
        .expect("File Not Found");

    let pool = MySqlPool::connect(
            &format!("mysql://{user}:{pass}@{host}/{db}")
        )
        .await
        .expect("Failed to connect to db");

    sqlx::query(&activities_query).execute(&pool).await.expect("Error");
    sqlx::query(&todos_query).execute(&pool).await.expect("Error");
    
    pool
}
