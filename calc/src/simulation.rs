use calc::described_struct;

described_struct! {
    struct PipeTypeParam {
        pressure_loss: f64 => "圧力損失(MPa)",
        pipe_count: usize => "パイプ本数(-)",
    }
}

described_struct! {
    struct CalcParam {
        mesh_count: usize => "メッシュ数(-)",
        time_step: f64 => "時間刻み(s)",
        calculation_step: usize => "計算ステップ数(-)",
        pipe_length: f64 => "パイプ長さ(m)",
        pipe_outdir: f64 => "パイプ外径(m)",
        pipe_indir: f64 => "パイプ内径(m)",
        pcm_init_thickness: f64 => "PCM初期厚み(m)",
        water_init_temp: f64 => "水初期温度(℃)",
        pcm_temp: f64 => "PCM温度(℃)",
    }
}

described_struct! {
    struct GeneralParam {
        water_dens: f64 => "水密度(kg/m3)",
        water_cond: f64 => "水熱伝導率(W/mK)",
        water_spec: f64 => "水比熱(J/kgK)",
        water_visc: f64 => "水粘性(Pa・s)",
        copper_cond: f64 => "銅熱伝導率(W/mK)",
        pcm_latent: f64 => "PCM潜熱(J/kg)",
        pcm_dens: f64 => "PCM密度(kg/m3)",
        nusscelt: f64 => "ヌセルト数(-)",
        pcm_cond: f64 => "PCM熱伝導率(W/mK)",
        g: f64 => "重力加速度(m/s2)",
    }
}



#[derive(Debug, Clone, Copy)]
pub struct Mesh{
    pub temp: f64,
    pub pcm_thickness: f64
}

pub struct Pipe{
    pub meshes: Vec<Mesh>,
    pub flow_rate: f64,
    pub composition_rate: f64,
}

#[derive(Default)]
pub struct CalcResult{
    pub pipe_courants : Vec<f64>,
    pub pipe_flow_rates : Vec<f64>,
    pub pipe_end_temperatures: Vec<Vec<f64>>,
    pub average_end_temperatures: Vec<f64>,
}

pub fn simulate(general_param: GeneralParam, calc_param: CalcParam, pipe_params: Vec<PipeTypeParam>) -> CalcResult{
    let x = calc_param.pipe_length / (calc_param.mesh_count as f64);
    //パイプ情報初期化
    let all_pipe_count = pipe_params.iter().map(|p| p.pipe_count).sum::<usize>();
    let mut pipes: Vec<Pipe> = pipe_params.into_iter()
        .map(|params| Pipe{
            //メッシュ初期化
            meshes: vec![Mesh{temp: calc_param.water_init_temp, pcm_thickness: calc_param.pcm_init_thickness}; calc_param.mesh_count],
            //流量計算
            flow_rate: 2.0 * calc_param.pipe_indir.powi(2) * params.pressure_loss * 10f64.powi(6) / (64.0 * calc_param.pipe_length * general_param.water_visc * general_param.water_dens),
            //パイプ組成割合
            composition_rate: params.pipe_count as f64 / all_pipe_count as f64,
        })
        .collect::<Vec<_>>();
    //リザルト初期化
    let mut result = CalcResult{
        pipe_flow_rates: pipes.iter().map(|p| p.flow_rate).collect(),
        //クーラン数計算
        pipe_courants: pipes.iter().map(|p| p.flow_rate * calc_param.time_step / x).collect(),
        pipe_end_temperatures: vec![vec![calc_param.water_init_temp]; pipes.len()],
        average_end_temperatures: vec![calc_param.water_init_temp],
    };
    //計算ループ
    for _ in 1..calc_param.calculation_step {
        for pipe in &mut pipes {
            //メッシュ毎の計算
            for mesh in &mut pipe.meshes {
                //熱通過率
                let k = 1.0 / (
                    1.0 / (general_param.nusscelt * general_param.water_cond)
                    + (calc_param.pipe_outdir / calc_param.pipe_indir).ln() / (2.0 * general_param.copper_cond)
                    + ((calc_param.pipe_outdir + 2.0 * mesh.pcm_thickness) / calc_param.pipe_outdir).ln() / (2.0 * general_param.pcm_cond));
                //温度変化
                let delta_temp = 4.0 * k * (calc_param.pcm_temp - mesh.temp) * calc_param.time_step / (calc_param.pipe_indir * general_param.water_spec * general_param.water_dens);
                //PCM厚み変化
                let delta_pcm_thickness = calc_param.pipe_indir.powi(2) * general_param.water_spec * general_param.water_dens * delta_temp 
                    / (4.0 * (calc_param.pipe_outdir + 2.0 * mesh.pcm_thickness) * general_param.pcm_dens * general_param.pcm_latent);
                //計算結果反映
                mesh.temp += delta_temp;
                mesh.pcm_thickness += delta_pcm_thickness;
            }
            let mut next_temps = vec![0.0; calc_param.mesh_count];
            for i in 0..calc_param.mesh_count {
                //高次風上差分
                next_temps[i] = {
                    let t_p2 = if i < 2 {
                        calc_param.water_init_temp
                    } else {
                        pipe.meshes[i - 2].temp
                    };
                    let t_p1 = if i < 1 {
                        calc_param.water_init_temp
                    } else {
                        pipe.meshes[i - 1].temp
                    };
                    let t_c = pipe.meshes[i].temp;
                    let t_n1 = if i + 1 >= calc_param.mesh_count {
                        pipe.meshes[i].temp
                    } else {
                        pipe.meshes[i + 1].temp
                    };
                    let t_n2 = if i + 2 >= calc_param.mesh_count {
                        t_n1
                    } else {
                        pipe.meshes[i + 2].temp
                    };
                    t_c - pipe.flow_rate * (2.0 * t_p2 - 10.0 * t_p1 + 9.0 * t_c - 2.0 * t_n1 + t_n2) * calc_param.time_step / (6.0 * x)
                };
            }
            //差分結果反映
            for i in 0..calc_param.mesh_count {
                pipe.meshes[i].temp = next_temps[i];
            }
        }
        //結果保存
        //各パイプの出口温度
        for (i, pipe) in pipes.iter().enumerate() {
            result.pipe_end_temperatures[i].push(pipe.meshes[calc_param.mesh_count - 1].temp);
        }
        //平均出口温度
        result.average_end_temperatures.push(
            pipes.iter().map(|p| p.flow_rate * p.composition_rate * p.meshes[calc_param.mesh_count - 1].temp).sum::<f64>()
            / pipes.iter().map(|p| p.flow_rate * p.composition_rate).sum::<f64>()
        );
    }
    return result;
}