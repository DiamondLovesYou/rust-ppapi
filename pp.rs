use ppapi::ffi;

mod globals {
    use ppapi::ffi;

    pub static mut MODULE: ffi::PP_Module = 0 as ffi::PP_Module;
}
// Initialize the global module handle.
pub fn initialize_globals(module: ffi::PP_Module) {
    unsafe {
        globals::MODULE = module;
    }
}

// Return a clone of the module handle.
pub fn get_module() -> ffi::PP_Module {
    unsafe {
        globals::MODULE
    }
}
