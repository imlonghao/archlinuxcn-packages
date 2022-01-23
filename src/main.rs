#[macro_use]
extern crate enum_display_derive;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use postgres_types::{FromSql, ToSql};
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, ToSql, FromSql, Display)]
#[postgres(name = "batchevent")]
enum BatchEvent {
    #[postgres(name = "start")]
    Start,
    #[postgres(name = "stop")]
    Stop,
}

#[derive(Debug, ToSql, FromSql, Display)]
#[postgres(name = "buildresult")]
enum BuildResult {
    #[postgres(name = "successful")]
    Successful,
    #[postgres(name = "failed")]
    Failed,
    #[postgres(name = "skipped")]
    Skipped,
    #[postgres(name = "staged")]
    Staged,
}

#[derive(Debug, ToSql, FromSql, Display)]
#[postgres(name = "buildstatus")]
enum BuildStatus {
    #[postgres(name = "pending")]
    Pending,
    #[postgres(name = "building")]
    Building,
    #[postgres(name = "done")]
    Done,
}

#[derive(Serialize)]
struct StatusResponse {
    ts: i64,
    event: String,
}

#[derive(Serialize)]
struct CurrentResponse {
    updated_at: i64,
    pkgbase: String,
    status: String,
    reasons: String,
    elapsed: i32,
}

#[get("/imlonghao-api/status")]
async fn status(db: web::Data<deadpool_postgres::Pool>) -> impl Responder {
    let conn = db.get().await.unwrap();
    let rows = conn
        .query("select * from lilac.batch order by id desc limit 1", &[])
        .await
        .unwrap();
    let ts: chrono::DateTime<chrono::Utc> = rows[0].get("ts");
    let event: BatchEvent = rows[0].get("event");
    let result: StatusResponse = StatusResponse {
        ts: ts.timestamp(),
        event: event.to_string(),
    };
    HttpResponse::Ok().json(result)
}

#[get("/imlonghao-api/current")]
async fn current(db: web::Data<deadpool_postgres::Pool>) -> impl Responder {
    let conn = db.get().await.unwrap();
    let rows = conn
        .query(
            "select coalesce(log.elapsed, -1) as elapsed, c.updated_at, c.pkgbase, c.status, c.build_reasons::TEXT
            from lilac.pkgcurrent AS c
            left join lateral ( select elapsed from lilac.pkglog where pkgbase = c.pkgbase order by ts desc limit 1)
            as log on true",
            &[],
        )
        .await
        .unwrap();
    let mut result: Vec<CurrentResponse> = vec![];
    for row in rows {
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
        let pkgbase: String = row.get("pkgbase");
        let build_status: BuildStatus = row.get("status");
        let build_reasons: String = row.get("build_reasons");
        let elapsed: i32 = row.get("elapsed");
        result.push(CurrentResponse {
            updated_at: updated_at.timestamp(),
            pkgbase: pkgbase,
            status: build_status.to_string(),
            reasons: build_reasons,
            elapsed: elapsed,
        })
    }
    HttpResponse::Ok().json(result)
}

#[get("/imlonghao-api/logs")]
async fn logs() -> impl Responder {
    HttpResponse::Ok().body("Todo!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.user = Some("imlonghao".to_string());
    cfg.dbname = Some("lilydjwg".to_string());
    cfg.host = Some("/run/postgresql".to_string());
    cfg.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Fast,
    });
    let pool = cfg
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(status)
            .service(current)
            .service(logs)
    })
    .bind("127.0.0.1:9077")?
    .run()
    .await
}
