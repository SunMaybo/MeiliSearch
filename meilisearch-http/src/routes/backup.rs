use actix_web::{get, post};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

use crate::backup::{init_backup_process, compressed_backup_path, BackupInfo, BackupStatus};
use crate::Data;
use crate::error::{Error, ResponseError};
use crate::helpers::Authentication;

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(trigger_backup)
        .service(get_backup_status);
}

#[post("/backups", wrap = "Authentication::Private")]
async fn trigger_backup(
    data: web::Data<Data>,
) -> Result<HttpResponse, ResponseError> {
    let backup_path = Path::new(&data.backup_path);
    match init_backup_process(&data, &backup_path) {
        Ok(resume) => Ok(HttpResponse::Accepted().json(resume)),
        Err(e) => Err(e.into())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupStatusResponse {
    status: String,
}

#[derive(Deserialize)]
struct BackupParam {
    backup_uid: String,
}

#[get("/backups/{backup_uid}/status", wrap = "Authentication::Private")]
async fn get_backup_status(
    data: web::Data<Data>,
    path: web::Path<BackupParam>,
) -> Result<HttpResponse, ResponseError> {
    let backup_path = Path::new(&data.backup_path);
    let backup_uid = &path.backup_uid;

    if let Some(resume) = BackupInfo::get_current() {
        if &resume.uid == backup_uid {
            return Ok(HttpResponse::Ok().json(resume));
        }
    }

    if File::open(compressed_backup_path(Path::new(backup_path), backup_uid)).is_ok() {
        let resume = BackupInfo::new(
            backup_uid.into(),
            BackupStatus::Done
        );

        Ok(HttpResponse::Ok().json(resume))
    } else {
        Err(Error::not_found("backup does not exist").into())
    }
}
