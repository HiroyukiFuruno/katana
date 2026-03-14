//! Katana three-pane egui shell.

#![deny(clippy::too_many_lines, clippy::cognitive_complexity)]

use eframe::egui;
use katana_platform::FilesystemService;

use crate::{
    app_state::{AppAction, AppState},
    preview_pane::PreviewPane,
};

pub struct KatanaApp {
    state: AppState,
    fs: FilesystemService,
    pending_action: AppAction,
    preview_pane: PreviewPane,
}

impl KatanaApp {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            preview_pane: PreviewPane::default(),
        }
    }

    fn take_action(&mut self) -> AppAction {
        std::mem::replace(&mut self.pending_action, AppAction::None)
    }

    fn refresh_preview(&mut self, source: &str) {
        // テキスト変更は即時反映（ダイアグラムは既存の画像を保持）。
        self.preview_pane.update_markdown_sections(source);
    }

    fn full_refresh_preview(&mut self, source: &str) {
        // ドキュメント選択や Refresh ボタン押下で全セクションを再レンダリング。
        self.preview_pane.full_render(source);
    }

    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        match self.fs.open_workspace(&path) {
            Ok(ws) => {
                let name = ws.name().unwrap_or("unknown").to_string();
                self.state.status_message = Some(format!("Opened workspace: {name}"));
                self.state.workspace = Some(ws);
                self.state.active_document = None;
            }
            Err(e) => {
                self.state.status_message = Some(format!("Cannot open workspace: {e}"));
            }
        }
    }

    fn handle_select_document(&mut self, path: std::path::PathBuf) {
        match self.fs.load_document(&path) {
            Ok(doc) => {
                // ドキュメント選択時はダイアグラム含め完全レンダリング。
                self.full_refresh_preview(&doc.buffer.clone());
                self.state.active_document = Some(doc);
            }
            Err(e) => {
                self.state.status_message = Some(format!("Cannot open file: {e}"));
            }
        }
    }

    fn handle_update_buffer(&mut self, content: String) {
        if let Some(doc) = &mut self.state.active_document {
            doc.update_buffer(content.clone());
        }
        self.refresh_preview(&content);
    }

    fn handle_save_document(&mut self) {
        let Some(doc) = &mut self.state.active_document else {
            return;
        };
        match self.fs.save_document(doc) {
            Ok(()) => self.state.status_message = Some("Saved.".to_string()),
            Err(e) => self.state.status_message = Some(format!("Save failed: {e}")),
        }
    }

    fn process_action(&mut self, action: AppAction) {
        match action {
            AppAction::OpenWorkspace(p) => self.handle_open_workspace(p),
            AppAction::SelectDocument(p) => self.handle_select_document(p),
            AppAction::UpdateBuffer(c) => self.handle_update_buffer(c),
            AppAction::SaveDocument => self.handle_save_document(),
            AppAction::RefreshDiagrams => {
                if let Some(doc) = &self.state.active_document {
                    let src = doc.buffer.clone();
                    self.full_refresh_preview(&src);
                }
            }
            AppAction::None => {}
        }
    }
}

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let action = self.take_action();
        self.process_action(action);

        render_menu_bar(ctx, &mut self.state, &mut self.pending_action);
        render_status_bar(ctx, &self.state);
        render_workspace_panel(ctx, &mut self.state, &mut self.pending_action);
        render_preview_panel(
            ctx,
            &mut self.preview_pane,
            &self.state,
            &mut self.pending_action,
        );
        render_editor_panel(ctx, &mut self.state, &mut self.pending_action);
    }
}

fn open_folder_dialog() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

fn render_menu_bar(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                render_file_menu(ui, state, action);
            });
            render_header_right(ui, state);
        });
    });
}

fn render_file_menu(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    if ui.button("Open Workspace…").clicked() {
        if let Some(path) = open_folder_dialog() {
            *action = AppAction::OpenWorkspace(path);
        }
        ui.close_menu();
    }
    ui.separator();
    if ui
        .add_enabled(state.is_dirty(), egui::Button::new("Save"))
        .clicked()
    {
        *action = AppAction::SaveDocument;
        ui.close_menu();
    }
}

fn render_header_right(ui: &mut egui::Ui, state: &AppState) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if state.is_dirty() {
            ui.label("●");
        }
        if !state.ai_available() {
            ui.label("[AI: unconfigured]");
        }
    });
}

fn render_status_bar(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        let msg = state.status_message.as_deref().unwrap_or("Ready");
        ui.label(msg);
    });
}

fn render_workspace_panel(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::SidePanel::left("workspace_tree")
        .resizable(true)
        .min_width(160.0)
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.heading("Workspace");
            ui.separator();
            render_workspace_content(ui, state, action);
        });
}

fn render_workspace_content(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    if let Some(ws) = &state.workspace {
        let entries = ws.tree.clone();
        let mut selected: Option<std::path::PathBuf> = None;
        for entry in &entries {
            render_tree_entry(ui, entry, &mut selected);
        }
        if let Some(path) = selected {
            *action = AppAction::SelectDocument(path);
        }
    } else {
        ui.label("No workspace open.");
        ui.add_space(8.0);
        if ui.button("Open Workspace…").clicked() {
            if let Some(path) = open_folder_dialog() {
                *action = AppAction::OpenWorkspace(path);
            }
        }
    }
}

fn render_preview_panel(
    ctx: &egui::Context,
    preview: &mut PreviewPane,
    state: &AppState,
    action: &mut AppAction,
) {
    egui::SidePanel::right("preview_pane")
        .resizable(true)
        .min_width(200.0)
        .default_width(400.0)
        .show(ctx, |ui| {
            render_preview_header(ui, state, action);
            ui.separator();
            if state.active_document.is_none() {
                ui.label("No document selected.");
            } else {
                preview.show(ui);
            }
        });
}

fn render_preview_header(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    ui.horizontal(|ui| {
        ui.heading("Preview");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let has_doc = state.active_document.is_some();
            if ui
                .add_enabled(has_doc, egui::Button::new("🔄"))
                .on_hover_text("Refresh diagrams")
                .clicked()
            {
                *action = AppAction::RefreshDiagrams;
            }
        });
    });
}

fn render_editor_panel(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let title = editor_title(state);
        ui.heading(title);
        ui.separator();
        render_editor_content(ui, state, action);
    });
}

fn editor_title(state: &AppState) -> String {
    state
        .active_document
        .as_ref()
        .and_then(|d| d.file_name())
        .map(|n| {
            if state.is_dirty() {
                format!("{n} ●")
            } else {
                n.to_string()
            }
        })
        .unwrap_or_else(|| "Editor".to_string())
}

fn render_editor_content(ui: &mut egui::Ui, state: &mut AppState, action: &mut AppAction) {
    if let Some(doc) = &state.active_document {
        let mut buffer = doc.buffer.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut buffer)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(40),
            );
            if response.changed() {
                *action = AppAction::UpdateBuffer(buffer);
            }
        });
    } else {
        render_empty_editor(ui);
    }
}

fn render_empty_editor(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.label("Open a workspace and select a Markdown file to begin editing.");
    });
}

fn render_tree_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    selected: &mut Option<std::path::PathBuf>,
) {
    use katana_core::workspace::TreeEntry;
    match entry {
        TreeEntry::Directory { path, children } => {
            render_directory_entry(ui, path, children, selected);
        }
        TreeEntry::File { path } => {
            render_file_entry(ui, entry, path, selected);
        }
    }
}

fn render_directory_entry(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    children: &[katana_core::workspace::TreeEntry],
    selected: &mut Option<std::path::PathBuf>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    egui::CollapsingHeader::new(format!("📁 {name}"))
        .default_open(true)
        .show(ui, |ui| {
            for child in children {
                render_tree_entry(ui, child, selected);
            }
        });
}

fn render_file_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    path: &std::path::Path,
    selected: &mut Option<std::path::PathBuf>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let label = if entry.is_markdown() {
        format!("📄 {name}")
    } else {
        format!("  {name}")
    };
    if ui.selectable_label(false, label).clicked() && entry.is_markdown() {
        *selected = Some(path.to_path_buf());
    }
}
