//! LibreOffice Calc-inspired classic menu and two-toolbar layout. All workbook actions delegate
//! to the same command registry used by the ribbon.

use egui::{Align, Layout, Sense, Ui, vec2};
use serde_json::{Value as Json, json};

use crate::{InterfaceMode, SheetApp, icons::Icon, theme::Tokens};

type MenuSection = &'static [(&'static str, &'static str)];
type MenuEntry = (&'static str, &'static [MenuSection]);

const MENUS: &[MenuEntry] = &[
    (
        "File",
        &[&[("New Workbook", "file.new"), ("Open…", "file.open"), ("Save", "file.save"), ("Save As…", "file.saveAs"), ("Print…", "file.print")]],
    ),
    (
        "Edit",
        &[
            &[("Undo", "edit.undo"), ("Redo", "edit.redo")],
            &[("Cut", "edit.cut"), ("Copy", "edit.copy"), ("Paste", "edit.paste")],
            &[("Find…", "edit.find")],
        ],
    ),
    ("View", &[]),
    (
        "Insert",
        &[
            &[("Insert Cells…", "home.insertCells"), ("Insert Rows", "home.insertRows"), ("Insert Columns", "home.insertColumns")],
            &[("Insert Sheet", "home.insertSheet"), ("Chart", "insert.chart")],
        ],
    ),
    (
        "Format",
        &[
            &[
                ("Format Cells…", "home.formatCells"),
                ("Bold", "home.bold"),
                ("Italic", "home.italic"),
                ("Underline", "home.underline"),
                ("Strikethrough", "home.strikethrough"),
            ],
            &[
                ("Merge & Center", "home.mergeCenter"),
                ("Merge Across", "home.mergeAcross"),
                ("Merge Cells", "home.mergeCells"),
                ("Unmerge Cells", "home.unmergeCells"),
                ("Wrap Text", "home.wrapText"),
            ],
        ],
    ),
    (
        "Sheet",
        &[
            &[("Insert Rows", "home.insertRows"), ("Insert Columns", "home.insertColumns"), ("Insert Sheet", "home.insertSheet")],
            &[("Delete Cells…", "home.deleteCells"), ("Delete Rows", "home.deleteRows"), ("Delete Columns", "home.deleteColumns"), ("Delete Sheet", "home.deleteSheet")],
            &[("Row Height…", "home.rowHeight"), ("Column Width…", "home.columnWidth"), ("AutoFit Column Width", "home.autofitColumnWidth")],
            &[("Hide Rows", "home.hideRows"), ("Hide Columns", "home.hideColumns"), ("Hide Sheet", "sheet.hide"), ("Unhide Sheet…", "sheet.unhide")],
        ],
    ),
    (
        "Data",
        &[&[
            ("Sort A to Z", "data.sortAscending"),
            ("Sort Z to A", "data.sortDescending"),
            ("Filter", "data.filter"),
            ("Remove Duplicates…", "data.removeDuplicates"),
        ]],
    ),
    ("Tools", &[&[("Options…", "file.options")]]),
    ("Window", &[]),
    ("Help", &[]),
];

pub(crate) fn show(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("classic_menu_bar").exact_size(30.0).frame(egui::Frame::NONE.fill(t.window)).show(ui, |ui| {
        egui::ScrollArea::horizontal()
            .id_salt("classic_menu_scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    for (menu, sections) in MENUS {
                        ui.menu_button(app.ui.language.tr(menu), |ui| {
                            if *menu == "View" {
                                let lang = app.ui.language;
                                ui.menu_button(lang.tr("User Interface"), |ui| interface_choices(app, ui));
                                ui.separator();
                                ui.checkbox(&mut app.ui.show_title_bar, lang.tr("Title Bar"));
                                ui.separator();
                                for (label, id) in [
                                    ("Normal", "view.normal"),
                                    ("Page Break Preview", "view.pageBreakPreview"),
                                    ("Page Layout", "view.pageLayout"),
                                    ("Formula Bar", "view.formulaBar"),
                                ] {
                                    command_item(app, ui, label, id, &Json::Null);
                                }
                                return;
                            }
                            if *menu == "Window" {
                                command_item(app, ui, "New Window", "view.newWindow", &Json::Null);
                                ui.separator();
                                for window in app.open_windows() {
                                    if ui.selectable_label(window.active, window.label()).clicked() {
                                        app.run_or_alert("view.switchWindows", json!({"window": window.window}));
                                        ui.close();
                                    }
                                }
                                return;
                            }
                            if *menu == "Help" {
                                if ui.button(app.ui.language.tr("About GridCraft")).clicked() {
                                    app.open_dialog("about", json!({}));
                                    ui.close();
                                }
                                if ui.button(app.ui.language.tr("Check for Updates")).clicked() {
                                    app.run_or_alert("app.checkForUpdates", json!({}));
                                    ui.close();
                                }
                                return;
                            }
                            for (section_index, section) in sections.iter().enumerate() {
                                if section_index > 0 {
                                    ui.separator();
                                }
                                for (label, id) in *section {
                                    command_item(app, ui, label, id, &Json::Null);
                                }
                            }
                        });
                    }
                })
            });
    });
    standard_toolbar(app, ui, &t);
    formatting_toolbar(app, ui, &t);
}

fn interface_choices(app: &mut SheetApp, ui: &mut Ui) {
    let lang = app.ui.language;
    for (mode, label) in [(InterfaceMode::GridCraftStandard, "GridCraft Standard"), (InterfaceMode::LibreOfficeClassic, "LibreOffice Classic")] {
        if ui.selectable_label(app.ui.interface == mode, lang.tr(label)).clicked() {
            app.ui.interface = mode;
            ui.close();
        }
    }
}

fn command_item(app: &mut SheetApp, ui: &mut Ui, label: &str, id: &str, params: &Json) {
    let dialog = match id {
        "home.insertCells" => Some("insertCells"),
        "home.deleteCells" => Some("deleteCells"),
        "home.rowHeight" => Some("rowHeight"),
        "home.columnWidth" => Some("columnWidth"),
        "home.defaultWidth" => Some("defaultWidth"),
        "home.formatCells" => Some("formatCells"),
        _ => None,
    };
    if let Some(dialog) = dialog {
        if ui.button(app.ui.language.tr(label)).clicked() {
            app.open_dialog(dialog, params.clone());
            ui.close();
        }
        return;
    }
    if id == "edit.find" {
        if ui.button(app.ui.language.tr(label)).clicked() {
            app.open_dialog("find", json!({}));
            ui.close();
        }
        return;
    }
    let Some(spec) = gridcraft_engine::find_command(id) else { return };
    let enabled = (spec.enabled)(&app.session).is_ok();
    let text = app.ui.language.tr(label);
    let response = ui.add_enabled(enabled, egui::Button::new(text).frame(false).min_size(vec2(205.0, 22.0)));
    let response = if let Some(shortcut) = spec.info(&app.session).shortcut_for_platform(cfg!(target_os = "macos")) {
        response.on_hover_text(format!("{label} ({shortcut})"))
    } else {
        response
    };
    if response.clicked() {
        app.run_or_alert(id, params.clone());
        ui.close();
    }
}

fn standard_toolbar(app: &mut SheetApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::top("classic_standard_toolbar")
        .exact_size(34.0)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(8, 3)))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().id_salt("classic_toolbar_scroll").show(ui, |ui| {
                let width = ui.available_width();
                ui.set_min_width(width);
                ui.with_layout(Layout::left_to_right(Align::Center).with_main_justify(true), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (label, icon, id, params) in [
                        ("New Workbook", Icon::NewFile, "file.new", Json::Null),
                        ("Open", Icon::OpenFolder, "file.open", Json::Null),
                        ("Save", Icon::Save, "file.save", Json::Null),
                        ("Print", Icon::Print, "file.print", Json::Null),
                        ] {
                            compact_icon_command(app, ui, icon, label, id, &params);
                        }
                    });
                    separator(ui, t);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (label, icon, id, params) in [
                        ("Cut", Icon::Cut, "edit.cut", Json::Null),
                        ("Copy", Icon::Copy, "edit.copy", Json::Null),
                        ("Paste", Icon::Paste, "edit.paste", Json::Null),
                        ("Undo", Icon::Undo, "edit.undo", Json::Null),
                        ("Redo", Icon::Redo, "edit.redo", Json::Null),
                        ] {
                            compact_icon_command(app, ui, icon, label, id, &params);
                        }
                    });
                    separator(ui, t);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (label, icon, id, params) in [
                        ("Insert Cells", Icon::Insert, "home.insertCells", Json::Null),
                        ("Insert Rows", Icon::InsertRows, "home.insertRows", Json::Null),
                        ("Insert Columns", Icon::InsertColumns, "home.insertColumns", Json::Null),
                        ("Delete Cells", Icon::Delete, "home.deleteCells", Json::Null),
                        ("Delete Rows", Icon::DeleteRows, "home.deleteRows", Json::Null),
                        ("Delete Columns", Icon::DeleteColumns, "home.deleteColumns", Json::Null),
                        ("Insert Sheet", Icon::Sheet, "home.insertSheet", Json::Null),
                        ] {
                            compact_icon_command(app, ui, icon, label, id, &params);
                        }
                    });
                    separator(ui, t);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (label, icon, id, params) in [
                        ("Sort A to Z", Icon::SortAsc, "data.sortAscending", Json::Null),
                        ("Filter", Icon::Filter, "data.filter", Json::Null),
                        ("AutoSum", Icon::Sum, "formulas.autoSum", Json::Null),
                        ("Find", Icon::Find, "edit.find", json!({"what":""})),
                        ("AutoFit Column Width", Icon::Format, "home.autofitColumnWidth", Json::Null),
                        ("AutoFit Row Height", Icon::Format, "home.autofitRowHeight", Json::Null),
                        ("Hide Rows", Icon::DeleteRows, "home.hideRows", Json::Null),
                        ("Hide Columns", Icon::DeleteColumns, "home.hideColumns", Json::Null),
                        ] {
                            compact_icon_command(app, ui, icon, label, id, &params);
                        }
                    });
                    ui.menu_button(app.ui.language.tr("User Interface"), |ui| interface_choices(app, ui));
                })
            });
        });
}

fn formatting_toolbar(app: &mut SheetApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::top("classic_formatting_toolbar")
        .exact_size(36.0)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(8, 3)))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().id_salt("classic_format_scroll").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.menu_button(
                        app.session
                            .active()
                            .and_then(|d| d.wb.active().map(|s| d.wb.styles.get(s.style_id(d.selection.active)).font.name.clone()))
                            .unwrap_or_else(|| "Calibri".into()),
                        |ui| {
                            for name in ["Arial", "Calibri", "Cambria", "Courier New", "Georgia", "Times New Roman", "Verdana"] {
                                if ui.button(name).clicked() {
                                    app.run_or_alert("home.fontName", json!({"name": name}));
                                    ui.close();
                                }
                            }
                        },
                    );
                    let size = app
                        .session
                        .active()
                        .and_then(|d| d.wb.active().map(|s| d.wb.styles.get(s.style_id(d.selection.active)).font.size))
                        .unwrap_or(11.0);
                    ui.menu_button(format_size(size), |ui| {
                        for size in [8, 9, 10, 11, 12, 14, 16, 18, 20, 24, 28, 36, 48, 72] {
                            if ui.button(size.to_string()).clicked() {
                                app.run_or_alert("home.fontSize", json!({"size": size}));
                                ui.close();
                            }
                        }
                    });
                    separator(ui, t);
                    for (label, icon, id, params) in [
                        ("Bold", Icon::Bold, "home.bold", json!({})),
                        ("Italic", Icon::Italic, "home.italic", json!({})),
                        ("Underline", Icon::Underline, "home.underline", json!({})),
                        ("Strikethrough", Icon::Strike, "home.strikethrough", json!({})),
                        ("Borders", Icon::BorderAll, "home.borders", json!({"preset":"all"})),
                    ] {
                        compact_icon_command(app, ui, icon, label, id, &params);
                    }
                    separator(ui, t);
                    for (label, icon, id, params) in [
                        ("Align Left", Icon::AlignLeft, "home.alignLeft", json!({})),
                        ("Center", Icon::AlignCenter, "home.alignCenter", json!({})),
                        ("Align Right", Icon::AlignRight, "home.alignRight", json!({})),
                        ("Top Align", Icon::AlignTop, "home.alignTop", json!({})),
                        ("Middle Align", Icon::AlignMiddle, "home.alignMiddle", json!({})),
                        ("Bottom Align", Icon::AlignBottom, "home.alignBottom", json!({})),
                        ("Decrease Indent", Icon::IndentDec, "home.decreaseIndent", json!({})),
                        ("Increase Indent", Icon::IndentInc, "home.increaseIndent", json!({})),
                        ("Wrap Text", Icon::Wrap, "home.wrapText", json!({})),
                        ("Merge & Center", Icon::Merge, "home.mergeCenter", json!({})),
                        ("Currency", Icon::Currency, "home.accounting", json!({})),
                        ("Percent", Icon::Percent, "home.percent", json!({})),
                        ("Comma Style", Icon::Comma, "home.comma", json!({})),
                        ("Increase Decimal", Icon::DecInc, "home.increaseDecimal", json!({})),
                        ("Decrease Decimal", Icon::DecDec, "home.decreaseDecimal", json!({})),
                    ] {
                        compact_icon_command(app, ui, icon, label, id, &params);
                    }
                    for (label, id) in [("Font Color", "home.fontColor"), ("Cell Background", "home.fillColor")] {
                        ui.menu_button(label, |ui| {
                            let choices: &[(&str, &str)] = if id == "home.fontColor" {
                                &[("Automatic", "auto"), ("Red", "#C00000"), ("Blue", "#0070C0"), ("Green", "#70AD47"), ("Yellow", "#FFFF00")]
                            } else {
                                &[("Red", "#C00000"), ("Blue", "#0070C0"), ("Green", "#70AD47"), ("Yellow", "#FFFF00"), ("No Fill", "none")]
                            };
                            for (name, value) in choices {
                                if ui.button(*name).clicked() {
                                    app.run_or_alert(id, json!({"color": *value}));
                                    ui.close();
                                }
                            }
                        });
                    }
                })
            });
        });
}

fn compact_icon_command(app: &mut SheetApp, ui: &mut Ui, icon: Icon, label: &str, id: &str, params: &Json) {
    if id == "home.insertCells" {
        let response = crate::widgets::icon_button(ui, icon, Tokens::get(ui.ctx()).text_dim, label, vec2(26.0, 26.0));
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, app.ui.language.tr(label)));
        if response.clicked() {
            app.open_dialog("insertCells", json!({}));
        }
        return;
    }
    if id == "home.deleteCells" {
        let response = crate::widgets::icon_button(ui, icon, Tokens::get(ui.ctx()).text_dim, label, vec2(26.0, 26.0));
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, app.ui.language.tr(label)));
        if response.clicked() {
            app.open_dialog("deleteCells", json!({}));
        }
        return;
    }
    if id == "edit.find" {
        let color = Tokens::get(ui.ctx()).text_dim;
        let response = crate::widgets::icon_button(ui, icon, color, label, vec2(26.0, 26.0));
        if response.clicked() {
            app.open_dialog("find", json!({}));
        }
        return;
    }
    let Some(spec) = gridcraft_engine::find_command(id) else { return };
    let enabled = (spec.enabled)(&app.session).is_ok();
    let t = Tokens::get(ui.ctx());
    let response = crate::widgets::icon_button(ui, icon, if enabled { t.text_dim } else { t.text_disabled }, label, vec2(26.0, 26.0));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, app.ui.language.tr(label)));
    if response.clicked() && enabled {
        app.run_or_alert(id, params.clone());
    }
}

fn format_size(size: f32) -> String {
    if size.fract() == 0.0 { format!("{size:.0}") } else { format!("{size:.1}") }
}

fn separator(ui: &mut Ui, t: &Tokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(8.0, 26.0), Sense::hover());
    ui.painter().line_segment([rect.center_top() + vec2(0.0, 3.0), rect.center_bottom() - vec2(0.0, 3.0)], egui::Stroke::new(1.0, t.separator));
}
