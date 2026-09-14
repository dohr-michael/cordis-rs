use cordis_core::{Context, Event, QueryOutcome, Routing};
use std::convert::Infallible;

struct Flow;
struct Query;
struct MoveOnly(String);

impl Event for Flow {
    const NAME: &'static str = "ui/issue34-waterfall-query-flow";
    type Args = MoveOnly;
    type Output = usize;
}
impl Event for Query {
    const NAME: &'static str = "ui/issue34-waterfall-query-query";
    type Args = MoveOnly;
    type Output = usize;
}

async fn check(ctx: Context) {
    let _ = ctx.waterfall_query::<Flow, Query, _, _, Infallible>(
        Routing::Unscoped,
        MoveOnly(String::new()),
        |result| async move {
            Ok(match result.unwrap() { QueryOutcome::Miss => 0, QueryOutcome::Answer(value) => value })
        },
    ).await;
}

fn main() {}
