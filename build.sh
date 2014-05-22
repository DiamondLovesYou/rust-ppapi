#!/bin/sh

BINDGEN_DIR=$1
NACL_DIR=$2
NACL_INCLUDE_DIR=$NACL_DIR/include

OUT=ffi.rs

rm -f $OUT
echo "#![allow(non_camel_case_types)]" >> $OUT
echo "#![allow(uppercase_variables)]"  >> $OUT

$BINDGEN_DIR/bindgen ppapi.hpp -I $NACL_INCLUDE_DIR -match ppapi/c -match nacl_io -match sys/mount -match ppapi.hpp -match usr/include/stdint.h -emit-clang-ast -o temp
#perl -i -e 's/pub\\sstruct\\sStruct_PP_Dummy_Struct_For_(PP_.*_Dev)\\s\\{\\s*_COMPILE_ASSERT_FAILED_The_type_named_\\g1_is_not_[0-9]+_bytes_wide:\\s\\[c_schar,\\s\\.\\.1u\\],\\s*\\}//' $OUT

cat temp >> $OUT
rm temp

echo "pub type PP_Var = Struct_PP_Var;"               >> $OUT
echo "pub type PP_Rect = Struct_PP_Rect;"             >> $OUT
echo "pub type PP_Point = Struct_PP_Point;"           >> $OUT
echo "pub type PP_FloatPoint = Struct_PP_FloatPoint;" >> $OUT
echo "pub type PP_TouchPoint = Struct_PP_TouchPoint;" >> $OUT
echo "pub type PP_Size = Struct_PP_Size;"             >> $OUT
echo "pub type PPB_OpenGLES2 = Struct_PPB_OpenGLES2;" >> $OUT
