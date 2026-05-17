use axum::Router;
use axum::http::header;
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveHeadersLayer, trace,
};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db;
use crate::routes;
use crate::state;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::get_health,
        routes::trip_handlers::create,
    ),
    components(schemas(
        routes::health::Status,
        crate::models::trips_api::CreateTripRequest,
        crate::models::trips_api::CreateTripResponse,
    )),
    tags((name = "Liftshare"))
)]
struct ApiDoc;

pub async fn create_app() -> Router {
    let shared_state: state::SharedState = Arc::new(state::AppState {
        db_pool: db::create_pool().await,
    });

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(routes::health::create_route())
        .merge(routes::trip_handlers::create_route())
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        // Mark the `Authorization` request header as sensitive so it doesn't
        // show in logs.
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        // Compress responses
        .layer(CompressionLayer::new())
        // Propagate `X-Request-Id`s from requests to responses
        .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
            "x-request-id",
        )))
        // CORS configuration. This should probably be more restrictive in
        // production.
        .layer(CorsLayer::permissive())
        .with_state(shared_state)
}
