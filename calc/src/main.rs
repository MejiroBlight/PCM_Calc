
use rfd::FileDialog;
mod simulation;

use simulation::{GeneralParam, CalcParam, simulate};
use calc::Described;
use eframe::{egui};

struct SimApp {
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
            calc_param: simulation::CalcParam::default(),
            general_param: simulation::GeneralParam::default(),
            pipe_params: vec![simulation::PipeTypeParam::default()],
            result: None,
        }
    }
}

impl SimApp {
    fn reset_params(&mut self) {
        self.calc_param = simulation::CalcParam::default();
        self.general_param = simulation::GeneralParam::default();
        self.pipe_params = vec![simulation::PipeTypeParam::default()];
        self.result = None;
    }
}

struct ParamArrays<'a>{
    calc_usize: Vec<&'a mut Described<usize>>,
    calc_f64: Vec<&'a mut Described<f64>>,
    general: Vec<&'a mut Described<f64>>
}

fn add_param<T: egui::emath::Numeric>(ui: &mut egui::Ui, param: &mut Described<T>) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", param.desc()));
        ui.add(egui::DragValue::new(&mut param.value));
    });
}

fn get_param_arrays<'a>(
    calc_param: &'a mut CalcParam,
    general_param: &'a mut GeneralParam,
) -> ParamArrays<'a> {
    let calc_usize = vec![
        &mut calc_param.mesh_count,
        &mut calc_param.calculation_step,
    ];
    let calc_f64 = vec![
        &mut calc_param.time_step,
        &mut calc_param.pipe_length,
        &mut calc_param.pipe_outdir,
        &mut calc_param.pipe_indir,
        &mut calc_param.pcm_init_thickness,
        &mut calc_param.water_init_temp,
        &mut calc_param.pcm_temp,
    ];
    let general = vec![
        &mut general_param.water_dens,
        &mut general_param.water_cond,
        &mut general_param.water_spec,
        &mut general_param.water_visc,
        &mut general_param.copper_cond,
        &mut general_param.pcm_latent,
        &mut general_param.pcm_dens,
        &mut general_param.nusscelt,
        &mut general_param.pcm_cond,
        &mut general_param.g,
    ];
    ParamArrays {
        calc_usize,
        calc_f64,
        general,
    }
}

impl eframe::App for SimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let ParamArrays {
                    calc_usize,
                    calc_f64,
                    general,
                } = get_param_arrays(&mut self.calc_param, &mut self.general_param);
                ui.heading("PCM Simulation");
                // CalcParam
                for param in calc_usize {
                    add_param(ui, param);
                }
                for param in calc_f64 {
                    add_param(ui, param);
                }
                ui.separator();

                // GeneralParam
                for param in general {
                    add_param(ui, param);
                }
                ui.separator();

                // PipeTypeParam
                ui.label("PipeType Parameters");
                let mut remove_idx = None;
                let pipe_count = self.pipe_params.len();
                for (i, param) in self.pipe_params.iter_mut().enumerate() {
                    ui.label(format!("PipeType {}", i + 1));
                    ui.horizontal(|ui| {
                        add_param(ui, &mut param.pressure_loss);
                        add_param(ui, &mut param.pipe_count);
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
                            let ParamArrays {
                                calc_usize,
                                calc_f64,
                                general,
                            } = get_param_arrays(&mut self.calc_param, &mut self.general_param);
                            let mut book = umya_spreadsheet::new_file();
                            // 1ページ目: パラメータ一覧（1,2行目は通常パラメータ、3列目以降にPipeTypeParamをpipeごとに2列ずつ）
                            let sheet1 = book.get_sheet_by_name_mut("Sheet1").unwrap();
                            // 1,2行目: 通常パラメータ
                            let mut row = 1;
                            for param in calc_usize.iter(){
                                sheet1.get_cell_mut((1, row)).set_value(param.desc());
                                sheet1.get_cell_mut((2, row)).set_value(param.value.to_string());
                                row += 1;
                            }
                            for param in calc_f64.iter(){
                                sheet1.get_cell_mut((1, row)).set_value(param.desc());
                                sheet1.get_cell_mut((2, row)).set_value(param.value.to_string());
                                row += 1;
                            }
                            for (r, param) in general.iter().enumerate() {
                                sheet1.get_cell_mut((3, (r + 1) as u32)).set_value(param.desc());
                                sheet1.get_cell_mut((4, (r + 1) as u32)).set_value(param.value.to_string());
                            }
                            for (i, param) in self.pipe_params.iter().enumerate() {
                                sheet1.get_cell_mut(((5 + i * 2) as u32, 1)).set_value(format!("PipeType {}", i + 1));
                                sheet1.get_cell_mut(((5 + i * 2) as u32, 2)).set_value(param.pipe_count.desc());
                                sheet1.get_cell_mut(((5 + i * 2 + 1) as u32, 2)).set_value(param.pipe_count.value.to_string());
                                sheet1.get_cell_mut(((5 + i * 2) as u32, 3)).set_value(param.pressure_loss.desc());
                                sheet1.get_cell_mut(((5 + i * 2 + 1) as u32, 3)).set_value(param.pressure_loss.value.to_string());
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
                                    self.reset_params();
                                    let ParamArrays {
                                        mut calc_usize,
                                        mut calc_f64,
                                        mut general,
                                    } = get_param_arrays(&mut self.calc_param, &mut self.general_param);
                                    let max_col = sheet.get_highest_column();
                                    let max_row = sheet.get_highest_row();
                                    for row in 1..=max_row {
                                        for col in [1u32, 3u32]{
                                            if let Some(cell) = sheet.get_cell((col, row)) {
                                                if let Some(param) = calc_usize.iter_mut().find(|p| p.desc() == cell.get_value()) {
                                                    if let Some(value_cell) = sheet.get_cell((col+1, row)) {
                                                        if let Ok(value) = value_cell.get_value().parse::<usize>() {
                                                            param.value = value;
                                                        }
                                                    }
                                                }else if let Some(param) = calc_f64.iter_mut().find(|p| p.desc() == cell.get_value()) {
                                                    if let Some(value_cell) = sheet.get_cell((col+1, row)) {
                                                        if let Ok(value) = value_cell.get_value().parse::<f64>() {
                                                            param.value = value;
                                                        }
                                                    }
                                                }else if let Some(param) = general.iter_mut().find(|p| p.desc() == cell.get_value()) {
                                                    if let Some(value_cell) = sheet.get_cell((col+1, row)) {
                                                        if let Ok(value) = value_cell.get_value().parse::<f64>() {
                                                            param.value = value;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    for col in (5..=max_col).step_by(2){
                                        if let Some(cell) = sheet.get_cell((col, 1)) {
                                            if cell.get_value().starts_with("PipeType") {
                                                let mut pipe_param = simulation::PipeTypeParam::default();
                                                for row in 2..=max_row {
                                                    if let Some(cell) = sheet.get_cell((col, row)) {
                                                        if cell.get_value() == pipe_param.pipe_count.desc() {
                                                            if let Some(value_cell) = sheet.get_cell((col+1, row)) {
                                                                if let Ok(value) = value_cell.get_value().parse::<usize>() {
                                                                    pipe_param.pipe_count.value = value;
                                                                }
                                                            }
                                                        } else if cell.get_value() == pipe_param.pressure_loss.desc() {
                                                            if let Some(value_cell) = sheet.get_cell((col+1, row)) {
                                                                if let Ok(value) = value_cell.get_value().parse::<f64>() {
                                                                    pipe_param.pressure_loss.value = value;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                self.pipe_params.push(pipe_param);
                                            }
                                        }
                                    }
                                }
                                println!("xlsxからパラメータを読込みました");
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