mod duolingo_client;

mod llm;
mod repository;
mod srs;

pub use duolingo_client::HttpDuolingoClient;
pub use llm::LlmServiceInvoker;
pub use llm::OpenAiLlm;
pub use repository::FirebaseUserRepository;
pub use srs::FsrsSrsService;

#[cfg(not(target_arch = "wasm32"))]
pub use repository::FileSystemUserRepository;
