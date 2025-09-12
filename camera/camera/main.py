import sys
import signal
import argparse
from PySide6.QtWidgets import QApplication
from PySide6.QtCore import QTimer
from view import CameraWindow


def signal_handler(signum, frame):
    """信号处理函数，用于处理Ctrl+C"""
    print("\n收到中断信号，正在关闭程序...")
    QApplication.quit()


def main():
    # 解析命令行参数
    parser = argparse.ArgumentParser(description="Camera")
    parser.add_argument("--full-screen", action="store_true", help="以全屏模式启动")
    args = parser.parse_args()

    app = QApplication(sys.argv)

    # 设置信号处理器来响应Ctrl+C
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # 创建一个定时器来让Qt事件循环定期检查Python信号
    # 这是让Qt应用程序响应Ctrl+C的关键
    timer = QTimer()
    timer.start(500)  # 每500ms检查一次
    timer.timeout.connect(lambda: None)  # 空操作，只是为了让事件循环运行

    window = CameraWindow(fullscreen=args.full_screen)

    if not args.full_screen:
        window.show()

    try:
        sys.exit(app.exec())
    except KeyboardInterrupt:
        print("\n程序被用户中断")
        sys.exit(0)


if __name__ == "__main__":
    main()
