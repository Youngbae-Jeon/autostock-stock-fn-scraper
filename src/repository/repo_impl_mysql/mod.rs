mod repo;
mod repo_tx;
mod repo_stocks;
mod convert;
mod repo_fi_annuals;
mod repo_fi_quarters;

pub use repo::RepoImpl;

pub async fn create_repository_impl(dburl: &str, max_connections: usize) -> RepoImpl {
	RepoImpl::new(dburl, max_connections).await
}
