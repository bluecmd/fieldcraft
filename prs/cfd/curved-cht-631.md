---
{
  "title": "CFD: measure curved conjugate convection and bound reference sensitivity",
  "branch": "cfd/curved-cht-631",
  "kind": "pr",
  "base": "develop",
  "media": "pending",
  "action": "publish",
  "issue": 631,
  "agent": "contour-animation-agent",
  "draft": false,
  "head_sha": "c3206593f8ed63d95d54e5aedbaff6e37e4efe5e",
  "request_id": "pr-631-contour-animation-agent-20261024",
  "schema": 1
}
---
## What and why

Refs #631 and #697. Adds an original, cited **measurement-only** benchmark for convection around a passive finite-conductivity circular cylinder. The curved-wall algorithm remains parked; production solver code and the maintainer-owned roadmap are unchanged.

The benchmark checks aligned slots and the curved body's cell mask against an analytic circle, checks CPU/device thermal agreement, and adds a bounded heavy coarse measurement that must actually stop as steady with stable wall heat. It does not freeze an unsupported reference oracle.

The three-grid values alone look close to the paper, but **padding and temporal/collision sensitivity exceed the last grid change**. Consequently this does not establish isolated curved-interface convergence or an independently met 1% accuracy target. The unchanged target and negative controls are reported below so the remaining uncertainty is reviewable.

## Reference and model

M. M. Rahman, M. A. Alim, S. Saha and M. K. Chowdhury, *Mixed Convection in a Vented Square Cavity with a Heat Conducting Horizontal Solid Circular Cylinder*, Journal of Naval Architecture and Marine Engineering 5(2), 2008, pp.37-46, [doi:10.3329/jname.v5i2.2504](https://doi.org/10.3329/jname.v5i2.2504). The primary full text describes the authors' Galerkin finite-element/Newton computation. These are published numerical values, not experiment, an empirical correlation, or output extracted from a commercial CAE product. No commercial code, decks, geometry or figures were used.

Original model from Fig. 1 and Table 1: square cavity L=1; centered passive cylinder D/L=0.2; ks/kf=5; Re=100; Pr=0.71; Ri=1. Uniform cold inlet occupies the bottom 0.1L of the left wall, the outlet the upper 0.1L of the right wall. The remaining **right wall**, length 0.9L, is hot; other walls are adiabatic. The cylinder solves conduction with neither imposed surface temperature nor volumetric generation. In-plane gravity and Boussinesq buoyancy are included; one periodic cell supplies the depth.

Theta=(T-Tin)/(Th-Tin). The compared Nu is the positive heat entering the fluid from the **heated cavity wall**, divided by kf * 0.9L * depth * (Th-Tin), with L=1. It is not a cylinder-surface Nu. Table 1 gives 4.842449 on its finest 7512-element grid and 4.832878 on the chosen 5918-element grid; the paper's approximately 0.2% nonmonotonic grid spread is separate from our predeclared 1% target. Its theta_av=0.197227 is secondary only: the text/formula do not resolve the fluid-only versus full cavity integration domain sufficiently for an acceptance gate. Our reported mean excludes the cylinder and short passages.

The current box-face BC representation requires short passages outside the cavity, with solid wall blocks masking each narrow slot. This differs from the paper's slot-plane BCs. Padding is measured separately, not attributed to curved-interface error. Grid n=40/80/160 keeps the 0.1L openings exactly aligned, giving D/dx=8/16/32. The default 512-segment cylinder has sagitta about 1.88e-6 L, below every spacing; its cell mask matches the analytic circle on all tested grids. No radius or reference value was adjusted to hit a target.

Scaled inputs: U=rho=cp=1, nu=0.01, alpha=0.01/0.71, kf=alpha, ks=5alpha, cylinder rho=cp=1, Tin=0 C, Th=1 C, beta=0.001 and gravity_y=-1000. These are dimensionless-reference-scaled properties, not real air or coolant. Default lattice speed 0.025 gives Mach 0.043301 and dt=0.025dx. Steady tolerance 1e-5, window 5, minimum time 20; automatic velocity/temperature residuals plus hot-wall heat and mean temperature must settle for two windows. Each reported run stopped as steady before its cap. Saved temperatures during steady capacity scaling are **not a physical transient**.

## Measured results

NVIDIA GeForce RTX 4090, strict hardware Vulkan, no software fallback. Nu averages the last tenth of recorded hot-wall heat history; tail relative spread is below 5.2e-5 in these device runs. These are accuracy measurements, not throughput benchmarks.

| Cavity n | Passage cells each side | Segments | Lattice speed | Steady t / steps | Nu | Difference from 4.842449 |
|---|---:|---:|---:|---:|---:|---:|
|40|1|512|0.025|100 /160000|4.824832997|-0.363783%|
|80|1|512|0.025|110 /352000|4.876086048|+0.694629%|
|160|1|512|0.025|145 /928000|4.880068872|+0.776877%|
|80|2|512|0.025|110 /352000|4.894713617|+1.079301%|
|160|2|512|0.025|150 /960000|4.898823204|+1.164167%|
|80|1|1024|0.025|110 /352000|4.876081311|+0.694531%|
|80|1|512|0.0125|115 /736000|4.911119575|+1.418096%|

The n80-to-n160 default change is about 0.082%, whereas doubling padding changes n160 by about 0.384%, and halving lattice speed changes n80 by about 0.718%. The last control changes dt, thermal relaxation and Mach together; it does not isolate Mach alone. Doubling tessellation changes Nu by less than 0.0001%, while leaving the cell mask unchanged. The three-grid errors are nonmonotonic, so no observed order or Richardson extrapolation is claimed.

The voxelized cylinder area is 0.0325 on n40/n80 and 0.03171875 on n160, versus pi*0.1^2=0.03141593. Fine-grid fluid mean theta is 0.193976893; cylinder theta spans 0.074985348..0.101811811, confirming solved nonuniform conduction rather than an isothermal obstacle. These values are diagnostics, not additional acceptance targets.

CPU n40 stopped steady at t95 / 152000 steps, Nu 4.824827744 (-0.363891%), agreeing closely with the device result. Different final steady sample times are reported honestly. No tolerance was changed after seeing results.

## Reproduce the bounded study

The source contains the enduring original geometry/model in crates/lbm/tests/common/curved_cht.rs. The ordinary ignored test runs only the bounded coarse CPU case; the long sweep is not added to CI. The complete original standalone harness is retained outside git, and the following equivalent code/commands reconstruct every reported device setting from the submitted source without unpublished helpers.

<details>
<summary>Complete bounded study harness and invocation matrix</summary>

Save this as .agent-files/rahman-study.rs in the source checkout (the directory is local-only):

```rust
#[path = "../crates/lbm/tests/common/mod.rs"]
mod common;
use fieldcraft_gpu::select::Policy;
use fieldcraft_lbm::{Flow, Limits, Setup};
use fieldcraft_lbm_gpu::{GpuFlow, Options};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse::<usize>().unwrap();
    let padding = args[2].parse::<usize>().unwrap();
    let segments = args[3].parse::<usize>().unwrap();
    let speed = args[4].parse::<f64>().unwrap();
    let end = args[5].parse::<f64>().unwrap();
    let (model, obstacles) = common::curved_cht::model_case(n, padding, segments, end, speed);
    let setup = Setup::new(&model, None, &obstacles, &Limits::default()).unwrap();
    let initial = Flow::new(&setup, &obstacles).unwrap();
    let mut gpu = GpuFlow::with_options(&initial, Policy::Hardware, Options::default()).unwrap();
    println!("hardware device: {}", gpu.adapter_name());
    let result = fieldcraft_lbm::run(&setup, &mut gpu, &Limits::default()).unwrap();
    println!("{:?}", common::curved_cht::observation(&setup, &result, padding, segments));
    println!("steady={:?}", result.steady.as_ref().unwrap().reason);
    serde_json::to_writer(std::fs::File::create(&args[6]).unwrap(), &result).unwrap();
}
```

PowerShell reproduction; requires a supported real Vulkan GPU. Policy::Hardware refuses software fallback. Serial runs avoid competing device studies:

```powershell
$env:CARGO_TARGET_DIR = "$PWD/target-study"
$env:CARGO_BUILD_JOBS = '3'
$env:FIELDCRAFT_THREADS = '3'
$env:FIELDCRAFT_GPU = 'hardware'
$env:FIELDCRAFT_REQUIRE_GPU = '1'
$env:WGPU_BACKEND = 'vulkan'
cargo test -p fieldcraft-lbm -p fieldcraft-lbm-gpu --no-run
$studyDeps = "$env:CARGO_TARGET_DIR/debug/deps"
$studyArgs = @('--edition=2024', '.agent-files/rahman-study.rs', '-L', "dependency=$studyDeps", '-o', '.agent-files/rahman-study.exe')
foreach ($studyCrate in @('fieldcraft_gpu','fieldcraft_lbm','fieldcraft_lbm_gpu','fieldcraft_mesh','fieldcraft_model','serde_json')) {
  $studyLib = Get-ChildItem $studyDeps -Filter "lib$studyCrate-*.rlib" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
  $studyArgs += @('--extern', "$studyCrate=$($studyLib.FullName)")
}
rustc @studyArgs
& .agent-files/rahman-study.exe 40 1 512 0.025 100 .agent-files/n40.json
& .agent-files/rahman-study.exe 80 1 512 0.025 160 .agent-files/n80.json
& .agent-files/rahman-study.exe 160 1 512 0.025 160 .agent-files/n160.json
& .agent-files/rahman-study.exe 80 2 512 0.025 160 .agent-files/n80-padding2.json
& .agent-files/rahman-study.exe 160 2 512 0.025 160 .agent-files/n160-padding2.json
& .agent-files/rahman-study.exe 80 1 1024 0.025 160 .agent-files/n80-segments1024.json
& .agent-files/rahman-study.exe 80 1 512 0.0125 160 .agent-files/n80-half-speed.json
```

</details>

## Demonstration

A real presented native-window recording, curved-conjugate-631.mp4, shows the coarse original model, saved accelerated steady iterations, settled temperature, and velocity/streamlines. 23 seconds, 1440 x 952, H.264 / 12 fps, 499982 bytes; SHA256 d4ce99bdc342ee829c0176932771ef21574580de4cd1768c696115a3c001cebc. The two companion stills are curved-conjugate-temperature.png and curved-conjugate-flow.png. The clip explicitly identifies the dimensionless case, 40 x 40 cavity / one periodic depth cell, passive cylinder, RTX 4090 / Vulkan, 160000 lattice steps, and nonphysical steady-playback semantics. The UI frame count denotes stored frames, not solver iterations. Surface colours read the adjacent fluid, not the cylinder's internal field.

The native app/CLI binaries were built from baseline 7d9c027d71b2c1ea3395e2552e95f0455a7244db; this feature changes tests only, so the recording demonstrates the unchanged production command/UI path rather than claiming a newly implemented algorithm. Original three-body geometry was generated with SolidCraft CLI 0.14.0, opened normally, and configured with model.units/material/fluid_domain/step commands, solve.settings (GPU, no fallback) and solve.fluid. The native solution also stops steady at t100 / 160000 steps. Saved binary projects, raw numerical result files and all media remain outside feature history. Captures are original FieldCraft output; captions use the repository's SIL OFL 1.1 Inter font. No paper figures or third-party CAD assets were copied.

<details>
<summary>Original native geometry recipe and source provenance</summary>

Original native geometry recipe, in millimetres, for SolidCraftCLI run --save:

```json
{"commands":[
 {"command":"solid.box","params":{"corner":[-50,100,-100],"length":50,"width":900,"height":200,"body_name":"LeftWall"}},
 {"command":"solid.box","params":{"corner":[1000,0,-100],"length":50,"width":900,"height":200,"body_name":"RightWall"}},
 {"command":"solid.cylinder","params":{"base":[500,500,-100],"radius":100,"height":200,"body_name":"Cylinder"}}
]}
```

The native wall's heated persistent face is F2:-x. Material/domain/step values match model_case(40,1,512,100,0.025), with the native kernel's own circle tessellation and 51 stored frames instead of 3. The native geometry is an independent original visualization of the same cavity, not the exact analytic test tessellation. All comparisons in the table use the submitted test geometry, not measurements taken from the video.

</details>

All three final media files were pushed to the separate ordinary-Git artifact inbox in commit 7e1fc52ee20b0a9873cbb33265581d8a8bbebc26, under objects/cfd/curved-cht-631/c3206593f8ed63d95d54e5aedbaff6e37e4efe5e/. The staging helper verified actual bytes and SHA256 values: the MP4 is 499982 bytes; the two stills are 227143 and 267835 bytes. Artifact received receipts are pending the maintainer scan; GitHub attachment upload is also pending. No GitHub sign-in, LFS or raw-repository video link is needed or claimed.

## Validation

Tested source c3206593f8ed63d95d54e5aedbaff6e37e4efe5e, rebased onto the latest mirrored upstream-develop 7d9c027d71b2c1ea3395e2552e95f0455a7244db. Windows x86_64; 3 build jobs and 3 CPU solver threads; strict NVIDIA RTX 4090 hardware Vulkan for device runs.

- cargo test -p fieldcraft-lbm -p fieldcraft-lbm-gpu:90 passed, 23 heavy/benchmark tests ignored by their annotations (including the new heavy coarse measurement); no failed tests. The device subset has 26 passed, including 23 integration and 3 unit cases; no missing-device skips or software fallback.
- cargo test -p fieldcraft-lbm --test curved_conjugate rahman_curved_conjugate_coarse_measurement -- --ignored --nocapture:1 passed in 36.87 seconds. Requires steady stop and stable tail heat, finite physically bounded temperature and a nonuniform passive conducting solid. This keeps the added automatic heavy workload bounded, rather than landing the exploratory sweep in CI.
- New fast regression compares the original tessellated geometry's cell mask to the analytic circle on 20 / 40 / 80 / 160 grids and checks exact slot counts/no cylinder heat source.
- New startup CPU/device test uses existing thermal tolerances: maximum temperature difference 2e-5 on the 1 K scale; wall heat relative difference 1e-4. Both arrays must be finite before the maximum difference is computed.
- cargo check -p fieldcraft-lbm -p fieldcraft-lbm-gpu; cargo clippy -p fieldcraft-lbm -p fieldcraft-lbm-gpu --all-targets -- -D warnings; cargo fmt --all --check; git diff --check: passed.
- Parent read-only review of model/equations, native image and test changes completed. No production numerics or oracle cases changed, so no local oracle regeneration was required. The full workspace gate, all ignored tests, wasm and other OS/device runs were not run locally; new-head remote CI is pending.

## Known limitations

The 1% default fine-grid comparison is not a validated interface-only acceptance: doubling passages and halving lattice speed exceed that target and exceed the last grid change. Boundary/model form, temporal/collision sensitivity, voxelized geometry and the paper's own grid spread are not isolated from one another. No convergence order, mesh independence or frozen accuracy fixture is claimed. The bulk mean has no reference acceptance gate because the reference integration domain is ambiguous. There is no higher-conductivity/otherRi/three-dimensional validation in this slice.

Steady solid-capacity scaling accelerates convergence; recorded intermediate temperatures are not a physical transient. The video shows only the coarse case, not every grid or a coupled two-circuit heat exchanger. Local diagnostic timings are not performance claims (the n160padding2 and native coarse solves briefly overlapped). Actual GitHub attachments remain maintainer-owned and pending. #631 stays open; the curved-wall production algorithm remains parked, and no renderer or backend-removal work is included.


