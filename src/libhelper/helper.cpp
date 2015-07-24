// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// This file is here to handle things that the ppapi has either inlined or
/// delegated to macros.

#ifdef __BINDGEN__
#include "builtin_defines.hpp" // For bindgen.
#endif

#include "ppapi.hpp"
#include "stdio.h"

#include "ppapi/c/pp_completion_callback.h"

#ifdef __cplusplus
extern "C" {
#endif

  PP_CompletionCallback make_completion_callback(PP_CompletionCallback_Func func,
                                                 void* user_data) {
    return PP_MakeCompletionCallback(func, user_data);
  }
  void run_completion_callback(PP_CompletionCallback func, const int32_t code) {
    PP_RunCompletionCallback(&func, code);
  }
  PP_CompletionCallback block_until_complete() {
    return PP_BlockUntilComplete();
  }

  const PP_Var make_undefined_var() {
    return PP_MakeUndefined();
  }
  const PP_Var make_null_var() {
    return PP_MakeNull();
  }

  const PP_Var bool_to_var(const bool value) {
    return PP_MakeBool(static_cast<PP_Bool>(value));
  }
  const bool bool_from_var(const PP_Var v) {
    return v.value.as_bool;
  }

  const PP_Var i32_to_var(const int32_t value) {
    return PP_MakeInt32(value);
  }
  const int32_t i32_from_var(const PP_Var v) {
    return v.value.as_int;
  }

  const PP_Var f64_to_var(const double value) {
    return PP_MakeDouble(value);
  }
  const double f64_from_var(const PP_Var v) {
    return v.value.as_double;
  }

  const PP_Var string_id_to_var(const int64_t id) {
    PP_VarValue vv;
    vv.as_id = id;
    const PP_Var v = {
      PP_VARTYPE_STRING,
      0,
      vv,
    };
    return v;
  }
  const PP_Var object_id_to_var(const int64_t id) {
    PP_VarValue vv;
    vv.as_id = id;
    const PP_Var v = {
      PP_VARTYPE_OBJECT,
      0,
      vv,
    };
    return v;
  }
  const PP_Var array_id_to_var(const int64_t id) {
    PP_VarValue vv;
    vv.as_id = id;
    const PP_Var v = {
      PP_VARTYPE_ARRAY,
      0,
      vv,
    };
    return v;
  }
  const PP_Var dictionary_id_to_var(const int64_t id) {
    PP_VarValue vv;
    vv.as_id = id;
    const PP_Var v = {
      PP_VARTYPE_DICTIONARY,
      0,
      vv,
    };
    return v;
  }
  const PP_Var array_buffer_id_to_var(const int64_t id) {
    PP_VarValue vv;
    vv.as_id = id;
    const PP_Var v = {
      PP_VARTYPE_ARRAY_BUFFER,
      0,
      vv,
    };
    return v;
  }
  const int64_t id_from_var(const PP_Var v) {
    return v.value.as_id;
  }

#ifdef __cplusplus
}
#endif
