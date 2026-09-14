//! scopes_tenants — opaque Service placement and explicit Event Scope routing.
//!
//! Service realms choose exact Service slots. Scope alone chooses Event reachability.
//! No derivation history or textual/numeric realm identity participates.

use std::{borrow::Cow, convert::Infallible, sync::Arc};

use cordis_core::event::{ListenerOptions, observer_sync};
use cordis_core::{
    BoxError, Context, Event, Plugin, PreparedPlugin, Routing, Service, ServiceRealm,
};
use examples_common::{Roster, section};
use parking_lot::Mutex;

struct Ledger {
    owner: &'static str,
}
impl Service for Ledger {
    const NAME: &'static str = "tenant/ledger";
}
struct Trail;
impl Service for Trail {
    const NAME: &'static str = "platform/trail";
}
struct Cache;
impl Service for Cache {
    const NAME: &'static str = "tenant/cache";
}

struct Ping;
impl Event for Ping {
    const NAME: &'static str = "tenant/ping";
    type Args = &'static str;
    type Output = ();
}

struct TrailPlugin;
impl Plugin for TrailPlugin {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;

    fn name(&self) -> Cow<'_, str> {
        "platform-trail".into()
    }
    fn prepare(&self, (): ()) -> Result<(), Infallible> {
        Ok(())
    }
    async fn apply(&self, ctx: Context, (): &()) -> Result<(), Infallible> {
        let _ = ctx.provide(Arc::new(Trail)).expect("trail publication");
        Ok(())
    }
}

struct TenantPlugin {
    tenant: &'static str,
}
impl Plugin for TenantPlugin {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;

    fn name(&self) -> Cow<'_, str> {
        format!("tenant-{}", self.tenant).into()
    }
    fn prepare(&self, (): ()) -> Result<(), Infallible> {
        Ok(())
    }
    async fn apply(&self, ctx: Context, (): &()) -> Result<(), Infallible> {
        let _ = ctx
            .provide(Arc::new(Ledger { owner: self.tenant }))
            .expect("private ledger publication");
        Ok(())
    }
}

fn map_platform(ctx: &Context, trail: &ServiceRealm) -> Context {
    ctx.with_service_realms([(Trail::NAME, trail.clone())])
        .expect("realm belongs to this Runtime")
}
fn map_tenant(ctx: &Context, ledger: &ServiceRealm, trail: &ServiceRealm) -> Context {
    ctx.with_service_realms([(Ledger::NAME, ledger.clone()), (Trail::NAME, trail.clone())])
        .expect("all realms belong to this Runtime")
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let root = Context::new();
    let shared_trail = root.new_service_realm();
    let platform_scope = root.with_child_scope();
    let acme_scope = platform_scope.with_child_scope();
    let globex_scope = platform_scope.with_child_scope();

    let acme_ledger_realm = root.new_service_realm();
    let globex_ledger_realm = root.new_service_realm();
    assert_ne!(acme_ledger_realm, globex_ledger_realm);
    assert_ne!(shared_trail, acme_ledger_realm);
    assert_ne!(shared_trail, globex_ledger_realm);

    let platform = map_platform(&platform_scope, &shared_trail);
    let acme = map_tenant(&acme_scope, &acme_ledger_realm, &shared_trail);
    let globex = map_tenant(&globex_scope, &globex_ledger_realm, &shared_trail);

    let mut roster = Roster::new();
    let trail = TrailPlugin;
    trail.prepare(())?;
    roster.push(
        platform
            .spawn(PreparedPlugin::from_input(trail, ()))
            .await?,
    );

    for (view, tenant) in [(&acme, "acme"), (&globex, "globex")] {
        let plugin = TenantPlugin { tenant };
        plugin.prepare(())?;
        roster.push(view.spawn(PreparedPlugin::from_input(plugin, ())).await?);
    }
    roster.report().await;

    section("Service placement is explicit and opaque");
    let acme_ledger = acme.try_service::<Ledger>()?;
    let globex_ledger = globex.try_service::<Ledger>()?;
    assert_eq!(acme_ledger.owner, "acme");
    assert_eq!(globex_ledger.owner, "globex");
    assert!(!Arc::ptr_eq(&acme_ledger, &globex_ledger));
    println!("  realm identity: private opaque handles compare unequal");
    println!("  placement: private realms stay isolated");
    let acme_trail = acme.try_service::<Trail>()?;
    let globex_trail = globex.try_service::<Trail>()?;
    assert!(Arc::ptr_eq(&acme_trail, &globex_trail));
    println!("  placement: explicit shared realm joins only trail");

    section("Scope routing is independent of placement");
    let route_log = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let ancestor_log = route_log.clone();
    platform_scope.on::<Ping, _>(observer_sync(move |_ctx: Context, _| {
        ancestor_log.lock().push("ancestor");
        Ok::<(), Infallible>(())
    }))?;
    let sibling_log = route_log.clone();
    globex_scope.on::<Ping, _>(observer_sync(move |_ctx: Context, _| {
        sibling_log.lock().push("sibling");
        Ok::<(), Infallible>(())
    }))?;
    let self_log = route_log.clone();
    acme_scope.on::<Ping, _>(observer_sync(move |_ctx: Context, _| {
        self_log.lock().push("self");
        Ok::<(), Infallible>(())
    }))?;
    let global_log = route_log.clone();
    root.on_with::<Ping, _>(
        observer_sync(move |_ctx: Context, _| {
            global_log.lock().push("global");
            Ok::<(), Infallible>(())
        }),
        ListenerOptions::default().global(),
    )?;

    root.emit::<Ping>(Routing::Scoped(acme_scope.scope()), "scoped")
        .await?;
    let routed = route_log.lock().clone();
    assert!(routed.contains(&"ancestor"));
    assert!(routed.contains(&"self"));
    assert!(!routed.contains(&"sibling"));
    assert!(routed.contains(&"global"));
    println!("  routing: ancestor reached");
    println!("  routing: sibling excluded");
    println!("  routing: global listener reached");

    let axis_log = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let direct_log = axis_log.clone();
    acme_scope.on::<Ping, _>(observer_sync(move |_ctx: Context, _| {
        direct_log.lock().push("direct");
        Ok::<(), Infallible>(())
    }))?;
    let isolated_same_scope = acme_scope.with_isolated_service(Cache::NAME);
    let isolated_log = axis_log.clone();
    isolated_same_scope.on::<Ping, _>(observer_sync(move |_ctx: Context, _| {
        isolated_log.lock().push("isolated");
        Ok::<(), Infallible>(())
    }))?;
    root.emit::<Ping>(Routing::Scoped(acme_scope.scope()), "axis")
        .await?;
    let axis = axis_log.lock().clone();
    assert!(axis.contains(&"direct") && axis.contains(&"isolated"));
    println!("  axes: isolate derivation kept the same Scope reachability");

    route_log.lock().clear();
    root.emit::<Ping>(Routing::Unscoped, "unscoped").await?;
    let unscoped = route_log.lock().clone();
    assert!(unscoped.contains(&"ancestor"));
    assert!(unscoped.contains(&"sibling"));
    assert!(unscoped.contains(&"global"));

    section("teardown");
    roster.teardown().await;
    println!("  scopes_tenants tour complete");
    Ok(())
}
