use crate::records::compile_options::CompileOptions;

pub(crate) fn set_compile_options_for_native_compilation(options: &mut CompileOptions) {
    options.optimization_level = 2; // note: this might be removed in the future in favor of --!optimize
    options.type_info_level = 1;
}
