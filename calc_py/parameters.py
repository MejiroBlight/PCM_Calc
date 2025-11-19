import math

class _PipeTypeParam:
    PRESSURE_LOSS: float  # MPa/m
    PIPE_COUNT: int  # -
    PCM_INIT_THICKNESS: float  # m
    
    def __init__(self, PRESSURE_LOSS: float, PIPE_COUNT: int, PCM_INIT_THICKNESS: float):
        self.PRESSURE_LOSS = PRESSURE_LOSS
        self.PIPE_COUNT = PIPE_COUNT
        self.PCM_INIT_THICKNESS = PCM_INIT_THICKNESS

class CalcParam:
    MESH_COUNT = 100  # -
    TIME_STEP = 0.05  # s
    CALCULATION_STEP = 12000  # -
    PIPE_LENGTH = 0.5  # m
    PIPE_OUTDIR = 0.0025  # m
    PIPE_INDIR = 0.002  # m
    WATER_INLET_TEMP = 5.0  # ℃
    WATER_INIT_TEMP = 58.0  # ℃
    PCM_TEMP = 58.0  # ℃
    PCM_MAX_THICKNESS = 0.05  # m
    VALVE_START_CLOSING_TEMP = 5  # ℃
    VALVE_END_CLOSING_TEMP = 3  # ℃
    USE_HIGH_ORDER_UPWIND_DIF = True  # 高次風上差分の使用
    WATER_VISC_REF_TEMP = None  # ℃ or None # 動粘度計算の基準温度（Noneの場合は平均温度参照）
    PIPES = [
        _PipeTypeParam(PRESSURE_LOSS=0.0001, PIPE_COUNT=1000, PCM_INIT_THICKNESS=0.0001),  # Type A
    ]

class GeneralParam:
    WATER_DENS = 998.0  # kg/m3
    WATER_COND = 0.602  # W/mK
    WATER_SPEC = 4200.0  # J/kgK
    COPPER_COND = 106.0  # W/mK
    PCM_LATENT = 190000.0  # J/kg
    PCM_DENS = 775.0  # kg/m3
    NUSSCELT = 3.66  # -
    PCM_COND = 0.172  # W/mK
    G = 9.80665  # m/s2
    _WATER_VISC_A = 1.83698 * 10**-6
    _WATER_VISC_B = 1855.2353

    def water_viscosity(self, temperature: float) -> float:
        return self._WATER_VISC_A * math.exp(self._WATER_VISC_B / (temperature + 273.15))  # Pa・s