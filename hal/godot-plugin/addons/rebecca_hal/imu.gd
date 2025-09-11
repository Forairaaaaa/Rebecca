## Rebecca-HAL IMU
class_name Imu
extends Node


enum ImuOrientation {
	Portrait,
	Landscape,
}

signal imu_data_received(data: ImuData)
signal orientation_changed(orientation: String)
signal orientation_angle_changed(angle_degrees: float) # 角度版本，0-360度

@export var host: String = "localhost"
@export var port: int = 12580
@export var device_id: String = "imu0"
@export var cli_tool_name: String = "rebecca-hal"

# 方向检测配置
@export var orientation_stability_threshold: float = 1.0 # 方向稳定阈值（秒）
@export var orientation_angle_threshold: float = 15.0 # 方向切换的角度阈值（度）

var _cs_imu: Node

# 方向检测相关变量
var _current_orientation: ImuOrientation = ImuOrientation.Portrait
var _current_angle: float = 0.0
var _last_orientation_time: float = 0.0
var _orientation_stable_time: float = 0.0
var _orientation_samples: Array[float] = []
var _max_samples: int = 10

func _ready():
	# Load cs node
	var imu_cs_class = load("res://addons/rebecca_hal/cs/Imu/Imu.cs")
	_cs_imu = imu_cs_class.new()
	add_child(_cs_imu)
	
	# Setup
	_cs_imu.host = host
	_cs_imu.port = port
	_cs_imu.deviceId = device_id
	_cs_imu.cliToolName = cli_tool_name
	_cs_imu.ImuDataReceived.connect(_on_cs_imu_data_received)


func _exit_tree():
	if _cs_imu:
		_cs_imu.Cleanup()


func _on_cs_imu_data_received(data: String):
	# print(data)
	var imu_data = ImuData.new()
	imu_data.init_from_json(data)
	imu_data_received.emit(imu_data)

	# 更新方向检测
	_update_orientation(imu_data)


func start_reading() -> bool:
	if _cs_imu:
		return _cs_imu.StartReading()
	return false


func stop_reading() -> bool:
	if _cs_imu:
		return _cs_imu.StopReading()
	return false


func get_orientation() -> String:
	return _get_orientation_string(_current_orientation)


func get_orientation_angle() -> float:
	return _current_angle


func _get_orientation_string(orientation: ImuOrientation) -> String:
	match orientation:
		ImuOrientation.Portrait:
			return "portrait"
		ImuOrientation.Landscape:
			return "landscape"
		_:
			return "unknown"


## 更新设备方向检测
func _update_orientation(imu_data: ImuData):
	# 计算设备相对于重力的角度
	var gravity = imu_data.get_gravity_direction()
	
	# 计算设备的倾斜角度（相对于垂直方向）
	var angle = _calculate_device_angle(gravity)
	
	# 添加到样本数组进行平滑处理
	_orientation_samples.append(angle)
	if _orientation_samples.size() > _max_samples:
		_orientation_samples.pop_front()
	
	# 计算平均角度
	var avg_angle = _get_average_angle()
	
	# 判断方向
	var new_orientation = _angle_to_orientation(avg_angle)
	
	# 检查方向稳定性
	var current_timestamp = Time.get_time_dict_from_system()
	var timestamp_seconds = current_timestamp.hour * 3600 + current_timestamp.minute * 60 + current_timestamp.second
	
	if new_orientation != _current_orientation:
		_current_orientation = new_orientation
		_last_orientation_time = timestamp_seconds
		_orientation_stable_time = 0.0
	else:
		_orientation_stable_time = timestamp_seconds - _last_orientation_time
		
		# 如果方向稳定超过阈值，发送信号
		if _orientation_stable_time >= orientation_stability_threshold:
			if _current_angle != avg_angle or _orientation_stable_time == orientation_stability_threshold:
				_current_angle = avg_angle
				orientation_changed.emit(_get_orientation_string(new_orientation))
				orientation_angle_changed.emit(avg_angle)


## 计算设备相对于重力的角度
func _calculate_device_angle(gravity: Vector3) -> float:
	# 假设设备的默认方向是Y轴向上
	# 计算重力向量与Y轴的夹角
	var up_vector = Vector3.UP
	var dot_product = gravity.dot(up_vector)
	var angle_rad = acos(clamp(dot_product, -1.0, 1.0))
	var angle_deg = rad_to_deg(angle_rad)
	
	# 根据X轴分量判断左右倾斜
	if gravity.x > 0:
		angle_deg = 360.0 - angle_deg
	
	return angle_deg


## 获取平均角度（考虑角度的循环性质）
func _get_average_angle() -> float:
	if _orientation_samples.is_empty():
		return 0.0
	
	var sin_sum = 0.0
	var cos_sum = 0.0
	
	for angle in _orientation_samples:
		sin_sum += sin(deg_to_rad(angle))
		cos_sum += cos(deg_to_rad(angle))
	
	var avg_angle_rad = atan2(sin_sum, cos_sum)
	var avg_angle_deg = rad_to_deg(avg_angle_rad)
	
	# 确保角度在0-360范围内
	if avg_angle_deg < 0:
		avg_angle_deg += 360.0
	
	return avg_angle_deg


## 将角度转换为方向字符串
func _angle_to_orientation(angle: float) -> ImuOrientation:
	# Portrait: 0-45度 或 315-360度
	# Landscape: 45-135度 或 225-315度
	# 中间区域需要考虑阈值避免频繁切换
	var normalized_angle = fmod(angle, 360.0)
	if normalized_angle < 0:
		normalized_angle += 360.0
	
	# Portrait 范围：315-45度（考虑0度交界）
	if (normalized_angle >= 315.0 or normalized_angle <= 45.0):
		return ImuOrientation.Portrait
	# Landscape 范围：45-135度 或 225-315度
	elif (normalized_angle >= 45.0 and normalized_angle <= 135.0) or (normalized_angle >= 225.0 and normalized_angle <= 315.0):
		return ImuOrientation.Landscape
	else:
		# 过渡区域，保持当前方向
		return _current_orientation
