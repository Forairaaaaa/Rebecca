extends Node2D


func _on_imu_imu_data_received(data: String) -> void:
	var json = JSON.new()
	var error = json.parse(data)
	if error == OK:
		var imu_data = json.data
		$LabelData.text = JSON.stringify(imu_data, "    ")
	else:
		print("JSON Parse Error: ", json.get_error_message(), " in ", data, " at line ", json.get_error_line())


func _on_button_start_pressed() -> void:
	$Imu.StartReading()


func _on_button_stop_pressed() -> void:
	$Imu.StopReading()
