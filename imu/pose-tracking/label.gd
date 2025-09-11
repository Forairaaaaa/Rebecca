extends Label


func _on_imu_imu_data_received(data: ImuData) -> void:
	text = JSON.stringify(data.to_dict(), "    ")
