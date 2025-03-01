# marco-sparko
# Release Process

These are the steps to create  release
## Create Tagged Source

Go to GitHub, on both the sparko_graphql and marco-sparko repos, create a release from ```main``` creating the tag ```releaseX.X.X``` where X.X.X is the version number.

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
total 64632
-rwxr-xr-x  1 bruce  staff  8680016  1 Mar 15:56 marco-sparko-0.3.0-aarch64-apple-darwin
-rwxr-xr-x  1 bruce  staff  7428096  1 Mar 15:56 marco-sparko-0.3.0-aarch64-pc-windows-msvc.exe
-rwxr-xr-x  1 bruce  staff  8777260  1 Mar 15:56 marco-sparko-0.3.0-x86_64-apple-darwin
-rwxr-xr-x  1 bruce  staff  8200192  1 Mar 15:56 marco-sparko-0.3.0-x86_64-pc-windows-msvc.exe
```

## Upload Binaries to GitHub
Go to the Release page on GitHub and upload the binaries.





