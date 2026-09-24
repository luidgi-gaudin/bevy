//! Integration with the [puffin](https://github.com/EmbarkStudios/puffin) profiler.
//!
//! Every [`tracing`] span entered while puffin scopes are on (see [`puffin::set_scopes_on`]) is
//! recorded as a puffin scope. The recorded frames are streamed by a [`puffin_http`] server, that
//! [`puffin_viewer`](https://crates.io/crates/puffin_viewer) can connect to.

use alloc::{string::String, vec::Vec};
use core::{cell::RefCell, fmt::Debug, fmt::Write};

use ::puffin::{GlobalProfiler, ScopeId, ThreadProfiler};
use bevy_app::App;
use bevy_ecs::{resource::Resource, system::Local};
use bevy_platform::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};
use tracing::{
    callsite::Identifier,
    field::{Field, Visit},
    info, span, warn, Metadata, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

/// The address the puffin server listens on if the `PUFFIN_SERVER_ADDR` environment variable
/// isn't set.
pub const DEFAULT_PUFFIN_SERVER_ADDR: &str = "127.0.0.1:8585";

/// Keeps the [`puffin_http::Server`] that streams profiling data to `puffin_viewer` alive.
///
/// Remove this resource to stop the server.
#[derive(Resource)]
pub struct PuffinServer(pub puffin_http::Server);

/// Starts a [`puffin_http::Server`] on the address set by the `PUFFIN_SERVER_ADDR` environment
/// variable, or on [`DEFAULT_PUFFIN_SERVER_ADDR`], and inserts it as a [`PuffinServer`] resource.
pub(crate) fn start_puffin_server(app: &mut App) {
    let address =
        std::env::var("PUFFIN_SERVER_ADDR").unwrap_or_else(|_| DEFAULT_PUFFIN_SERVER_ADDR.into());
    match puffin_http::Server::new(&address) {
        Ok(server) => {
            info!("Tracing with puffin is active, connect puffin_viewer to {address}");
            app.insert_resource(PuffinServer(server));
        }
        Err(error) => warn!("Failed to start the puffin server on {address}: {error:#}"),
    }
}

/// Ends the current puffin frame and starts a new one.
///
/// Scopes are turned on when this runs for the first time rather than when the app is built,
/// so that spans covering the whole lifetime of the app (like the span entered by `App::run`)
/// are not recorded: puffin only sends the scopes of a thread once all its scopes are closed.
pub(crate) fn puffin_new_frame(mut started: Local<bool>) {
    if !*started {
        *started = true;
        ::puffin::set_scopes_on(true);
    }
    GlobalProfiler::lock().new_frame();
}

/// The values of the fields of a span, displayed next to its scope name by puffin viewers.
struct PuffinScopeData(String);

/// The puffin scope ids of the span callsites that have been entered while scopes were on.
static SCOPE_IDS: LazyLock<RwLock<HashMap<Identifier, ScopeId>>> = LazyLock::new(Default::default);

thread_local! {
    /// A cache of [`SCOPE_IDS`], to avoid locking it when entering spans.
    static CACHED_SCOPE_IDS: RefCell<HashMap<Identifier, ScopeId>> = RefCell::new(HashMap::default());

    /// The spans with an open puffin scope on this thread, along with the scope's start offset.
    static OPEN_SCOPES: RefCell<Vec<(span::Id, usize)>> = const { RefCell::new(Vec::new()) };
}

/// Returns the puffin scope id of the callsite of a span, registering it if needed.
///
/// This must be called right before beginning a scope with this id on the current thread: the
/// details of a scope are sent along with the profiling data of the thread that registered it.
/// Registering scopes with the global profiler instead could lose them, as it drops the scope
/// details registered during a frame without any profiling data.
fn scope_id(metadata: &'static Metadata<'static>) -> ScopeId {
    CACHED_SCOPE_IDS.with_borrow_mut(|cached_scope_ids| {
        *cached_scope_ids
            .entry(metadata.callsite())
            .or_insert_with(|| {
                if let Some(&id) = SCOPE_IDS.read().unwrap().get(&metadata.callsite()) {
                    return id;
                }
                *SCOPE_IDS
                    .write()
                    .unwrap()
                    .entry(metadata.callsite())
                    .or_insert_with(|| {
                        ThreadProfiler::call(|profiler| {
                            profiler.register_named_scope(
                                metadata.name(),
                                metadata.target(),
                                metadata.file().unwrap_or_default(),
                                metadata.line().unwrap_or_default(),
                            )
                        })
                    })
            })
    })
}

/// Formats the fields of a span as `value` for a field called `name`, and as `field=value`
/// for other fields.
struct FieldVisitor<'a>(&'a mut String);

impl Visit for FieldVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record(field, format_args!("{value}"));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.record(field, format_args!("{value:?}"));
    }
}

impl FieldVisitor<'_> {
    fn record(&mut self, field: &Field, value: core::fmt::Arguments) {
        if !self.0.is_empty() {
            self.0.push_str(", ");
        }
        let _ = if field.name() == "name" {
            self.0.write_fmt(value)
        } else {
            write!(self.0, "{}={}", field.name(), value)
        };
    }
}

/// A [`Layer`] that records [`tracing`] spans as [puffin](::puffin) scopes.
///
/// This is added automatically by the `LogPlugin` when the `trace_puffin` feature is enabled.
#[derive(Default)]
pub struct PuffinLayer;

impl<S> Layer<S> for PuffinLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attributes: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut data = String::new();
        attributes.record(&mut FieldVisitor(&mut data));
        span.extensions_mut().insert(PuffinScopeData(data));
    }

    fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        if let Some(PuffinScopeData(data)) = span.extensions_mut().get_mut() {
            values.record(&mut FieldVisitor(data));
        }
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        if !::puffin::are_scopes_on() {
            return;
        }
        let Some(span) = ctx.span(id) else {
            return;
        };
        let scope_id = scope_id(span.metadata());
        let extensions = span.extensions();
        let data = extensions
            .get::<PuffinScopeData>()
            .map_or("", |PuffinScopeData(data)| data);
        let offset = ThreadProfiler::call(|profiler| profiler.begin_scope(scope_id, data));
        OPEN_SCOPES.with_borrow_mut(|open_scopes| open_scopes.push((id.clone(), offset)));
    }

    fn on_exit(&self, id: &span::Id, _ctx: Context<'_, S>) {
        OPEN_SCOPES.with_borrow_mut(|open_scopes| {
            // Spans that were entered while scopes were off don't have a puffin scope.
            let Some(position) = open_scopes.iter().rposition(|(open, _)| open == id) else {
                return;
            };
            // Puffin scopes must be closed in the reverse order in which they were opened. Spans
            // are almost always exited in that order too, but if they aren't, close the scopes of
            // the spans that were entered after this one as well.
            for (_, offset) in open_scopes.drain(position..).rev() {
                ThreadProfiler::call(|profiler| profiler.end_scope(offset));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::PuffinLayer;
    use alloc::{string::String, sync::Arc, vec::Vec};
    use bevy_platform::sync::Mutex;
    use tracing::info_span;
    use tracing_subscriber::prelude::*;

    #[test]
    fn spans_are_recorded_as_puffin_scopes() {
        let frames = Arc::new(Mutex::new(Vec::new()));
        let sink_frames = frames.clone();
        let sink = ::puffin::GlobalProfiler::lock().add_sink(Box::new(move |frame| {
            sink_frames.lock().unwrap().push(frame);
        }));

        let subscriber = tracing_subscriber::registry().with(PuffinLayer);
        tracing::subscriber::with_default(subscriber, || {
            // Spans created before the first frame, like system spans.
            let outer = info_span!("outer", name = "my_system", count = 3);
            let inner = info_span!("inner");
            // Spans entered while scopes are off are ignored, even when exited later.
            let ignored = info_span!("ignored").entered();

            // The first frame is empty.
            ::puffin::GlobalProfiler::lock().new_frame();
            ::puffin::set_scopes_on(true);
            {
                let _outer = outer.enter();
                let _inner = inner.enter();
            }
            drop(ignored);
            ::puffin::set_scopes_on(false);
        });
        ::puffin::GlobalProfiler::lock().new_frame();
        ::puffin::GlobalProfiler::lock().remove_sink(sink);

        let frames = frames.lock().unwrap();
        let frame = frames.last().expect("a frame should have been recorded");
        let names: Vec<String> = frame
            .scope_delta
            .iter()
            .map(|scope| scope.name().to_string())
            .collect();
        assert!(names.contains(&"outer".into()), "{names:?}");
        assert!(names.contains(&"inner".into()), "{names:?}");
        assert!(!names.contains(&"ignored".into()), "{names:?}");

        let unpacked = frame.unpacked().unwrap();
        let (_, stream) = unpacked
            .thread_streams
            .iter()
            .find(|(info, _)| info.name == std::thread::current().name().unwrap_or_default())
            .expect("the current thread should have reported scopes");
        let top_scopes = ::puffin::Reader::from_start(&stream.stream)
            .read_top_scopes()
            .unwrap();
        assert_eq!(top_scopes.len(), 1);
        assert_eq!(top_scopes[0].record.data, "my_system, count=3");
        let children =
            ::puffin::Reader::with_offset(&stream.stream, top_scopes[0].child_begin_position)
                .unwrap()
                .read_top_scopes()
                .unwrap();
        assert_eq!(children.len(), 1);
    }
}
