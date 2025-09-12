import cv2
import numpy as np
import os
from PySide6.QtWidgets import (
    QMainWindow,
    QLabel,
    QVBoxLayout,
    QWidget,
    QPushButton,
)
from PySide6.QtCore import QTimer, Qt, QPropertyAnimation, QEasingCurve
from PySide6.QtGui import QImage, QPixmap, QIcon, QFont
from PySide6.QtWidgets import QGraphicsOpacityEffect
from camera import Camera


class CameraWindow(QMainWindow):
    def __init__(self, fullscreen=False):
        super().__init__()
        self.setWindowTitle("CAMERA")
        self.setMinimumSize(260, 123)

        # 设置窗口图标
        if os.path.exists("icon.svg"):
            icon = QIcon("icon.svg")
            self.setWindowIcon(icon)

        if fullscreen:
            self.showFullScreen()
        else:
            self.resize(320, 480)

        # 初始化摄像头
        self.camera = Camera()

        # UI状态
        self.rotation_angle = 0
        self.scale_mode = "fill"
        self.is_recording = False

        # 长按检测
        self.long_press_timer = QTimer()
        self.long_press_timer.setSingleShot(True)
        self.long_press_timer.timeout.connect(self._on_long_press)
        self.is_long_press = False

        # 录制时长更新定时器
        self.recording_timer = QTimer()
        self.recording_timer.timeout.connect(self._update_recording_time)

        # 创建中心部件和布局
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        layout = QVBoxLayout(central_widget)
        layout.setContentsMargins(5, 5, 5, 5)

        # 创建显示标签
        self.image_label = QLabel()
        self.image_label.setAlignment(Qt.AlignCenter)
        self.image_label.setStyleSheet("background-color: black;")
        layout.addWidget(self.image_label)

        # 创建画面透明度效果用于闪光
        self.image_opacity_effect = QGraphicsOpacityEffect()
        self.image_opacity_effect.setOpacity(1.0)  # 确保初始透明度为1.0
        self.image_label.setGraphicsEffect(self.image_opacity_effect)

        # 创建闪光动画（画面变黑再恢复）
        self.flash_animation = QPropertyAnimation(self.image_opacity_effect, b"opacity")
        self.flash_animation.setDuration(200)
        self.flash_animation.setKeyValueAt(0.0, 1.0)  # 开始：正常显示
        self.flash_animation.setKeyValueAt(0.3, 0.0)  # 30%时：完全变黑
        self.flash_animation.setKeyValueAt(1.0, 1.0)  # 结束：恢复正常
        self.flash_animation.setEasingCurve(QEasingCurve.InOutQuad)

        # 创建录制时长显示器（iOS风格）
        self.recording_time_label = QLabel("", central_widget)
        self.recording_time_label.setAlignment(Qt.AlignCenter)
        self.recording_time_label.setStyleSheet("""
            QLabel {
                background-color: rgba(255, 59, 48, 230);
                color: white;
                border-radius: 12px;
                font-size: 14px;
                font-weight: bold;
                padding: 6px 12px;
            }
        """)
        self.recording_time_label.setFont(QFont("Arial", 14, QFont.Bold))
        self.recording_time_label.hide()  # 初始隐藏

        # 创建悬浮旋转按钮
        self.rotate_button = QPushButton("旋转 0°", central_widget)
        self.rotate_button.clicked.connect(self.rotate_frame)

        # 设置按钮位置在左下角
        self.rotate_button.move(20, self.height() - 60)

        # 创建缩放模式切换按钮
        self.scale_button = QPushButton("Fill", central_widget)
        self.scale_button.clicked.connect(self.toggle_scale_mode)

        # 设置缩放按钮位置在右下角
        self.scale_button.move(self.width() - 100, self.height() - 60)

        # 创建拍摄按钮
        self.capture_button = QPushButton("", central_widget)

        # 重写按钮事件处理
        self.capture_button.mousePressEvent = self.capture_button_press
        self.capture_button.mouseReleaseEvent = self.capture_button_release

        # 初始按钮位置设置
        self.update_button_positions()

        # 连接摄像头信号
        self._connect_camera_signals()

    def _connect_camera_signals(self):
        """连接摄像头信号到UI槽函数"""
        self.camera.frame_ready.connect(self._on_frame_ready)
        self.camera.photo_saved.connect(self._on_photo_saved)
        self.camera.video_started.connect(self._on_video_started)
        self.camera.video_stopped.connect(self._on_video_stopped)
        self.camera.error_occurred.connect(self._on_camera_error)

    def _on_frame_ready(self, frame):
        """处理新的摄像头帧"""
        # 获取当前窗口大小
        current_size = self.size()
        window_width = current_size.width()
        window_height = current_size.height()

        # 根据缩放模式调整图像（仅用于显示）
        display_frame = frame.copy()
        if self.scale_mode == "letterbox":
            display_frame = self.letterbox_resize(
                display_frame, window_width, window_height
            )
        else:  # fill mode
            display_frame = self.fill_resize(display_frame, window_width, window_height)

        # 转换为Qt格式
        rgb_image = cv2.cvtColor(display_frame, cv2.COLOR_BGR2RGB)
        h, w, ch = rgb_image.shape
        bytes_per_line = ch * w

        qt_image = QImage(rgb_image.data, w, h, bytes_per_line, QImage.Format_RGB888)
        pixmap = QPixmap.fromImage(qt_image)

        # 显示图像
        self.image_label.setPixmap(pixmap)

    def _on_photo_saved(self, filename):
        """照片保存完成"""
        print(f"照片已保存: {filename}")

    def _on_video_started(self, filename):
        """录像开始"""
        self.is_recording = True

        # 显示录制时长标签
        self.recording_time_label.show()
        self.update_button_positions()

        # 开始录制时长更新定时器
        self.recording_timer.start(100)

        # 更新按钮样式为录像模式 - 通过重新调用位置更新来应用样式
        self.update_button_positions()

        print(f"开始录像: {filename}")

    def _on_video_stopped(self, filename):
        """录像停止"""
        self.is_recording = False
        self.recording_timer.stop()

        # 隐藏录制时长标签
        self.recording_time_label.hide()

        # 恢复按钮样式为拍照模式 - 通过重新调用位置更新来应用样式
        self.update_button_positions()

        print(f"录像已停止: {filename}")

    def _on_camera_error(self, error_message):
        """处理摄像头错误"""
        print(f"摄像头错误: {error_message}")

    def letterbox_resize(self, image, target_width, target_height):
        """
        使用letterbox方式调整图像大小，保持宽高比
        """
        h, w = image.shape[:2]

        # 计算缩放比例
        scale = min(target_width / w, target_height / h)

        # 计算新的尺寸
        new_w = int(w * scale)
        new_h = int(h * scale)

        # 缩放图像
        resized = cv2.resize(image, (new_w, new_h), interpolation=cv2.INTER_AREA)

        # 创建目标大小的黑色画布
        result = np.zeros((target_height, target_width, 3), dtype=np.uint8)

        # 计算居中位置
        x_offset = (target_width - new_w) // 2
        y_offset = (target_height - new_h) // 2

        # 将缩放后的图像放到画布中心
        result[y_offset : y_offset + new_h, x_offset : x_offset + new_w] = resized

        return result

    def fill_resize(self, image, target_width, target_height):
        """
        使用fill方式调整图像大小，填满整个窗口（可能会裁剪）
        """
        h, w = image.shape[:2]

        # 计算缩放比例，选择较大的比例以填满窗口
        scale = max(target_width / w, target_height / h)

        # 计算新的尺寸
        new_w = int(w * scale)
        new_h = int(h * scale)

        # 缩放图像
        resized = cv2.resize(image, (new_w, new_h), interpolation=cv2.INTER_AREA)

        # 计算裁剪位置（居中裁剪）
        x_offset = (new_w - target_width) // 2
        y_offset = (new_h - target_height) // 2

        # 裁剪到目标尺寸
        if x_offset >= 0 and y_offset >= 0:
            result = resized[
                y_offset : y_offset + target_height, x_offset : x_offset + target_width
            ]
        else:
            # 如果缩放后的图像小于目标尺寸，创建黑色背景
            result = np.zeros((target_height, target_width, 3), dtype=np.uint8)
            start_x = max(0, -x_offset)
            start_y = max(0, -y_offset)
            end_x = min(target_width, new_w + start_x)
            end_y = min(target_height, new_h + start_y)
            result[start_y:end_y, start_x:end_x] = resized[
                max(0, y_offset) : max(0, y_offset) + (end_y - start_y),
                max(0, x_offset) : max(0, x_offset) + (end_x - start_x),
            ]

        return result

    def update_button_positions(self):
        """
        更新按钮位置和大小以适应当前窗口大小
        """
        current_size = self.size()
        window_width = current_size.width()
        window_height = current_size.height()

        # 计算缩放因子，基于窗口的最小尺寸
        # 基准尺寸设为320x480（原始设计尺寸）
        base_width = 320
        base_height = 480
        scale_factor = min(window_width / base_width, window_height / base_height)

        # 限制缩放因子的范围，避免按钮过小或过大
        scale_factor = max(0.8, min(scale_factor, 3.0))

        # 计算按钮尺寸
        small_button_width = int(80 * scale_factor)
        small_button_height = int(30 * scale_factor)
        capture_button_size = int(60 * scale_factor)

        # 更新按钮尺寸
        self.rotate_button.setFixedSize(small_button_width, small_button_height)
        self.scale_button.setFixedSize(small_button_width, small_button_height)
        self.capture_button.setFixedSize(capture_button_size, capture_button_size)

        # 更新按钮圆角半径以保持比例
        small_button_radius = int(15 * scale_factor)
        capture_button_radius = int(30 * scale_factor)

        # 更新按钮样式
        self._update_button_styles(
            small_button_radius, capture_button_radius, scale_factor
        )

        # 计算按钮中心对齐的Y坐标，根据最大按钮尺寸动态调整底部边距
        # 确保最大的按钮（拍摄按钮）底部不会越界
        bottom_margin = max(60, capture_button_size // 2 + 50)  # 至少20px边距
        button_center_y = window_height - bottom_margin

        # 左下角：旋转按钮
        self.rotate_button.move(20, button_center_y - small_button_height // 2)

        # 右下角：缩放模式按钮
        self.scale_button.move(
            window_width - small_button_width - 20,
            button_center_y - small_button_height // 2,
        )

        # 底部中央：拍摄按钮
        self.capture_button.move(
            (window_width - capture_button_size) // 2,
            button_center_y - capture_button_size // 2,
        )

        # 顶部中央：录制时长显示器
        if hasattr(self, "recording_time_label"):
            self.recording_time_label.adjustSize()
            label_width = self.recording_time_label.width()
            self.recording_time_label.move((window_width - label_width) // 2, 20)

    def _update_button_styles(
        self, small_button_radius, capture_button_radius, scale_factor
    ):
        """
        更新按钮样式以适应缩放
        """
        # 计算字体大小
        small_font_size = int(12 * scale_factor)

        # 更新旋转按钮样式
        self.rotate_button.setStyleSheet(f"""
            QPushButton {{
                background-color: rgba(0, 0, 0, 60);
                color: white;
                border-radius: {small_button_radius}px;
                font-size: {small_font_size}px;
            }}
            QPushButton:hover {{
                background-color: rgba(0, 0, 0, 120);
            }}
            QPushButton:pressed {{
                background-color: rgba(0, 0, 0, 160);
            }}
        """)

        # 更新缩放按钮样式
        self.scale_button.setStyleSheet(f"""
            QPushButton {{
                background-color: rgba(0, 0, 0, 60);
                color: white;
                border-radius: {small_button_radius}px;
                font-size: {small_font_size}px;
            }}
            QPushButton:hover {{
                background-color: rgba(0, 0, 0, 120);
            }}
            QPushButton:pressed {{
                background-color: rgba(0, 0, 0, 160);
            }}
        """)

        # 更新拍摄按钮样式（根据当前录制状态）
        if self.is_recording:
            self.capture_button.setStyleSheet(f"""
                QPushButton {{
                    background-color: rgba(255, 59, 48, 233);
                    border-radius: {capture_button_radius}px;
                }}
                QPushButton:hover {{
                    background-color: rgba(255, 59, 48, 168);
                }}
                QPushButton:pressed {{
                    background-color: rgba(255, 59, 48, 123);
                }}
            """)
        else:
            self.capture_button.setStyleSheet(f"""
                QPushButton {{
                    background-color: rgba(255, 255, 255, 233);
                    border-radius: {capture_button_radius}px;
                }}
                QPushButton:hover {{
                    background-color: rgba(255, 255, 255, 168);
                }}
                QPushButton:pressed {{
                    background-color: rgba(255, 255, 255, 123);
                }}
            """)

    def rotate_frame(self):
        """
        切换旋转角度
        """
        self.rotation_angle = (self.rotation_angle + 90) % 360
        # 更新摄像头旋转角度
        self.camera.set_rotation(self.rotation_angle)
        # 更新按钮文字显示当前角度
        self.rotate_button.setText(f"旋转 {self.rotation_angle}°")
        print(f"当前旋转角度: {self.rotation_angle}°")

    def toggle_scale_mode(self):
        """
        切换缩放模式
        """
        if self.scale_mode == "letterbox":
            self.scale_mode = "fill"
            self.scale_button.setText("Fill")
        else:
            self.scale_mode = "letterbox"
            self.scale_button.setText("Letterbox")
        print(f"当前缩放模式: {self.scale_mode}")

    def capture_button_press(self, event):
        """
        拍摄按钮按下事件
        """
        if not self.is_recording:
            # 重置长按状态
            self.is_long_press = False
            # 开始长按计时器
            self.long_press_timer.start(500)
        else:
            # 如果正在录像，点击停止录像
            self.camera.stop_recording()

    def capture_button_release(self, event):
        """
        拍摄按钮释放事件
        """
        if self.long_press_timer.isActive():
            # 如果长按计时器还在运行，说明是短按，执行拍照
            self.long_press_timer.stop()
            if not self.is_long_press:  # 确保不是长按触发的录像
                self._capture_photo()

    def _on_long_press(self):
        """
        长按触发事件
        """
        self.is_long_press = True
        self.camera.start_recording()

    def _capture_photo(self):
        """
        拍摄照片
        """
        # 触发拍照特效
        self.trigger_capture_effect()
        # 调用摄像头拍照
        self.camera.capture_photo()

    def _update_recording_time(self):
        """
        更新录制时长显示
        """
        if not self.is_recording:
            return

        # 获取录制时长
        total_seconds = self.camera.get_recording_duration()
        minutes = total_seconds // 60
        seconds = total_seconds % 60

        # 更新显示文本
        time_text = f"{minutes:02d}:{seconds:02d}"
        self.recording_time_label.setText(time_text)
        self.recording_time_label.adjustSize()

        # 重新定位标签到顶部中央
        window_width = self.width()
        label_width = self.recording_time_label.width()
        self.recording_time_label.move((window_width - label_width) // 2, 20)

    def trigger_capture_effect(self):
        """
        触发iOS风格的拍摄效果
        """
        # 确保动画停止并重置透明度
        if self.flash_animation.state() == QPropertyAnimation.Running:
            self.flash_animation.stop()

        # 重置透明度为1.0，确保动画从正确状态开始
        self.image_opacity_effect.setOpacity(1.0)

        # 开始画面闪烁动画
        self.flash_animation.start()

    def resizeEvent(self, event):
        """
        窗口大小改变时更新按钮位置
        """
        super().resizeEvent(event)
        if hasattr(self, "rotate_button"):  # 确保按钮已经创建
            self.update_button_positions()

    def closeEvent(self, event):
        """
        窗口关闭时释放摄像头资源
        """
        # 释放摄像头资源
        if hasattr(self, "camera"):
            self.camera.release()
        event.accept()
