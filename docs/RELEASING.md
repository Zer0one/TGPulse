# TGPulse-Next desktop releases

The public version takes the upstream Cargo workspace version and appends a
fork counter starting at zero. With upstream `0.1.0`, the first tag is
`0.1.0.0`, then `0.1.0.1`. Tag names have no `v` prefix. A new upstream Cargo
version resets the counter to zero. The release workflow rejects a tag whose
first three components do not match `Cargo.toml`.

`.github/workflows/release.yml` is the release authority. Run it manually on
the intended commit first (`workflow_dispatch`): it builds and tests macOS
ARM64, Linux x86_64 and Windows x86_64 and uploads CI artifacts, but **does not
publish** a release. Inspect all three archives, the included binary version,
architecture and runtime dependencies. Then create and push the annotated
four-part tag on that verified commit. The tag run repeats the checks and
publishes a GitHub prerelease only if all jobs succeed, attaching the three
archives and `SHA256SUMS`. GitHub's generated notes are supplemental to these
checks, not evidence that a binary runs on a user's machine.

If the publishing job fails after a tag has already been pushed, fix the
workflow on `main` and manually dispatch it there with `release_tag` set to
that existing tag. The job checks out and rebuilds the tagged source, then
publishes to the same tag; do not move an existing release tag. Leave
`release_tag` empty for an ordinary dry run.

The archives contain the executable, project README and license. The macOS
archive also contains SDL3 and its license, with the executable linked to the
adjacent dylib rather than to a Homebrew path. It uses Metal and is ad-hoc
signed for packaging, **not notarized**. Linux may still need the usual system
audio/gamepad/display libraries; Windows uses the workspace's static runtime
link settings. ROMs, NVRAM, personal configuration and save states are never
release assets. Put complete ZIP romsets in `roms/` under the directory from
which the executable is launched, or pass `--roms`.

Android is not part of the first release matrix: a distributable APK requires
a release signing key and a separately verified package. Do not publish an
unsigned or debug-signed APK under the desktop release tag.

TGPulse development and `macos-emulation-toolkit` are separate projects. The
release workflow does not update the toolkit's `current` installation. After a
release is verified, handle that deployment in the toolkit's own workflow and
record the release source revision and binary hash there; do not replace
`current` during an unverified CI run.
