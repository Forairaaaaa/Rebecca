extends Node2D


func _ready():
	$Imu.imu_data_received.connect(_on_imu_data_received)
	$Imu.orientation_changed.connect(_on_imu_orientation_changed)
	$Imu.orientation_angle_changed.connect(_on_imu_orientation_angle_changed)

	$LabelOrientation.text = $Imu.get_orientation()
	$LabelOrientationAngle.text = str($Imu.get_orientation_angle()) + "°"


func _on_imu_data_received(data: ImuData) -> void:
	$LabelData.text = JSON.stringify(data.to_dict(), "    ")


func _on_imu_orientation_changed(orientation: String) -> void:
	$LabelOrientation.text = orientation


func _on_imu_orientation_angle_changed(angle: float) -> void:
	$LabelOrientationAngle.text = str(angle) + "°"


func _on_button_start_pressed() -> void:
	$Imu.start_reading()


func _on_button_stop_pressed() -> void:
	$Imu.stop_reading()
