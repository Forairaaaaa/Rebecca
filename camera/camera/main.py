import sys
import cv2
import numpy as np
import os
import argparse
from datetime import datetime
from PySide6.QtWidgets import (
    QApplication,
    QMainWindow,
    QLabel,
    QVBoxLayout,
    QWidget,
    QPushButton,
)
from PySide6.QtCore import QTimer, Qt, QPropertyAnimation, QEasingCurve
from PySide6.QtGui import QImage, QPixmap, QIcon
from PySide6.QtWidgets import QGraphicsOpacityEffect


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

        # 旋转角度状态 (0, 90, 180, 270)
        self.rotation_angle = 0

        # 缩放模式状态 (letterbox, fill)
        self.scale_mode = "fill"

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

        # 创建悬浮旋转按钮
        self.rotate_button = QPushButton("旋转 0°", central_widget)
        self.rotate_button.clicked.connect(self.rotate_frame)
        self.rotate_button.setFixedSize(80, 30)
        self.rotate_button.setStyleSheet("""
            QPushButton {
                background-color: rgba(0, 0, 0, 60);
                color: white;
                border-radius: 15px;
                font-size: 12px;
            }
            QPushButton:hover {
                background-color: rgba(0, 0, 0, 120);
            }
            QPushButton:pressed {
                background-color: rgba(0, 0, 0, 160);
            }
        """)

        # 设置按钮位置在左下角
        self.rotate_button.move(20, self.height() - 60)

        # 创建缩放模式切换按钮
        self.scale_button = QPushButton("Fill", central_widget)
        self.scale_button.clicked.connect(self.toggle_scale_mode)
        self.scale_button.setFixedSize(80, 30)
        self.scale_button.setStyleSheet("""
            QPushButton {
                background-color: rgba(0, 0, 0, 60);
                color: white;
                border-radius: 15px;
                font-size: 12px;
            }
            QPushButton:hover {
                background-color: rgba(0, 0, 0, 120);
            }
            QPushButton:pressed {
                background-color: rgba(0, 0, 0, 160);
            }
        """)

        # 设置缩放按钮位置在右下角
        self.scale_button.move(self.width() - 100, self.height() - 60)

        # 创建拍摄按钮
        self.capture_button = QPushButton("", central_widget)
        self.capture_button.clicked.connect(self.capture_photo)
        self.capture_button.setFixedSize(60, 60)
        self.capture_button.setStyleSheet("""
            QPushButton {
                background-color: rgba(255, 255, 255, 123);
                border-radius: 30px;
            }
            QPushButton:hover {
                background-color: rgba(255, 255, 255, 255);
            }
            QPushButton:pressed {
                background-color: rgba(255, 255, 255, 200);
            }
        """)

        # 初始按钮位置设置
        self.update_button_positions()

        # 初始化摄像头
        self.camera = cv2.VideoCapture(0)
        if not self.camera.isOpened():
            print("无法打开摄像头")
            return

        # 创建定时器用于更新画面
        self.timer = QTimer()
        self.timer.timeout.connect(self.update_frame)
        self.timer.start(30)  # 约33fps

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
        更新按钮位置以适应当前窗口大小
        """
        current_size = self.size()
        window_width = current_size.width()
        window_height = current_size.height()

        # 计算按钮中心对齐的Y坐标
        button_center_y = window_height - 60

        # 左下角：旋转按钮 (高度30px，所以Y坐标要减15px)
        self.rotate_button.move(20, button_center_y - 15)

        # 右下角：缩放模式按钮 (高度30px，所以Y坐标要减15px)
        self.scale_button.move(window_width - 100, button_center_y - 15)

        # 底部中央：拍摄按钮 (高度60px，所以Y坐标要减30px)
        self.capture_button.move((window_width - 60) // 2, button_center_y - 30)

    def rotate_image(self, image, angle):
        """
        根据角度旋转图像
        """
        if angle == 0:
            return image
        elif angle == 90:
            return cv2.rotate(image, cv2.ROTATE_90_CLOCKWISE)
        elif angle == 180:
            return cv2.rotate(image, cv2.ROTATE_180)
        elif angle == 270:
            return cv2.rotate(image, cv2.ROTATE_90_COUNTERCLOCKWISE)
        return image

    def rotate_frame(self):
        """
        切换旋转角度
        """
        self.rotation_angle = (self.rotation_angle + 90) % 360
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

    def capture_photo(self):
        """
        拍摄并保存当前画面
        """
        # 触发拍照特效
        self.trigger_capture_effect()

        ret, frame = self.camera.read()
        if not ret:
            print("无法捕获画面")
            return

        # 只应用旋转，不进行缩放
        processed_frame = self.rotate_image(frame, self.rotation_angle)

        # 创建保存目录
        if not os.path.exists("captures"):
            os.makedirs("captures")

        # 生成文件名（时间戳）
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"captures/photo_{timestamp}.jpg"

        # 保存图片
        success = cv2.imwrite(filename, processed_frame)
        if success:
            print(f"照片已保存: {filename}")
        else:
            print("保存照片失败")

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

    def update_frame(self):
        """
        更新摄像头画面
        """
        ret, frame = self.camera.read()
        if not ret:
            return

        # 应用旋转
        frame = self.rotate_image(frame, self.rotation_angle)

        # 获取当前窗口大小
        current_size = self.size()
        window_width = current_size.width()
        window_height = current_size.height()

        # 根据缩放模式调整图像
        if self.scale_mode == "letterbox":
            frame = self.letterbox_resize(frame, window_width, window_height)
        else:  # fill mode
            frame = self.fill_resize(frame, window_width, window_height)

        # 转换为Qt格式
        rgb_image = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        h, w, ch = rgb_image.shape
        bytes_per_line = ch * w

        qt_image = QImage(rgb_image.data, w, h, bytes_per_line, QImage.Format_RGB888)
        pixmap = QPixmap.fromImage(qt_image)

        # 显示图像
        self.image_label.setPixmap(pixmap)

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
        if hasattr(self, "camera"):
            self.camera.release()
        event.accept()


def main():
    # 解析命令行参数
    parser = argparse.ArgumentParser(description="摄像头实时显示程序")
    parser.add_argument("--full-screen", action="store_true", help="以全屏模式启动")
    args = parser.parse_args()

    app = QApplication(sys.argv)
    window = CameraWindow(fullscreen=args.full_screen)

    if not args.full_screen:
        window.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
