#!/usr/bin/env python3
"""
多屏幕视频循环播放程序
支持主屏幕(cvlc)和两个副屏(kava)的独立随机播放
"""

import os
import sys
import random
import subprocess
import threading
import time
import argparse
import signal
from pathlib import Path
from typing import List, Optional


class VideoPlayer:
    """视频播放器基类"""

    def __init__(self, name: str):
        self.name = name
        self.current_process: Optional[subprocess.Popen] = None
        self.is_running = False
        self.video_list: List[str] = []

    def load_videos(self, video_dir: str) -> None:
        """加载指定目录下的所有视频文件"""
        video_extensions = {
            ".mp4",
            ".avi",
            ".mkv",
            ".mov",
            ".wmv",
            ".flv",
            ".webm",
            ".m4v",
        }
        video_path = Path(video_dir)

        if not video_path.exists():
            print(f"错误：目录 {video_dir} 不存在")
            return

        self.video_list = []
        for file_path in video_path.rglob("*"):
            if file_path.is_file() and file_path.suffix.lower() in video_extensions:
                self.video_list.append(str(file_path))

        if not self.video_list:
            print(f"警告：在目录 {video_dir} 中未找到视频文件")
            return

        # 随机打乱播放列表
        random.shuffle(self.video_list)
        print(f"{self.name}: 加载了 {len(self.video_list)} 个视频文件")

    def play_video(self, video_path: str) -> None:
        """播放单个视频 - 子类需要实现"""
        raise NotImplementedError

    def stop(self) -> None:
        """停止当前播放"""
        if self.current_process:
            try:
                self.current_process.terminate()
                self.current_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.current_process.kill()
            except Exception as e:
                print(f"{self.name}: 停止播放时出错: {e}")
            finally:
                self.current_process = None
        self.is_running = False

    def run(self) -> None:
        """主播放循环"""
        if not self.video_list:
            print(f"{self.name}: 没有视频可播放")
            return

        self.is_running = True
        video_index = 0

        while self.is_running:
            if video_index >= len(self.video_list):
                # 重新打乱播放列表
                random.shuffle(self.video_list)
                video_index = 0
                print(f"{self.name}: 重新打乱播放列表")

            current_video = self.video_list[video_index]
            print(f"{self.name}: 正在播放 {os.path.basename(current_video)}")

            try:
                self.play_video(current_video)
                video_index += 1
            except Exception as e:
                print(f"{self.name}: 播放 {current_video} 时出错: {e}")
                video_index += 1
                time.sleep(1)


class MainScreenPlayer(VideoPlayer):
    """主屏幕播放器 - 使用 cvlc"""

    def __init__(self):
        super().__init__("主屏幕")

    def play_video(self, video_path: str) -> None:
        """使用 cvlc 在主屏幕全屏播放视频"""
        cmd = [
            "cvlc",
            "--intf",
            "dummy",
            "--no-video-title-show",
            "--fullscreen",
            "--no-osd",
            "--play-and-exit",
            video_path,
        ]

        self.current_process = subprocess.Popen(
            cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )

        # 等待播放结束
        self.current_process.wait()
        self.current_process = None


class CoverScreenPlayer(VideoPlayer):
    """副屏播放器 - 使用 kava"""

    def __init__(self, screen_name: str):
        super().__init__(f"副屏幕({screen_name})")
        self.screen_name = screen_name

    def play_video(self, video_path: str) -> None:
        """使用 kava 在副屏播放视频"""
        cmd = ["kava", self.screen_name, "--video", video_path]

        self.current_process = subprocess.Popen(
            cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )

        # 等待播放结束
        self.current_process.wait()
        self.current_process = None


class MultiScreenVideoPlayer:
    """多屏幕视频播放管理器"""

    def __init__(self, video_dir: str):
        self.video_dir = video_dir
        self.players = []
        self.threads = []
        self.running = True

        # 创建播放器实例
        self.main_player = MainScreenPlayer()
        self.sub_player1 = CoverScreenPlayer("screen0")
        self.sub_player2 = CoverScreenPlayer("screen1")

        self.players = [self.main_player, self.sub_player1, self.sub_player2]

        # 为每个播放器加载视频列表
        for player in self.players:
            player.load_videos(video_dir)

    def start(self) -> None:
        """启动所有屏幕的播放"""
        print("启动多屏幕视频播放...")

        # 为每个播放器创建独立线程
        for player in self.players:
            if player.video_list:  # 只有有视频的播放器才启动
                thread = threading.Thread(target=player.run, daemon=True)
                thread.start()
                self.threads.append(thread)

        if not self.threads:
            print("错误：没有找到可播放的视频文件")
            return

        print("所有屏幕播放已启动，按 Ctrl+C 退出...")

        try:
            # 主线程等待
            while self.running:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\n正在停止播放...")
            self.stop()

    def stop(self) -> None:
        """停止所有播放器"""
        self.running = False

        for player in self.players:
            player.stop()

        # 等待所有线程结束
        for thread in self.threads:
            if thread.is_alive():
                thread.join(timeout=2)

        print("所有播放器已停止")


def signal_handler(signum, frame):
    """信号处理器"""
    print("\n收到退出信号，正在停止...")
    sys.exit(0)


def main():
    """主函数"""
    parser = argparse.ArgumentParser(
        description="多屏幕视频循环播放程序",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
使用示例:
  python main.py /path/to/videos    # 播放指定目录下的所有视频
  python main.py ~/Movies           # 播放家目录Movies文件夹的视频
        """,
    )

    parser.add_argument("video_dir", help="包含视频文件的目录路径")

    args = parser.parse_args()

    # 检查目录是否存在
    if not os.path.exists(args.video_dir):
        print(f"错误：目录 '{args.video_dir}' 不存在")
        sys.exit(1)

    if not os.path.isdir(args.video_dir):
        print(f"错误：'{args.video_dir}' 不是一个目录")
        sys.exit(1)

    # 注册信号处理器
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # 创建并启动多屏播放器
    player = MultiScreenVideoPlayer(args.video_dir)
    player.start()


if __name__ == "__main__":
    main()
