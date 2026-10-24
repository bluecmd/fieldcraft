---
{
  "schema": 1,
  "kind": "pr",
  "action": "publish",
  "request_id": "pr-480-windows-dxc-agent-20261024-1",
  "agent": "windows-dxc-agent",
  "branch": "mcp/skill-prompts-480",
  "issue": 480,
  "head_sha": "c515f79ec5856e898154be9e238a139851d8a559",
  "base": "develop",
  "draft": false,
  "title": "MCP: serve exact bundled skills as discoverable prompts",
  "media": "attached"
}
---
## What and why

Closes #480. MCP clients currently receive an empty prompt catalogue. The server now advertises its immutable prompt capability and lists two stable, argument-free names: `fieldcraft-meshing` and `fieldcraft-modelling`. `prompts/get` returns the complete corresponding `SKILL.md` as its first user message, byte-identical to the compile-time source via `include_str!`. There is no separately maintained prompt copy and retrieval never executes model commands.

**Stacked dependency: [PR #718](https://github.com/tangency-labs/fieldcraft/pull/718), exact modelling-skill commit `443160e0bbc3c6a8f8310c1366bb6e593eb6eda6`.** This branch includes that already-reviewed four-file skill/recipe commit, followed by one five-file MCP integration commit. The dependency is unchanged; it must land before this integration. The feature-specific diff is from that predecessor, rather than a second implementation of the skill.

Three small modelling references are bundled as separate embedded resources and fixed `resources/list`/`resources/read` entries: checks.md and the original axial-rod/conductor JSON recipes. Their texts are also source-identical; the explicit URI map in docs/mcp.md makes them usable by a client without a source checkout. The existing model/command resources remain available. Names/URIs are a compile-time allowlist, with no arbitrary filesystem lookup, source-file access or automatic recipe execution. Unknown names, malformed/nonempty arguments and unusable continuation cursors return bounded protocol errors.

## Demonstration

The 24-second native-window clip shows a real connected MCP workflow. The new CLI first retrieves both skills and their references while the existing cantilever model remains unchanged (315 nodes/160 hex elements). The client then explicitly requests a separate tools/call batch to execute the retrieved conductor recipe, and the native app displays its actual temperature field and probe. Retrieval and model actions are deliberately separate; no in-app prompt selector is added or implied.

The connected original recipe uses 36 hex8 elements/84 nodes and gives midpoint 323.1499999999999 K and axial heat flux -7500.000000000019 W/m2, matching its simple steady-conduction calculation. This is a synthetic implementation check with assumed conductivity, not experimental validation, design accuracy, cooling-solid behavior or a coupled exchanger. The steady result is not presented as a physical transient. The solver is CPU; RTX 4090 Vulkan renders the viewport.

The CLI was built after the exact requested commit; its SHA-256 is `105724ad395812114beeb730dd356c57507c6453487e4a7bc24b8e5f6f6a556f`. The native host executable is reused from original build `99d8536`, SHA-256 `ff37388057d4eee4dd5490a4e927c45be6c20dcbf3fae4766dad2355d3a606c0`, with production source equivalent to `7d9c027d71b2c1ea3395e2552e95f0455a7244db`; it was not rebuilt at this MCP head. This change is in the new CLI/MCP server, not the viewport. The owned demo app was closed after capture.

`mcp-skill-prompts.mp4`, `mcp-model-unchanged.png` and `mcp-connected-conductor.png`, plus the small safe wire-qa.json, were normal-pushed as ordinary Git blobs by the coordinator in artifact-queue commit `bc8c32a560263e01351545be0a684dde68b76bf4`, under `objects/mcp/skill-prompts-480/c515f79ec5856e898154be9e238a139851d8a559/`. Index SHA-256 hashes and byte sizes were verified before the push. The maintainer receipts confirm all four artifacts received, with SHA-256 hashes and byte sizes matching their immutable manifests. All three native image/video artifacts are now attached at the verified GitHub URLs below; wire-qa.json remains report evidence in the inbox. Private raw protocol/capture logs and original reproduction/encoding scripts are retained externally; no demo binaries, raster images or LFS pointers are in feature history.

Native GitHub attachments (original capture revision `c515f79ec5856e898154be9e238a139851d8a559`, with the reused viewport/new CLI provenance described above):

- [mcp-skill-prompts.mp4](https://github.com/user-attachments/assets/4594ef85-25ad-40cd-8292-0d355a16fc11)
- [mcp-model-unchanged.png](https://github.com/user-attachments/assets/2b34560b-7df0-43f5-8171-87c457050d4e)
- [mcp-connected-conductor.png](https://github.com/user-attachments/assets/b64c6221-e6b6-4ebf-8cb7-a981fd3ebaff)

## Validation

- `cargo test -p fieldcraft-mcp`: all 15 unit tests and the corpus-part mesh/load/solve/render MCP integration test pass; zero failures/ignored tests. Doc tests: zero.
- Added parity regression enumerates every bundled skill directory and verifies its stable discovery name and exact source payload, including optional empty arguments, across protocol versions 2025-06-18, 2025-03-26 and 2024-11-05. It also confirms retrieval leaves model data unchanged.
- Added resource regression compares all three embedded and resources/read payloads with the source files, checks MIME/discovery and bounded sizes, and rejects arbitrary/traversal/file URIs. Hostile prompt-name/argument/cursor regressions verify protocol errors and continued server responsiveness.
- `cargo clippy -p fieldcraft-mcp --all-targets -- -D warnings`, `cargo check -p fieldcraft-mcp --target wasm32-unknown-unknown`, `cargo fmt --all -- --check`, and `git diff --check` pass. `cargo build -p fieldcraft-cli` passes at the requested commit.
- Actual newly built CLI wire QA runs from an external directory: all three negotiated versions in confined headless sessions, plus a real connected-app session at 2025-06-18. All return source-identical skills/references with equal SHA-256 values; unknown-name/argument/traversal checks pass and retrieval leaves the model unchanged. The connected case asserts it did not fall back headless before explicitly executing the recipe.
- Rebased on the current fork upstream-develop mirror before the normal feature push; rebase was a no-op. No solver code or numerical fixture changed. No broad local CI gate, numerical oracle, sustained GPU solve or non-Windows runtime check was run for this protocol change.

- Relayed CI on exact `c515f79`: main gate, heavy tests, oracle and dependency/licence/source checks succeeded; Windows app tests and the macOS oracle were skipped. No skipped job is reported as runtime verification.

## Known limitations

The modelling prompt depends on unmerged PR #718 as declared above. The protocol does not automatically choose or run a skill: clients expose user-selected prompts and explicitly invoke model tools afterward. Clients that do not render embedded resources can retrieve the three fixed URIs separately. Other repository documentation links still refer to the matching checkout; the server does not expose arbitrary repository files.

Native evidence is Windows-only, with the exact reused viewport provenance distinguished from the newly built CLI. It demonstrates actual protocol-driven model work, not a new desktop prompt-picker UI. Source parity is checked against the actual compiler-input checkout bytes, including its line endings. The synthetic conduction example does not establish engineering material properties, mesh convergence for a realistic part, or exchanger/cooling accuracy. Backend support, solver numerics and ROADMAP are unchanged.

The three native attachments are verified by maintainer receipts. The separate wire QA JSON remains inbox evidence; it is not a native GitHub video/image attachment.
