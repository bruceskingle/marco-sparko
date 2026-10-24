# marco-sparko
# Release Process

Each release is developed on a branch called `vX.X.X` in both sparko_graphql and marco-sparko, which is squash
merged to `main` and tagged `releaseX.X.X`. Publishing the marco-sparko release on GitHub builds and attaches the
installers. Once it is released, the branches for the next version are started (step 5).

The commands below are for zsh on the main dev machine, with both repos checked out in `~/git`. They need the GitHub
CLI, `gh`, signed in with `gh auth login`. Start each terminal session by setting the version being released:

```
export RELEASE=0.4.4
```

## 1. Try the Installers
The installers can be built from the release branch before anything is merged, to try them on other machines. The
workflow builds a tag, so with the release branches checked out and pushed make a temporary one in both repos:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag test$RELEASE
  git push origin test$RELEASE
done

cd ~/git/marco-sparko
gh workflow run release.yml --ref v$RELEASE -f tag=test$RELEASE
```

`--ref` runs the version of the workflow on the release branch, so changes to it can be tried before they are merged.

Wait a few seconds for the run to start, then watch it (about 30 to 40 minutes, the Intel Mac is the slowest):

```
cd ~/git/marco-sparko
RUN=$(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch $RUN
```

### Downloading the Installers
Each build's installers are kept as an artifact named `macos-aarch64`, `macos-x64`, `windows-x64` or
`windows-arm64` for 90 days. Download them all once the run has finished:

```
gh run download $RUN --dir ~/Downloads/marco-sparko-$RELEASE
```

Or download particular builds with `-n`, e.g. while the slower builds are still running (if `gh` refuses because the
run is in progress, wait for it to finish):

```
gh run download $RUN -n windows-arm64 -n macos-aarch64 --dir ~/Downloads/marco-sparko-$RELEASE
```

To list what has been uploaded so far:

```
gh api repos/bruceskingle/marco-sparko/actions/runs/$RUN/artifacts --jq '.artifacts[] | "\(.name) \(.size_in_bytes)"'
```

On a Windows VM (with `gh` installed from https://github.com/cli/cli/releases and signed in), in PowerShell:

```
cd ~\git\marco-sparko
$RUN = gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId'
gh run download $RUN -n windows-arm64 --dir $HOME\Downloads\marco-sparko-test
```

The artifacts can also be downloaded from the run's page on the Actions tab of the marco-sparko repo, under
**Artifacts** at the bottom of the Summary (only the name is a link).

### If a Build Fails
See which step failed and why:

```
gh run view $RUN --log-failed
```

If that prints nothing, get the whole log of the failed job (its ID is shown next to it by `gh run watch`):

```
gh api repos/bruceskingle/marco-sparko/actions/jobs/<job id>/logs | grep -B2 -A10 -i "error\|panicked"
```

Fix the problem on the release branch, commit and push, then move the temporary tags and run it again:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag -f test$RELEASE
  git push -f origin test$RELEASE
done

cd ~/git/marco-sparko
gh workflow run release.yml --ref v$RELEASE -f tag=test$RELEASE
```

### Afterwards
Delete the temporary tags, they can be made again for another try:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag -d test$RELEASE
  git push origin --delete test$RELEASE
done
```

## 2. Merge to Main and Tag
Make sure everything is committed and pushed. Then, sparko_graphql first, create a pull request and squash merge it
to `main`:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git checkout v$RELEASE
  git status --short
  git push
  gh pr create --base main --head v$RELEASE --title "V$RELEASE" --body "Release $RELEASE"
  gh pr merge v$RELEASE --squash
done
```

`gh pr merge` asks for confirmation. The pull requests can be merged on GitHub instead, choose **Squash and merge**.

Tag the merged `main` in both repos:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git checkout main
  git pull
  git tag release$RELEASE
  git push origin release$RELEASE
done
```

### More Changes After Merging
A squash merge does not record that the branch's commits are on `main`, so before making more changes on the release
branch merge `main` back into it. This changes no files, but without it the next pull request shows every change in
the release again:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git checkout v$RELEASE
  git pull
  git fetch origin
  git merge origin/main -m "Merge main after squash merge"
  git push
done
```

If the release has been tagged but not published, delete the tags, make the changes, and repeat this step:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag -d release$RELEASE
  git push origin --delete release$RELEASE
done
```

## 3. Final Check
Do a clean build of the tagged source and click around the app:

```
cd ~/git/sparko_graphql
git checkout release$RELEASE
cd ~/git/marco-sparko
git checkout release$RELEASE
cargo clean
dx run --release
```

If something needs fixing, go back to the release branch as in "More Changes After Merging" in step 2.

## 4. Release
Publish the release. This starts the `Release` workflow in
[.github/workflows/release.yml](../.github/workflows/release.yml), which builds and tests the installers on GitHub and
attaches them to the release:

```
cd ~/git/marco-sparko
gh release create release$RELEASE --verify-tag --title "Release $RELEASE" --generate-notes
```

Wait a few seconds, then watch the build (about 30 to 40 minutes):

```
RUN=$(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch $RUN
gh release view release$RELEASE
```

When it has finished the release should have these files:

```
MarcoSparko_X.X.X_aarch64.dmg
MarcoSparko_X.X.X_x64.dmg
MarcoSparko_X.X.X_x64-setup.exe
MarcoSparko_X.X.X_x64_en-US.msi
MarcoSparko_X.X.X_arm64-setup.exe
MarcoSparko_X.X.X_arm64_en-US.msi
```

Download them, e.g. to try the released installers:

```
gh release download release$RELEASE --dir ~/Downloads/marco-sparko-$RELEASE
```

or on a Windows VM in PowerShell, just the Windows ones:

```
gh release download release0.4.4 -R bruceskingle/marco-sparko -p '*.msi' -p '*-setup.exe' --dir $HOME\Downloads\marco-sparko
```

The files are only attached if all four builds succeed. If one fails, see "If a Build Fails" in step 1. If the
problem was temporary, re-run the failed jobs:

```
gh run rerun $RUN --failed
```

Otherwise delete the release (leaving the tags), fix the problem as in "More Changes After Merging" in step 2, and
create the release again:

```
gh release delete release$RELEASE --yes
```

## 5. Start the Next Release Branch
Set `RELEASE` to the next version:

```
export RELEASE=0.4.5
```

Create the release branch in both repos from the latest `main`:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git checkout main
  git pull
  git checkout -b v$RELEASE
done
```

Set the new version in both Cargo.toml files, and the version of sparko_graphql which marco-sparko uses, then build
so that the Cargo.lock files are updated too:

```
cd ~/git/sparko_graphql
sed -i '' "1,/^version = /s/^version = .*/version = \"$RELEASE\"/" Cargo.toml
cargo build

cd ~/git/marco-sparko
sed -i '' -e "1,/^version = /s/^version = .*/version = \"$RELEASE\"/" \
  -e "s/^sparko_graphql = {version = \"[^\"]*\"/sparko_graphql = {version = \"$RELEASE\"/" Cargo.toml
cargo build
```

Check the changes, then commit and push the branches:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git diff --stat
  git commit -am "Start v$RELEASE"
  git push -u origin v$RELEASE
done
```

## 6. Develop
Commit to `v$RELEASE` and push as usual. When it is ready to release, go back to step 1.

If a change needs its own branch, e.g. because someone else is working on it, branch from the release branch and
merge it back into it:

```
git checkout -b my-feature v$RELEASE
# ... commit and push my-feature, then when it is done
git checkout v$RELEASE
git pull
git merge my-feature
git push
```

Someone else's feature branch can be merged with a pull request on GitHub instead, with `v$RELEASE` as the base.

# Old Release Instructions
The manual build steps used before the workflow are in [Release-archive.md](Release-archive.md).
