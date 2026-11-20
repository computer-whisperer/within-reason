Headers copied verbatim from RecoilEngine (GPL-2.0-or-later), commit e9d1993, engine release 2026.09.01.
Source paths: rts/ExternalAI/Interface/*.h, rts/System/{ExportDefines,MainDefines}.h. Re-copy when bumping the engine.

Checked 2026-11-19: every file here is byte-identical to the same path at the tags 2026.07.04, 2026.09.03 (the lobby's
release since 2026-11-16) and 2026.11.04 (a pre-release), so one build of the libraries serves all of them and
nothing was re-copied. The check, from the repository's root with the tag fetched into `upstream/RecoilEngine`
(`git -C upstream/RecoilEngine fetch --depth 1 origin tag <release>`):

    for f in $(cd crates/recoil-ai-sys/vendor && find rts -type f); do
        git -C upstream/RecoilEngine show <release>:$f | cmp -s - crates/recoil-ai-sys/vendor/$f || echo "DIFFERS $f"
    done
