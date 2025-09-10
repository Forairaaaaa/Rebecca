extends CharacterBody3D

# No available yaw in 6dof
@export var is_6dof: bool = true


func _ready() -> void:
	print("k")


func _on_imu_imu_data_received(data: ImuData) -> void:
	print("update pose with:", data.quaternion)
	quaternion = data.quaternion.normalized()
