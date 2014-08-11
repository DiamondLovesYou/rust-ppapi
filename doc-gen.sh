#/bin/sh

SYSROOT="`readlink -f $1`"
NACL_SDK="`readlink -f $2`"
BUILD_DIR="`readlink -f build`"
DOCS_DIR="`readlink -f docs`"

export LD_LIBRARY_PATH=$SYSROOT/lib:$LD_LIBRARY_PATH

mkdir -p $BUILD_DIR
rm -fr tmp_src
rm -fr $DOCS_DIR/*
git clone https://github.com/DiamondLovesYou/rust-ppapi.git tmp_src &&
  cd tmp_src &&
  git submodule update --init &&
# We strictly speaking don't need to build libressl, but meh.
  remake clean &&
  remake SYSROOT="$SYSROOT" NACL_SDK="$NACL_SDK" BUILD_DIR="$BUILD_DIR" &&
  rustdoc -L $BUILD_DIR --target=le32-unknown-nacl -o $DOCS_DIR lib.rs

rm -fr tmp_src
