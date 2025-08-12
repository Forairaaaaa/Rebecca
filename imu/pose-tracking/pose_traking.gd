extends CharacterBody3D

func _ready() -> void:
    print("???")
    var output = ``
    OS.execute("rebecca-imu", ["-h"], output, true)
    print(output)
