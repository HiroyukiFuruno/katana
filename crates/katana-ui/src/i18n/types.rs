use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct I18nMessages {
    pub menu: MenuMessages,
    pub workspace: WorkspaceMessages,
    pub preview: PreviewMessages,
    pub plantuml: PlantumlMessages,
    pub view_mode: ViewModeMessages,
    pub split_toggle: SplitToggleMessages,
    pub error: ErrorMessages,
    pub status: StatusMessages,
    pub action: ActionMessages,
    pub ai: AiMessages,
    pub tool: ToolMessages,
    pub settings: SettingsMessages,
    pub tab: TabMessages,
    pub search: SearchMessages,
    pub about: AboutMessages,
    pub update: UpdateMessages,
    pub toc: TocMessages,
    #[serde(default)]
    pub export: ExportMessages,
    #[serde(default)]
    pub terms: TermsMessages,
    #[serde(default)]
    pub dialog: DialogMessages,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExportMessages {
    pub success: String,
    pub failed: String,
    pub tool_missing: String,
    pub temp_file_error: String,
    pub write_error: String,
    pub persist_error: String,
    pub exporting: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TermsMessages {
    pub title: String,
    pub version_label: String,
    pub content: String,
    pub accept: String,
    pub decline: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TocMessages {
    pub title: String,
    pub empty: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchMessages {
    pub modal_title: String,
    pub query_hint: String,
    pub include_pattern_hint: String,
    pub exclude_pattern_hint: String,
    pub no_results: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AboutMessages {
    pub basic_info: String,
    pub version: String,
    pub build: String,
    pub copyright: String,
    pub runtime: String,
    pub platform: String,
    pub architecture: String,
    pub rust: String,
    pub license: String,
    pub links: String,
    pub source_code: String,
    pub documentation: String,
    pub report_issue: String,
    pub support: String,
    pub sponsor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMessages {
    pub title: String,
    pub checking_for_updates: String,
    pub update_available: String,
    pub update_available_desc: String,
    pub up_to_date: String,
    pub up_to_date_desc: String,
    pub failed_to_check: String,
    pub action_close: String,
    #[serde(default = "default_install_update")]
    pub install_update: String,
    #[serde(default = "default_downloading")]
    pub downloading: String,
    #[serde(default = "default_installing")]
    pub installing: String,
    #[serde(default = "default_restart_confirm")]
    pub restart_confirm: String,
    #[serde(default = "default_action_later")]
    pub action_later: String,
    #[serde(default = "default_action_skip_version")]
    pub action_skip_version: String,
    #[serde(default = "default_action_restart")]
    pub action_restart: String,
}

pub(crate) fn default_install_update() -> String {
    "Install and Relaunch".to_string()
}
pub(crate) fn default_downloading() -> String {
    "Downloading update...".to_string()
}
pub(crate) fn default_installing() -> String {
    "Installing update...".to_string()
}
pub(crate) fn default_restart_confirm() -> String {
    "Update is ready. Restart now?".to_string()
}
pub(crate) fn default_action_later() -> String {
    "Later".to_string()
}
pub(crate) fn default_action_skip_version() -> String {
    "Skip This Version".to_string()
}
pub(crate) fn default_action_restart() -> String {
    "Restart Now".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuMessages {
    pub file: String,
    pub settings: String,
    pub language: String,
    pub open_workspace: String,
    pub save: String,
    pub open_all: String,
    pub about: String,
    pub quit: String,
    pub hide: String,
    pub hide_others: String,
    pub show_all: String,
    #[serde(default)]
    pub export: String,
    #[serde(default)]
    pub export_html: String,
    #[serde(default)]
    pub export_pdf: String,
    #[serde(default)]
    pub export_png: String,
    #[serde(default)]
    pub export_jpg: String,
    pub help: String,
    pub check_updates: String,
    #[serde(default = "default_menu_release_notes")]
    pub release_notes: String,
}

pub(crate) fn default_menu_release_notes() -> String {
    "Release Notes".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceMessages {
    pub no_workspace_open: String,
    pub no_document_selected: String,
    pub workspace_title: String,
    pub recent_workspaces: String,
    #[serde(default = "default_metadata_tooltip")]
    pub metadata_tooltip: String,
    pub path_label: String,
}

pub(crate) fn default_metadata_tooltip() -> String {
    "Size: {size} B\nModified: {mod_time}".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagramControllerMessages {
    pub pan_up: String,
    pub pan_down: String,
    pub pan_left: String,
    pub pan_right: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub reset: String,
    pub fullscreen: String,
    pub close: String,
    #[serde(default = "default_trackpad_help")]
    pub trackpad_help: String,
}

pub(crate) fn default_trackpad_help() -> String {
    "Trackpad: 2-finger pinch to zoom, 1-finger drag to pan".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewMessages {
    pub preview_title: String,
    pub refresh_diagrams: String,
    pub rendering: String,
    pub no_preview: String,
    #[serde(default = "default_diagram_controller")]
    pub diagram_controller: DiagramControllerMessages,
}

pub(crate) fn default_diagram_controller() -> DiagramControllerMessages {
    DiagramControllerMessages {
        pan_up: "Move up".to_string(),
        pan_down: "Move down".to_string(),
        pan_left: "Move left".to_string(),
        pan_right: "Move right".to_string(),
        zoom_in: "Zoom in".to_string(),
        zoom_out: "Zoom out".to_string(),
        reset: "Reset position and size".to_string(),
        fullscreen: "Fullscreen".to_string(),
        close: "Close".to_string(),
        trackpad_help: default_trackpad_help(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlantumlMessages {
    pub downloading_plantuml: String,
    pub plantuml_installed: String,
    pub download_error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewModeMessages {
    pub preview: String,
    pub code: String,
    pub split: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SplitToggleMessages {
    pub horizontal: String,
    pub vertical: String,
    pub editor_first: String,
    pub preview_first: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMessages {
    pub missing_dependency: String,
    pub curl_launch_failed: String,
    pub download_failed: String,
    pub render_error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusMessages {
    pub ready: String,
    pub saved: String,
    pub save_failed: String,
    pub opened_workspace: String,
    pub cannot_open_workspace: String,
    pub cannot_open_file: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ActionMessages {
    pub expand_all: String,
    pub collapse_all: String,
    pub collapse_sidebar: String,
    pub refresh_workspace: String,
    pub toggle_filter: String,
    pub remove_workspace: String,
    #[serde(default)]
    pub recursive_expand: String,
    #[serde(default)]
    pub recursive_open_all: String,
    #[serde(default)]
    pub toggle_toc: String,
    pub show_meta_info: String,
    #[serde(default = "default_action_new_file")]
    pub new_file: String,
    #[serde(default = "default_action_new_directory")]
    pub new_directory: String,
    #[serde(default = "default_action_open")]
    pub open: String,
    #[serde(default = "default_action_rename")]
    pub rename: String,
    #[serde(default = "default_action_delete")]
    pub delete: String,
    #[serde(default = "default_action_copy_path")]
    pub copy_path: String,
    #[serde(default = "default_action_copy_relative_path")]
    pub copy_relative_path: String,
    #[serde(default = "default_action_reveal_in_os")]
    pub reveal_in_os: String,
    #[serde(default = "default_action_save")]
    pub save: String,
    #[serde(default = "default_action_cancel")]
    pub cancel: String,
    #[serde(default = "default_action_discard")]
    pub discard: String,
    #[serde(default = "default_action_confirm")]
    pub confirm: String,
}

pub(crate) fn default_action_confirm() -> String {
    "Confirm".to_string()
}

pub(crate) fn default_action_new_file() -> String {
    "New File".to_string()
}
pub(crate) fn default_action_new_directory() -> String {
    "New Folder".to_string()
}
pub(crate) fn default_action_open() -> String {
    "Open".to_string()
}
pub(crate) fn default_action_rename() -> String {
    "Rename".to_string()
}
pub(crate) fn default_action_delete() -> String {
    "Delete".to_string()
}
pub(crate) fn default_action_copy_path() -> String {
    "Copy Path".to_string()
}
pub(crate) fn default_action_copy_relative_path() -> String {
    "Copy Relative Path".to_string()
}
pub(crate) fn default_action_reveal_in_os() -> String {
    "Reveal in OS".to_string()
}
pub(crate) fn default_action_save() -> String {
    "Save".to_string()
}
pub(crate) fn default_action_cancel() -> String {
    "Cancel".to_string()
}
pub(crate) fn default_action_discard() -> String {
    "Discard".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DialogMessages {
    pub new_file_title: String,
    pub new_directory_title: String,
    pub rename_title: String,
    pub delete_title: String,
    pub delete_confirm_msg: String,
    #[serde(default = "default_unsaved_changes_title")]
    pub unsaved_changes_title: String,
    #[serde(default = "default_unsaved_changes_msg")]
    pub unsaved_changes_msg: String,
}

pub(crate) fn default_unsaved_changes_title() -> String {
    "Unsaved Changes".to_string()
}
pub(crate) fn default_unsaved_changes_msg() -> String {
    "Do you want to save the changes you made to {name}?\n\nYour changes will be lost if you don't save them.".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiMessages {
    pub ai_unconfigured: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolMessages {
    pub not_installed: String,
    pub install_path: String,
    pub download: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsMessages {
    pub title: String,
    pub tabs: Vec<SettingsTabMessage>,
    pub toc_visible: String,
    pub theme: SettingsThemeMessages,
    pub font: SettingsFontMessages,
    pub layout: SettingsLayoutMessages,
    pub workspace: SettingsWorkspaceMessages,
    #[serde(default)]
    pub updates: SettingsUpdatesMessages,
    #[serde(default)]
    pub behavior: SettingsBehaviorMessages,
    pub preview: SettingsPreviewMessages,
    pub color: SettingsColorMessages,
}

impl SettingsMessages {
    pub fn tab_name(&self, key: &str) -> String {
        self.tabs
            .iter()
            .find(|t| t.key == key)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| key.to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsThemeMessages {
    pub preset: String,
    pub dark_section: String,
    pub light_section: String,
    pub custom_colors: String,
    pub reset_custom: String,
    #[serde(default = "default_custom_section")]
    pub custom_section: String,
    #[serde(default = "default_delete_custom")]
    pub delete_custom: String,
    #[serde(default = "default_save_custom_theme")]
    pub save_custom_theme: String,
    #[serde(default = "default_save_custom_theme_title")]
    pub save_custom_theme_title: String,
    #[serde(default = "default_theme_name_label")]
    pub theme_name_label: String,
    #[serde(default = "default_duplicate")]
    pub duplicate: String,
    #[serde(default = "default_ui_contrast_offset")]
    pub ui_contrast_offset: String,
    #[serde(default = "default_show_more")]
    pub show_more: String,
    #[serde(default = "default_show_less")]
    pub show_less: String,
}

pub(crate) fn default_show_more() -> String {
    "Show more...".to_string()
}

pub(crate) fn default_show_less() -> String {
    "Show less...".to_string()
}

pub(crate) fn default_ui_contrast_offset() -> String {
    "UI Contrast Offset".to_string()
}

pub(crate) fn default_duplicate() -> String {
    "Duplicate...".to_string()
}

pub(crate) fn default_custom_section() -> String {
    "Custom".to_string()
}

pub(crate) fn default_delete_custom() -> String {
    "Delete Custom Theme".to_string()
}

pub(crate) fn default_save_custom_theme() -> String {
    "Save as Custom Theme...".to_string()
}

pub(crate) fn default_save_custom_theme_title() -> String {
    "Save Custom Theme".to_string()
}

pub(crate) fn default_theme_name_label() -> String {
    "Theme Name:".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsTabMessage {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsFontMessages {
    pub size: String,
    pub family: String,
    pub size_slider_hint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsLayoutMessages {
    pub split_direction: String,
    pub horizontal: String,
    pub vertical: String,
    pub pane_order: String,
    pub editor_first: String,
    pub preview_first: String,
    pub toc_position: String,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsWorkspaceMessages {
    pub max_depth: String,
    pub ignored_directories: String,
    pub ignored_directories_hint: String,
    #[serde(default = "default_visible_extensions_msg")]
    pub visible_extensions: String,
    #[serde(default = "default_no_extension_label")]
    pub no_extension_label: String,
    #[serde(default = "default_no_extension_warning_title")]
    pub no_extension_warning_title: String,
    #[serde(default = "default_no_extension_warning")]
    pub no_extension_warning: String,
    #[serde(default = "default_extensionless_excludes")]
    pub extensionless_excludes: String,
    #[serde(default = "default_extensionless_excludes_hint")]
    pub extensionless_excludes_hint: String,
}

pub(crate) fn default_extensionless_excludes() -> String {
    "Ignored Extensionless Files".to_string()
}
pub(crate) fn default_extensionless_excludes_hint() -> String {
    "Comma-separated list of exact file names to ignore when 'No Extension' is enabled (e.g., .DS_Store, .gitignore).".to_string()
}

pub(crate) fn default_no_extension_label() -> String {
    "No Extension".to_string()
}
pub(crate) fn default_no_extension_warning_title() -> String {
    "Warning".to_string()
}
pub(crate) fn default_no_extension_warning() -> String {
    "There is no guarantee that files without extensions can be displayed correctly as Markdown. Furthermore, the application may crash due to unexpected behavior. Are you sure you want to enable this?".to_string()
}

pub(crate) fn default_visible_extensions_msg() -> String {
    "Visible Extensions".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SettingsUpdatesMessages {
    pub section_title: String,
    pub interval: String,
    pub never: String,
    pub daily: String,
    pub weekly: String,
    pub monthly: String,
    pub check_now: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SettingsBehaviorMessages {
    pub section_title: String,
    pub confirm_close_dirty_tab: String,
    pub scroll_sync: String,
    pub auto_save: String,
    pub auto_save_interval: String,
    pub seconds: String,
    pub close_confirm_title: String,
    pub close_confirm_msg: String,
    pub close_confirm_discard: String,
    pub close_confirm_cancel: String,

    #[serde(default = "default_clear_http_cache")]
    pub clear_http_cache: String,

    #[serde(default = "default_cache_retention_days")]
    pub cache_retention_days: String,

    #[serde(default = "default_days_suffix")]
    pub days_suffix: String,
}

pub(crate) fn default_clear_http_cache() -> String {
    "Clear All Caches".to_string()
}
pub(crate) fn default_cache_retention_days() -> String {
    "Cache Retention Days".to_string()
}
pub(crate) fn default_days_suffix() -> String {
    " days".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsPreviewMessages {
    pub title: String,
    pub heading: String,
    pub normal_text: String,
    pub accent_link: String,
    pub secondary_text: String,
    pub code_sample: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsColorMessages {
    pub background: String,
    pub panel_background: String,
    pub text: String,
    pub text_secondary: String,
    pub accent: String,
    pub border: String,
    pub selection: String,
    pub code_background: String,
    pub preview_background: String,
    #[serde(default = "default_section_system")]
    pub section_system: String,
    #[serde(default = "default_section_code")]
    pub section_code: String,
    #[serde(default = "default_section_preview")]
    pub section_preview: String,
    #[serde(default = "default_group_basic")]
    pub group_basic: String,
    #[serde(default = "default_group_text")]
    pub group_text: String,
    #[serde(default = "default_group_ui_elements")]
    pub group_ui_elements: String,
    #[serde(default = "default_highlight")]
    pub highlight: String,
    #[serde(default = "default_code_text")]
    pub code_text: String,
    #[serde(default = "default_preview_text")]
    pub preview_text: String,

    #[serde(default = "d_title_bar_text")]
    pub title_bar_text: String,

    #[serde(default = "d_active_file_highlight")]
    pub active_file_highlight: String,
    #[serde(default = "d_file_tree_text")]
    pub file_tree_text: String,

    #[serde(default = "d_success_text")]
    pub success_text: String,

    #[serde(default = "d_warning_text")]
    pub warning_text: String,

    #[serde(default = "d_error_text")]
    pub error_text: String,

    #[serde(default = "d_button_bg")]
    pub button_background: String,

    #[serde(default = "d_button_active")]
    pub button_active_background: String,

    #[serde(default = "d_splash_bg")]
    pub splash_background: String,

    #[serde(default = "d_splash_prog")]
    pub splash_progress: String,

    #[serde(default = "d_line_num")]
    pub line_number_text: String,

    #[serde(default = "d_line_num_act")]
    pub line_number_active_text: String,

    #[serde(default = "d_curr_bg")]
    pub current_line_background: String,

    #[serde(default = "d_hover_bg")]
    pub hover_line_background: String,
}

pub(crate) fn default_highlight() -> String {
    "Highlight".to_string()
}

pub(crate) fn default_section_system() -> String {
    "System".to_string()
}
pub(crate) fn default_section_code() -> String {
    "Code".to_string()
}
pub(crate) fn default_section_preview() -> String {
    "Preview".to_string()
}
pub(crate) fn default_group_basic() -> String {
    "Basic".to_string()
}
pub(crate) fn default_group_text() -> String {
    "Text & Typography".to_string()
}
pub(crate) fn default_group_ui_elements() -> String {
    "UI Elements".to_string()
}
pub(crate) fn default_code_text() -> String {
    "Code Text".to_string()
}
pub(crate) fn default_preview_text() -> String {
    "Preview Text".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct TabMessages {
    pub nav_prev: String,
    pub nav_next: String,
    #[serde(default)]
    pub close: String,
    #[serde(default)]
    pub close_others: String,
    #[serde(default)]
    pub close_all: String,
    #[serde(default)]
    pub close_right: String,
    #[serde(default)]
    pub close_left: String,
    #[serde(default)]
    pub pin: String,
    #[serde(default)]
    pub unpin: String,
    #[serde(default)]
    pub restore_closed: String,
}
pub(crate) fn d_title_bar_text() -> String {
    "Title Bar Text".to_string()
}
pub(crate) fn d_active_file_highlight() -> String {
    "Active File".to_string()
}
pub(crate) fn d_success_text() -> String {
    "Success Text".to_string()
}
pub(crate) fn d_warning_text() -> String {
    "Warning Text".to_string()
}
pub(crate) fn d_error_text() -> String {
    "Error Text".to_string()
}
pub(crate) fn d_button_bg() -> String {
    "Button Background".to_string()
}
pub(crate) fn d_button_active() -> String {
    "Active Button".to_string()
}
pub(crate) fn d_splash_bg() -> String {
    "Splash Background".to_string()
}
pub(crate) fn d_splash_prog() -> String {
    "Splash Progress".to_string()
}
pub(crate) fn d_line_num() -> String {
    "Line Number".to_string()
}
pub(crate) fn d_line_num_act() -> String {
    "Active Line Num".to_string()
}
pub(crate) fn d_curr_bg() -> String {
    "Current Line".to_string()
}
pub(crate) fn d_hover_bg() -> String {
    "Hover Line".to_string()
}
pub(crate) fn d_file_tree_text() -> String {
    "File Tree Text".to_string()
}