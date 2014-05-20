// for now, we just include every file one might need for one very large mod.
// TODO(richard): separate mods for each header

#include <ppapi/c/pp_macros.h>

#include <ppapi/c/ppb.h>
#include <ppapi/c/ppp.h>

#include <ppapi/c/ppb_audio.h>
#if PPAPI_RELEASE >= 34
#include <ppapi/c/ppb_audio_buffer.h>
#endif
#include <ppapi/c/ppb_audio_config.h>
#include <ppapi/c/ppb_console.h>
#include <ppapi/c/ppb_core.h>
#include <ppapi/c/ppb_file_io.h>
#include <ppapi/c/ppb_file_ref.h>
#include <ppapi/c/ppb_file_system.h>
#include <ppapi/c/ppb_fullscreen.h>
#include <ppapi/c/ppb_gamepad.h>
#include <ppapi/c/ppb_graphics_2d.h>
#include <ppapi/c/ppb_graphics_3d.h>
#include <ppapi/c/ppb_host_resolver.h>
#include <ppapi/c/ppb_image_data.h>
#include <ppapi/c/ppb_input_event.h>
#include <ppapi/c/ppb_instance.h>
#if PPAPI_RELEASE >= 34
#include <ppapi/c/ppb_media_stream_audio_track.h>
#include <ppapi/c/ppb_media_stream_video_track.h>
#endif
#include <ppapi/c/ppb_message_loop.h>
#include <ppapi/c/ppb_messaging.h>
#include <ppapi/c/ppb_mouse_cursor.h>
#include <ppapi/c/ppb_mouse_lock.h>
#include <ppapi/c/ppb_net_address.h>
#include <ppapi/c/ppb_network_list.h>
#include <ppapi/c/ppb_network_monitor.h>
#include <ppapi/c/ppb_network_proxy.h>
#include <ppapi/c/ppb_opengles2.h>
#include <ppapi/c/ppb_tcp_socket.h>
#include <ppapi/c/ppb_text_input_controller.h>
#include <ppapi/c/ppb_udp_socket.h>
#include <ppapi/c/ppb_url_loader.h>
#include <ppapi/c/ppb_url_request_info.h>
#include <ppapi/c/ppb_url_response_info.h>
#include <ppapi/c/ppb_var.h>
#include <ppapi/c/ppb_var_array.h>
#include <ppapi/c/ppb_var_array_buffer.h>
#include <ppapi/c/ppb_var_dictionary.h>
#include <ppapi/c/ppb_view.h>
#include <ppapi/c/ppb_websocket.h>

#include <ppapi/c/pp_completion_callback.h>
#include <ppapi/c/pp_directory_entry.h>
#include <ppapi/c/pp_errors.h>
#include <ppapi/c/pp_file_info.h>
#include <ppapi/c/pp_graphics_3d.h>
#include <ppapi/c/pp_input_event.h>
#include <ppapi/c/pp_instance.h>
#include <ppapi/c/pp_module.h>
#include <ppapi/c/pp_point.h>
#include <ppapi/c/pp_rect.h>
#include <ppapi/c/pp_resource.h>
#include <ppapi/c/pp_size.h>
#include <ppapi/c/pp_stdint.h>
#include <ppapi/c/pp_time.h>
#include <ppapi/c/pp_touch_point.h>
#include <ppapi/c/pp_var.h>

#include <ppapi/c/ppp_graphics_3d.h>
#include <ppapi/c/ppp_input_event.h>
#include <ppapi/c/ppp_instance.h>
#include <ppapi/c/ppp_messaging.h>
#include <ppapi/c/ppp_mouse_lock.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppb_audio_input_dev.h>
#include <ppapi/c/dev/ppb_buffer_dev.h>
#include <ppapi/c/dev/ppb_char_set_dev.h>
#include <ppapi/c/dev/ppb_crypto_dev.h>
#endif
#include <ppapi/c/dev/ppb_cursor_control_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppb_device_ref_dev.h>
#endif
#include <ppapi/c/dev/ppb_file_chooser_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppb_find_dev.h>
#endif
#include <ppapi/c/dev/ppb_font_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppb_gles_chromium_texture_mapping_dev.h>
#include <ppapi/c/dev/ppb_graphics_2d_dev.h>
#include <ppapi/c/dev/ppb_ime_input_event_dev.h>
#endif
#if PPAPI_RELEASE < 34
#include <ppapi/c/dev/ppb_keyboard_input_event_dev.h>
#endif
#include <ppapi/c/dev/ppb_memory_dev.h>
#include <ppapi/c/dev/ppb_opengles2ext_dev.h>
#include <ppapi/c/dev/ppb_printing_dev.h>
#if PPAPI_RELEASE < 34
#include <ppapi/c/dev/ppb_resource_array_dev.h>
#endif
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppb_scrollbar_dev.h>
#include <ppapi/c/dev/ppb_text_input_dev.h>
#endif
#include <ppapi/c/dev/ppb_trace_event_dev.h>
#include <ppapi/c/dev/ppb_truetype_font_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppb_url_util_dev.h>
#include <ppapi/c/dev/ppb_video_capture_dev.h>
#include <ppapi/c/dev/ppb_video_decoder_dev.h>
#endif
#include <ppapi/c/dev/ppb_view_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppb_widget_dev.h>
#endif
#include <ppapi/c/dev/ppb_zoom_dev.h>
#include <ppapi/c/dev/pp_cursor_type_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppp_find_dev.h>
#endif
#include <ppapi/c/dev/ppp_network_state_dev.h>
#include <ppapi/c/dev/ppp_printing_dev.h>
#include <ppapi/c/dev/pp_print_settings_dev.h>
#include <ppapi/c/dev/ppp_scrollbar_dev.h>
#include <ppapi/c/dev/ppp_selection_dev.h>
#include <ppapi/c/dev/ppp_text_input_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/ppp_video_capture_dev.h>
#include <ppapi/c/dev/ppp_video_decoder_dev.h>
#include <ppapi/c/dev/ppp_widget_dev.h>
#endif
#include <ppapi/c/dev/ppp_zoom_dev.h>
#if PPAPI_RELEASE <= 34
#include <ppapi/c/dev/pp_video_capture_dev.h>
#include <ppapi/c/dev/pp_video_dev.h>
#endif

#include <nacl_io/nacl_io.h>
#include <sys/mount.h>       // for mount.

// helpers:
extern "C" {
  PP_CompletionCallback make_completion_callback(PP_CompletionCallback_Func func,
                                                 void* user_data);
  const PP_Var make_undefined_var();
  const PP_Var make_null_var();

  const PP_Var bool_to_var(const bool value);
  const bool bool_from_var(const PP_Var v);

  const PP_Var i32_to_var(const int32_t value);
  const int32_t i32_from_var(const PP_Var v);

  const PP_Var f64_to_var(const double value);
  const double f64_from_var(const PP_Var v);

  const PP_Var string_id_to_var(const int64_t id);
  const PP_Var object_id_to_var(const int64_t id);
  const PP_Var array_id_to_var(const int64_t id);
  const PP_Var dictionary_id_to_var(const int64_t id);
  const PP_Var array_buffer_id_to_var(const int64_t id);
  const int64_t id_from_var(const PP_Var v);
}
