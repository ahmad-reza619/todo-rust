use actix_web::{web, App, HttpServer};
use sqlx::{mysql::{MySqlPool}, Pool, MySql};
use std::fs;

mod controllers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = setup_db().await;

    HttpServer::new(move || {
        App::new()
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
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}

async fn setup_db() -> Pool<MySql> {
    let host = env!("MYSQL_HOST");
    let user = env!("MYSQL_USER");
    let pass = env!("MYSQL_PASSWORD");
    let db = env!("MYSQL_DBNAME");

    let activities_query = fs::read_to_string("src/schema/activities.sql")
        .expect("File Not Found");
    let todos_query = fs::read_to_string("src/schema/todos.sql")
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
