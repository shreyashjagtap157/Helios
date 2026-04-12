Omni Self-Hosting Release v2 — Effects Feature

Contents
- OMNI_SELF_HOSTING_RELEASE_v2_EFFECTS.md  (release notes)
- MANIFEST.txt                             (list of files included in the release bundle)
- package_release.ps1                      (Windows PowerShell packaging helper)

Purpose
This release bundle contains the artifacts and instructions required to verify and bootstrap the Omni v2 self-hosting compiler with effect-clause support. It references prebuilt OVM stage files produced during validation.

How to produce the ZIP release locally (Windows PowerShell)
1. Open PowerShell in the repository root (d:\Project\Helios).
2. Run the packaging helper script:

   .\release\omni-self-hosting-v2-effects\package_release.ps1

The script will copy the listed artifacts into a temporary folder and create a zip file `omni-self-hosting-v2-effects.zip` in `release/`.

Files referenced by this manifest are located in the `build/` directory and in the repository root.

If you want me to produce additional artifacts (e.g., an installer, tarball, or embed the `omnc` binary), tell me and I will prepare the steps and scripts to do so.