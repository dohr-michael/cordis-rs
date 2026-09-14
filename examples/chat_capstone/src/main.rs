//! chat_capstone — EX-07 final-v3 Event/lifecycle capstone.
//!
//! The executable is fully scripted: two frontend Plugins drive typed Events with
//! explicit Routing, the chat path uses owned waterfall/query composition, a
//! Session Plugin demonstrates typed same-Fiber update control and identity-breaking
//! era replacement, and logout is ordinary disposal. No TTY input is required.

use std::borrow::Cow;
use std::convert::Infallible;
use std::io;
use std::sync::Arc;

use cordis_core::event::{
    ListenerOptions, Next, around, mapper_sync, observer_sync, responder, responder_sync,
    with_state,
};
use cordis_core::{
    BoxError, Context, Event, FiberState, InjectSpec, Plugin, PreparedChange, PreparedPlugin,
    QueryOutcome, Routing, Service, UpdateOutcome,
};
use examples_common::{Roster, section};

fn io_error(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

#[derive(Clone)]
struct Inbound {
    user: String,
    text: String,
}

struct Command;
impl Event for Command {
    const NAME: &'static str = "chat/command";
    type Args = String;
    type Output = String;
}

struct Message;
impl Event for Message {
    const NAME: &'static str = "chat/message";
    type Args = Inbound;
    type Output = String;
}

struct BuddyReply;
impl Event for BuddyReply {
    const NAME: &'static str = "chat/buddy-reply";
    type Args = Inbound;
    type Output = String;
}

struct Notice;
impl Event for Notice {
    const NAME: &'static str = "chat/notice";
    type Args = String;
    type Output = ();
}

struct RegistrationProbe;
impl Event for RegistrationProbe {
    const NAME: &'static str = "chat/registration-probe";
    type Args = ();
    type Output = String;
}

struct Session {
    user: String,
}
impl Service for Session {
    const NAME: &'static str = "chat/session";
}

#[derive(Clone)]
struct SessionConfig {
    user: String,
}

struct SessionPlugin;
impl Plugin for SessionPlugin {
    type Config = SessionConfig;
    type Input = SessionConfig;
    type PrepareError = Infallible;
    type ApplyError = io::Error;

    fn name(&self) -> Cow<'_, str> {
        "chat-session".into()
    }

    fn prepare(&self, config: SessionConfig) -> Result<SessionConfig, Infallible> {
        Ok(config)
    }

    async fn apply(&self, ctx: Context, input: &SessionConfig) -> Result<(), io::Error> {
        let _publication = ctx
            .provide(Arc::new(Session {
                user: input.user.clone(),
            }))
            .map_err(io_error)?;
        Ok(())
    }
}

struct ChatPlugin;
impl Plugin for ChatPlugin {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = io::Error;

    fn inject(&self) -> InjectSpec {
        InjectSpec::none().require(Session::NAME)
    }

    fn prepare(&self, (): ()) -> Result<(), Infallible> {
        Ok(())
    }

    async fn apply(&self, _ctx: Context, (): &()) -> Result<(), io::Error> {
        Ok(())
    }
}

struct Frontend {
    label: &'static str,
    commands: Vec<String>,
    route: Routing,
}
impl Plugin for Frontend {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = io::Error;

    fn name(&self) -> Cow<'_, str> {
        format!("frontend-{}", self.label).into()
    }

    fn prepare(&self, (): ()) -> Result<(), Infallible> {
        Ok(())
    }

    async fn apply(&self, ctx: Context, (): &()) -> Result<(), io::Error> {
        for command in &self.commands {
            let answer = ctx
                .query::<Command>(self.route.clone(), command.clone())
                .await
                .map_err(io_error)?;
            if let QueryOutcome::Answer(answer) = answer {
                println!("  {} -> {answer}", self.label);
            }
        }
        Ok(())
    }
}

fn prepared<P: Plugin>(plugin: P, config: P::Config) -> Result<PreparedPlugin, P::PrepareError> {
    let input = plugin.prepare(config)?;
    Ok(PreparedPlugin::from_input(plugin, input))
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let root = Context::new();
    let app = root.with_child_scope();
    let sibling = root.with_child_scope();
    let app_routing = Routing::Scoped(app.scope());
    let mut roster = Roster::new();

    section("typed Event roles and explicit Routing");
    app.on::<RegistrationProbe, _>(responder_sync(|_, (): ()| {
        Ok::<_, Infallible>(Some("app-registration".to_owned()))
    }))?;
    sibling.on::<RegistrationProbe, _>(responder_sync(|_, (): ()| {
        Ok::<_, Infallible>(Some("sibling-registration".to_owned()))
    }))?;
    app.on::<Command, _>(responder::<Command, _>(with_state(
        || 1usize,
        move |callback_ctx: Context, invocation: usize, command: String| async move {
            assert_eq!(invocation, 1);
            let probe = callback_ctx
                .query::<RegistrationProbe>(Routing::Scoped(callback_ctx.scope()), ())
                .await
                .map_err(io_error)?;
            assert_eq!(probe, QueryOutcome::Answer("app-registration".to_owned()));
            let answer = (command == "who").then(|| "scoped-command".to_owned());
            Ok::<_, io::Error>(answer)
        },
    )))?;
    sibling.on::<Command, _>(responder_sync(|_, _: String| {
        Ok::<_, Infallible>(Some("wrong-sibling".into()))
    }))?;
    let command = root
        .query::<Command>(app_routing.clone(), "who".into())
        .await?;
    assert_eq!(command, QueryOutcome::Answer("scoped-command".into()));
    println!("  routing: scoped command reached registration context");
    println!("  registration context: callback kept registration Scope");

    root.on::<Notice, _>(observer_sync(|_, text: String| {
        println!("  notice: {text}");
        Ok::<_, Infallible>(())
    }))?;

    app.on::<BuddyReply, _>(responder_sync(|_, inbound: Inbound| {
        Ok::<_, Infallible>(Some(format!(
            "buddy heard {}: {}",
            inbound.user, inbound.text
        )))
    }))?;
    app.on::<Message, _>(mapper_sync(|_, mut inbound: Inbound| {
        inbound.text = inbound.text.trim().to_lowercase();
        Ok::<_, Infallible>(inbound)
    }))?;
    app.on::<Message, _>(around(
        |_, inbound: Inbound, next: Next<Message>| async move {
            if inbound.text.contains("secret") {
                return Ok::<_, io::Error>("blocked".into());
            }
            next.call(inbound).await.map_err(io_error)
        },
    ))?;

    section("dependent chat before session");
    let chat = roster.push(app.spawn(prepared(ChatPlugin, ())?).await?);
    assert_eq!(chat.state(), FiberState::Pending);

    let session = app
        .spawn(prepared(
            SessionPlugin,
            SessionConfig {
                user: "Alice".into(),
            },
        )?)
        .await?;
    assert_eq!(chat.ready().await?, FiberState::Active);

    section("owned waterfall/query");
    let reply = root
        .waterfall_query::<Message, BuddyReply, _, _, io::Error>(
            app_routing.clone(),
            Inbound {
                user: root.try_service::<Session>()?.user.clone(),
                text: " HELLO ".into(),
            },
            |query| async move {
                match query {
                    Ok(QueryOutcome::Answer(answer)) => Ok(answer),
                    Ok(QueryOutcome::Miss) => Err(io::Error::other("query miss")),
                    Err(error) => Err(io_error(error)),
                }
            },
        )
        .await?;
    assert_eq!(reply, "buddy heard Alice: hello");
    println!("  waterfall/query: mapped and answered");

    section("typed same-Fiber precommit update policy");
    sibling.on_update::<SessionPlugin, _>(
        mapper_sync::<SessionPlugin, _>(|_, mut config: SessionConfig| {
            config.user = "sibling-policy".to_owned();
            Ok::<_, Infallible>(config)
        }),
        ListenerOptions::default(),
    )?;
    app.on_update::<SessionPlugin, _>(
        mapper_sync::<SessionPlugin, _>(
            move |_callback_ctx: Context, mut config: SessionConfig| {
                config.user = format!("policy:{}", config.user.trim().to_lowercase());
                Ok::<_, Infallible>(config)
            },
        ),
        ListenerOptions::default(),
    )?;
    let same_id = session.id().clone();
    assert_eq!(
        session
            .update(PreparedChange::from_input::<SessionPlugin>(SessionConfig {
                user: " ALICE-V2 ".into(),
            }))
            .await?,
        UpdateOutcome::Committed(FiberState::Active)
    );
    assert_eq!(session.id(), same_id);
    assert_eq!(root.try_service::<Session>()?.user, "policy:alice-v2");
    println!("  update policy: target-scoped and same FiberId preserved");

    section("identity-breaking era replacement and dependent convergence");
    let successor = session
        .era_swap(PreparedChange::from_input::<SessionPlugin>(SessionConfig {
            user: "bob".into(),
        }))
        .await?;
    assert_ne!(successor.id(), same_id);
    println!("  era replacement: new FiberId observed");
    assert_eq!(chat.ready().await?, FiberState::Active);
    assert_eq!(root.try_service::<Session>()?.user, "bob");
    println!("  dependent convergence: chat active on successor");

    section("two scripted frontends");
    let frontend_one = root
        .spawn(prepared(
            Frontend {
                label: "one",
                commands: vec!["who".into()],
                route: app_routing.clone(),
            },
            (),
        )?)
        .await?;
    frontend_one.dispose().await?;
    println!("  frontend one: terminated cleanly");

    let frontend_two = root
        .spawn(prepared(
            Frontend {
                label: "two",
                commands: vec!["who".into()],
                route: app_routing.clone(),
            },
            (),
        )?)
        .await?;
    frontend_two.dispose().await?;
    println!("  frontend two: terminated cleanly");

    root.emit::<Notice>(Routing::Unscoped, "logout".into())
        .await?;
    successor.dispose().await?;
    assert_eq!(chat.ready().await?, FiberState::Pending);
    println!("  logout: ordinary dispose reached pending");

    roster.teardown().await;
    println!("chat capstone complete");
    Ok(())
}
