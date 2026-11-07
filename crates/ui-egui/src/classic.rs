//! LibreOffice Calc-inspired classic menu and two-toolbar layout. All workbook actions delegate
//! to the same command registry used by the ribbon.

use egui::{Align, Layout, Sense, Ui, vec2};
use gridcraft_engine::model::{HAlign, Style, VAlign};
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
            &[
                ("Delete Cells…", "home.deleteCells"),
                ("Delete Rows", "home.deleteRows"),
                ("Delete Columns", "home.deleteColumns"),
                ("Delete Sheet", "home.deleteSheet"),
            ],
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
    if let Some(dialog) = id.strip_prefix("dialog:") {
        if ui.button(app.ui.language.tr(label)).clicked() {
            app.open_dialog(dialog, params.clone());
            ui.close();
        }
        return;
    }
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
    if id == "edit.paste" {
        if ui.button(app.ui.language.tr(label)).clicked() {
            app.paste_from_clipboard();
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
        .exact_size(36.0)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(8, 3)))
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                file_toolbar_group(app, ui);
                separator(ui, t);
                clipboard_toolbar_group(app, ui, t);
                separator(ui, t);
                font_toolbar_group(app, ui);
                separator(ui, t);
                editing_toolbar_group(app, ui);
            });
        });
}

fn file_toolbar_group(app: &mut SheetApp, ui: &mut Ui) {
    let (open_clicked, open_menu) = crate::widgets::split_button(ui, Icon::OpenFolder, None, "Open");
    if open_clicked {
        app.run_or_alert("file.open", json!({}));
    }
    egui::Popup::menu(&open_menu).show(|ui| {
        command_menu(app, ui, &[("New Workbook", "file.new", Json::Null), ("Open…", "file.open", Json::Null)]);
    });
    let (save_clicked, save_menu) = crate::widgets::split_button(ui, Icon::Save, None, "Save");
    if save_clicked {
        app.run_or_alert("file.save", json!({}));
    }
    egui::Popup::menu(&save_menu).show(|ui| {
        command_menu(app, ui, &[("Save", "file.save", Json::Null), ("Save As…", "file.saveAs", Json::Null)]);
    });
}

fn clipboard_toolbar_group(app: &mut SheetApp, ui: &mut Ui, t: &Tokens) {
    let cut = crate::widgets::icon_button(ui, Icon::Cut, t.text_dim, "Cut", vec2(26.0, 26.0));
    if cut.clicked() {
        app.copy_to_clipboard(ui.ctx(), "edit.cut");
    }
    let copy = crate::widgets::icon_button(ui, Icon::Copy, t.text_dim, "Copy", vec2(26.0, 26.0));
    if copy.clicked() {
        app.copy_to_clipboard(ui.ctx(), "edit.copy");
    }
    let (paste_clicked, paste_menu) = crate::widgets::split_button(ui, Icon::Paste, None, "Paste");
    if paste_clicked {
        app.paste_from_clipboard();
    }
    egui::Popup::menu(&paste_menu).show(|ui| paste_menu_items(app, ui));
    compact_icon_command(app, ui, Icon::Undo, "Undo", "edit.undo", &Json::Null);
    compact_icon_command(app, ui, Icon::Redo, "Redo", "edit.redo", &Json::Null);
}

fn paste_menu_items(app: &mut SheetApp, ui: &mut Ui) {
    command_menu(
        app,
        ui,
        &[
            ("Paste", "edit.paste", json!({})),
            ("Paste Values", "edit.pasteSpecial", json!({"what":"values"})),
            ("Paste Formulas", "edit.pasteSpecial", json!({"what":"formulas"})),
            ("Paste Formatting", "edit.pasteSpecial", json!({"what":"formats"})),
            ("Transpose", "edit.pasteSpecial", json!({"what":"all","transpose":true})),
            ("Paste Link", "edit.pasteSpecial", json!({"what":"all","link":true})),
            ("-", "", Json::Null),
            ("Paste Special…", "dialog:pasteSpecial", json!({})),
        ],
    );
}

fn font_toolbar_group(app: &mut SheetApp, ui: &mut Ui) {
    let style = active_style(app);
    let font_name = app
        .session
        .active()
        .and_then(|d| d.wb.active().map(|s| d.wb.styles.get(s.style_id(d.selection.active)).font.name.clone()))
        .unwrap_or_else(|| "Calibri".into());
    ui.menu_button(font_name, |ui| {
        for name in crate::ribbon::FONTS {
            if ui.button(*name).clicked() {
                app.run_or_alert("home.fontName", json!({"name": name}));
                ui.close();
            }
        }
    });
    ui.menu_button(format_size(style.font.size), |ui| {
        for size in [8, 9, 10, 11, 12, 14, 16, 18, 20, 24, 28, 36, 48, 72] {
            if ui.button(size.to_string()).clicked() {
                app.run_or_alert("home.fontSize", json!({"size": size}));
                ui.close();
            }
        }
    });
    font_color_control(app, ui);
    compact_toggle(app, ui, Icon::Bold, "Bold", "home.bold", style.font.bold);
    compact_toggle(app, ui, Icon::Italic, "Italic", "home.italic", style.font.italic);
    underline_control(app, ui);
    compact_toggle(app, ui, Icon::Strike, "Strikethrough", "home.strikethrough", style.font.strike);
}

fn editing_toolbar_group(app: &mut SheetApp, ui: &mut Ui) {
    let sort_filter = crate::widgets::small_button(ui, Icon::SortFilter, "", "Sort & Filter", true);
    egui::Popup::menu(&sort_filter).show(|ui| sort_filter_menu(app, ui));
    let orientation = crate::widgets::small_button(ui, Icon::Orientation, "", "Orientation", true);
    egui::Popup::menu(&orientation).show(|ui| orientation_menu(app, ui));
    let style = active_style(app);
    let wrap = crate::widgets::small_toggle_button(ui, Icon::Wrap, "Wrap Text", "Wrap Text", style.align.wrap);
    if wrap.clicked() {
        app.run_or_alert("home.wrapText", json!({}));
    }
}

fn formatting_toolbar(app: &mut SheetApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::top("classic_formatting_toolbar")
        .exact_size(36.0)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(8, 3)))
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                currency_control(app, ui);
                compact_icon_command(app, ui, Icon::Percent, "Percent", "home.percent", &Json::Null);
                compact_icon_command(app, ui, Icon::Comma, "Comma Style", "home.comma", &Json::Null);
                compact_icon_command(app, ui, Icon::DecInc, "Increase Decimal", "home.increaseDecimal", &Json::Null);
                compact_icon_command(app, ui, Icon::DecDec, "Decrease Decimal", "home.decreaseDecimal", &Json::Null);
                separator(ui, t);
                borders_control(app, ui);
                separator(ui, t);

                let style = active_style(app);
                compact_toggle(app, ui, Icon::AlignLeft, "Align Left", "home.alignLeft", style.align.h == HAlign::Left);
                compact_toggle(app, ui, Icon::AlignCenter, "Center", "home.alignCenter", style.align.h == HAlign::Center);
                compact_toggle(app, ui, Icon::AlignRight, "Align Right", "home.alignRight", style.align.h == HAlign::Right);
                compact_icon_command(app, ui, Icon::IndentDec, "Decrease Indent", "home.decreaseIndent", &Json::Null);
                compact_icon_command(app, ui, Icon::IndentInc, "Increase Indent", "home.increaseIndent", &Json::Null);
                compact_toggle(app, ui, Icon::AlignTop, "Top Align", "home.alignTop", style.align.v == VAlign::Top);
                compact_toggle(app, ui, Icon::AlignMiddle, "Middle Align", "home.alignMiddle", style.align.v == VAlign::Center);
                compact_toggle(app, ui, Icon::AlignBottom, "Bottom Align", "home.alignBottom", style.align.v == VAlign::Bottom);
                separator(ui, t);
                fill_color_control(app, ui);
                format_painter_control(app, ui);
                separator(ui, t);
                compact_icon_command(app, ui, Icon::InsertRows, "Insert Rows", "home.insertRows", &Json::Null);
                compact_icon_command(app, ui, Icon::InsertColumns, "Insert Columns", "home.insertColumns", &Json::Null);
                compact_icon_command(app, ui, Icon::DeleteRows, "Delete Rows", "home.deleteRows", &Json::Null);
                compact_icon_command(app, ui, Icon::DeleteColumns, "Delete Columns", "home.deleteColumns", &Json::Null);
                separator(ui, t);
                merge_control(app, ui);
            });
        });
}

fn command_menu(app: &mut SheetApp, ui: &mut Ui, items: &[(&str, &str, Json)]) {
    for (label, id, params) in items {
        if *label == "-" {
            ui.separator();
        } else {
            command_item(app, ui, label, id, params);
        }
    }
}

fn sort_filter_menu(app: &mut SheetApp, ui: &mut Ui) {
    command_menu(
        app,
        ui,
        &[
            ("Sort A to Z", "data.sortAscending", json!({})),
            ("Sort Z to A", "data.sortDescending", json!({})),
            ("Custom Sort…", "dialog:sort", json!({})),
            ("-", "", Json::Null),
            ("Filter", "data.filter", json!({})),
            ("Clear", "data.clearFilter", json!({})),
            ("Reapply", "data.reapply", json!({})),
        ],
    );
}

fn orientation_menu(app: &mut SheetApp, ui: &mut Ui) {
    command_menu(
        app,
        ui,
        &[
            ("Angle Counterclockwise", "home.orientation", json!({"angle":"counterclockwise"})),
            ("Angle Clockwise", "home.orientation", json!({"angle":"clockwise"})),
            ("Vertical Text", "home.orientation", json!({"angle":"vertical"})),
            ("Rotate Text Up", "home.orientation", json!({"angle":"up"})),
            ("Rotate Text Down", "home.orientation", json!({"angle":"down"})),
            ("Horizontal", "home.orientation", json!({"angle":0})),
            ("-", "", Json::Null),
            ("Format Cell Alignment…", "dialog:formatCells", json!({"tab":"Alignment"})),
        ],
    );
}

fn underline_control(app: &mut SheetApp, ui: &mut Ui) {
    let (clicked, menu) = crate::widgets::split_button(ui, Icon::Underline, None, "Underline");
    if clicked {
        app.run_or_alert("home.underline", json!({}));
    }
    egui::Popup::menu(&menu).show(|ui| {
        command_menu(
            app,
            ui,
            &[
                ("Underline", "home.underline", json!({"style":"single"})),
                ("Double Underline", "home.underline", json!({"style":"double"})),
            ],
        );
    });
}

fn currency_control(app: &mut SheetApp, ui: &mut Ui) {
    let (clicked, menu) = crate::widgets::split_button(ui, Icon::Currency, None, "Accounting Number Format");
    if clicked {
        app.run_or_alert("home.accounting", json!({}));
    }
    egui::Popup::menu(&menu).show(|ui| {
        let mut items: Vec<_> = crate::ribbon::ACCOUNTING_SYMBOLS
            .iter()
            .map(|(label, code)| (*label, "home.numberFormat", json!({"code":code})))
            .collect();
        items.push(("-", "", Json::Null));
        items.push(("More Accounting Formats…", "dialog:formatCells", json!({"tab":"Number"})));
        command_menu(app, ui, &items);
    });
}

fn borders_control(app: &mut SheetApp, ui: &mut Ui) {
    let (clicked, menu) = crate::widgets::split_button(ui, Icon::Borders, None, "Borders");
    if clicked {
        let preset = app.grid.last_border.clone();
        app.run_or_alert("home.borders", json!({"preset":preset}));
    }
    egui::Popup::menu(&menu).show(|ui| {
        for (label, preset) in [
            ("Bottom Border", "bottom"),
            ("Top Border", "top"),
            ("Left Border", "left"),
            ("Right Border", "right"),
            ("No Border", "none"),
            ("All Borders", "all"),
            ("Outside Borders", "outside"),
            ("Thick Outside Borders", "thickOutside"),
            ("Bottom Double Border", "doubleBottom"),
            ("Thick Bottom Border", "thickBottom"),
            ("Top and Bottom Border", "topBottom"),
            ("Top and Thick Bottom Border", "topThickBottom"),
            ("Top and Double Bottom Border", "topDoubleBottom"),
            ("Inside Borders", "inside"),
        ] {
            let response = crate::border_preview::preset_button(ui, app.ui.language.tr(label), preset);
            if response.clicked() {
                app.grid.last_border = preset.to_string();
                app.run_or_alert("home.borders", json!({"preset":preset}));
                ui.close();
            }
        }
        ui.separator();
        if ui.button(app.ui.language.tr("More Borders…")).clicked() {
            app.open_dialog("formatCells", json!({"tab":"Border"}));
            ui.close();
        }
    });
}

fn font_color_control(app: &mut SheetApp, ui: &mut Ui) {
    color_control(app, ui, "home.fontColor");
}

fn fill_color_control(app: &mut SheetApp, ui: &mut Ui) {
    color_control(app, ui, "home.fillColor");
}

fn color_control(app: &mut SheetApp, ui: &mut Ui, id: &str) {
    let is_fill = id == "home.fillColor";
    let (icon, previous, none_label) = if is_fill {
        (Icon::Fill, app.grid.last_fill.clone(), "No Fill")
    } else {
        (Icon::FontColor, app.grid.last_font_color.clone(), "Automatic")
    };
    let accent = gridcraft_engine::model::Color::from_hex(&previous)
        .and_then(|color| color.resolve(&Default::default()))
        .map(crate::theme::color32);
    let (clicked, menu) = crate::widgets::split_button(ui, icon, accent, if is_fill { "Fill Color" } else { "Font Color" });
    if clicked {
        app.run_or_alert(id, json!({"color":previous}));
    }
    let theme_colors = app
        .session
        .active()
        .map(|d| d.wb.theme.colors)
        .unwrap_or(gridcraft_engine::model::Theme::default().colors);
    egui::Popup::menu(&menu)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            if let Some(color) = crate::widgets::color_palette(ui, &theme_colors, app.ui.language.tr(none_label)) {
                if color != "none" {
                    if is_fill {
                        app.grid.last_fill = color.clone();
                    } else {
                        app.grid.last_font_color = color.clone();
                    }
                }
                let color = if !is_fill && color == "none" { "auto".to_string() } else { color };
                app.run_or_alert(id, json!({"color":color}));
                ui.close();
            }
        });
}

fn compact_toggle(app: &mut SheetApp, ui: &mut Ui, icon: Icon, label: &str, id: &str, on: bool) {
    let response = crate::widgets::toggle_button(ui, icon, on, app.ui.language.tr(label));
    if response.clicked() {
        app.run_or_alert(id, json!({}));
    }
}

fn format_painter_control(app: &mut SheetApp, ui: &mut Ui) {
    let on = app.session.format_painter.is_some();
    let response = crate::widgets::toggle_button(ui, Icon::Brush, on, app.ui.language.tr("Format Painter"));
    if response.double_clicked() {
        app.run_or_alert("edit.formatPainter", json!({"sticky":true}));
    } else if response.clicked() {
        if on {
            app.session.format_painter = None;
        } else {
            app.run_or_alert("edit.formatPainter", json!({}));
        }
    }
}

fn merge_control(app: &mut SheetApp, ui: &mut Ui) {
    let (main, menu) = crate::widgets::small_split_button(ui, Icon::Merge, "Merge & Center", "Merge & Center");
    if main.clicked() {
        app.run_or_alert("home.mergeCenter", json!({}));
    }
    egui::Popup::menu(&menu).show(|ui| {
        command_menu(
            app,
            ui,
            &[
                ("Merge & Center", "home.mergeCenter", json!({})),
                ("Merge Across", "home.mergeAcross", json!({})),
                ("Merge Cells", "home.mergeCells", json!({})),
                ("Unmerge Cells", "home.unmergeCells", json!({})),
            ],
        );
    });
}

fn active_style(app: &SheetApp) -> Style {
    app.session
        .active()
        .and_then(|d| {
            let sheet = d.wb.active()?;
            Some(d.wb.styles.get(sheet.style_id(d.selection.active)).clone())
        })
        .unwrap_or_default()
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
