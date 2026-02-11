use origa::infrastructure::FileSystemUserRepository;
use std::path::PathBuf;

pub async fn build_repository() -> Result<FileSystemUserRepository, origa::domain::OrigaError> {
    let path = PathBuf::from("./data/users");
    FileSystemUserRepository::new(path).await
}
