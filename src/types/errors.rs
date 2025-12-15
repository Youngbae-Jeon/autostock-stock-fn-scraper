#[derive(Debug)]
pub struct Error {
	pub message: String,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.message)
	}
}

impl From<mysql_async::Error> for Error {
	fn from(err: mysql_async::Error) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}

impl From<std::fmt::Error> for Error {
	fn from(err: std::fmt::Error) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}

impl From<std::str::Utf8Error> for Error {
	fn from(err: std::str::Utf8Error) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}

impl From<std::num::ParseIntError> for Error {
	fn from(err: std::num::ParseIntError) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}

impl From<std::num::ParseFloatError> for Error {
	fn from(err: std::num::ParseFloatError) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}

impl From<chrono::format::ParseError> for Error {
	fn from(err: chrono::format::ParseError) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}

impl From<String> for Error {
	fn from(message: String) -> Self {
		Error { message }
	}
}

impl From<&str> for Error {
	fn from(message: &str) -> Self {
		Error {
			message: message.to_string(),
		}
	}
}

impl From<csv::Error> for Error {
	fn from(err: csv::Error) -> Self {
		Error {
			message: err.to_string(),
		}
	}
}
