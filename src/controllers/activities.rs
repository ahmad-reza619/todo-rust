use actix_web::{get, post, delete, HttpResponse, Responder, web, patch};
use sqlx::mysql::{MySql};
use sqlx::Pool;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, sqlx::FromRow, Serialize)]
struct Activity {
    id: i32,
    title: String,
    email: String,
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

#[get("/activity-groups")]
pub async fn activity_index(pool: web::Data<Pool<MySql>>) -> impl Responder {
    let pool_ref = pool.get_ref();
    let result = sqlx::query_as::<_, Activity>("select * from activities")
        .fetch_all(pool_ref)
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
            let response = ResponseResult {
                status: "Error".to_string(),
                message: e.to_string(),
                data: Some(Vec::<Activity>::new()),
            };
            HttpResponse::InternalServerError().json(response)
        }
    }
}

#[get("/activity-groups/{id}")]
pub async fn activity_find(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();
    let pool_ref = pool.get_ref();
    let result = sqlx::query_as::<_, Activity>("select * from activities where id = ?")
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
            match e {
               sqlx::Error::RowNotFound => {
                    let response: ResponseResult<_> = ResponseResult::<Activity> {
                        status: "Not Found".to_string(),
                        message: format!("Activity with ID {} Not Found", &id).to_string(),
                        data: None,
                    };
                    HttpResponse::NotFound().json(response)
               }
               _ => {
                    let response: ResponseResult<_> = ResponseResult::<Activity> {
                        status: "Error".to_string(),
                        message: e.to_string(),
                        data: None,
                    };
                    HttpResponse::InternalServerError().json(response)
               }
            }
        }
    }
}

#[derive(Deserialize, Validate, Debug)]
pub struct ActivitityCreatePayload {
    #[validate(required)]
    title: Option<String>,
    email: String,
} 

#[post("/activity-groups")]
pub async fn activity_create(
    payload: web::Json<ActivitityCreatePayload>,
    pool: web::Data<Pool<MySql>>
) -> impl Responder {
    let pool_ref = pool.get_ref();
    let is_valid = payload.validate();
    match is_valid {
        Ok(_) => {
            let result = sqlx::query("insert into activities(email, title) values(?, ?)")
                .bind(&payload.email)
                .bind(&payload.title)
                .execute(pool_ref)
                .await;
            match result {
                Ok(p) => {
                    let last_insert_id = p.last_insert_id();
                    let activity = sqlx::query_as::<_, Activity>(
                        "SELECT * FROM activities WHERE id = ?",
                    )
                        .bind(last_insert_id)
                        .fetch_one(pool_ref)
                        .await
                        .expect("Failed to select activity");
                    let res = ResponseResult {
                        status: "Success".to_string(),
                        message: "Success".to_string(),
                        data: Some(activity),
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
        },
        Err(_) => {
            let response: ResponseResult<_> = ResponseResult::<Activity> {
                status: "Bad Request".to_string(),
                message: "title cannot be null".to_string(),
                data: None,
            };
            HttpResponse::BadRequest().json(response)
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct EditPayload {
    #[validate(required)]
    title: Option<String>,
}

#[patch("/activity-groups/{id}")]
pub async fn activity_edit(
    path: web::Path<i32>,
    pool: web::Data<Pool<MySql>>,
    payload: web::Json<EditPayload>,
) -> impl Responder {
    let id = path.into_inner();
    let pool_ref = pool.get_ref();
    let existing_activity = sqlx::query("SELECT * from activities where id = ?")
        .bind(id)
        .fetch_optional(pool_ref)
        .await
        .unwrap();
    if existing_activity.is_none() {
        let response: ResponseResult<_> = ResponseResult::<Activity> {
            status: "Not Found".to_string(),
            message: format!("Activity with id {} cannot be found", id).to_string(),
            data: None,
        };
        return HttpResponse::NotFound().json(response);
    }
    let validation = payload.validate();
    if validation.is_err() {
        let response: ResponseResult<_> = ResponseResult::<Activity> {
            status: "Bad Request".to_string(),
            message: "title cannot be null".to_string(),
            data: None,
        };
        return HttpResponse::NotFound().json(response);
    }
    let result = sqlx::query("UPDATE activities SET title = ? WHERE id = ?")
        .bind(&payload.title)
        .bind(id)
        .execute(pool_ref)
        .await;
    match result {
        Ok(_) => {
            let activity = sqlx::query_as::<_, Activity>(
                "SELECT * FROM activities WHERE id = ?",
            )
                .bind(id)
                .fetch_one(pool_ref)
                .await
                .expect("Failed to select activity");
            let res = ResponseResult {
                status: "Success".to_string(),
                message: "Success".to_string(),
                data: Some(activity),
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

#[delete("/activity-groups/{id}")]
pub async fn activity_delete(
    path: web::Path<i32>,
    pool: web::Data<Pool<MySql>>
) -> impl Responder {
    let id = path.into_inner();
    let pool_ref = pool.get_ref();
    match sqlx::query("select * from activities where id = ?")
        .bind(id)
        .fetch_one(pool_ref)
        .await {
            Ok(_) => {
                let result = sqlx::query("delete from activities where id = ?")
                    .bind(id)
                    .execute(pool_ref)
                    .await;
                match result {
                    Ok(_) => {
                        let res: ResponseResult<_> = ResponseResult::<Activity> {
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
            },
            Err(err) => match err {
                sqlx::Error::RowNotFound => {
                    let response: ResponseResult<_> = ResponseResult::<Activity> {
                        status: "Not Found".to_string(),
                        message: format!("Activity with ID {} Not Found", &id).to_string(),
                        data: None,
                    };
                    HttpResponse::NotFound().json(response)
                }
                _ => {
                    let response: ResponseResult<_> = ResponseResult::<Activity> {
                        status: "Internal Server Error".to_string(),
                        message: "Interval Server Error".to_string(),
                        data: None,
                    };
                    HttpResponse::InternalServerError().json(response)
                }
            }
    }
}
