/// Initialize tracing using the calling crate's package name.
#[macro_export]
macro_rules! init_tracing {
    () => {{
        use $crate::__private::tracing_subscriber::{
            EnvFilter, fmt::format::FmtSpan, layer::SubscriberExt as _,
            util::SubscriberInitExt as _,
        };

        let default_directive = format!("{}=info", env!("CARGO_PKG_NAME").replace("-", "_"),);

        match $crate::__private::tracing_journald::layer() {
            Ok(journald_layer) => $crate::__private::tracing_subscriber::registry()
                .with(
                    EnvFilter::builder()
                        .with_default_directive(default_directive.parse()?)
                        .from_env_lossy(),
                )
                .with(journald_layer)
                .init(),
            Err(_) => $crate::__private::tracing_subscriber::registry()
                .with(
                    EnvFilter::builder()
                        .with_default_directive(default_directive.parse()?)
                        .from_env_lossy(),
                )
                .with(
                    $crate::__private::tracing_subscriber::fmt::layer()
                        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
                )
                .init(),
        };

        Ok::<(), $crate::__private::anyhow::Error>(())
    }};
}
