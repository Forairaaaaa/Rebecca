extends Node2D


func _ready():
	$Imu.imu_data_received.connect(_on_imu_data_received)


func _on_imu_data_received(data: ImuData) -> void:
	$LabelData.text = JSON.stringify(data.to_dict(), "    ")


func _on_button_start_pressed() -> void:
	$Imu.start_reading()


func _on_button_stop_pressed() -> void:
	$Imu.stop_reading()
