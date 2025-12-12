mod repo;
mod repo_tx;
mod repo_stocks;
mod convert;

pub use repo::RepoImpl;

pub async fn create_repository_impl(dburl: &str, max_connections: usize) -> RepoImpl {
	RepoImpl::new(dburl, max_connections).await
}
