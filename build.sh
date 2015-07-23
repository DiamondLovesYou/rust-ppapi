#!/bin/sh

BINDGEN_DIR=$1
NACL_DIR=$NACL_SDK_ROOT
NACL_INCLUDE_DIR=$NACL_DIR/include

OUT=src/lib/ffi.rs

rm -f $OUT
echo "#![allow(non_camel_case_types)]" >> $OUT
echo "#![allow(non_snake_case)]"       >> $OUT
echo "#![allow(raw_pointer_derive)]"   >> $OUT
echo "#![allow(missing_copy_implementations)]" >> $OUT

($NACL_DIR/toolchain/linux_pnacl/bin/pnacl-clang++ -std=c++11 -dM -E -x c++ - < /dev/null) > src/libhelper/builtin_defines.hpp

export LD_LIBRARY_PATH=$BINDGEN_DIR:$LD_LIBRARY_PATH

$BINDGEN_DIR/bindgen -nostdinc -I $NACL_INCLUDE_DIR -isystem $NACL_DIR/toolchain/linux_pnacl/le32-nacl/include -isystem $NACL_DIR/toolchain/linux_pnacl/le32-nacl/include/c++/v1 -isystem $NACL_DIR/include/pnacl -isystem $NACL_DIR/toolchain/linux_pnacl/lib/clang/3.7.0/include -target le32-unknown-nacl src/libhelper/helper.cpp -pthread -o temp -D__BINDGEN__ -std=c++11
#perl -i -e 's/pub\\sstruct\\sStruct_PP_Dummy_Struct_For_(PP_.*_Dev)\\s\\{\\s*_COMPILE_ASSERT_FAILED_The_type_named_\\g1_is_not_[0-9]+_bytes_wide:\\s\\[c_schar,\\s\\.\\.1u\\],\\s*\\}//' $OUT

# -isystem $NACL_DIR/toolchain/linux_pnacl/usr/include/c++/v1/arm-none-linux-gnueabi -isystem $NACL_DIR/toolchain/linux_pnacl/usr/include/c++/v1/backward -isystem $NACL_DIR/toolchain/linux_pnacl/include/c++/v1

cat temp >> $OUT
rm temp

echo "pub type PP_Var = Struct_PP_Var;"               >> $OUT
echo "pub type PP_Rect = Struct_PP_Rect;"             >> $OUT
echo "pub type PP_Point = Struct_PP_Point;"           >> $OUT
echo "pub type PP_FloatPoint = Struct_PP_FloatPoint;" >> $OUT
echo "pub type PP_TouchPoint = Struct_PP_TouchPoint;" >> $OUT
echo "pub type PP_Size = Struct_PP_Size;"             >> $OUT
echo "pub type PPB_OpenGLES2 = Struct_PPB_OpenGLES2;" >> $OUT
echo "pub type PP_CompletionCallback = Struct_PP_CompletionCallback;" >> $OUT
