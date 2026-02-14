#[cfg(not(target_arch = "wasm32"))]
mod file_user_repository;
mod firebase_user_repository;

#[cfg(not(target_arch = "wasm32"))]
pub use file_user_repository::FileSystemUserRepository;
pub use firebase_user_repository::FirebaseUserRepository;
