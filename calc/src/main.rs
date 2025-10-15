use umya_spreadsheet::*;
use rfd::FileDialog;
mod simulation;
use std::collections::HashMap;

use calc::Described;
use simulation::{GeneralParam, CalcParam, PipeTypeParam, simulate};
use eframe::{egui};

struct SimApp {
    str_buffer: HashMap<&'static str, String>,
    // パラメータ
    calc_param: simulation::CalcParam,
    general_param: simulation::GeneralParam,
    pipe_params: Vec<simulation::PipeTypeParam>,
    // 結果
    result: Option<simulation::CalcResult>,
}

impl Default for SimApp {
    fn default() -> Self {
        Self {
            str_buffer: HashMap::new(),
            calc_param: simulation::CalcParam::default(),
            general_param: simulation::GeneralParam::default(),
            pipe_params: vec![simulation::PipeTypeParam::default()],
            result: None,
        }
    }
}

fn add_param<T>(str_buffer: &mut HashMap<&'static str, String>, ui: &mut egui::Ui, param: &mut Described<T>)
where
    T: std::str::FromStr + ToString,
{
    ui.horizontal(|ui| {
        ui.label(format!("{}:", param.desc()));
        let entry = str_buffer.entry(param.desc()).or_insert_with(|| param.value.to_string());
        ui.add(egui::TextEdit::singleline(entry).desired_width(80.0));
        if let Ok(val) = entry.parse() {
            param.value = val;
        }
    });
}

impl eframe::App for SimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("PCM Simulation");
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.mesh_count);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.time_step);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.calculation_step);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.pipe_length);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.pipe_outdir);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.pipe_indir);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.pcm_init_thickness);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.water_init_temp);
                add_param(&mut self.str_buffer, ui, &mut self.calc_param.pcm_temp);
                ui.separator();
                add_param(&mut self.str_buffer, ui, &mut self.general_param.water_dens);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.water_cond);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.water_spec);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.water_visc);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.copper_cond);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.pcm_latent);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.pcm_dens);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.nusscelt);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.pcm_cond);
                add_param(&mut self.str_buffer, ui, &mut self.general_param.g);
                ui.separator();
                ui.label("PipeType Parameters");
                let mut remove_idx = None;
                let pipe_count = self.pipe_params.len();
                for (i, param) in self.pipe_params.iter_mut().enumerate() {
                    // descにインデックスを付与
                    param.pressure_loss.desc = Box::leak(format!("圧力損失(MPa) (PipeType {})", i+1).into_boxed_str());
                    param.pipe_count.desc = Box::leak(format!("パイプ本数(-) (PipeType {})", i+1).into_boxed_str());
                    ui.horizontal(|ui| {
                        add_param(&mut self.str_buffer, ui, &mut param.pressure_loss);
                        add_param(&mut self.str_buffer, ui, &mut param.pipe_count);
                    });
                    if pipe_count > 1 {
                        if ui.button("削除").clicked() {
                            remove_idx = Some(i);
                        }
                    }
                    ui.separator();
                }
                if let Some(idx) = remove_idx {
                    self.pipe_params.remove(idx);
                }
                if ui.button("パイプタイプ追加").clicked() {
                    self.pipe_params.push(simulation::PipeTypeParam::default());
                }
                if ui.button("計算実行").clicked() {
                    self.result = Some(simulate(
                        self.general_param.clone(),
                        self.calc_param.clone(),
                        self.pipe_params.clone(),
                    ));
                }
                if ui.button("計算結果をxlsxで保存").clicked() {
                    if let Some(result) = &self.result {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Excelファイル", &["xlsx"])
                            .set_file_name("calc_result.xlsx")
                            .save_file() {
                            let mut book = umya_spreadsheet::new_file();
                            // 1ページ目: パラメータ一覧（1,2行目は通常パラメータ、3列目以降にPipeTypeParamをpipeごとに2列ずつ）
                            let sheet1 = book.get_sheet_by_name_mut("Sheet1").unwrap();
                            // 1,2行目: 通常パラメータ
                            let mut keys: Vec<_> = self.str_buffer.keys().filter(|k| !k.contains("PipeType")).collect();
                            for (i, key) in keys.iter().enumerate() {
                                let k: &str = &**key;
                                let value = self.str_buffer.get(k);
                                let v: &str = value.map(|s| s.as_str()).unwrap_or("");
                                sheet1.get_cell_mut((1, (i+1) as u32)).set_value(k);
                                sheet1.get_cell_mut((2, (i+1) as u32)).set_value(v);
                            }
                            // 3列目以降: PipeTypeParam
                            let pipe_count = self.pipe_params.len();
                            let pipe_params = ["圧力損失(MPa)", "パイプ本数(-)"];
                            // 1行目: Pipe1, Pipe2, ...
                            for i in 0..pipe_count {
                                let base_col = 3 + (i as u32) * 2;
                                sheet1.get_cell_mut((base_col, 1)).set_value(format!("Pipe{}", i+1));
                            }
                            // 2行目: 空欄
                            // 3行目以降: パラメータ名と値
                            for (j, param_name) in pipe_params.iter().enumerate() {
                                for i in 0..pipe_count {
                                    let base_col = 3 + (i as u32) * 2;
                                    let key = format!("{} (PipeType {})", param_name, i+1);
                                    let value = self.str_buffer.get(&*Box::leak(key.clone().into_boxed_str())).map(|s| s.as_str()).unwrap_or("");
                                    sheet1.get_cell_mut((base_col, (j+2) as u32)).set_value(*param_name);
                                    sheet1.get_cell_mut((base_col+1, (j+2) as u32)).set_value(value);
                                }
                            }
                            sheet1.set_name("Parameters");
                            // 2ページ目: 各管の出口温度と平均出口温度（最左列に計算ステップ）
                            let sheet2 = book.new_sheet("PipeOutTemps").unwrap();
                            // ヘッダー
                            sheet2.get_cell_mut((1, 1)).set_value("Step");
                            for (i, _) in result.pipe_end_temperatures.iter().enumerate() {
                                sheet2.get_cell_mut(((i+2) as u32, 1)).set_value(format!("Pipe {}", i+1));
                            }
                            let col_count = result.pipe_end_temperatures.len();
                            let max_len = result.pipe_end_temperatures.iter().map(|v| v.len()).max().unwrap_or(0);
                            let avg_col = (col_count+2) as u32;
                            sheet2.get_cell_mut((avg_col, 1)).set_value("Average");
                            for row in 0..max_len {
                                // ステップ番号
                                sheet2.get_cell_mut((1, (row+2) as u32)).set_value(row.to_string());
                                // 各パイプの出口温度
                                for (col, temps) in result.pipe_end_temperatures.iter().enumerate() {
                                    if let Some(val) = temps.get(row) {
                                        sheet2.get_cell_mut(((col+2) as u32, (row+2) as u32)).set_value(val.to_string());
                                    }
                                }
                                // 平均出口温度
                                if let Some(temp) = result.average_end_temperatures.get(row) {
                                    sheet2.get_cell_mut((avg_col, (row+2) as u32)).set_value(temp.to_string());
                                }
                            }

                            // 3ページ目: その他のresult値
                            let sheet3 = book.new_sheet("OtherResults").unwrap();
                            // pipe_courants
                            sheet3.get_cell_mut((1, 1)).set_value("pipe_courants");
                            for (i, v) in result.pipe_courants.iter().enumerate() {
                                sheet3.get_cell_mut(((i+2) as u32, 1)).set_value(v.to_string());
                            }
                            // pipe_flow_rates
                            sheet3.get_cell_mut((1, 2)).set_value("pipe_flow_rates");
                            for (i, v) in result.pipe_flow_rates.iter().enumerate() {
                                sheet3.get_cell_mut(((i+2) as u32, 2)).set_value(v.to_string());
                            }

                            let _ = umya_spreadsheet::writer::xlsx::write(&book, path);
                            println!("計算結果をExcelファイルに保存しました");
                        }
                    } else {
                        println!("計算結果がありません");
                    }
                }
                if ui.button("xlsxからパラメータ読込").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Excelファイル", &["xlsx"])
                        .pick_file() {
                        match umya_spreadsheet::reader::xlsx::read(path) {
                            Ok(book) => {
                                if let Some(sheet) = book.get_sheet_by_name("Parameters") {
                                    // 1,2行目: 通常パラメータ
                                    let mut max_col = sheet.get_highest_column();
                                    let mut max_row = sheet.get_highest_row();
                                    // 通常パラメータ（1,2列目）
                                    for col in 1..=2 {
                                        for row in 1..=max_row {
                                            let key = sheet.get_cell((col, row)).map(|cell| cell.get_value().to_string()).unwrap_or_default();
                                            let value = if col == 1 {
                                                // 1列目はパラメータ名、2列目は値
                                                sheet.get_cell((col+1, row)).map(|cell| cell.get_value().to_string()).unwrap_or_default()
                                            } else {
                                                continue;
                                            };
                                            if !key.is_empty() && !value.is_empty() {
                                                self.str_buffer.insert(Box::leak(key.clone().into_boxed_str()), value.clone());
                                            }
                                        }
                                    }

                                    // PipeTypeParam（3列目以降、2列ごとに1Pipe）
                                    let mut pipe_params: Vec<simulation::PipeTypeParam> = Vec::new();
                                    let pipe_param_names = ["圧力損失(MPa)", "パイプ本数(-)"];
                                    let mut pipe_col = 3;
                                    while pipe_col <= max_col {
                                        // 1行目: PipeN
                                        let pipe_label = sheet.get_cell((pipe_col, 1)).map(|cell| cell.get_value().to_string()).unwrap_or_default();
                                        if pipe_label.is_empty() {
                                            break;
                                        }
                                        // 2行目: 空欄（スキップ）
                                        // 3行目以降: パラメータ名と値
                                        let mut pipe_param = simulation::PipeTypeParam::default();
                                        for (j, param_name) in pipe_param_names.iter().enumerate() {
                                            let name_cell = sheet.get_cell((pipe_col, (j+2) as u32)).map(|cell| cell.get_value().to_string()).unwrap_or_default();
                                            let value_cell = sheet.get_cell((pipe_col+1, (j+2) as u32)).map(|cell| cell.get_value().to_string()).unwrap_or_default();
                                            if name_cell == *param_name {
                                                // str_bufferにも反映
                                                let key = format!("{} (PipeType {})", param_name, (pipe_col-2)/2 + 1);
                                                self.str_buffer.insert(Box::leak(key.clone().into_boxed_str()), value_cell.clone());
                                                // PipeTypeParam構造体にも反映
                                                if param_name == &"圧力損失(MPa)" {
                                                    if let Ok(v) = value_cell.parse() {
                                                        pipe_param.pressure_loss.value = v;
                                                    }
                                                } else if param_name == &"パイプ本数(-)" {
                                                    if let Ok(v) = value_cell.parse() {
                                                        pipe_param.pipe_count.value = v;
                                                    }
                                                }
                                            }
                                        }
                                        pipe_params.push(pipe_param);
                                        pipe_col += 2;
                                    }
                                    if !pipe_params.is_empty() {
                                        self.pipe_params = pipe_params;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("xlsx読込エラー: {}", e);
                            }
                        }
                    }
                }
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "PCM Simulation GUI",
        options,
        Box::new(|cc| {
            // フォント設定
            use egui::FontFamily::{Proportional, Monospace};
            use egui::{FontData, FontDefinitions};
            let mut fonts = FontDefinitions::default();
            // NotSansJPのttfファイルをバイナリ埋め込み
            fonts.font_data.insert(
                "NotSansJP".to_owned(),
                FontData::from_static(include_bytes!("../NotoSansJP-Medium.ttf")).into(),
            );
            fonts.families.get_mut(&Proportional).unwrap().insert(0, "NotSansJP".to_owned());
            fonts.families.get_mut(&Monospace).unwrap().insert(0, "NotSansJP".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(SimApp::default()))
        }),
    );
}