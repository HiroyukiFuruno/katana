use crate::app_state::AppAction;

#[cfg(target_os = "macos")]
mod ffi {
    pub const TAG_OPEN_WORKSPACE: i32 = 1;
    pub const TAG_SAVE: i32 = 2;
    pub const TAG_LANG_EN: i32 = 3;
    pub const TAG_LANG_JA: i32 = 4;
    pub const TAG_ABOUT: i32 = 5;
    pub const TAG_SETTINGS: i32 = 6;
    pub const TAG_LANG_ZH_CN: i32 = 7;
    pub const TAG_LANG_ZH_TW: i32 = 8;
    pub const TAG_LANG_KO: i32 = 9;
    pub const TAG_LANG_PT: i32 = 10;
    pub const TAG_LANG_FR: i32 = 11;
    pub const TAG_LANG_DE: i32 = 12;
    pub const TAG_LANG_ES: i32 = 13;
    pub const TAG_LANG_IT: i32 = 14;
    pub const TAG_CHECK_UPDATES: i32 = 15;
    pub const TAG_RELEASE_NOTES: i32 = 16;

    #[allow(dead_code)]
    extern "C" {
        pub fn katana_setup_native_menu();
        pub fn katana_poll_menu_action() -> i32;
        pub fn katana_set_app_icon_png(png_data: *const u8, png_len: std::ffi::c_ulong);
        pub fn katana_set_process_name();
        pub fn native_free_menu_actions();
        pub fn katana_update_menu_strings(
            file: *const std::ffi::c_char,
            open_workspace: *const std::ffi::c_char,
            save: *const std::ffi::c_char,
            settings: *const std::ffi::c_char,
            preferences: *const std::ffi::c_char,
            language: *const std::ffi::c_char,
            about: *const std::ffi::c_char,
            quit: *const std::ffi::c_char,
            hide: *const std::ffi::c_char,
            hide_others: *const std::ffi::c_char,
            show_all: *const std::ffi::c_char,
            check_updates: *const std::ffi::c_char,
            help: *const std::ffi::c_char,
            release_notes: *const std::ffi::c_char,
        );
    }
}

#[allow(clippy::missing_safety_doc)]
#[cfg(all(target_os = "macos", not(test)))]
pub unsafe fn native_menu_setup() {
    ffi::katana_setup_native_menu();
}

#[allow(clippy::missing_safety_doc)]
#[cfg(all(target_os = "macos", not(test)))]
pub unsafe fn native_set_process_name() {
    ffi::katana_set_process_name();
}

#[allow(clippy::missing_safety_doc)]
#[cfg(all(target_os = "macos", not(test)))]
pub unsafe fn native_set_app_icon_png(png_data: *const u8, png_len: usize) {
    ffi::katana_set_app_icon_png(png_data, png_len as std::ffi::c_ulong);
}

#[cfg(all(target_os = "macos", not(test)))]
#[allow(clippy::too_many_arguments)]
unsafe fn native_update_menu_strings(
    file: &str,
    open_workspace: &str,
    save: &str,
    settings: &str,
    preferences: &str,
    language: &str,
    about: &str,
    quit: &str,
    hide: &str,
    hide_others: &str,
    show_all: &str,
    check_updates: &str,
    help: &str,
    release_notes: &str,
) {
    let f = std::ffi::CString::new(file).unwrap_or_default();
    let ow = std::ffi::CString::new(open_workspace).unwrap_or_default();
    let s = std::ffi::CString::new(save).unwrap_or_default();
    let st = std::ffi::CString::new(settings).unwrap_or_default();
    let p = std::ffi::CString::new(preferences).unwrap_or_default();
    let l = std::ffi::CString::new(language).unwrap_or_default();
    let a = std::ffi::CString::new(about).unwrap_or_default();
    let q = std::ffi::CString::new(quit).unwrap_or_default();
    let h = std::ffi::CString::new(hide).unwrap_or_default();
    let ho = std::ffi::CString::new(hide_others).unwrap_or_default();
    let sa = std::ffi::CString::new(show_all).unwrap_or_default();
    let cu = std::ffi::CString::new(check_updates).unwrap_or_default();
    let hlp = std::ffi::CString::new(help).unwrap_or_default();
    let rn = std::ffi::CString::new(release_notes).unwrap_or_default();
    ffi::katana_update_menu_strings(
        f.as_ptr(),
        ow.as_ptr(),
        s.as_ptr(),
        st.as_ptr(),
        p.as_ptr(),
        l.as_ptr(),
        a.as_ptr(),
        q.as_ptr(),
        h.as_ptr(),
        ho.as_ptr(),
        sa.as_ptr(),
        cu.as_ptr(),
        hlp.as_ptr(),
        rn.as_ptr(),
    );
}

#[cfg(all(target_os = "macos", not(test)))]
pub fn update_native_menu_strings_from_i18n() {
    let msgs = crate::i18n::get();
    let preferences = format!("{}…", msgs.menu.settings);
    unsafe {
        native_update_menu_strings(
            &msgs.menu.file,
            &msgs.menu.open_workspace,
            &msgs.menu.save,
            &msgs.menu.settings,
            &preferences,
            &msgs.menu.language,
            &msgs.menu.about,
            &msgs.menu.quit,
            &msgs.menu.hide,
            &msgs.menu.hide_others,
            &msgs.menu.show_all,
            &msgs.menu.check_updates,
            &msgs.menu.help,
            &msgs.menu.release_notes,
        );
    }
}

#[cfg(any(not(target_os = "macos"), test))]
pub fn update_native_menu_strings_from_i18n() {}

#[cfg(target_os = "macos")]
pub(crate) fn poll_native_menu(
    show_about: &mut bool,
    open_folder_dialog: fn() -> Option<std::path::PathBuf>,
) -> AppAction {
    let action = unsafe { ffi::katana_poll_menu_action() };
    match action {
        ffi::TAG_OPEN_WORKSPACE => {
            if let Some(path) = open_folder_dialog() {
                AppAction::OpenWorkspace(path)
            } else {
                AppAction::None
            }
        }
        ffi::TAG_SAVE => AppAction::SaveDocument,
        ffi::TAG_LANG_EN => AppAction::ChangeLanguage("en".to_string()),
        ffi::TAG_LANG_JA => AppAction::ChangeLanguage("ja".to_string()),
        ffi::TAG_LANG_ZH_CN => AppAction::ChangeLanguage("zh-CN".to_string()),
        ffi::TAG_LANG_ZH_TW => AppAction::ChangeLanguage("zh-TW".to_string()),
        ffi::TAG_LANG_KO => AppAction::ChangeLanguage("ko".to_string()),
        ffi::TAG_LANG_PT => AppAction::ChangeLanguage("pt".to_string()),
        ffi::TAG_LANG_FR => AppAction::ChangeLanguage("fr".to_string()),
        ffi::TAG_LANG_DE => AppAction::ChangeLanguage("de".to_string()),
        ffi::TAG_LANG_ES => AppAction::ChangeLanguage("es".to_string()),
        ffi::TAG_LANG_IT => AppAction::ChangeLanguage("it".to_string()),
        ffi::TAG_ABOUT => {
            *show_about = !*show_about;
            AppAction::None
        }
        ffi::TAG_CHECK_UPDATES => AppAction::CheckForUpdates,
        ffi::TAG_RELEASE_NOTES => AppAction::ShowReleaseNotes,
        ffi::TAG_SETTINGS => AppAction::ToggleSettings,
        _ => AppAction::None,
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn poll_native_menu(
    _show_about: &mut bool,
    _open_folder_dialog: fn() -> Option<std::path::PathBuf>,
) -> AppAction {
    AppAction::None
}
