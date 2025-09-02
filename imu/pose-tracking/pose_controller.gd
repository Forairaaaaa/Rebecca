extends CharacterBody3D

# No available yaw in 6dof
@export var is_6dof: bool = true


func _ready() -> void:
	print("k")


func _update_pose(quat: Quaternion):
	print("update pose with:", quat)
	quaternion = quat.normalized()


func _on_imu_imu_data_received(data: String) -> void:
	# print(data)
	var json = JSON.new()
	var error = json.parse(data)
	if error == OK:
		var imu_data = json.data

		# Take out quaternion
		var quat = imu_data.quaternion as Array
		if typeof(quat) == TYPE_ARRAY and quat.size() == 4:
			_update_pose(Quaternion(quat[0], quat[1], quat[2], quat[3]))
		else:
			print("wtf?")
	else:
		print("JSON Parse Error: ", json.get_error_message(), " in ", data, " at line ", json.get_error_line())
