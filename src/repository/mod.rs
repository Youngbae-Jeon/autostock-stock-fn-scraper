#[cfg(feature = "mysql")]
mod repo_impl_mysql;

use std::env;

#[cfg(feature = "mysql")]
use repo_impl_mysql::create_repository_impl;
#[cfg(feature = "mysql")]
pub use repo_impl_mysql::RepoImpl as Repo;

use crate::entities::Repository;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
	pub url: String,
	pub max_connections: usize,
}

impl DatabaseConfig {
	pub fn from_env() -> Self {
		let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
		let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
			.ok()
			.and_then(|s| s.parse().ok())
			.unwrap_or(10);
		Self {
			url,
			max_connections,
		}
	}
}

pub async fn create(conf: &DatabaseConfig) -> Repo {
	log::debug!("Connecting to database: {}", conf.url);
	let repo = create_repository_impl(&conf.url, conf.max_connections).await;
	repo.test_connection().await
		.unwrap_or_else(|err| {
			log::error!("Can't connect to database: {}", err);
			panic!("Can't connect to database: {}", err);
		});

	log::info!("Connected to database");
	repo
}
