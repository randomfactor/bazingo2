use rocket::fs::{FileServer, NamedFile};
use rocket::http::Method;
use rocket::Request;
use std::path::PathBuf;

pub fn frontend_dist_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/dist")
}

pub fn index_html_path() -> PathBuf {
    frontend_dist_root().join("index.html")
}

#[catch(404)]
pub async fn spa_fallback_catcher(request: &Request<'_>) -> Option<NamedFile> {
    let path = request.uri().path();

    if request.method() != Method::Get || path.starts_with("/api") || path.starts_with("/auth") {
        return None;
    }

    NamedFile::open(index_html_path()).await.ok()
}

pub fn file_server() -> FileServer {
    FileServer::from(frontend_dist_root())
}

#[cfg(test)]
mod tests {
    use super::{index_html_path};

    #[test]
    fn frontend_index_exists_for_spa_fallback() {
        assert!(index_html_path().exists());
    }
}
