---
title: Paseo plugin API
date: 2026-10-20
tags: [digest, paseo, plugins, extensibility, ade]
type: digest
source:
  - https://github.com/getpaseo/paseo
  - https://github.com/getpaseo/paseo/blob/main/docs/plugins.md
  - https://github.com/getpaseo/paseo/blob/main/public-docs/plugins/reference.md
  - https://github.com/getpaseo/paseo/blob/main/public-docs/plugins/providers.md
  - https://paseo.sh/docs/plugins/reference
  - https://paseo.sh/plugins
  - https://github.com/getpaseo/plugins/blob/main/PROTOCOL.md
  - https://x.com/moboudra/status/2094786865970962569
  - https://x.com/moboudra/status/2110803767021211901
---

# Paseo plugin API

Related: [[2026-10-12 Paseo multi-platform and plugin API]] (manifest, compile-on-daemon, client `eval`, hook list, distribution, first-party plugins; not repeated in detail) · [[2026-10-20 Paseo daemon-app API]] (wire protocol the plugin bundles ride on) · [[2026-10-10 Paseo]] · [[2026-09-29 OpenCode2 plugin development]] · [[2026-09-28 Minimal orchestrator core Web UI plugin]] · [[2026-10-20 ADE landscape]]

**Source pin:** `getpaseo/paseo` `main` @ `068f272a00ffef3f00ec30ec4cfa3a631be97624` (2026-10-20 17:26 UTC+8), **0.12.0**. `@getpaseo/plugin` is 0.12.0, versioned in lockstep. Clone `/tmp/rnr/paseo`, diffed against the 10-12 pin `06f0522` (0.11.2).

## TL;DR

- **A plugin is a directory with `paseo-plugin.json` plus up to two entries**: `index.server.ts` runs in a daemon-owned Node subprocess and gets `PluginServerContext`; `index.client.tsx` runs inside every connected app and gets `PluginClientContext`. Each entry default-exports `contribute(ctx) => cleanup`. All other code must live in `client/`, `server/` or `shared/`, and the esbuild compiler on the daemon enforces this.
- **The plugin reaches both sides over the normal protocol.** The client bundle is a string in `plugin.catalog.get.response`. Client→server calls are `plugin.rpc.invoke.request {pluginId, method, input}`, typed by a shared `defineRpc` zod contract that is validated on both ends. The subprocess itself is a protocol client: an exclusive `plugin:<id>` WebSocket session tunneled over IPC (`paseo_frame`), giving it the same `PaseoApi` the app has.
- **Extension points:** 15 client registrations (screens, sidebar header/footer, workspace/agent panels, Command Center, slash commands, header buttons, composer pills, attachment sources, themes, timeline transformers/renderers, settings screens, `openPanel`, `playAudio`). Server side: `handle`, `registerSettings`, `registerProvider`, `registerUsageSource`, plus 9 `on` events and 3 `before` transforms (12 hooks; the docs still say "all eleven hooks", but `lifecycle.ts` and `plugin-examples/lifecycle-logger` both have 12). MCP is not an extension point of its own; you inject MCP servers into agents via `before("agent.create")`.
- **No sandbox, no permission manifest.** "Plugin code is trusted and unsandboxed." The server `require` falls through to Node's `require` for anything not host-provided. Isolation is only per-plugin process (crash containment). Built-ins run **in-process** through the same worker protocol (`InternalPluginChild`).
- **Changed since 0.11.2:** `agent.turn_ended.timeline` is now deprecated and always `[]` (`068f272`, #7477, today). Icons accept inline `{ svg }` or components (`b6dde61`, #7368). `ProviderRegistration.requiresConfiguration` is new and the built-in `hoplite-provider` uses it (#6953, 0.11.3). Plugin reload waits instead of timing out (#6306). Type-only imports of `@getpaseo/protocol` work without local deps (#7023). The gallery grew to **326 plugins by 132 authors** (from 235/73 on 10-12).

## 1. What a plugin is and how it is laid out

```text
my-plugin/
  paseo-plugin.json      # manifest (required)
  index.client.tsx       # PluginClientContext → app, per client   (optional)
  index.server.ts        # PluginServerContext → daemon subprocess (optional; ≥1 entry required)
  client/  server/  shared/   # only places other modules may live
  package.json tsconfig.json  # dev-only: @getpaseo/plugin etc. for typechecking
```

Manifest schema (`packages/server/src/server/plugins/manifest.ts`, unchanged since 10-12):

```ts
const PluginManifestSchema = z.object({
  id: PluginIdSchema,
  name: z.string().trim().min(1).optional(),
  description: z.string().trim().min(1).optional(),
  icon: PluginAssetPathSchema.refine((v) => /\.png$/i.test(v), "Expected a relative path to a PNG").optional(),
  media: z.array(z.union([PluginAssetPathSchema, z.url({ protocol: /^https$/ })])).optional(),
  // A misspelled requirement must not silently disable compatibility checks.
  requirements: PluginRequirementsSchema.strict().optional(),
  build: z.array(PluginBuildCommandSchema).min(1).optional(),   // argv arrays, no shell
});
```

The manifest has no capabilities, permissions, contribution list or activation events. Contributions are discovered only by running the entry. An omitted `requirements.paseo` means `<0.8.0` and is rejected. IDs everywhere (plugin, screen, panel, command…) match lowercase-hyphen.

## 2. Discovery, loading and runtime

**Sources** (`docs/plugins.md`): directory (`paseo plugin install /abs/path`), registry `owner/slug` or `host/owner/slug` (reviewed pinned artifact), `git:`/`github:owner/repo[:subpath]` (+`--ref` once), and npm. They are stored under `plugins.<runtimeId>` in `$PASEO_HOME/config.json`, and the global switch is `pluginsEnabled`. "Only the daemon reads plugin files and acquires sources." `ManagedPluginSources` stages Git/npm. npm lifecycle scripts are disabled; the manifest's `build` argv runs instead, logged to `daemon.log`. A failed candidate leaves the running revision untouched. Updates are preview/apply (`plugin.source.update.preview.request` / `.apply.request`), and apply re-validates the previewed target.

**Registry protocol v1** (`getpaseo/plugins/PROTOCOL.md`): static HTTPS JSON. The daemon reads only `/plugins/<owner>/<slug>.json` and uses only `id` + `artifact` (e.g. `{kind:"git", remote, commit, pluginPath}`). It fetches with redirects disabled. `index.json` is for gallery clients only. Private registries take `pluginRegistries.<host>.authorization`. This confirms `github.com/getpaseo/plugins` as the official registry repo (open in the 10-12 digest).

**Compile:** `packages/server/src/server/plugins/compiler.ts` (esbuild) builds one bundle per entry and enforces boundaries: no `client/` in server, no `server/` or `node:*` (including bare `fs`) in client, no React in server, nothing at the root. "Forbidden imports fail compilation; never replace them with empty module stubs." `.svg` imports load as text.

**Server runtime** (`runtime.ts`, `plugin-process.ts`, `plugin-process-protocol.ts`, `bundle-evaluator.ts`):

```ts
// runtime.ts
return fork(fileURLToPath(resolveWorkerUrl()), [], {
  execArgv: resolveWorkerExecArgv(),       // --experimental-strip-types + a TS loader
  serialization: "advanced",
  stdio: ["ignore", "pipe", "pipe", "ipc"],
});
// bundle-evaluator.ts: host instances for @getpaseo/plugin{,/server,/server/provider,/server/acp,/server/usage}
// and zod; client-only specifiers throw; everything else → nodeRequire(name)
```

- IPC is a strict zod union. Daemon → worker: `initialize`, `invoke`, `hook`, `hook.cancel`, `provider.{status,catalog_key,connect,send,close}`, `usage.{fetch,discover}`, `settings.changed`, `shutdown`, `paseo_frame`, `paseo_close`. Worker → daemon: `ready`, `result`, `error`, `fatal`, `hooks.changed`, `provider.{connected,connect_failed,accepted,rejected,event,closed}`, `paseo_frame`, `paseo_close`.
- **Built-ins** (`packages/server/src/server/plugins/builtin/index.ts`: `antigravity-provider`, `hoplite-provider`, `muse-provider`, nine `*-usage-source`) "run in process and remain active independently of `pluginsEnabled`". `InternalPluginChild` (`internal-child.ts`) is a fake child: same worker, messages are `structuredClone`d and delivered on a microtask. This was already true at 0.11.2; the 10-12 digest didn't mention it.
- **Crash handling:** `handleChildClose` removes the plugin from the catalog, rejects pending RPCs and provider sends, and notifies. I found **no automatic respawn** (code reading; docs say "A failed reload stays failed; Paseo does not restore the old code").
- **Startup ordering:** "plugin sessions may connect while application WebSockets remain paused; the daemon accepts clients only after configured plugins have settled and the initial catalog is complete."

**Client runtime** (`packages/app/src/plugins/evaluate.ts`, `registry.ts`): `globalThis.eval(bundle)` with a fixed `runtimeRequire`: `react`, `react/jsx-runtime`, `react-native`, `zod`, `@tanstack/react-query`, `@getpaseo/plugin`, `/client` (augmented with `useSettings`, `openExternalUrl`, `getPaseoClient(serverId)`, `useHosts`), `/client/react-native`, `/client/ui`. Anything else throws ``Module "<name>" is not available in plugin client code``. Each installation gets its own React Query client and one `InstalledPlugin.paseo` client over the host's existing connection. The app checks `requirements.paseo` against **its own** version before evaluating.

## 3. Plugin-facing types

```ts
// packages/plugin/src/server/contracts.ts
export interface PluginServerContext extends PluginLifecycleRegistration {
  registerSettings<Schema extends ZodType>(definition: SettingsDefinition<Schema>): PluginSettings<Schema>;
  handle<I extends ZodType, O extends ZodType>(contract: PluginRpcContract<I, O>,
    handler: (input: ZodOutput<I>, context: PluginHandlerContext) => ZodInput<O> | Promise<ZodInput<O>>): void;
  registerProvider(provider: ProviderRegistration): void;
  registerUsageSource(source: UsageSourceRegistration): void;
}
export interface PluginHandlerContext { paseo: PaseoApi }
```

```ts
// packages/plugin/src/client/contracts.ts (abridged)
export interface PluginClientContext extends PluginCommandCapabilities, PluginClientContextAliases {
  playAudio(source: { base64: string; mimeType: string }): Promise<void>;
  addSettingsScreen(c): PluginCleanup;   addScreen(c): PluginCleanup;
  addSidebarHeaderItem(c): PluginCleanup; addSidebarFooterItem(c): PluginCleanup;
  addWorkspacePanel(c): PluginCleanup;   addCommandCenterItem(c): PluginCleanup;
  addSlashCommand(c): PluginCleanup;     addAttachmentSource(c): PluginCleanup;
  addHeaderButton(c): PluginButtonRegistration;  addComposerPill(c): PluginButtonRegistration;
  addTheme(c): PluginCleanup;
  addTimelineTransformer<T extends AgentTimelineItem["type"]>(c): PluginCleanup;
  addTimelineRenderer<S extends ZodType>(c): PluginCleanup;
  openPanel(id: string, options: PluginClientOpenPanelOptions): void;
}
// COMPAT(pluginSidebarAliases): added in v0.11.0, remove after 2027-03-29 → addSurface, addSidebarItem
```

```ts
// packages/plugin/src/rpc.ts
const RPC_NAME = /^[a-z][a-z0-9._-]*$/;
export function defineRpc(definition) { /* trims, validates name */ }
export async function callPluginRpc(contract, invoke, input) {
  const parsedInput = await contract.input.parseAsync(input);
  const output = await invoke(contract.name, parsedInput);
  return contract.output.parseAsync(output);
}
```

SDK entry points (`packages/plugin/package.json` exports): `.` (shared: `defineRpc`, `defineSettings`, icons, attachments), `./server`, `./server/provider`, `./server/acp`, `./server/usage`, `./client`, `./client/react-native`, `./client/ui`, and `./client/host` ("App-owned rendering integration; not a plugin-author entry"; the server evaluator throws "private to the app host").

## 4. How a plugin reaches daemon and app

| Path | Wire |
|---|---|
| Daemon → app: UI code | `plugin.catalog.get.response { plugins: [{ id, clientBundle: string, requirements? }] }`; gated by `server_info.features.plugins` |
| App → plugin backend | `plugin.rpc.invoke.request { requestId, pluginId, method, input: unknown }` → `plugin.rpc.invoke.response`; daemon forwards as IPC `invoke` |
| Plugin backend → Paseo | Own WS session `plugin:<id>` over IPC (`PluginSessionSocket` in `session-socket.ts` wraps frames as `{type:"paseo_frame", data, isBinary}`); `websocket-server.ts` rejects any other socket claiming a plugin clientId ("Invalid plugin clientId") |
| Plugin backend → chat | `paseo.agents.ref(id).timeline.append({ type:"plugin", id, kind, version, data })` → `agent.timeline.append.request`; daemon stamps `pluginId`, ≤64 KiB `data`, in-memory only (lost on daemon restart); gated by `features.pluginTimelineItems` |
| Settings | `defineSettings({ id, scope: "host", version, schema, migrate? })` + `settingsRpc(id)` → `settings.<id>.read|write` RPCs; storage keyed by runtime installation ID, outside the source dir |

Multi-host: plugins are installed **per daemon**. If several hosts expose the same contribution, the app shows one sidebar item plus a host picker, and "Calls never fall through to another host when the selected host is offline." Themes coalesce via `pluginThemeId = <pluginId>/theme/<themeId>`; attachment sources stay scoped to the composer's host.

## 5. Extension points in more depth

- **Lifecycle hooks** (`packages/plugin/src/server/lifecycle.ts`). `server.on`: `agent.created`, `agent.closed`, `agent.turn_started`, `agent.turn_ended`, `agent.permission_requested`, `agent.permission_resolved`, `agent.archived`, `workspace.created`, `workspace.archived`. `server.before`: `agent.create` (editable: public config except `cwd`, plus `env`), `agent.session_open` (only `env`; `reason: create|resume|refresh|import`, `purpose: interactive|history`), `workspace.create` (whole request). Contract (reference "Context and cleanup"): **30 s timeout** aborting `signal`, before-hook error fails the operation, event errors are only logged, "Live, best effort; no replay, persistence, or automatic retry", order = plugin-ID then registration order, no deep merge. Hooks "must not be awaited inside agent mutations", which would deadlock. `PluginHookAgent {id, workspaceId, parentAgentId, provider, cwd, title}` and `PluginTurnOutcome` are discriminated `completed|failed|canceled`.
  - **Change today (#7477):** `agent.turn_ended` carries `deprecated timeline: []`. Fetch with `context.paseo.agents.ref(id).timeline.refetch({ direction: "tail", limit: 20 })`. Plugins written against 0.11 that read `event.timeline` silently get nothing.
- **Agent providers** (`@getpaseo/plugin/server/provider`): `ProviderRegistration { id, label, description?, icon? (SVG path), command?, status?({launch}), connect(request), getCatalogCacheKey?, requiresConfiguration? }`. `connect` returns `ProviderConnection { version, capabilities, send(input), onEvent(listener), close() }`, a message protocol with `PROVIDER_PROTOCOL_VERSION = 1` and negotiated `PROVIDER_CAPABILITIES` (`prompt.{message,command,image,output_schema,steer}`, `session.{archive,configure,list,persistence,revert.*,subsession,unarchive}`, `permission`, `permission.tool_policy`, `timeline.plugin`) via `negotiateProviderCapabilities`. Inputs: `session.{open,prompt,interrupt,configure,permission,revert,archive,unarchive,close}`, `catalog`, `sessions`. Events include `session.opened` (with `parentSessionId`, `toolCallId` for subagents), `timeline.item`, `session.turn`, `session.permission`, `session.persistence`, `session.usage`, `request.completed|failed`. The daemon resolves `agents.providers.<id>.command/env`, sanitizes env, and hands `launch {command,args,env}`: "do not merge the plugin process's environment back in". There are ACP shortcuts (`runAcpProvider`, `/server/acp` transformers). Built-in omp/Pi/Claude/Codex are **not** plugins (`packages/server/src/server/agent/providers/{omp,…}`).
- **MCP:** no `registerMcpServer`/tool API in the SDK (`rg registerTool packages/plugin/src` finds nothing). The documented route is `before("agent.create")` adding `config.mcpServers.<name> = { type: "stdio"|"http"|"sse", … }` (`ProviderMcpServerConfig`); "The selected provider … must support the configured MCP servers."
- **UI contributions:** see TL;DR. Notable contracts: the host owns route/header/error boundary while the plugin owns the body; screen params live in the route query (`param.` prefix); panels declare `locations: ["workspace","explorer"]`. Panel props are frozen, readonly snapshots (no store access, no Expo Router). Slash commands never send text to the agent. Built-ins win name clashes and plugins beat provider commands. Timeline transformers are synchronous and deterministic, receive accumulated text with `phase: "streaming"|"complete"`, and are memoized by source-item reference. Icons are Lucide name, `{ svg }` (validated once by `findSvgIconProblem` in `packages/plugin/src/icons.ts`), or a component. Attachment sources need `PluginSerializableIcon`.
- **Commands:** Command Center items (global/workspace/agent) and slash commands. Callbacks get `paseo`, `rpc`, `openScreen`, `openPanel`, and lazy context snapshots.
- **Settings:** host-scoped only (`defineSettings` throws otherwise), versioned with `migrate`, server `read()`/`subscribe()`. "Settings-change notifications must not enter the catalog reload path."
- **Usage sources:** `registerUsageSource` (`/server/usage`); used by the nine built-in `*-usage-source` plugins.
- **Processes:** `spawnProcess`, `execCommand`, `terminateProcess` from `@getpaseo/plugin/server` (`process.ts`).

## 6. Permissions, isolation, versioning, stability

- **Permissions:** none declared or enforced. Gates are human: global `pluginsEnabled`, per-plugin enable, registry review, update preview. `docs/plugins.md` tells contributors and agents: "Never enable plugins on a user's behalf without explicit permission." The plugin's daemon session gets owner-level access as far as I can tell. I didn't find a reduced principal for `plugin:<id>` sessions; unverified.
- **Isolation:** separate Node process per non-built-in plugin; built-ins share the daemon process. Client code runs in the app's JS realm with host React. A render error is caught by the plugin error boundary, but there is no realm isolation.
- **Versioning:** `requirements.paseo` semver range, checked by the daemon (install/prepare/load/startup/enable/reload) and by **each app** separately. Prereleases match their stable core. Daemon features gate SDK areas: `plugins`, `pluginManagement`, `pluginLogs`, `pluginGitManagement`, `pluginSourceInstallation`, `pluginSourceUpdates`, `pluginThemes` ("A daemon that predates this flag keeps `addTheme` in the server bundle it compiles, so a theme plugin cannot start there at all"), `pluginSettings`, `pluginTimelineItems`. Client cap: `plugin_timeline_items`.
- **Stability signals.** Breaking eras: 0.8 (single `index.ts` → split entries, `public-docs/plugins/migration.md`) and 0.11 (`owner/repo` = registry). Deprecated but live: `addSurface`/`addSidebarItem` (remove after 2027-03-29), `turn_ended.timeline` (now empty). `client/host` is explicitly not for authors. The SDK has no separate semver track, and the docs advise "Add an upper bound when a later release is incompatible; a minimum alone does not promise protection from future breaking changes." The stable-feeling surface is RPC contracts, hooks, `PaseoApi`, screens/panels/commands, and providers; churn shows up in sidebar/screen naming, icon forms, and hook payloads.

## 7. Examples and ecosystem

- Official examples (`plugin-examples/`, 14): `agent-configuration`, `buttons`, `catppuccin`, `hosts`, `inline-thinking`, `lifecycle-actions`, `lifecycle-logger` (registers all 12 hooks; docs say eleven), `linear` (attachment source), `local-plugin`, `modal-ui`, `provider-acp-transformer`, `provider-direct`, `settings`, `timeline-items`. First-party plugins: `plugins/` (12 built-ins incl. new `hoplite-provider`, manifest `{"id":"hoplite-provider","requirements":{"paseo":">=0.11.0"}}`, opt-in via `requiresConfiguration`).
- Gallery (paseo.sh/plugins, 2026-10-20 via Exa): "326 plugins by 132 authors". Featured: Zerosub (multi-subscription switching), Cafe (in-app plugin directory), Shell Yeah, Flowly. Most installed: Workspaces (411), Beautiful Chat (273), Skills (181).
- Maintainer (X, 2026-09-01): "extend the daemon and the clients (including the mobile app) with custom functionality and UI … do it all via chat and the app hot reloads in place … people are mostly building custom dashboards and integrations" ([post](https://x.com/moboudra/status/2094786865970962569)). 2026-10-15: Hoplite as the first sponsor, added from Settings → Providers ([post](https://x.com/moboudra/status/2110803767021211901)). Community: a sidebar-organizer plugin ([pedrovzsoares](https://x.com/pedrovzsoares/status/2109756350637179257)). HN (2026-10-14, [50097147](https://news.ycombinator.com/item?id=50097147)): "Look at Paseo and build your own backend and UI plugins while still using opencode." Reddit: nothing plugin-specific found.

## For ade

**Borrow**
- **Two entries + `shared/` contracts, enforced by the compiler** (`client/` / `server/` / `shared/`). With a zod protocol package already shared, a plugin's `shared/` RPC schemas slot in as `defineRpc({ name, input, output })` validated on both ends.
- **Daemon compiles and serves the client bundle over the existing socket.** Phones never install anything, and relay clients get plugins for free. Pair it with an explicit host-module allowlist (`react`, `react-native`, `zod`, `zustand`, your Uniwind/RNR kit) so plugins don't bundle their own React.
- **Plugin backend = an ordinary protocol client** with a reserved `plugin:<id>` identity over IPC. One API (`PaseoApi`) for app, CLI and plugins means no second plugin-only API to maintain.
- **`before` hooks that edit a typed request** (`agent.create` config/env/MCP, `session_open` env) with a fixed timeout and ordered, no-merge semantics. This is high leverage for few concepts. For omp-only ade, `before("agent.create")` adding MCP servers or omp extensions covers most "tool plugins".
- **Daemon-pushed plugin timeline rows** (`type:"plugin", kind, version, data` ≤64 KiB) plus **client renderers keyed by kind**, with an "unavailable" placeholder when the renderer is missing.
- **Reviewed static registry** (pinned commit/artifact, redirects off) that anyone can self-host.

**Avoid or do differently**
- **Capability-less manifest + unsandboxed `nodeRequire` fallback.** If ade will ever load third-party plugins, declare capabilities (fs roots, network, agent control, UI slots) in the manifest and give the `plugin:<id>` session a reduced permission set. Paseo's `DAEMON_PERMISSIONS` machinery exists but isn't applied to plugins as far as I found.
- **Hook payloads that copy history** (Paseo just had to empty `turn_ended.timeline`). Pass ids and let plugins fetch bounded pages.
- **Lockstep SDK versioning with only `>=` ranges** caused silent breakage (the `timeline` field). Version the plugin API separately (`apiVersion` in manifest) and treat removals as breaking.
- **Ad hoc aliases** (`addSurface`, `addSidebarItem`, `owner/repo` meaning changes). Settle names before opening the gallery.
- Since ade runs omp only, skip Paseo's generic provider-plugin protocol at first. A single omp adapter plus omp's own extension system is enough; revisit only if ade adds providers.

## Unverified / not checked

- Whether `plugin:<id>` sessions receive the full owner permission set (inferred from `OWNER_SESSION_ADMISSION` usage; not traced end to end).
- No automatic plugin restart after a crash (code reading of `runtime.ts` `handleChildClose`; not tested).
- Exact compile-time boundary checks: line numbers in `compiler.ts` were not re-read this time.
- Gallery counts and install numbers are a live snapshot (Exa fetch, 2026-10-20). Registry review process details beyond `PROTOCOL.md` were not read.
- Hub interaction with plugins (e.g. whether Hub-triggered agents fire plugin hooks) was not checked.

## Sources

- Code @ `068f272`: `packages/plugin/{package.json, src/rpc.ts, src/settings.ts, src/icons.ts, src/server/{contracts,lifecycle,provider,process}.ts, src/client/contracts.ts}`; `packages/server/src/server/plugins/{manifest,runtime,plugin-process-protocol,bundle-evaluator,internal-child,session-socket,builtin/index}.ts`; `packages/app/src/plugins/evaluate.ts`; `packages/protocol/src/messages.ts` (`PluginCatalogGetResponseSchema`, `PluginRpcInvokeRequestSchema`, `AgentTimelineAppendRequestSchema`, `server_info.features`); `plugins/hoplite-provider/`; `plugin-examples/`.
- Docs: `docs/plugins.md`; `public-docs/plugins/{reference,providers,migration,publishing,index}.md`; live https://paseo.sh/docs/plugins/reference (Exa; matches repo); https://paseo.sh/plugins; https://github.com/getpaseo/plugins/blob/main/PROTOCOL.md.
- Commits since the 10-12 pin: `068f272` (#7477), `b6dde61` (#7368), `e083890` (#7023), `74c8172` (#6306), `1839de5` (#6760), `16bb179` (#6953), `cafe4b0` (0.12.0). `CHANGELOG.md` 0.11.3, 0.11.4, 0.12.0.
- X `search_posts_all` (2 queries), HN Algolia (`query=paseo`), Reddit via Exa.
