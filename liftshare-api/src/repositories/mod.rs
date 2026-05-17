pub mod trip_repo;

pub type RepositoryResult<T> = Result<T, sqlx::Error>;
