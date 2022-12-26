use actix_web::{get, post, delete, HttpResponse, Responder, web, patch};
use sqlx::mysql::{MySql};
use sqlx::Pool;
use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::FromRow, Serialize)]
struct Todo {
    id: i32,
    activity_group_id: i32,
    title: String,
    is_active: String,
    priority: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
struct ResponseResult<T> {
    message: String,
    status: String,
    data: Option<T>,
}

#[derive(Serialize)]
struct ResponseFail {
    message: String,
    status: String,
}

#[derive(Deserialize)]
pub struct TodoAllQueries {
    activity_group_id: Option<i32>,
}

#[get("/todo-items")]
pub async fn todo_index(pool: web::Data<Pool<MySql>>, query: web::Query<TodoAllQueries>) -> impl Responder {
    let pool_ref = pool.get_ref();
    let result = match query.activity_group_id {
        None => sqlx::query_as::<_, Todo>("select * from todos")
            .fetch_all(pool_ref)
            .await,
        Some(group_id) => sqlx::query_as::<_, Todo>("select * from todos where activity_group_id = ?")
            .bind(group_id)
            .fetch_all(pool_ref)
            .await
    };
    match result {
        Ok(results) => {
            let response = ResponseResult {
                status: "Success".to_string(),
                message: "Success".to_string(),
                data: Some(results),
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            let response = ResponseResult {
                status: "Error".to_string(),
                message: e.to_string(),
                data: Some(Vec::<Todo>::new()),
            };
            HttpResponse::InternalServerError().json(response)
        }
    }
}

#[get("/todo-items/{id}")]
pub async fn todo_find(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();
    let pool_ref = pool.get_ref();
    let result = sqlx::query_as::<_, Todo>("select * from todos where id = ?")
        .bind(id)
        .fetch_one(pool_ref)
        .await;
    match result {
        Ok(results) => {
            let response = ResponseResult {
                status: "Success".to_string(),
                message: "Success".to_string(),
                data: Some(results),
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            let response: ResponseResult<_> = ResponseResult::<Todo> {
                status: "Error".to_string(),
                message: e.to_string(),
                data: None,
            };
            HttpResponse::InternalServerError().json(response)
        }
    }
}

#[derive(Deserialize)]
pub struct TodoCreatePayload {
    activity_group_id: i32,
    title: String,
} 

#[post("/todo-items")]
pub async fn todo_create(
    payload: web::Json<TodoCreatePayload>,
    pool: web::Data<Pool<MySql>>
) -> impl Responder {
    let pool_ref = pool.get_ref();
    let result = sqlx::query("insert into todos(activity_group_id, title, is_active, priority) values(?, ?, ?, ?)")
        .bind(&payload.activity_group_id)
        .bind(&payload.title)
        .bind("1")
        .bind("very-high")
        .execute(pool_ref)
        .await;
    match result {
        Ok(p) => {
            let last_insert_id = p.last_insert_id();
            let todo = sqlx::query_as::<_, Todo>(
                "SELECT * FROM todos WHERE id = ?",
            )
                .bind(last_insert_id)
                .fetch_one(pool_ref)
                .await
                .expect("Failed to select activity");
            let res = ResponseResult {
                status: "Success".to_string(),
                message: "Success".to_string(),
                data: Some(todo),
            };
            HttpResponse::Ok().json(res)
        },
        Err(e) => {
            let res = ResponseFail {
                status: "Failed".to_string(),
                message: e.to_string(),
            };
            HttpResponse::Ok().json(res)
        }
    }
}

#[derive(Deserialize)]
pub struct EditTodoPayload {
    title: String,
    is_active: Option<String>,
}

#[patch("/todo-items/{id}")]
pub async fn todo_edit(
    path: web::Path<i32>,
    pool: web::Data<Pool<MySql>>,
    payload: web::Json<EditTodoPayload>
) -> impl Responder {
    let id = path.into_inner();
    let pool_ref = pool.get_ref();
    let result = match &payload.is_active {
        Some(active) => {
            sqlx::query("UPDATE todos SET title = ?, is_active = ? WHERE id = ?")
                .bind(&payload.title)
                .bind(active)
                .bind(id)
                .execute(pool_ref)
                .await
        },
        None => {
            sqlx::query("UPDATE todos SET title = ? WHERE id = ?")
                .bind(&payload.title)
                .bind(id)
                .execute(pool_ref)
                .await
        }
    };
    match result {
        Ok(_) => {
            let todo = sqlx::query_as::<_, Todo>(
                "SELECT * FROM todos WHERE id = ?",
            )
                .bind(id)
                .fetch_one(pool_ref)
                .await
                .expect("Failed to select todo");
            let res = ResponseResult {
                status: "Success".to_string(),
                message: "Success".to_string(),
                data: Some(todo),
            };
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            let res = ResponseFail {
                status: "Failed".to_string(),
                message: e.to_string(),
            };
            HttpResponse::Ok().json(res)
        }
    }
}

#[delete("/todo-items/{id}")]
pub async fn todo_delete(
    path: web::Path<i32>,
    pool: web::Data<Pool<MySql>>
) -> impl Responder {
    let id = path.into_inner();
    let pool_ref = pool.get_ref();
    let result = sqlx::query("delete from todos where id = ?")
        .bind(id)
        .execute(pool_ref)
        .await;
    match result {
        Ok(_) => {
            let res: ResponseResult<_> = ResponseResult::<Todo> {
                status: "Success".to_string(),
                message: "Success".to_string(),
                data: None,
            };
            HttpResponse::Ok().json(res)
        },
        Err(e) => {
            let res = ResponseFail {
                status: "Failed".to_string(),
                message: e.to_string(),
            };
            HttpResponse::Ok().json(res)
        }
    }
}
