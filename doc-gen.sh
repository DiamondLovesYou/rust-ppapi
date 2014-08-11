#/bin/sh

SYSROOT="`readlink -f $2`"
NACL_SDK="`readlink -f $3`"
BUILD_DIR="`readlink -f build`"
DOCS_DIR="`readlink -f docs`"

export LD_LIBRARY_PATH=$SYSROOT/lib:$LD_LIBRARY_PATH

mkdir -p $BUILD_DIR
git rm -fr $DOCS_DIR
mkdir -p $DOCS_DIR
cd $1 &&
# We strictly speaking don't need to build libressl, but meh.
  remake SYSROOT="$SYSROOT" NACL_SDK="$NACL_SDK" BUILD_DIR="$BUILD_DIR" &&
  rustdoc -L $BUILD_DIR --target=le32-unknown-nacl -o $DOCS_DIR src/lib.rs

cd $4 &&
  git add docs
