# Investigation: AT-SPI property caching crashed Qt apps (issue #6)

**Status:** Resolved — decision: keep property caching **off** (it is the correct design,
not a workaround). The crash itself is an upstream Qt AT-SPI bridge bug.

## Symptom

While building the Phase-0 spike, reading the focused element used the raw zbus proxy:

```rust
AccessibleProxy::builder(conn.connection())
    .destination(sender)?.path(path)?.build().await?   // caching left at the default
```

Querying focused Qt apps (`kdialog`, `kcalc`) then **SIGSEGV-ed the app** — it started,
registered on the a11y bus, emitted a focus event, then crashed; our follow-up query hit a
dead connection (`org.freedesktop.DBus.Error.NoReply` / `ServiceUnknown`). GTK apps were
unaffected. Setting `cache_properties(CacheProperties::No)` fixed it completely.

## Root cause

1. **zbus caches properties by default.** A zbus proxy built without `cache_properties(No)`
   eagerly calls `org.freedesktop.DBus.Properties.GetAll("org.a11y.atspi.Accessible")` on the
   target object at build time (and subscribes to `PropertiesChanged`).
2. **AT-SPI's `Accessible` properties are not `org.freedesktop.DBus.Properties`-cache-friendly.**
   They have no `PropertiesChanged` signal — AT-SPI surfaces changes through its *own* event
   model (`object:property-change`, `object:state-changed`, …), not the standard D-Bus
   Properties signal. So caching would also serve **stale** values even if it didn't crash.
3. **The Qt AT-SPI bridge crashes on that `GetAll`.** The GNOME/AT-SPI client ecosystem
   fetches accessible properties individually (or via the registry cache), so a `GetAll` over
   the whole `Accessible` interface is an under-exercised path in Qt's bridge. That path
   SIGSEGVs here (Parrot 7 / Plasma 6 / Qt 6). It reproduces only by crashing the app, so we
   did not pursue a live repro.

## Authoritative evidence: the atspi crate already does this

The `atspi` crate **never caches** AT-SPI proxy properties — every proxy constructor sets
`CacheProperties::No`:

- `atspi-proxies-0.14.0/src/proxy_ext.rs` — all interface proxies (`action`, `application`,
  `component`, `text`, `value`, …) build with `.cache_properties(zbus::proxy::CacheProperties::No)`.
- `atspi-proxies-0.14.0/src/accessible.rs:301,323` — the accessible-proxy helpers do the same.

Our crash came from bypassing those helpers with a raw `AccessibleProxy::builder(...).build()`,
which inherits zbus's caching-on default. Using `cache_properties(No)` simply restores the
crate's intended behaviour.

## Conclusion & recommendation

- **Do not re-enable property caching.** It is wrong for AT-SPI (stale data; no
  `PropertiesChanged`) *and* triggers the Qt-bridge crash. `cache_properties(No)` is the
  correct, permanent design — matching the `atspi` crate's own convention. Issue #6's
  "safely re-enable caching" goal is therefore resolved as **won't re-enable, by design**.
- **Optional hardening:** prefer the crate's idiomatic constructors / `ProxyExt` over the raw
  `AccessibleProxy::builder` so the no-cache setting can't be dropped accidentally. Our
  explicit `cache_properties(No)` already achieves the same; this is a tidiness nicety.
- **Upstream:** the SIGSEGV in Qt's AT-SPI bridge on `Properties.GetAll` for the `Accessible`
  interface looks like a genuine Qt bug worth reporting to KDE/Qt if it can be reproduced
  safely (e.g. with `busctl --user call … org.freedesktop.DBus.Properties GetAll …` against a
  Qt app's accessible object). Out of scope for oxeye; noted for whoever has a throwaway
  session.

## References

- atspi crate — crates.io/crates/atspi; source: `atspi-proxies` `proxy_ext.rs` / `accessible.rs`
  (`CacheProperties::No` throughout).
- zbus property caching — docs.rs/zbus (proxy `cache_properties`, `CacheProperties`).
- AT-SPI2 event model — wiki.freedesktop.org/www/Accessibility/AT-SPI2/ (property changes via
  AT-SPI events, not `org.freedesktop.DBus.Properties.PropertiesChanged`).
- KDE Qt AT-SPI bridge — community.kde.org/Accessibility/qt-atspi.
