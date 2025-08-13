extends Label


func _on_rebecca_imu_imu_data_received(data: String) -> void:
	var json = JSON.new()
	var error = json.parse(data)
	if error == OK:
		var imu_data = json.data
		text = JSON.stringify(imu_data, "    ")
	else:
		print("JSON Parse Error: ", json.get_error_message(), " in ", data, " at line ", json.get_error_line())
