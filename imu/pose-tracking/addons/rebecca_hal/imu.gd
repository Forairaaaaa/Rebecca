## Rebecca-HAL IMU
class_name Imu
extends Node


signal imu_data_received(data: ImuData)

@export var host: String = "localhost"
@export var port: int = 12580
@export var device_id: String = "imu0"
@export var cli_tool_name: String = "rebecca-hal"

var _cs_imu: Node

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


func start_reading() -> bool:
	if _cs_imu:
		return _cs_imu.StartReading()
	return false


func stop_reading() -> bool:
	if _cs_imu:
		return _cs_imu.StopReading()
	return false
