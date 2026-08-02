use data_lib::common::{I18N_FILE_ENDINGS, I18N_LOCALE_CODES};

use super::check_files::RepoFileSummary;

pub(super) fn has_i18n_files(summary: &RepoFileSummary) -> bool {
    summary.i18n_files
}

pub(super) fn is_i18n_file(file: &str) -> bool {
    let file_name = file.rsplit('/').next().unwrap_or(file);
    I18N_LOCALE_CODES.iter().any(|code| {
        I18N_FILE_ENDINGS
            .iter()
            .any(|ending| file_name == format!("{code}{ending}"))
    })
}
