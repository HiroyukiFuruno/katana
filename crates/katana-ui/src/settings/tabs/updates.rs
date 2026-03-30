use crate::app_state::AppAction;
use crate::settings::*;
use crate::widgets::StyledComboBox;

pub(crate) fn render_updates_tab(
    ui: &mut egui::Ui,
    state: &mut crate::app_state::AppState,
) -> Option<AppAction> {
    let update_msgs = &crate::i18n::get().settings.updates;
    let settings = &mut state.config.settings;

    section_header(ui, &update_msgs.section_title);

    let ver_str = format!("Current version: v{}", env!("CARGO_PKG_VERSION"));
    ui.label(egui::RichText::new(ver_str).weak().size(HINT_FONT_SIZE));

    ui.horizontal(|ui| {
        ui.label(&update_msgs.interval);

        let mut interval = settings.settings().updates.interval;
        use katana_platform::settings::UpdateInterval;
        let mut changed = false;

        StyledComboBox::new(
            "update_interval",
            match interval {
                UpdateInterval::Never => update_msgs.never.as_str(),
                UpdateInterval::Daily => update_msgs.daily.as_str(),
                UpdateInterval::Weekly => update_msgs.weekly.as_str(),
                UpdateInterval::Monthly => update_msgs.monthly.as_str(),
            },
        )
        .show(ui, |ui| {
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Never, &update_msgs.never)
                .changed();
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Daily, &update_msgs.daily)
                .changed();
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Weekly, &update_msgs.weekly)
                .changed();
            changed |= ui
                .selectable_value(&mut interval, UpdateInterval::Monthly, &update_msgs.monthly)
                .changed();
        });

        if changed {
            settings.settings_mut().updates.interval = interval;
            let _ = settings.save();
        }
    });

    ui.add_space(SUBSECTION_SPACING);

    if ui.button(&update_msgs.check_now).clicked() {
        return Some(AppAction::CheckForUpdates);
    }
    None
}