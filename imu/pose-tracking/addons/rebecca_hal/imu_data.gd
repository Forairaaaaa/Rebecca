class_name ImuData
extends Resource


@export var accel: Vector3
@export var gyro: Vector3
@export var mag: Vector3
@export var quaternion: Quaternion
@export var euler_angles: Vector3
@export var temp: float
@export var timestamp: int


func _init():
	pass


func _array_to_vector3(arr: Array) -> Vector3:
	if arr and arr.size() >= 3:
		return Vector3(arr[0], arr[1], arr[2])
	return Vector3.ZERO


func _array_to_quaternion(arr: Array) -> Quaternion:
	if arr and arr.size() >= 4:
		return Quaternion(arr[0], arr[1], arr[2], arr[3])
	return Quaternion.IDENTITY


func init_from_json(data: String):
	# Parse json
	var json = JSON.new()
	var error = json.parse(data)
	if error != OK:
		print("[ImuData] JSON Parse Error: ", json.get_error_message(), " in ", data, " at line ", json.get_error_line())
		return
	
	var imu_data = json.data
	accel = _array_to_vector3(imu_data.get("accel"))
	gyro = _array_to_vector3(imu_data.get("gyro"))
	mag = _array_to_vector3(imu_data.get("mag"))
	quaternion = _array_to_quaternion(imu_data.get("quaternion"))
	euler_angles = _array_to_vector3(imu_data.get("euler_angles"))
	temp = imu_data.get("temp", 0.0)
	timestamp = imu_data.get("timestamp", 0)

		
## 获取加速度模长
func get_accel_magnitude() -> float:
	return accel.length()


## 获取陀螺仪模长
func get_gyro_magnitude() -> float:
	return gyro.length()


## 获取磁力计模长
func get_mag_magnitude() -> float:
	return mag.length()


## 判断设备是否静止（基于加速度和陀螺仪阈值）
func is_stationary(accel_threshold: float = 0.1, gyro_threshold: float = 0.05) -> bool:
	return get_accel_magnitude() < accel_threshold and get_gyro_magnitude() < gyro_threshold


## 获取重力方向（归一化的加速度向量）
func get_gravity_direction() -> Vector3:
	return accel.normalized()


## 转换为Dictionary，便于调试和序列化
func to_dict() -> Dictionary:
	return {
		"timestamp": timestamp,
		"accel": [accel.x, accel.y, accel.z],
		"gyro": [gyro.x, gyro.y, gyro.z],
		"mag": [mag.x, mag.y, mag.z],
		"temp": temp,
		"quaternion": [quaternion.x, quaternion.y, quaternion.z, quaternion.w],
		"euler_angles": [euler_angles.x, euler_angles.y, euler_angles.z]
	}


## 获取格式化的字符串表示
func to_string() -> String:
	return "ImuData(timestamp=%d, accel=(%f,%f,%f), gyro=(%f,%f,%f), temp=%f)" % [
		timestamp, accel.x, accel.y, accel.z, gyro.x, gyro.y, gyro.z, temp
	]
