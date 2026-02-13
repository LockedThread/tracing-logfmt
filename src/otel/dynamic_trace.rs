//! Extract OpenTelemetry trace_id and span_id from tracing span extensions (OtelData).
//!
//! This follows the same approach as [json-subscriber's with_opentelemetry_ids][json-subscriber]:
//! read `OtelData` from the current span's extensions for each supported
//! tracing-opentelemetry version. Enable the feature that matches your
//! tracing-opentelemetry version (otel-0-28, otel-0-29, otel-0-30, otel-0-31, or otel-0-32).
//!
//! [json-subscriber]: https://github.com/mladedav/json-subscriber/blob/7491bd305c8ec3cc31a16d65f047ca82b3a66894/src/layer/mod.rs#L1007

use tracing_subscriber::registry::LookupSpan;

/// Expands to an `ids.or_else(|| ...)` block for OtelData that has `trace_id()` and `span_id()` methods (0.32).
macro_rules! extract_otel_ids_direct {
    ($ids:ident, $span:ident, $feature:literal, $tracing_otel:ident) => {
        #[cfg(feature = $feature)]
        {
            $ids = $ids.or_else(|| {
                $span
                    .extensions()
                    .get::<$tracing_otel::OtelData>()
                    .and_then(|otel_data| {
                        let trace_id = otel_data.trace_id()?;
                        let span_id = otel_data.span_id()?;
                        Some((trace_id.to_string(), span_id.to_string()))
                    })
            });
        }
    };
}

/// Expands to an `ids.or_else(|| ...)` block for OtelData that has parent_cx + builder (0.28–0.31).
macro_rules! extract_otel_ids_parent_cx_builder {
    ($ids:ident, $span:ident, $feature:literal, $tracing_otel:ident, $otel_trace:ident) => {
        #[cfg(feature = $feature)]
        {
            use $otel_trace::trace::TraceContextExt;
            $ids = $ids.or_else(|| {
                $span
                    .extensions()
                    .get::<$tracing_otel::OtelData>()
                    .and_then(|otel_data| {
                        let trace_id = otel_data.parent_cx.span().span_context().trace_id();
                        let trace_id = if trace_id == $otel_trace::trace::TraceId::INVALID {
                            otel_data.builder.trace_id?
                        } else {
                            trace_id
                        };
                        let span_id = otel_data.builder.span_id?;
                        Some((trace_id.to_string(), span_id.to_string()))
                    })
            });
        }
    };
}

/// Returns `(trace_id, span_id)` from the given span's extensions if it contains
/// OtelData from a supported tracing-opentelemetry version.
pub(crate) fn trace_id_span_id_from_span<S>(
    span: &tracing_subscriber::registry::SpanRef<'_, S>,
) -> Option<(String, String)>
where
    S: for<'a> LookupSpan<'a>,
{
    let mut ids = None;

    extract_otel_ids_direct!(ids, span, "otel-0-32", tracing_opentelemetry_0_32);
    extract_otel_ids_parent_cx_builder!(
        ids,
        span,
        "otel-0-31",
        tracing_opentelemetry_0_31,
        opentelemetry_0_30
    );
    extract_otel_ids_parent_cx_builder!(
        ids,
        span,
        "otel-0-30",
        tracing_opentelemetry_0_30,
        opentelemetry_0_29
    );
    extract_otel_ids_parent_cx_builder!(
        ids,
        span,
        "otel-0-29",
        tracing_opentelemetry_0_29,
        opentelemetry_0_28
    );
    extract_otel_ids_parent_cx_builder!(
        ids,
        span,
        "otel-0-28",
        tracing_opentelemetry_0_28,
        opentelemetry_0_27
    );

    ids
}
