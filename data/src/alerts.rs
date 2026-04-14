use std::{
    error::Error,
    io, panic,
    sync::{Mutex, OnceLock},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertKind {
    RateLimit,
    UnexpectedError,
}

#[derive(Debug, Clone)]
struct PipelineAlert {
    kind: AlertKind,
    context: String,
    details: String,
}

static ALERTS: OnceLock<Mutex<Vec<PipelineAlert>>> = OnceLock::new();

fn alerts() -> &'static Mutex<Vec<PipelineAlert>> {
    ALERTS.get_or_init(|| Mutex::new(Vec::new()))
}

fn red_banner(title: &str, context: &str, details: &str) -> String {
    let border = "############################################################";
    format!(
        "\x1b[1;37;41m{border}\n{title}\nContext: {context}\nDetails: {details}\n{border}\x1b[0m"
    )
}

fn push_alert(kind: AlertKind, context: impl Into<String>, details: impl Into<String>) {
    let context = context.into();
    let details = details.into();
    let title = match kind {
        AlertKind::RateLimit => "DATA PIPELINE BLOCKED: RATE LIMIT DETECTED",
        AlertKind::UnexpectedError => "DATA PIPELINE BLOCKED: UNEXPECTED ERROR",
    };

    eprintln!("{}", red_banner(title, &context, &details));

    alerts().lock().unwrap().push(PipelineAlert {
        kind,
        context,
        details,
    });
}

pub fn record_rate_limit(context: impl Into<String>, details: impl Into<String>) {
    push_alert(AlertKind::RateLimit, context, details);
}

pub fn record_unexpected_error(context: impl Into<String>, details: impl Into<String>) {
    push_alert(AlertKind::UnexpectedError, context, details);
}

pub fn install_panic_hook() {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        record_unexpected_error("panic", panic_info.to_string());
        previous_hook(panic_info);
    }));
}

pub fn print_summary() {
    let alerts = alerts().lock().unwrap();
    if alerts.is_empty() {
        return;
    }

    let rate_limit_count = alerts
        .iter()
        .filter(|alert| alert.kind == AlertKind::RateLimit)
        .count();
    let error_count = alerts.len().saturating_sub(rate_limit_count);
    let details = alerts
        .iter()
        .map(|alert| format!("- {}: {}", alert.context, alert.details))
        .collect::<Vec<_>>()
        .join("\n");

    eprintln!(
        "{}",
        red_banner(
            "INCOMPLETE DATA - DO NOT PUBLISH",
            &format!(
                "{} blocking alerts (rate limits: {}, unexpected errors: {})",
                alerts.len(),
                rate_limit_count,
                error_count
            ),
            &details,
        )
    );
}

pub fn alert_count() -> usize {
    alerts().lock().unwrap().len()
}

pub fn fail_if_any() -> Result<(), Box<dyn Error>> {
    let count = alert_count();
    if count == 0 {
        return Ok(());
    }

    print_summary();
    Err(Box::new(io::Error::other(format!(
        "data pipeline recorded {count} blocking alert(s)"
    ))))
}
