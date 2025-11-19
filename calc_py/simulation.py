import parameters as params
import math
from dataclasses import dataclass
from typing import List

@dataclass
class Mesh:
    temp: float
    pcm_thickness: float

@dataclass
class Pipe:
    meshes: List[Mesh]
    flow_rate: float
    composition_rate: float
    valve_open_rate: float

@dataclass
class CalcResult:
    pipe_flow_rates: List[List[float]]
    pipe_end_temperatures: List[List[float]]
    pipe_re: List[List[float]]
    pipe_open_rates: List[List[float]]
    pipe_flow_amounts: List[List[float]]
    average_end_temperatures: List[float]
    total_flow_amounts: List[float]
    pipe_last_pcm_thicknesses: List[List[float]]
    total_integ_flow_amounts: List[float]
    total_req_area: float

def run_simulation() -> CalcResult:
    # パラメータ取得
    general_param = params.GeneralParam()
    calc_param = params.CalcParam()
    pipe_params = calc_param.PIPES
    
    # 初期計算
    x = calc_param.PIPE_LENGTH / calc_param.MESH_COUNT
    
    # パイプ情報初期化
    all_pipe_count = sum(p.PIPE_COUNT for p in pipe_params)
    pipes : list[Pipe] = []
    
    for pipe_param in pipe_params:
        # メッシュ初期化
        meshes = [Mesh(temp=calc_param.WATER_INIT_TEMP, 
                      pcm_thickness=pipe_param.PCM_INIT_THICKNESS) 
                 for _ in range(calc_param.MESH_COUNT)]
        
        # パイプ組成割合
        composition_rate = pipe_param.PIPE_COUNT / all_pipe_count
        
        pipes.append(Pipe(meshes=meshes, composition_rate=composition_rate, flow_rate=0.0, valve_open_rate=1.0))
    
    # 結果初期化
    result = CalcResult(
        pipe_end_temperatures=[[] for _ in pipes],
        pipe_flow_rates=[[] for _ in pipes],
        pipe_flow_amounts=[[] for _ in pipes],
        pipe_re=[[] for _ in pipes],
        pipe_open_rates=[[] for _ in pipes],
        pipe_last_pcm_thicknesses=[[] for _ in pipes],
        average_end_temperatures=[],
        total_flow_amounts=[],
        total_integ_flow_amounts=[],
        total_req_area=0.0
    )
    
    print("計算開始")
    total_steps = calc_param.CALCULATION_STEP
    
    for step in range(1, total_steps):
        if step % 100 == 0 or step == 1:
            print(f"進行状況: {step}/{total_steps}")
        
        for pipe in pipes:
            # メッシュ毎の計算
            for mesh in pipe.meshes:
                if mesh.pcm_thickness >= calc_param.PCM_MAX_THICKNESS:
                    continue
                # 熱抵抗
                r = (
                    1.0 / (general_param.NUSSCELT * general_param.WATER_COND) +
                    math.log(calc_param.PIPE_OUTDIR / calc_param.PIPE_INDIR) / (2.0 * general_param.COPPER_COND) +
                    math.log((calc_param.PIPE_OUTDIR + 2.0 * mesh.pcm_thickness) / calc_param.PIPE_OUTDIR) / (2.0 * general_param.PCM_COND)
                ) / (math.pi * x)
                
                # 温度変化
                delta_temp = 4 * (calc_param.PCM_TEMP - mesh.temp) * calc_param.TIME_STEP / (r * general_param.WATER_SPEC * general_param.WATER_DENS * math.pi * calc_param.PIPE_INDIR**2 * x)
                # PCM厚み変化
                delta_pcm_thickness = (calc_param.PIPE_INDIR**2 * general_param.WATER_SPEC * general_param.WATER_DENS * delta_temp / 
                                     (4.0 * (calc_param.PIPE_OUTDIR + 2.0 * mesh.pcm_thickness) * general_param.PCM_DENS * general_param.PCM_LATENT))
                # 計算結果反映
                mesh.temp += delta_temp
                mesh.pcm_thickness += max(delta_pcm_thickness, 0)
            
            #流量計算
            average_temp = sum(mesh.temp for mesh in pipe.meshes) / calc_param.MESH_COUNT
            pressure_loss = calc_param.PIPES[pipes.index(pipe)].PRESSURE_LOSS * 10**6 * pipe.valve_open_rate # Pa/m
            viscosity = general_param.water_viscosity(
                average_temp if calc_param.WATER_VISC_REF_TEMP is None 
                else calc_param.WATER_VISC_REF_TEMP
            )
            v = pressure_loss * calc_param.PIPE_INDIR**2 / (32 * viscosity * calc_param.PIPE_LENGTH)
            for _ in range(50):  # 収束計算ループ
                Re = general_param.WATER_DENS * v * calc_param.PIPE_INDIR / viscosity
                if Re == 0:
                    break

                if Re < 2300:
                    f = 64.0 / Re
                else:
                    f = 0.3164 * Re ** -0.25
                # 新しい速度を計算
                v_new = math.sqrt(2 * pressure_loss * calc_param.PIPE_INDIR / (general_param.WATER_DENS * f * calc_param.PIPE_LENGTH))
                if abs(v_new - v) < 1e-6:
                    v = v_new
                    break
                v = v_new
            pipe.flow_rate = v
            
            # 流速・レイノルズ数・流量記録
            result.pipe_flow_rates[pipes.index(pipe)].append(v)
            result.pipe_re[pipes.index(pipe)].append(Re)
            result.pipe_flow_amounts[pipes.index(pipe)].append(v * math.pi * (calc_param.PIPE_INDIR / 2)**2 * general_param.WATER_DENS * 60)  # L/min
            if step % 1000 == 0 or step == 1:
                print(f"クーラン数 (パイプ {pipes.index(pipe)+1}): {v*calc_param.TIME_STEP/x :.2f}")
            
            # メッシュ温度更新
            next_temps = [0.0] * calc_param.MESH_COUNT
            if calc_param.USE_HIGH_ORDER_UPWIND_DIF:
                # 高次風上差分法の実装
                for i in range(calc_param.MESH_COUNT):
                    # 各方向の温度を取得
                    t_p2 = (
                        calc_param.WATER_INLET_TEMP if i < 2 
                        else pipe.meshes[i - 2].temp
                    )
                    t_p1 = (
                        calc_param.WATER_INLET_TEMP if i < 1 
                        else pipe.meshes[i - 1].temp
                    )
                    t_c = pipe.meshes[i].temp
                    t_n1 = (
                        pipe.meshes[i].temp if i + 1 >= calc_param.MESH_COUNT 
                        else pipe.meshes[i + 1].temp
                    )
                    t_n2 = (
                        t_n1 if i + 2 >= calc_param.MESH_COUNT 
                        else pipe.meshes[i + 2].temp
                    )
                    
                    # 高次風上差分による温度更新
                    next_temps[i] = t_c - pipe.flow_rate * (2.0 * t_p2 - 10.0 * t_p1 + 9.0 * t_c - 2.0 * t_n1 + t_n2) * calc_param.TIME_STEP / (6.0 * x)
            else :
                overrun = pipe.flow_rate * calc_param.TIME_STEP / x
                for i in range(calc_param.MESH_COUNT): 
                    offset = math.floor(overrun)
                    prev_temp = (
                        calc_param.WATER_INLET_TEMP if i - offset - 1 < 0 
                        else pipe.meshes[i - offset - 1].temp
                    )
                    current_temp = (
                        calc_param.WATER_INLET_TEMP if i - offset < 0 
                        else pipe.meshes[i - offset].temp
                    )
                    mix_rate = overrun - offset
                    next_temps[i] = prev_temp * mix_rate + current_temp * (1 - mix_rate)
            for i in range(calc_param.MESH_COUNT):
                pipe.meshes[i].temp = next_temps[i]

            # バルブ開閉率
            last_mesh_temp = pipe.meshes[calc_param.MESH_COUNT - 1].temp
            if last_mesh_temp <= params.CalcParam.VALVE_START_CLOSING_TEMP:
                pipe.valve_open_rate = max(0.0, (last_mesh_temp - params.CalcParam.VALVE_END_CLOSING_TEMP) / max(1 ,params.CalcParam.VALVE_START_CLOSING_TEMP - params.CalcParam.VALVE_END_CLOSING_TEMP))
        
        # 結果保存
        # 各パイプの出口温度
        for i, pipe in enumerate(pipes):
            result.pipe_end_temperatures[i].append(pipe.meshes[calc_param.MESH_COUNT - 1].temp)
            result.pipe_open_rates[i].append(pipe.valve_open_rate)
        
        # 平均出口温度
        weighted_temp_sum = sum(p.flow_rate * p.composition_rate * p.meshes[calc_param.MESH_COUNT - 1].temp for p in pipes)
        weight_sum = sum(p.flow_rate * p.composition_rate for p in pipes)
        if weight_sum == 0:
            result.average_end_temperatures.append(0.0)
        else:
            result.average_end_temperatures.append(weighted_temp_sum / weight_sum)

        # 合計流量
        total_flow = sum(amounts[-1] * calc_param.PIPES[i].PIPE_COUNT for i, amounts in enumerate(result.pipe_flow_amounts))
        result.total_flow_amounts.append(total_flow)
        # 合計積算流量
        result.total_integ_flow_amounts.append(
            result.total_integ_flow_amounts[-1] + total_flow * calc_param.TIME_STEP / 60.0 if step > 1 
            else total_flow * calc_param.TIME_STEP / 60.0
        )
    
    
    # PCM厚み保存
    for i, pipe in enumerate(pipes):
        result.pipe_last_pcm_thicknesses[i] = [mesh.pcm_thickness for mesh in pipe.meshes]
    # 必要面積計算
    result.total_req_area = sum(
        math.pi * (calc_param.PIPE_OUTDIR / 2.0 + pipe.meshes[0].pcm_thickness)**2 * calc_param.PIPES[i].PIPE_COUNT
        for i, pipe in enumerate(pipes))

    print("計算完了")

    return result
