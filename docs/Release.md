# marco-sparko
# Release Process

These are the steps to create  release
## Create Tagged Source
Ensure that all feature branches have been merged to main and that Cargo.toml shows the correct version number for the new release.

Go to GitHub, on both the sparko_graphql and marco-sparko repos, create a release from ```main``` creating the tag ```releaseX.X.X``` where X.X.X is the version number.

# New Process
## Build Apple Silicone OSX Release

Go to the main dev machine

```
cd ~/git/sparko_graphql
git checkout releaseX.X.X
cd ~/git/marco-sparko
git checkout releaseX.X.X
dx bundle --release
ls -l target/dx/marco-sparko/bundle/macos/bundle/dmg
```

You should see a dmg bundle like this:

```
total 11232
-rwxrwxrwx  1 bruce  staff    19300 15 Jan 19:11 bundle_dmg.sh
-rw-r--r--@ 1 bruce  staff  5323104 15 Jan 19:12 MarcoSparko_0.3.0_aarch64.dmg
-rw-r--r--@ 1 bruce  staff   402941 15 Jan 17:57 MarcoSparko.icns
```

# Old Process
## Build Apple Silicone OSX and Windows Releases

Go to the main dev machine

```
cd ~/git/sparko_graphql
git checkout releaseX.X.X
cd ~/git/marco-sparko
git checkout releaseX.X.X
cargo build --release
cross build --release --target x86_64-pc-windows-msvc
cross build --release --target aarch64-pc-windows-msvc
```

## Build x86_64 OSX and Windows Releases

Go to the OSX x86_64 build machine

```
cd ~/git/sparko_graphql
git checkout releaseX.X.X
cd ~/git/marco-sparko
git checkout releaseX.X.X
cargo build --release
```

## Gather Release Binaries
Go to the main dev machine

```
cd ~/git/marco-sparko
./gather_builds.sh
% ls -l build
```

You should see newly created binaries like this:

```
-rw-r--r--@ 1 bruce  staff  3331675  3 Mar 16:46 marco-sparko-0.3.0-aarch64-apple-darwin.dmg
-rwxr-xr-x  1 bruce  staff  7417856  3 Mar 16:46 marco-sparko-0.3.0-aarch64-pc-windows-msvc.exe
-rw-r--r--@ 1 bruce  staff  3415115  3 Mar 16:46 marco-sparko-0.3.0-x86_64-apple-darwin.dmg
-rwxr-xr-x  1 bruce  staff  8187904  3 Mar 16:46 marco-sparko-0.3.0-x86_64-pc-windows-msvc.exe
```

## Upload Binaries to GitHub
Go to the Release page on GitHub and upload the binaries.





