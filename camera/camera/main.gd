extends Node2D

func _ready():
	var feed_count = CameraServer.get_feed_count()
	if feed_count == 0:
		printerr("No camera found")
		return

	var feed = CameraServer.get_feed(0)
	feed.set_active(true)
