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
        
        # 1. 平均出口温度
        df_avg_temp = pd.DataFrame({
            'Step': range(len(result.average_end_temperatures)),
            'Average_End_Temperature': result.average_end_temperatures
        })
        df_avg_temp.to_excel(writer, sheet_name='Average_End_Temperature', index=False)
        
        # 2. 各パイプの出口温度
        max_steps = max(len(temps) for temps in result.pipe_end_temperatures)
        pipe_temp_data = {'Step': range(max_steps)}
        
        for i, temps in enumerate(result.pipe_end_temperatures):
            # データ長を合わせるためにNoneで埋める
            padded_temps = temps + [None] * (max_steps - len(temps))
            pipe_temp_data[f'Pipe_{i+1}_End_Temperature'] = padded_temps
        
        df_pipe_temps = pd.DataFrame(pipe_temp_data)
        df_pipe_temps.to_excel(writer, sheet_name='Pipe_End_Temperatures', index=False)
        
        # 3. 各パイプの流量
        max_flow_steps = max(len(flows) for flows in result.pipe_flow_rates)
        pipe_flow_data = {'Step': range(max_flow_steps)}
        
        for i, flows in enumerate(result.pipe_flow_rates):
            # データ長を合わせるためにNoneで埋める
            padded_flows = flows + [None] * (max_flow_steps - len(flows))
            pipe_flow_data[f'Pipe_{i+1}_Flow_Rate'] = padded_flows
        
        df_pipe_flows = pd.DataFrame(pipe_flow_data)
        df_pipe_flows.to_excel(writer, sheet_name='Pipe_Flow_Rates', index=False)
        
        # 4. 各パイプのレイノルズ数
        max_re_steps = max(len(res) for res in result.pipe_re)
        pipe_re_data = {'Step': range(max_re_steps)}
        
        for i, res in enumerate(result.pipe_re):
            # データ長を合わせるためにNoneで埋める
            padded_res = res + [None] * (max_re_steps - len(res))
            pipe_re_data[f'Pipe_{i+1}_Reynolds_Number'] = padded_res
        
        df_pipe_re = pd.DataFrame(pipe_re_data)
        df_pipe_re.to_excel(writer, sheet_name='Pipe_Reynolds_Numbers', index=False)

        # 5. 各パイプのバルブ開閉率
        max_open_rate_steps = max(len(rates) for rates in result.pipe_open_rates)
        pipe_open_rate_data = {'Step': range(max_open_rate_steps)}
        for i, rates in enumerate(result.pipe_open_rates):
            # データ長を合わせるためにNoneで埋める
            padded_rates = rates + [None] * (max_open_rate_steps - len(rates))
            pipe_open_rate_data[f'Pipe_{i+1}_Valve_Open_Rate'] = padded_rates
        
        df_pipe_valve = pd.DataFrame(pipe_open_rate_data)
        df_pipe_valve.to_excel(writer, sheet_name='Pipe_Valve_Open_Rates', index=False)

        # 6. 各パイプの最終PCM厚さ
        max_pcm_thickness_steps = max(len(thicknesses) for thicknesses in result.pipe_last_pcm_thicknesses)
        pipe_pcm_thickness_data = {'Step': range(max_pcm_thickness_steps)}
        for i, thicknesses in enumerate(result.pipe_last_pcm_thicknesses):
            # データ長を合わせるためにNoneで埋める
            padded_thicknesses = thicknesses + [None] * (max_pcm_thickness_steps - len(thicknesses))
            pipe_pcm_thickness_data[f'Pipe_{i+1}_Last_PCM_Thickness'] = padded_thicknesses
        df_pipe_pcm = pd.DataFrame(pipe_pcm_thickness_data)
        df_pipe_pcm.to_excel(writer, sheet_name='Pipe_Last_PCM_Thicknesses', index=False)
    
    print(f"結果を {filename} に保存しました")
    return filename

# シミュレーション実行
result = simulation.run_simulation()

# 結果をExcelファイルに保存
save_result_to_xlsx(result)