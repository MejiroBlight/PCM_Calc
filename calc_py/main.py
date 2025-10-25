import simulation
import pandas as pd
from datetime import datetime
import os

def save_result_to_xlsx(result: simulation.CalcResult, filename: str = None):
    """
    計算結果をExcelファイルに保存する関数
    
    Args:
        result: 計算結果
        filename: 保存するファイル名（指定しない場合は日時を使用）
    """
    if filename is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"calc_result_{timestamp}.xlsx"
    
    # Excelファイルを作成
    with pd.ExcelWriter(filename, engine='openpyxl') as writer:
        # 0. パラメータ一覧を最初のシートに出力
        from parameters import CalcParam, GeneralParam
        param_dict = {k: v for k, v in CalcParam.__dict__.items() if not k.startswith("__") and not callable(v)}
        general_dict = {k: v for k, v in GeneralParam.__dict__.items() if not k.startswith("__") and not callable(v)}
        # PIPES分解
        pipes = param_dict.pop("PIPES", [])
        pipe_param_dict = {}
        for idx, pipe in enumerate(pipes):
            for field in pipe.__dict__:
                pipe_param_dict[f"PIPES_{field}_{idx}"] = getattr(pipe, field)
        # まとめる
        all_params = {**param_dict, **pipe_param_dict, **general_dict}
        df_params = pd.DataFrame(list(all_params.items()), columns=["Parameter", "Value"])
        df_params.to_excel(writer, sheet_name='Parameters', index=False)

        steps = [s * CalcParam.TIME_STEP for s in range(CalcParam.CALCULATION_STEP - 1)]
        
        # 1. 各時刻の推移データを出力
        total_data = {'Time(s)': steps}
        total_data['Average_End_Temperature'] = result.average_end_temperatures
        total_data['Total_Flow_Amount(L/m)'] = result.total_flow_amounts
        df_total = pd.DataFrame(total_data)
        df_total.to_excel(writer, sheet_name= 'Total_Lapse_Data' , index=False)

        # 2. 各パイプごとの時刻推移データを出力
        for i in range(len(CalcParam.PIPES)):
            pipe_data = {'Time(s)': steps}
            pipe_data[f'End_Temperature'] = result.pipe_end_temperatures[i]
            pipe_data[f'Flow_Rate(m/s)'] = result.pipe_flow_rates[i]
            pipe_data[f'Flow_Amount(L/m)'] = result.pipe_flow_amounts[i]
            pipe_data[f'Reynolds_Number'] = result.pipe_re[i]
            pipe_data[f'Valve_Open_Rate'] = result.pipe_open_rates[i]
            df_pipe = pd.DataFrame(pipe_data)
            df_pipe.to_excel(writer, sheet_name=f'Pipe_{i+1}_Lapse_Data', index=False)

        for i in range(len(CalcParam.PIPES)):
            pipe_data = {'mesh_index': range(CalcParam.MESH_COUNT)}
            pipe_data[f'Last_PCM_Thickness'] = result.pipe_last_pcm_thicknesses[i]
            df_pipe = pd.DataFrame(pipe_data)
            df_pipe.to_excel(writer, sheet_name=f'Pipe_{i+1}_Last_Data', index=False)

        df_other = pd.DataFrame(
            [
                ["Total_Required_Area(m2)", result.total_req_area],
            ],
            columns=["Label", "Value"]
        )
        df_other.to_excel(writer, sheet_name='Other_Results', index=False)
        

    print(f"結果を {filename} に保存しました")
    return filename


# シミュレーション実行
result = simulation.run_simulation()

# 結果をExcelファイルに保存
save_result_to_xlsx(result)