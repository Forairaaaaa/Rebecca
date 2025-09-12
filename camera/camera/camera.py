import cv2
import numpy as np
import os
from datetime import datetime
from PySide6.QtCore import QTimer, QObject, Signal


class Camera(QObject):
    """
    摄像头管理类，负责摄像头初始化、画面捕获、拍照和录像功能
    """

    # 信号定义
    frame_ready = Signal(np.ndarray)  # 新帧准备就绪
    photo_saved = Signal(str)  # 照片保存完成
    video_started = Signal(str)  # 录像开始
    video_stopped = Signal(str)  # 录像停止
    error_occurred = Signal(str)  # 错误发生

    def __init__(self):
        super().__init__()
        self.camera = None
        self.video_writer = None
        self.current_video_filename = ""
        self.rotation_angle = 0
        self.is_recording = False
        self.recording_start_time = None

        # 初始化摄像头
        self._init_camera()

        # 创建定时器用于更新画面
        self.timer = QTimer()
        self.timer.timeout.connect(self._capture_frame)
        self.timer.start(30)  # 约33fps

    def _init_camera(self):
        """初始化摄像头，尝试不同的后端"""
        backends_to_try = [
            cv2.CAP_DSHOW,  # DirectShow (Windows推荐)
            cv2.CAP_MSMF,  # Media Foundation
            cv2.CAP_ANY,  # 自动选择
        ]

        self.camera = None
        for backend in backends_to_try:
            try:
                camera = cv2.VideoCapture(0, backend)
                if camera.isOpened():
                    self.camera = camera
                    print(f"摄像头初始化成功，使用后端: {backend}")
                    break
                else:
                    camera.release()
            except Exception as e:
                print(f"后端 {backend} 初始化失败: {e}")
                continue

        if not self.camera or not self.camera.isOpened():
            self.error_occurred.emit("无法打开摄像头")
            return

        # 设置摄像头参数以提高稳定性
        self.camera.set(cv2.CAP_PROP_BUFFERSIZE, 1)  # 减少缓冲区
        self.camera.set(cv2.CAP_PROP_FPS, 30)  # 设置帧率

    def _capture_frame(self):
        """捕获并处理摄像头帧"""
        if not self.camera or not self.camera.isOpened():
            return

        ret, frame = self.camera.read()
        if not ret:
            return

        # 应用旋转
        rotated_frame = self._rotate_image(frame, self.rotation_angle)

        # 如果正在录像，将帧写入视频文件
        if self.is_recording and self.video_writer:
            self.video_writer.write(rotated_frame)

        # 发送帧信号给UI
        self.frame_ready.emit(rotated_frame)

    def _rotate_image(self, image, angle):
        """根据角度旋转图像"""
        if angle == 0:
            return image
        elif angle == 90:
            return cv2.rotate(image, cv2.ROTATE_90_CLOCKWISE)
        elif angle == 180:
            return cv2.rotate(image, cv2.ROTATE_180)
        elif angle == 270:
            return cv2.rotate(image, cv2.ROTATE_90_COUNTERCLOCKWISE)
        return image

    def set_rotation(self, angle):
        """设置旋转角度"""
        self.rotation_angle = angle

    def capture_photo(self):
        """拍摄并保存当前画面"""
        if not self.camera or not self.camera.isOpened():
            self.error_occurred.emit("摄像头未就绪")
            return

        ret, frame = self.camera.read()
        if not ret:
            self.error_occurred.emit("无法捕获画面")
            return

        # 只应用旋转，不进行缩放
        processed_frame = self._rotate_image(frame, self.rotation_angle)

        # 创建保存目录
        if not os.path.exists("captures"):
            os.makedirs("captures")

        # 生成文件名（时间戳）
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"captures/photo_{timestamp}.jpg"

        # 保存图片
        success = cv2.imwrite(filename, processed_frame)
        if success:
            self.photo_saved.emit(filename)
        else:
            self.error_occurred.emit("保存照片失败")

    def start_recording(self):
        """开始录像"""
        if self.is_recording:
            return

        # 创建保存目录
        if not os.path.exists("captures"):
            os.makedirs("captures")

        # 生成视频文件名
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # 获取摄像头参数
        ret, frame = self.camera.read()
        if not ret:
            self.error_occurred.emit("无法获取摄像头画面")
            return

        # 应用旋转
        frame = self._rotate_image(frame, self.rotation_angle)
        height, width = frame.shape[:2]

        # 尝试多种编码器，提高兼容性
        codecs_to_try = [
            ("XVID", "avi"),
            ("MJPG", "avi"),
            ("mp4v", "mp4"),
            ("H264", "mp4"),
        ]

        self.video_writer = None
        for codec, ext in codecs_to_try:
            try:
                fourcc = cv2.VideoWriter_fourcc(*codec)
                filename = f"captures/video_{timestamp}.{ext}"
                writer = cv2.VideoWriter(filename, fourcc, 20.0, (width, height))

                if writer.isOpened():
                    self.video_writer = writer
                    self.current_video_filename = filename
                    print(f"使用编码器: {codec}")
                    break
                else:
                    writer.release()
            except Exception as e:
                print(f"编码器 {codec} 失败: {e}")
                continue

        if not self.video_writer or not self.video_writer.isOpened():
            self.error_occurred.emit("无法初始化任何视频编码器")
            return

        # 设置录像状态
        self.is_recording = True
        self.recording_start_time = datetime.now()

        self.video_started.emit(self.current_video_filename)

    def stop_recording(self):
        """停止录像"""
        if not self.is_recording:
            return

        # 停止录像状态
        self.is_recording = False

        # 释放视频编码器
        if self.video_writer:
            self.video_writer.release()
            self.video_writer = None

        filename = self.current_video_filename
        self.current_video_filename = ""

        self.video_stopped.emit(filename)

    def get_recording_duration(self):
        """获取录制时长"""
        if not self.is_recording or not self.recording_start_time:
            return 0

        elapsed = datetime.now() - self.recording_start_time
        return int(elapsed.total_seconds())

    def release(self):
        """释放摄像头资源"""
        if self.is_recording:
            self.stop_recording()

        if self.timer.isActive():
            self.timer.stop()

        if self.camera:
            self.camera.release()
