# marco-sparko
# Release Process

These are the steps to create  release
## Create Tagged Source
Ensure that all feature branches have been merged to main and that Cargo.toml shows the correct version number for the new release.

Go to GitHub and create a release from ```main``` creating the tag ```releaseX.X.X``` where X.X.X is the version number, on sparko_graphql **first** and then on marco-sparko. Use the same tag on both.

## Build the Installers
Publishing the marco-sparko release starts the ```Release``` workflow in [.github/workflows/release.yml](../.github/workflows/release.yml), which builds, tests and attaches the installers to the release. Nothing needs to be done on any of the build machines.

Watch it on the Actions tab of the marco-sparko repo, it takes about 30 to 40 minutes. When it has finished the release should have these files:

```
MarcoSparko_X.X.X_aarch64.dmg
MarcoSparko_X.X.X_x64.dmg
MarcoSparko_X.X.X_x64-setup.exe
MarcoSparko_X.X.X_x64_en-US.msi
MarcoSparko_X.X.X_arm64-setup.exe
MarcoSparko_X.X.X_arm64_en-US.msi
```

The files are only attached if all four builds succeed. If one fails, open the failed run on the Actions tab, fix the problem, and either use **Re-run failed jobs** on the run (if the problem was temporary) or move the tag to the fixed commit, delete the release and create it again.

## Try a Build Without a Release
On the Actions tab choose **Release**, then **Run workflow**, and enter a tag. The installers are kept on the run's page under **Artifacts** for 90 days and no release is changed. The tag must be on a commit which contains the workflow, and the same tag must exist in sparko_graphql.

# Old Release Instructions
The manual build steps used before the workflow are in [Release-archive.md](Release-archive.md).
