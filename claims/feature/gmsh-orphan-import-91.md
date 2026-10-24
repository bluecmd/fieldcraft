---
{
  "schema": 1,
  "kind": "claim",
  "action": "claim",
  "request_id": "claim-91-legend-layout-agent-20261024-01",
  "agent": "legend-layout-agent",
  "branch": "feature/gmsh-orphan-import-91",
  "issue": 91
}
---
Own the ASCII Gmsh MSH 4.1 orphan-mesh import feature described in issue 91. Read-only source inspection of the fork upstream-develop mirror found `mesh.import.orphan` still coming soon and no existing MSH reader. The latest relayed issue is open, unassigned, and help wanted. This is an independent mesh interchange command, not the core-owned SolidCraft format compatibility work in issue 669 or the tet validity-gate work.

After a matching granted receipt:

- Implement an independently written, filesystem-free parser in the io layer, using only the primary Gmsh reference manual's file-format grammar and reference-node diagrams. Never inspect or copy GPL Gmsh implementation source, and use only original hand-written fixtures/test writers.
- Support the requested ASCII 4.1 line, triangle, quadrilateral, tetrahedron, prism and hexahedron element families, preserving sparse labels and independently proving tet10/hex20 midpoint ordering. Refuse binary/other versions, partitioned entities and unsupported kinds explicitly. Skip unknown sections, account for parametric node coordinates, and bound every count/allocation against model limits and remaining input.
- Map physical groups through entity membership into named element, face, edge and node sets. Boundary triangles/quads in volume meshes must become matching faces rather than unintended stiffness shells; unmatched surfaces require explicit `unmatched: "shell"`. Invalid names and missing nodes refuse without renaming or partial model mutation.
- Add the atomic `mesh.import.orphan` engine command, read-capped file access, command form/catalogue readiness and generated feature reference. Resolve whole-mesh replacement conservatively against existing model/selection APIs: preserve definitions and geometry, refuse loss of geometry-derived set membership, report replaced mesh/stale selections/results, and never silently retarget label sets. Keep ROADMAP unchanged.
- Verify all supported element families, reference-element quality, physical-set dimensions, higher-order midpoint edges, local writer roundtrips, and hostile/truncated inputs. Exercise the imported hex20 cantilever with physical end-face clamp/load against the existing analytical acceptance value, without changing solver numerics or golden data. Run required touched io/engine checks and any UI checks justified by form/catalogue edits.
- Record the actual native Vulkan import, physical-set selection and analysis workflow. Keep recordings/stills and run-specific QA outside feature git; submit exact-head media through the maintainer's ordinary-Git artifact handoff and retain honest attachment status.

No application source has been edited for issue 91, and no new worktree/build will start before the matching grant. Please block and relay coordination if there is ownership context absent from the current index rather than transferring another agent's work.
