# TGPulse working framework

## Before each substantial activity or new phase

- Give a short Italian preflight: objective/scope, recommended available model
  and reasoning effort (`low`, `medium`, `high`, `xhigh` as appropriate), expected
  consumption, uncertainty and planned verification. For trivial follow-ups,
  keep this to one sentence; do not turn it into a second planning task.
- Distinguish the recommended model/effort from the actual active settings.
  Never claim to have changed either without confirmation from the product.
  If a change is needed, explain why and wait for explicit user approval before
  starting that phase. Do not silently substitute a model or start subagents.
- Estimate consumption as low/medium/high/very high initially. Supply a numeric
  range only when comparable observed runs or reliable telemetry justify one;
  label it as an estimate with confidence and assumptions. Do not invent a
  conversion between tokens, money, runtime and account allowance percentages.
- Read the available usage-limit tool at the start of a macro-activity. Keep a
  transient baseline: timestamp, bucket, window duration, used percentage and
  reset time. Do not store account IDs, credit balances or usage snapshots in Git.
- For large work, propose one bounded milestone and a checkpoint, not an
  uncalibrated estimate for completing an entire hardware subsystem.

## During work

- If scope, difficulty or the anticipated consumption materially increases,
  announce a revised estimate before the new phase. Seek approval for a model
  change, expanded scope, installations or other newly required authority.
- Reuse project tools. Prefer narrow, upstream-reapplicable fixes; avoid a new
  automation/runner merely to report consumption. Build time is not AI usage.
- Treat user-named references as authoritative at the requested layer: inspect
  their current implementation, not only prior summaries. For Model 2 controls,
  the tested SM2 Libretro reference and its workbook remain authoritative.

## End-of-activity recap (including partial/blocked outcomes)

- Report completed work, evidence/tests, remaining limits, and the exact binary
  or document for testing. Separate build, loader/smoke and real gameplay proof.
- Refresh account usage and report remaining percentages for every available
  window plus reset times in Europe/Rome. Missing windows mean unavailable,
  not zero usage. If the tool fails, say so; do not claim a stale value is live.
- Compare only the same bucket and reset window: delta is end used percentage
  minus start used percentage, in **percentage points**. Label it as the observed
  account-wide delta, not exact task consumption; concurrent tasks can contribute.
  An unchanged rounded reading is not proof of zero cost. Across a reset or a
  changed bucket, do not calculate a misleading delta.
- Report exact per-task tokens/cost only if supplied by reliable per-task
  telemetry. Otherwise explicitly state that exact attribution is unavailable.
- Keep the recap compact; include consumption even when the work is unfinished.

Suggested format:

> Prima: scope — recommended model/effort — expected consumption/confidence —
> verification — current remaining allowance.
>
> Dopo: outcome/tests — pending work — observed account delta (if comparable) —
> remaining allowance/reset — publication status.

## Project boundaries and verification

- This repository is development TGPulse. MAME and macos-emulation-toolkit are
  separate projects: do not edit/deploy them as a side effect.
- Preserve existing uncommitted work. Never commit ROMs, settings, saves or
  account telemetry. Ask before installing additional software or dependencies.
- Commit/push only with explicit authorization for the current work. A past
  publication request is not blanket permission. Do not replace toolkit/current
  releases without the corresponding release request.
- Use `cargo test --offline --workspace` and proportionate targeted checks.
  Development build: `cargo build --offline --release -p tgpulse`;
  executable: `target/release/tgpulse`, launched by `tgpulse.dev` when installed.
  Do not equate `tgpulse.my` or `tgpulse` with this development binary.
- For Model 1 priorities and evidence, read `docs/MODEL1_ROADMAP.md`. Keep roadmap
  status and verification boundaries current when its activities change.

This is a repository-scoped working agreement, not an automatic model switch,
hard quota enforcement or a global modification to other projects.
