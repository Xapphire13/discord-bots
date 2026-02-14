use std::sync::Arc;

use maud::{Markup, PreEscaped, html};
use rocket::{State, get};

use crate::state::{AppState, ONLINE_GRACE_PERIOD};

use super::breadcrumbs::{Breadcrumb, breadcrumbs};
use super::{format_relative, page_shell};

#[get("/bot/<name>")]
pub fn bot_detail(name: &str, state: &State<Arc<AppState>>) -> Option<Markup> {
    let registry = state.registry.read().unwrap();
    let bot = registry.get(name)?;
    let online = registry.is_online(name, ONLINE_GRACE_PERIOD);
    let ago = (chrono::Utc::now() - bot.last_heartbeat).num_seconds();
    let bot_name = bot.name.clone();
    drop(registry);

    let content = html! {
        (breadcrumbs(&[
            Breadcrumb { label: "bots", href: Some("/")},
            Breadcrumb { label: name, href: None }])
        )

        @if online {
            div.status.online { "[ONLINE]" }
        } @else {
            div.status.offline { "[OFFLINE]" }
        }
        div.meta { "Last seen: " (format_relative(ago)) }

        div
            hx-get=(format!("/fragments/bot/{bot_name}/charts"))
            hx-trigger="every 60s"
            hx-swap="innerHTML"
        {
            (fragment_bot_charts(name, state)?)
        }
    };
    Some(page_shell(&format!("{bot_name} | Dashboard"), content))
}

fn charts_inner(uptime_svg: &str, metric_charts: &[(String, &str)]) -> Markup {
    html! {
        h2 { "> uptime" }
        div.chart-container {
            (PreEscaped(uptime_svg))
        }

        @if !metric_charts.is_empty() {
            h2 { "> metrics" }
            @for (_event_id, svg) in metric_charts {
                div.chart-container {
                    (PreEscaped(svg))
                }
            }
        }
    }
}

#[get("/fragments/bot/<name>/charts")]
pub fn fragment_bot_charts(name: &str, state: &State<Arc<AppState>>) -> Option<Markup> {
    let uptime_svg = "TODO uptime chart";

    let metrics = state.metrics.read().unwrap();
    let event_ids = metrics.event_ids(name);
    let mut metric_charts = Vec::new();
    for eid in &event_ids {
        let events = metrics.query(name, Some(eid));
        let has_values = events.iter().any(|e| e.value.is_some());
        let svg = if has_values {
            "TODO value chart"
        } else {
            "TODO count chart"
        };
        metric_charts.push((eid.clone(), svg));
    }
    drop(metrics);

    Some(charts_inner(uptime_svg, &metric_charts))
}
