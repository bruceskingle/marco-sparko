VERSION=`grep "^version = " Cargo.toml | sed -e 's/[^"]*"\([^"]*\)"/\1/'`
echo Gathering build artefacts for version $VERSION...
mkdir -p build
scp $BUILDER_X86_64_APPLE_DARWIN:git/marco-sparko/target/release/marco-sparko build/marko-sparko-$VERSION-x86_64-apple-darwin
cp target/release/marco-sparko build/marko-sparko-$VERSION-aarch64-apple-darwin
cp target/x86_64-pc-windows-msvc/release/marco-sparko.exe build/marko-sparko-$VERSION-x86_64-pc-windows-msvc
