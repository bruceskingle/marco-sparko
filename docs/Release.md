# marco-sparko
# Release Process

Each release is developed on a branch called `vX.X.X` in both sparko_graphql and marco-sparko, which is squash
merged to `main` and tagged `releaseX.X.X`. The installers are built on GitHub and attached to a draft release, which
is published once they have been tried. Then the branches for the next version are started.

The steps are run with [scripts/release.sh](../scripts/release.sh) on the main dev machine, with both repos checked
out side by side in `~/git`. It needs the GitHub CLI, `gh`, signed in with `gh auth login`. The version comes from
marco-sparko's Cargo.toml, which `start` sets at the beginning of each release, so it never needs to be typed again.
Each command checks that both repos are in the state it expects (the right branch, nothing uncommitted, the tags it
needs) and stops with an explanation at the first problem.

```
cd ~/git/marco-sparko
./scripts/release.sh status
```

shows where things are: the branch, version and uncommitted changes of each repo, the test and release tags, the
GitHub release and the latest workflow runs.

The manual commands for each step are in the collapsed sections, in case the script cannot be used. They need the
version in `RELEASE`, e.g. `export RELEASE=0.4.4`.

## 1. Try the Installers (optional)

```
./scripts/release.sh try
```

Builds the installers from the release branches before anything is merged, to try them on other machines. It tags both
release branches `testX.X.X` (moving the tags if they exist), runs the workflow, watches it (about 30 to 40 minutes,
the Intel Mac is the slowest) and downloads the installers to `~/Downloads/marco-sparko-X.X.X-test`. Run it again
after fixing anything. The test tags are deleted by `merge`.

If a build fails, it shows the commands to see why and to re-run the failed jobs.

Ctrl+C stops watching but not the run. To pick it up again, e.g. after a re-run of failed jobs, and download the
installers when it finishes:

```
./scripts/release.sh watch
```

<details>
<summary>Manual commands</summary>

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag -f test$RELEASE
  git push -f origin test$RELEASE
done

cd ~/git/marco-sparko
gh workflow run release.yml --ref v$RELEASE -f tag=test$RELEASE
```

`--ref` runs the version of the workflow on the release branch, so changes to it can be tried before they are merged.
Wait a few seconds for the run to start, then watch it and download the installers:

```
RUN=$(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch $RUN
gh run download $RUN --dir ~/Downloads/marco-sparko-$RELEASE-test
```

Each build's installers are kept as an artifact named `macos-aarch64`, `macos-x64`, `windows-x64` or `windows-arm64`
for 90 days. Particular builds can be downloaded with `-n`, e.g. while the slower builds are still running (if `gh`
refuses because the run is in progress, wait for it to finish):

```
gh run download $RUN -n windows-arm64 -n macos-aarch64 --dir ~/Downloads/marco-sparko-$RELEASE-test
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

If a build fails, see why with the first command, or the second if that prints nothing (the job ID is shown next to
the job by `gh run watch`), and re-run the failed jobs with the third:

```
gh run view $RUN --log-failed
gh api repos/bruceskingle/marco-sparko/actions/jobs/<job id>/logs | grep -B2 -A10 -i "error\|panicked"
gh run rerun $RUN --failed
```

Delete the test tags afterwards:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag -d test$RELEASE
  git push origin --delete test$RELEASE
done
```

</details>

## 2. Merge to Main and Tag

```
./scripts/release.sh merge
```

For each repo, sparko_graphql first: creates a pull request from the release branch (unless there is one), squash
merges it to `main` (`gh` asks for confirmation), and tags `main` `releaseX.X.X`. It then merges `main` back into the
release branch, which changes no files but means a later pull request only shows new changes, and deletes the test
tags. If it fails part way, run it again and it carries on.

To make more changes after merging, before the release is published:

```
./scripts/release.sh untag
```

deletes the release tags (and the draft release, if `build` has made one). Commit the changes to the release
branches, then run `merge` again.

<details>
<summary>Manual commands</summary>

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

The pull requests can be merged on GitHub instead, choose **Squash and merge**. Then tag `main` and merge it back
into the release branches:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git fetch origin
  git tag release$RELEASE origin/main
  git push origin release$RELEASE
  git merge origin/main -m "Merge main after squash merge"
  git push
done
```

To make more changes, delete the draft release (if there is one) and the release tags:

```
gh release delete release$RELEASE --yes
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag -d release$RELEASE
  git push origin --delete release$RELEASE
done
```

</details>

## 3. Final Check

```
./scripts/release.sh check
```

Checks out the release tags in both repos, does a clean build and runs the app. Click around, then quit the app and it
switches both repos back to the release branches. If something needs fixing, use `untag` as in step 2.

<details>
<summary>Manual commands</summary>

```
cd ~/git/sparko_graphql
git checkout release$RELEASE
cd ~/git/marco-sparko
git checkout release$RELEASE
cargo clean
dx run --release
```

Then go back to the release branches. If Cargo.lock shows as changed, it was regenerated (e.g. by the editor) and it
is safe to discard it:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git restore Cargo.lock
  git checkout v$RELEASE
done
```

</details>

## 4. Build the Draft Release

```
./scripts/release.sh build
```

Creates a draft GitHub release for the tag (with notes generated from the merged pull requests), then runs the
[Release workflow](../.github/workflows/release.yml) on the tag, which builds and tests the installers and attaches
them to the draft. The files are only attached if all four builds succeed. It downloads them to
`~/Downloads/marco-sparko-X.X.X` and shows the draft's URL.

Try the installers, and edit the release notes on GitHub (the draft is at the top of the Releases page, choose
**Edit**), or with `gh release edit releaseX.X.X --notes-file <file>`. A draft is only visible to people who can push
to the repo.

If a build fails, fix it as in step 2 (`untag`, commit, `merge`, `check`) and run `build` again. Running `build` again
on an existing draft replaces its installers. As with `try`, `./scripts/release.sh watch` picks up the run again after
Ctrl+C or a re-run of failed jobs, and downloads the installers from the draft when it finishes.

<details>
<summary>Manual commands</summary>

```
cd ~/git/marco-sparko
gh release create release$RELEASE --draft --verify-tag --title "Release $RELEASE" --generate-notes
gh workflow run release.yml --ref release$RELEASE -f tag=release$RELEASE -f attach=true
```

Wait a few seconds for the run to start, then watch it and download the installers from the draft:

```
RUN=$(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch $RUN
gh release download release$RELEASE --dir ~/Downloads/marco-sparko-$RELEASE
```

On a Windows VM in PowerShell, just the Windows installers (with the version in place of 0.4.4):

```
gh release download release0.4.4 -R bruceskingle/marco-sparko -p '*.msi' -p '*-setup.exe' --dir $HOME\Downloads\marco-sparko
```

The release should have these files:

```
MarcoSparko_X.X.X_aarch64.dmg
MarcoSparko_X.X.X_x64.dmg
MarcoSparko_X.X.X_x64-setup.exe
MarcoSparko_X.X.X_x64_en-US.msi
MarcoSparko_X.X.X_arm64-setup.exe
MarcoSparko_X.X.X_arm64_en-US.msi
```

</details>

## 5. Publish

```
./scripts/release.sh publish
```

Checks that the draft has all six installers, then publishes it as the latest release. Nothing is built again, so the
installers are the ones which were tried.

<details>
<summary>Manual commands</summary>

```
gh release edit release$RELEASE --draft=false --latest
```

or choose **Publish release** on the draft's edit page on GitHub.

</details>

## 6. Start the Next Release

```
./scripts/release.sh start
```

Creates the `vX.X.X` branches for the next patch version (e.g. 0.4.4 after 0.4.3) in both repos from `main`, sets
the version in both Cargo.toml files and the version of sparko_graphql which marco-sparko uses, builds to update the
Cargo.lock files, commits and pushes. For a different version, give it, e.g. `./scripts/release.sh start 0.5.0`. If
the branches already exist it switches to them.

Then commit to the `vX.X.X` branches and push as usual. When it is ready to release, go back to step 1.

If a change needs its own branch, e.g. because someone else is working on it, branch from the release branch and
merge it back into it, or merge their pull request on GitHub with the release branch as the base:

```
git checkout -b my-feature v0.4.4
# ... commit and push my-feature, then when it is done
git checkout v0.4.4
git pull
git merge my-feature
git push
```

<details>
<summary>Manual commands</summary>

```
export RELEASE=0.4.5

for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git checkout main
  git pull
  git checkout -b v$RELEASE
done

cd ~/git/sparko_graphql
sed -i '' "1,/^version = /s/^version = .*/version = \"$RELEASE\"/" Cargo.toml
cargo build

cd ~/git/marco-sparko
sed -i '' -e "1,/^version = /s/^version = .*/version = \"$RELEASE\"/" \
  -e "s/^sparko_graphql = {version = \"[^\"]*\"/sparko_graphql = {version = \"$RELEASE\"/" Cargo.toml
cargo build

for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git diff --stat
  git commit -am "Start v$RELEASE"
  git push -u origin v$RELEASE
done
```

</details>

# Old Release Instructions
The manual build steps used before the workflow are in [Release-archive.md](Release-archive.md).
