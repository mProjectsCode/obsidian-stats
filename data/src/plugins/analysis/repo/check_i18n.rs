use data_lib::common::{I18N_FILE_ENDINGS, I18N_LOCALE_CODES};

pub(super) fn has_i18n_files(files: &[String]) -> bool {
    files.iter().any(|file| {
        I18N_LOCALE_CODES.iter().any(|code| {
            I18N_FILE_ENDINGS
                .iter()
                .any(|ending| file == &format!("{code}{ending}"))
        })
    })
}
