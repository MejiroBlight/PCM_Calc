mod simulation;
use simulation::{GeneralParam, CalcParam, PipeTypeParam, simulate};
use eframe::{egui};

struct SimApp {
    // 入力パラメータ
    mesh_count: usize,
    time_step: f64,
    calculation_step: usize,
    pipe_length: f64,
    pipe_outdir: f64,
    pipe_indir: f64,
    pcm_init_thickness: f64,
    water_init_temp: f64,
    pcm_temp: f64,
    // GeneralParam
    water_dens: f64,
    water_cond: f64,
    water_spec: f64,
    water_visc: f64,
    copper_cond: f64,
    pcm_latent: f64,
    pcm_dens: f64,
    nusscelt: f64,
    pcm_cond: f64,
    g: f64,
    // PipeTypeParam
    pressure_loss: f64,
    pipe_count: usize,
    // 結果
    result: Option<simulation::CalcResult>,
}

impl Default for SimApp {
    fn default() -> Self {
        Self {
            mesh_count: 10,
            time_step: 0.1,
            calculation_step: 100,
            pipe_length: 1.0,
            pipe_outdir: 0.03,
            pipe_indir: 0.025,
            pcm_init_thickness: 0.01,
            water_init_temp: 20.0,
            pcm_temp: 60.0,
            water_dens: 1000.0,
            water_cond: 0.6,
            water_spec: 4180.0,
            water_visc: 0.001,
            copper_cond: 400.0,
            pcm_latent: 200000.0,
            pcm_dens: 800.0,
            nusscelt: 100.0,
            pcm_cond: 0.2,
            g: 9.8,
            pressure_loss: 0.1,
            pipe_count: 1,
            result: None,
        }
    }
}

impl SimApp {
    fn add_param<T>(ui: &mut egui::Ui, name: String, value: &mut T){
        ui.horizontal(|ui| {
            ui.label("pcm_dens:");
            if ui.add(egui::TextEdit::singleline(&mut self.pcm_dens_str).desired_width(80.0)).changed() {
                if let Ok(val) = self.pcm_dens_str.parse() {
                    self.general_params.pcm_dens = val;
                }
            }
            ui.label("kg/m³");
        });
    }
}

impl eframe::App for SimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PCM Simulation");
            
            ui.horizontal(|ui| {
                ui.label("mesh_count");
                ui.add(egui::DragValue::new(&mut self.mesh_count));
                ui.label("time_step");
                ui.add(egui::DragValue::new(&mut self.time_step));
                ui.label("calculation_step");
                ui.add(egui::DragValue::new(&mut self.calculation_step));
            });
            ui.horizontal(|ui| {
                ui.label("pipe_length");
                ui.add(egui::DragValue::new(&mut self.pipe_length));
                ui.label("pipe_outdir");
                ui.add(egui::DragValue::new(&mut self.pipe_outdir));
                ui.label("pipe_indir");
                ui.add(egui::DragValue::new(&mut self.pipe_indir));
            });
            ui.horizontal(|ui| {
                ui.label("pcm_init_thickness");
                ui.add(egui::DragValue::new(&mut self.pcm_init_thickness));
                ui.label("water_init_temp");
                ui.add(egui::DragValue::new(&mut self.water_init_temp));
                ui.label("pcm_temp");
                ui.add(egui::DragValue::new(&mut self.pcm_temp));
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("water_dens");
                ui.add(egui::DragValue::new(&mut self.water_dens));
                ui.label("water_cond");
                ui.add(egui::DragValue::new(&mut self.water_cond));
                ui.label("water_spec");
                ui.add(egui::DragValue::new(&mut self.water_spec));
            });
            ui.horizontal(|ui| {
                ui.label("water_visc");
                ui.add(egui::DragValue::new(&mut self.water_visc));
                ui.label("copper_cond");
                ui.add(egui::DragValue::new(&mut self.copper_cond));
                ui.label("pcm_latent");
                ui.add(egui::DragValue::new(&mut self.pcm_latent));
            });
            ui.horizontal(|ui| {
                ui.label("pcm_dens");
                ui.add(egui::DragValue::new(&mut self.pcm_dens));
                ui.label("nusscelt");
                ui.add(egui::DragValue::new(&mut self.nusscelt));
                ui.label("pcm_cond");
                ui.add(egui::DragValue::new(&mut self.pcm_cond));
                ui.label("g");
                ui.add(egui::DragValue::new(&mut self.g));
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("pressure_loss");
                ui.add(egui::DragValue::new(&mut self.pressure_loss));
                ui.label("pipe_count");
                ui.add(egui::DragValue::new(&mut self.pipe_count));
            });
            if ui.button("計算実行").clicked() {
                let general_param = GeneralParam {
                    water_dens: self.water_dens,
                    water_cond: self.water_cond,
                    water_spec: self.water_spec,
                    water_visc: self.water_visc,
                    copper_cond: self.copper_cond,
                    pcm_latent: self.pcm_latent,
                    pcm_dens: self.pcm_dens,
                    nusscelt: self.nusscelt,
                    pcm_cond: self.pcm_cond,
                    g: self.g,
                };
                let calc_param = CalcParam {
                    mesh_count: self.mesh_count,
                    time_step: self.time_step,
                    calculation_step: self.calculation_step,
                    pipe_length: self.pipe_length,
                    pipe_outdir: self.pipe_outdir,
                    pipe_indir: self.pipe_indir,
                    pcm_init_thickness: self.pcm_init_thickness,
                    water_init_temp: self.water_init_temp,
                    pcm_temp: self.pcm_temp,
                };
                let pipe_params = vec![PipeTypeParam {
                    pressure_loss: self.pressure_loss,
                    pipe_count: self.pipe_count,
                }];
                self.result = Some(simulate(general_param, calc_param, pipe_params));
            }
            if let Some(result) = &self.result {
                ui.separator();
                ui.label("平均出口温度 (一部):");
                for (i, temp) in result.average_end_temperatures.iter().take(10).enumerate() {
                    ui.label(format!("step {}: {:.2}℃", i, temp));
                }
            }
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