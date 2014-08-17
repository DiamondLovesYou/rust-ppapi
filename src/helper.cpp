// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// Rust PPApi is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Rust PPApi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Rust PPApi. If not, see <http://www.gnu.org/licenses/>.

/// This file is here to handle thing that the ppapi has either inlined or delegated to macros.
#include "ppapi.hpp"
#include "stdio.h"

#include "ppapi/c/pp_completion_callback.h"

extern "C" {
  PP_CompletionCallback make_completion_callback(PP_CompletionCallback_Func func,
                                                 void* user_data) {
    return PP_MakeCompletionCallback(func, user_data);
  }
  void run_completion_callback(PP_CompletionCallback func, const int32_t code) {
    PP_RunCompletionCallback(&func, code);
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


  FILE* stdout_file() {
    return stdin;
  }
  FILE* stderr_file() {
    return stderr;
  }
}
