
use crate::types::Error;

pub trait IntoRepoResult<T> {
	fn into_repo_result(self) -> Result<Vec<T>, Error>;
}

impl<S, T, E> IntoRepoResult<T> for Vec<S>
where
	T: TryFrom<S, Error = E>,
	E: std::error::Error + Send + Sync + 'static,
{
	fn into_repo_result(self) -> Result<Vec<T>, Error> {
		self.into_iter()
			.map(|ent| ent.try_into())
			.collect::<Result<Vec<T>, E>>()
			.map_err(|err| Error::from(format!("Failed to convert entity: {}", err)))
	}
}
