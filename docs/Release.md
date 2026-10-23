# marco-sparko
# Release Process

Each release is developed on a branch called `vX.X.X` in both sparko_graphql and marco-sparko, which is squash
merged to `main` and tagged `releaseX.X.X`. Publishing the marco-sparko release on GitHub builds and attaches the
installers.

The commands below are for zsh on the main dev machine, with both repos checked out in `~/git`. They need the GitHub
CLI, `gh`, signed in with `gh auth login`. Start each terminal session by setting the version being worked on:

```
export RELEASE=0.4.4
```

## 1. Start the Release Branch
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

## 2. Develop
Commit to `v$RELEASE` and push as usual.

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

## 3. Try the Installers (optional)
The installers can be built from the release branch before anything is merged, to try them on other machines. The
workflow builds a tag, so with the release branches checked out make a temporary one in both repos:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag test$RELEASE
  git push origin test$RELEASE
done

cd ~/git/marco-sparko
gh workflow run release.yml --ref v$RELEASE -f tag=test$RELEASE
```

Wait a few seconds for the run to start, then watch it and download the installers when it has finished (about
30 to 40 minutes):

```
cd ~/git/marco-sparko
gh run watch $(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
gh run download $(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId') --dir ~/Downloads/marco-sparko-$RELEASE
```

`--ref` runs the version of the workflow on the release branch, so changes to it can be tried before they are
merged. GitHub may refuse to run a workflow which is not on `main` yet, in which case this step is only possible once
the workflow has been released.

Delete the temporary tags afterwards, they can be made again for another try:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
  git tag -d test$RELEASE
  git push origin --delete test$RELEASE
done
```

## 4. Merge to Main and Tag
Make sure everything is committed and pushed. Then, sparko_graphql first, create a pull request and squash merge it
to `main`:

```
for repo in sparko_graphql marco-sparko; do
  cd ~/git/$repo
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

## 5. Final Check
Do a clean build of the tagged source and click around the app:

```
cd ~/git/sparko_graphql
git checkout release$RELEASE
cd ~/git/marco-sparko
git checkout release$RELEASE
cargo clean
dx run --release
```

If something needs fixing, fix it on `main`, then move the tags to the fixed commit (in whichever repos changed):

```
git checkout main
# ... commit the fix
git push
git tag -f release$RELEASE
git push -f origin release$RELEASE
```

## 6. Release
Publish the release. This starts the `Release` workflow in
[.github/workflows/release.yml](../.github/workflows/release.yml), which builds and tests the installers on GitHub and
attaches them to the release:

```
cd ~/git/marco-sparko
gh release create release$RELEASE --verify-tag --title "Release $RELEASE" --generate-notes
```

Wait a few seconds, then watch the build (about 30 to 40 minutes):

```
gh run watch $(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
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

The files are only attached if all four builds succeed. If one fails, look at the failed step in the run on the
Actions tab of the marco-sparko repo. If the problem was temporary, re-run the failed jobs:

```
gh run rerun $(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId') --failed
```

Otherwise fix it as in step 5, then delete the release (leaving the tag) and create it again:

```
gh release delete release$RELEASE --yes
gh release create release$RELEASE --verify-tag --title "Release $RELEASE" --generate-notes
```

Then set `RELEASE` to the next version and start again at step 1.

# Old Release Instructions
The manual build steps used before the workflow are in [Release-archive.md](Release-archive.md).
