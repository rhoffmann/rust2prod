use actix_files::NamedFile;
use actix_web::HttpRequest;

pub async fn static_files(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    let file = NamedFile::open(path)?;
    Ok(file.use_last_modified(true))
}