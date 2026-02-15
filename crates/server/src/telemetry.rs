use axum::{body::Body, http::Request, response::Response};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use std::{
    future::Future,
    pin::Pin,
    sync::OnceLock,
    task::{Context as TaskContext, Poll},
};
use tower::{Layer, Service};

use crate::auth::jwt::Claims;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Keep the LoggerProvider alive for the process lifetime.
static LOGGER_PROVIDER: OnceLock<opentelemetry_sdk::logs::SdkLoggerProvider> = OnceLock::new();

/// Tokio runtime for the OTLP gRPC exporters. Tonic's `connect_lazy()`
/// calls `tokio::spawn` which requires a Tokio runtime context. When
/// `dioxus::serve` calls our init closure the runtime context may not be
/// propagated yet, so we ensure one exists here.
static OTEL_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Set up the OpenTelemetry TracerProvider and register it globally.
///
/// Dioxus owns the tracing subscriber — this only configures the OTLP
/// trace exporter so HTTP spans (via `OtelTraceLayer`) reach SigNoz.
///
/// Reads config from environment:
///   - `OTEL_EXPORTER_OTLP_ENDPOINT` — collector gRPC address
///       Local: `http://localhost:4317`
///       SigNoz Cloud: `https://ingest.{region}.signoz.cloud:443`
///   - `OTEL_SERVICE_NAME` — service name tag (default: project name)
///   - `SIGNOZ_INGESTION_KEY` — SigNoz Cloud access token (optional for local)
///   - `DEPLOY_ENV` — deployment environment tag (default: `development`)
pub fn init_telemetry() {
    let _ = dotenvy::dotenv();

    let endpoint = match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(ep) => ep,
        Err(_) => {
            eprintln!("OTEL_EXPORTER_OTLP_ENDPOINT not set, skipping OTLP telemetry");
            return;
        }
    };

    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "lexodus".to_string());
    let environment = std::env::var("DEPLOY_ENV").unwrap_or_else(|_| "development".to_string());

    // Tonic's connect_lazy() calls tokio::spawn which requires a Tokio
    // runtime context. When dioxus::serve calls our init closure the runtime
    // may not be propagated, so we ensure one exists here.
    let rt = OTEL_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(1)
            .build()
            .expect("Failed to create OTEL runtime")
    });
    let _guard = rt.enter();

    use opentelemetry_otlp::WithTonicConfig;

    let mut builder = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint);

    // Enable TLS with system root certs for HTTPS endpoints (e.g. SigNoz Cloud)
    if endpoint.starts_with("https://") {
        builder = builder.with_tls_config(
            opentelemetry_otlp::tonic_types::transport::ClientTlsConfig::new().with_native_roots(),
        );
    }

    // Attach SigNoz Cloud ingestion key as gRPC metadata when present
    if let Ok(key) = std::env::var("SIGNOZ_INGESTION_KEY") {
        if !key.is_empty() {
            let mut metadata = opentelemetry_otlp::tonic_types::metadata::MetadataMap::new();
            metadata.insert(
                "signoz-ingestion-key",
                key.parse().expect("Invalid SIGNOZ_INGESTION_KEY value"),
            );
            builder = builder.with_metadata(metadata);
        }
    }

    let exporter = builder.build().expect("Failed to create OTLP exporter");

    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(service_name)
        .with_attribute(KeyValue::new("service.version", APP_VERSION))
        .with_attribute(KeyValue::new("deployment.environment", environment))
        .build();

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.clone())
        .build();

    global::set_tracer_provider(provider);

    // -- Log exporter (uses the `log` crate, not `tracing` subscriber) --
    let mut log_builder = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint);
    if endpoint.starts_with("https://") {
        log_builder = log_builder.with_tls_config(
            opentelemetry_otlp::tonic_types::transport::ClientTlsConfig::new().with_native_roots(),
        );
    }
    if let Ok(key) = std::env::var("SIGNOZ_INGESTION_KEY") {
        if !key.is_empty() {
            let mut md = opentelemetry_otlp::tonic_types::metadata::MetadataMap::new();
            md.insert(
                "signoz-ingestion-key",
                key.parse().expect("Invalid SIGNOZ_INGESTION_KEY value"),
            );
            log_builder = log_builder.with_metadata(md);
        }
    }
    let log_exporter = log_builder
        .build()
        .expect("Failed to create OTLP log exporter");

    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource)
        .build();
    let _ = LOGGER_PROVIDER.set(logger_provider);

    // Bridge the `log` crate → OpenTelemetry. This is separate from the
    // `tracing` subscriber (owned by Dioxus) so there's no conflict.
    let bridge =
        opentelemetry_appender_log::OpenTelemetryLogBridge::new(LOGGER_PROVIDER.get().unwrap());
    match log::set_boxed_logger(Box::new(bridge)) {
        Ok(()) => {
            log::set_max_level(log::LevelFilter::Info);
            eprintln!("Log bridge active — logs exporting to SigNoz");
        }
        Err(_) => {
            eprintln!("Log bridge skipped — log crate logger already set");
        }
    }

    let mode = if std::env::var("SIGNOZ_INGESTION_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        "cloud"
    } else {
        "local"
    };
    eprintln!(
        "Telemetry initialized v{APP_VERSION} — traces + logs exporting to {endpoint} ({mode})"
    );
}

/// Detect client platform from User-Agent and optional X-Client-Platform header.
///
/// Priority: explicit `X-Client-Platform` header > User-Agent heuristic.
/// Dioxus native clients (desktop/mobile) don't send User-Agent, so they
/// show as "native" unless the app sets X-Client-Platform.
fn detect_platform(ua: &str, explicit: Option<&str>) -> &'static str {
    if let Some(p) = explicit {
        return match p {
            "ios" => "ios",
            "android" => "android",
            "desktop" => "desktop",
            "mobile" => "mobile",
            "web" => "web",
            _ => "unknown",
        };
    }

    if ua == "unknown" || ua.is_empty() {
        return "native";
    }
    if ua.contains("iPhone") || ua.contains("iPad") || ua.contains("CFNetwork") {
        "ios"
    } else if ua.contains("Android") {
        "android"
    } else if ua.contains("Mozilla") || ua.contains("Chrome") || ua.contains("Safari") {
        "web"
    } else {
        "native"
    }
}

/// Tower layer that creates an OpenTelemetry span for each HTTP request.
///
/// Captures: method, path, user-agent, client platform, request ID,
/// response status, and authenticated user info (if present).
#[derive(Clone)]
pub struct OtelTraceLayer;

impl<S> Layer<S> for OtelTraceLayer {
    type Service = OtelTraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelTraceService { inner }
    }
}

#[derive(Clone)]
pub struct OtelTraceService<S> {
    inner: S,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_header_takes_priority() {
        assert_eq!(detect_platform("Mozilla/5.0 Chrome", Some("ios")), "ios");
        assert_eq!(detect_platform("", Some("desktop")), "desktop");
    }

    #[test]
    fn explicit_header_known_values() {
        for (input, expected) in [
            ("ios", "ios"),
            ("android", "android"),
            ("desktop", "desktop"),
            ("mobile", "mobile"),
            ("web", "web"),
        ] {
            assert_eq!(detect_platform("", Some(input)), expected);
        }
    }

    #[test]
    fn explicit_header_unknown_value() {
        assert_eq!(detect_platform("", Some("smartwatch")), "unknown");
    }

    #[test]
    fn empty_ua_returns_native() {
        assert_eq!(detect_platform("", None), "native");
    }

    #[test]
    fn unknown_ua_returns_native() {
        assert_eq!(detect_platform("unknown", None), "native");
    }

    #[test]
    fn ios_user_agents() {
        assert_eq!(
            detect_platform("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)", None),
            "ios"
        );
        assert_eq!(
            detect_platform("Mozilla/5.0 (iPad; CPU OS 17_0)", None),
            "ios"
        );
        assert_eq!(detect_platform("CFNetwork/1485 Darwin/23.1.0", None), "ios");
    }

    #[test]
    fn android_user_agent() {
        assert_eq!(
            detect_platform("Mozilla/5.0 (Linux; Android 14; Pixel 8)", None),
            "android"
        );
    }

    #[test]
    fn web_browser_user_agents() {
        assert_eq!(
            detect_platform("Mozilla/5.0 (Macintosh; Intel Mac OS X)", None),
            "web"
        );
        assert_eq!(
            detect_platform("Chrome/120.0.0.0 Safari/537.36", None),
            "web"
        );
    }

    #[test]
    fn unrecognized_ua_returns_native() {
        assert_eq!(detect_platform("curl/8.4.0", None), "native");
        assert_eq!(detect_platform("custom-http-client/1.0", None), "native");
    }
}

impl<S> Service<Request<Body>> for OtelTraceService<S>
where
    S: Service<Request<Body>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let tracer = global::tracer("lexodus");
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let explicit_platform = req
            .headers()
            .get("x-client-platform")
            .and_then(|v| v.to_str().ok());
        let client_platform = detect_platform(&user_agent, explicit_platform);

        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let auth_attrs: Vec<KeyValue> = if let Some(claims) = req.extensions().get::<Claims>() {
            vec![
                KeyValue::new("user.id", claims.sub),
                KeyValue::new("user.email", claims.email.clone()),
                KeyValue::new("user.role", claims.role.clone()),
                KeyValue::new("user.tier", claims.tier.clone()),
                KeyValue::new("auth.status", "authenticated"),
            ]
        } else {
            vec![KeyValue::new("auth.status", "anonymous")]
        };

        let mut attributes = vec![
            KeyValue::new("http.method", method.clone()),
            KeyValue::new("http.target", path.clone()),
            KeyValue::new("http.user_agent", user_agent),
            KeyValue::new("client.platform", client_platform),
            KeyValue::new("http.request_id", request_id),
        ];
        attributes.extend(auth_attrs);

        let route = path
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();

        let span = tracer
            .span_builder(format!("{} {}", &method, &route))
            .with_kind(SpanKind::Server)
            .with_attributes(attributes)
            .start(&tracer);

        let cx = Context::current_with_span(span);
        let mut inner = self.inner.clone();

        let guard = cx.clone().attach();
        let future = inner.call(req);
        drop(guard);

        Box::pin(async move {
            let response = future.await?;

            let span = cx.span();
            let status = response.status();
            span.set_attribute(KeyValue::new("http.status_code", status.as_u16() as i64));

            if status.is_server_error() {
                span.set_status(opentelemetry::trace::Status::error(status.to_string()));
            } else if status.is_client_error() {
                span.set_attribute(KeyValue::new("error.type", "client_error"));
            }

            Ok(response)
        })
    }
}
