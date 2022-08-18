use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{error, HttpResponse};
use deadpool_postgres::PoolError;
use derive_more::{Display, Error};
use tokio_postgres::error::Error as PGError;

#[derive(Display, Debug, Error)]
pub enum IdentityServerError {
    NotFound,
    PGError(PGError),
    PoolError(PoolError),
    #[display(fmt = "Validation error with: {}", reason)]
    ValidationError {
        reason: String,
    },
    AuthenticationError {
        reason: String,
    },
}

impl std::convert::From<tokio_postgres::Error> for IdentityServerError {
    fn from(error: PGError) -> Self {
        IdentityServerError::PGError(error)
    }
}

impl IdentityServerError {
    pub fn authanticationError(reason: &str) -> IdentityServerError {
        IdentityServerError::AuthenticationError { reason: reason.to_owned() }
    }
    pub fn validationError(reason: &str) -> IdentityServerError {
        IdentityServerError::ValidationError { reason: reason.to_owned() }
    }
}

impl error::ResponseError for IdentityServerError {
    fn status_code(&self) -> StatusCode {
        match *self {
            IdentityServerError::NotFound => StatusCode::NOT_FOUND,
            IdentityServerError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            IdentityServerError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            IdentityServerError::AuthenticationError { .. } => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
        /*
        IdentityServerError::NotFound => HttpResponse::NotFound().finish(),
        IdentityServerError::PoolError(ref err) => {
            HttpResponse::InternalServerError().body(err.to_string())
        },
        IdentityServerError::ValidationError {}
        _ => HttpResponse::InternalServerError().finish(),
         */
    }
}
