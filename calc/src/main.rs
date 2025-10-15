use umya_spreadsheet::*;
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
        if ui.add(egui::TextEdit::singleline(entry).desired_width(80.0)).changed() {
            if let Ok(val) = entry.parse() {
                param.value = val;
            }
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
                        let mut book = umya_spreadsheet::new_file();
                        // 1ページ目: パラメータ一覧（str_bufferのkeyとvalueを列挙）
                        let sheet1 = book.get_sheet_by_name_mut("Sheet1").unwrap();
                        let mut row = 1u32;
                        let mut keys: Vec<_> = self.str_buffer.keys().collect();
                        for key in keys {
                            let k: &str = &**key;
                            let value = self.str_buffer.get(k);
                            let v: &str = value.map(|s| s.as_str()).unwrap_or("");
                            sheet1.get_cell_mut((1, row)).set_value(k);
                            sheet1.get_cell_mut((2, row)).set_value(v);
                            row += 1;
                        }

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

                        let _ = umya_spreadsheet::writer::xlsx::write(&book, "calc_result.xlsx");
                        println!("計算結果をcalc_result.xlsxに保存しました");
                    } else {
                        println!("計算結果がありません");
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