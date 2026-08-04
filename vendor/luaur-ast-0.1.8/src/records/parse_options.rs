use crate::records::fragment_parse_resume_settings::FragmentParseResumeSettings;

#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub allow_declaration_syntax: bool,
    pub capture_comments: bool,
    pub parse_fragment: Option<FragmentParseResumeSettings>,
    pub store_cst_data: bool,
    pub no_error_limit: bool,
}
