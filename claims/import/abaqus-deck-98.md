---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-98-contour-animation-agent-20261024-reader",
  "agent": "contour-animation-agent",
  "branch": "import/abaqus-deck-98",
  "issue": 98
}
---
Request the complete explicitly stated first-PR scope of #98: a pure-text, wasm-compatible Abaqus model-data reader in fieldcraft-decks and the normal atomic `deck.read` command/catalogue/form. Read nodes/elements with labels, node/element sets, element-based surfaces, elastic/density materials, solid/shell/2D/truss sections, and the beam-section profiles/orientations the current writer emits. Invert all current writer-table element/formulation mappings (including pyramid5), preserving connectivity and refusing mixed formulations by section name rather than guessing.

Bounded lexer handles case-insensitive keywords/names, quoted names, comments, blank lines and connectivity continuation. Resolve forward set references with explicit bounded expansion/cycle detection. Report each unread keyword with its original line (bounded command output plus total count); retain no raw unknown/history text in Model. Refuse INCLUDE, PART/ASSEMBLY/INSTANCE, unsupported element types, missing referenced labels, invalid material context, duplicate/conflicting names, bad shapes/labels, nonfinite numbers and oversized lines/files/expansions with actionable line numbers. Unknown variants must not silently become supported default physics.

Use model checked setters and central Quantity/unit conversion to store SI; make input unit assumptions explicit through command parameters/defaults rather than infer units from magnitudes. Read through existing confined Access::Read and capped native IO; construct and validate a replacement model before mutating the session, then clear incompatible results/history through the normal replacement path. Preserve the previous model on every parse/validation/read failure. Follow the current maintainer policy for Quantity and ROADMAP rather than the stale issue's requested ROADMAP edit.

Reviewable buildable commits: (1) independent lexer/parser/model mapping with writer-generated and original handwritten regression fixtures; (2) atomic engine command, form/catalogue/docs and process/hostile tests; fold fixes into their owning layer. Test writer/read/writer supported mesh/set/surface/material/section roundtrips, all element/formulation mappings, units, a read/reconstrained cantilever solve, unknown reporting and each refusal/truncation/cycle/large-range hostile input. No commercial CAE program, output deck, manual example, external model or GPL implementation is used. Consult only public primary keyword/element documentation and the existing original writer/model mappings.

No history/loads/steps import, unknown-keyword passthrough storage/re-emission, assemblies/instances/includes, other deck formats, solver algorithm, kernel pin or ROADMAP changes. These remain separate work as the issue explicitly states. Capture a real native imported-model/unknown-report demonstration on the final tested source and transfer media through the shared artifact handoff, outside feature history. Run touched decks/engine tests and all-target clippy, fmt, targeted wasm and command-document regeneration; no full local workspace gate.

No matching owned claim was found during read-only triage and issue98 is unassigned in the relayed index. Wait for the normal matching granted receipt and read the full relayed discussion before application coding. Reject or reconcile explicitly if another PR/owner overlaps this reader scope.
