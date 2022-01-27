#[macro_use]
extern crate enum_display_derive;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{Context, Result};
use bigdecimal::ToPrimitive;
use cached::proc_macro::cached;
use postgres_types::{FromSql, ToSql};
use serde::Serialize;
use std::fmt::Display;
use yaml_rust::YamlLoader;

const STYLE_HTML: &'static str = r#"<style>
code {
    font-family: "SFMono-Regular", Monaco, Menlo, Consolas, "Liberation Mono", monospace;
    font-size: 12px;
    line-height: 20px;
    white-space: pre-wrap;
}
</style>"#;

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

#[derive(Serialize)]
struct LogsResponse {
    ts: i64,
    pkgbase: String,
    pkg_version: String,
    maintainer: String,
    elapsed: i32,
    result: String,
    cpu: i32,
    memory: f64,
}

#[cached(time = 86400, result = true)]
fn get_maintainer(pkg: String) -> Result<String> {
    let contents = std::fs::read_to_string(format!(
        "/data/archgitrepo-webhook/archlinuxcn/{}/lilac.yaml",
        pkg
    ))?;
    let docs = YamlLoader::load_from_str(&contents)?;
    let doc = &docs[0];
    let maintainers_yaml = doc["maintainers"]
        .as_vec()
        .context("maintainers is empty")?;
    let mut maintainers: Vec<&str> = vec![];
    for m in maintainers_yaml {
        maintainers.push(m["github"].as_str().unwrap_or("None"));
    }
    Ok(maintainers.join(", "))
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
async fn logs(db: web::Data<deadpool_postgres::Pool>) -> impl Responder {
    let conn = db.get().await.unwrap();
    let rows = conn
        .query(
            "select ts, pkgbase, COALESCE(pkg_version, '') AS pkg_version, elapsed, result, COALESCE(case when elapsed = 0 then 0 else cputime * 100 / elapsed end, -1) AS cpu, COALESCE(memory / 1073741824.0, -1) AS memory from  (
                select *, row_number() over (partition by pkgbase order by ts desc) as k
                from lilac.pkglog
            ) as w where k = 1 order by ts desc",
            &[],
        )
        .await
        .unwrap();
    let mut results: Vec<LogsResponse> = vec![];
    for row in rows {
        let ts: chrono::DateTime<chrono::Utc> = row.get("ts");
        let pkgbase: String = row.get("pkgbase");
        let pkg_version: String = row.get("pkg_version");
        let maintainer = get_maintainer(pkgbase.clone()).unwrap_or("Unknown".to_string());
        let elapsed: i32 = row.get("elapsed");
        let result: BuildResult = row.get("result");
        let cpu: i32 = row.get("cpu");
        let memory: pg_bigdecimal::PgNumeric = row.get("memory");
        let memory_bd: bigdecimal::BigDecimal = memory.n.unwrap();
        results.push(LogsResponse {
            ts: ts.timestamp(),
            pkgbase: pkgbase,
            pkg_version: pkg_version,
            maintainer: maintainer,
            elapsed: elapsed,
            result: result.to_string(),
            cpu: cpu,
            memory: memory_bd.to_f64().unwrap(),
        })
    }
    HttpResponse::Ok().json(results)
}

#[get("/imlonghao-api/pkg/{name}")]
async fn get_pkg(
    name: web::Path<String>,
    db: web::Data<deadpool_postgres::Pool>,
) -> impl Responder {
    let conn = db.get().await.unwrap();
    let rows = conn
        .query(
            "select ts, pkgbase, COALESCE(pkg_version, '') AS pkg_version, elapsed, result, COALESCE(case when elapsed = 0 then 0 else cputime * 100 / elapsed end, -1) AS cpu, COALESCE(memory / 1073741824.0, -1) AS memory from lilac.pkglog WHERE pkgbase=$1 order by id desc",
            &[&name.to_string()],
        )
        .await
        .unwrap();
    let mut results: Vec<LogsResponse> = vec![];
    for row in rows {
        let ts: chrono::DateTime<chrono::Utc> = row.get("ts");
        let pkgbase: String = row.get("pkgbase");
        let pkg_version: String = row.get("pkg_version");
        let maintainer = get_maintainer(pkgbase.clone()).unwrap_or("Unknown".to_string());
        let elapsed: i32 = row.get("elapsed");
        let result: BuildResult = row.get("result");
        let cpu: i32 = row.get("cpu");
        let memory: pg_bigdecimal::PgNumeric = row.get("memory");
        let memory_bd: bigdecimal::BigDecimal = memory.n.unwrap();
        results.push(LogsResponse {
            ts: ts.timestamp(),
            pkgbase: pkgbase,
            pkg_version: pkg_version,
            maintainer: maintainer,
            elapsed: elapsed,
            result: result.to_string(),
            cpu: cpu,
            memory: memory_bd.to_f64().unwrap(),
        })
    }
    HttpResponse::Ok().json(results)
}

#[get("/imlonghao-api/pkg/{name}/log/{ts}")]
async fn get_pkg_log(
    path: web::Path<(String, i64)>,
    db: web::Data<deadpool_postgres::Pool>,
) -> impl Responder {
    let (name, ts) = path.into_inner();
    let dt = chrono::DateTime::<chrono::Utc>::from_utc(
        chrono::naive::NaiveDateTime::from_timestamp(ts, 0),
        chrono::Utc,
    );
    let conn = db.get().await.unwrap();
    let rows = conn
        .query(
            "select logdir from lilac.batch where ts < $1 and event = 'start' and logdir is not null order by id desc limit 1",
            &[&dt],
        )
        .await
        .unwrap();
    if rows.len() == 0 {
        return HttpResponse::BadRequest().body("ts is too old");
    }
    let logdir: String = rows[0].get("logdir");
    let filename = format!("/home/lilydjwg/.lilac/log/{}/{}.log", logdir, name);
    let contents = match std::fs::read_to_string(&filename) {
        Ok(x) => x,
        Err(_) => return HttpResponse::NotFound().body(format!("Log {} not exist", &filename)),
    };
    let converted = ansi_to_html::convert(&contents, true, false).unwrap();
    let html = format!("{}<code>{}</code>", STYLE_HTML, converted);
    HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _guard = sentry::init((
        std::env::var("SENTRY").unwrap(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    std::env::set_var("RUST_BACKTRACE", "1");

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
            .service(get_pkg)
            .service(get_pkg_log)
    })
    .bind("127.0.0.1:9077")?
    .run()
    .await
}
