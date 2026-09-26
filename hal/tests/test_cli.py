"""Run after building hal/service and hal/cli-tool/rebecca-hal (debug profile)."""
import json
import math
from pathlib import Path
import signal
import selectors
import socket
import subprocess
import time
import unittest
import urllib.request


class CliTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        hal = Path(__file__).resolve().parents[1]
        cls.cli = hal / "cli-tool/rebecca-hal/target/debug/rebecca-hal"
        service = hal / "service/target/debug/rebecca-hal-service"
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            cls.port = sock.getsockname()[1]
        cls.service = subprocess.Popen(
            [str(service), "--port", str(cls.port), "--mock-imu", "--mock-backlight"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        cls.addClassCleanup(cls.stop_service)
        for _ in range(100):
            if cls.service.poll() is not None:
                raise RuntimeError("HAL service exited before becoming ready")
            try:
                with urllib.request.urlopen(f"http://127.0.0.1:{cls.port}/devices", timeout=1) as response:
                    if {"imu0", "backlight0"}.issubset(json.load(response)):
                        return
            except OSError:
                pass
            time.sleep(0.05)
        raise RuntimeError("HAL service did not become ready")

    @classmethod
    def stop_service(cls):
        cls.service.terminate()
        try:
            cls.service.wait(timeout=5)
        except subprocess.TimeoutExpired:
            cls.service.kill()
            cls.service.wait()

    def command(self, *args):
        return [str(self.cli), "--port", str(self.port), *args]

    def test_imu_wire_data_decodes(self):
        subprocess.run(self.command("imu", "imu0", "start"), check=True,
                       capture_output=True, timeout=5)
        reader = subprocess.Popen(self.command("imu", "imu0", "read"),
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(reader.stdout, selectors.EVENT_READ)
                self.assertTrue(selector.select(timeout=10), "No IMU data received")
                first_sample = reader.stdout.readline()
            reader.send_signal(signal.SIGINT)
            output, error = reader.communicate(timeout=5)
            output = first_sample + output
        finally:
            if reader.poll() is None:
                reader.kill()
                reader.communicate()
            subprocess.run(self.command("imu", "imu0", "stop"), check=True,
                           capture_output=True, timeout=5)
        self.assertEqual(reader.returncode, 0, error)
        samples = [json.loads(line) for line in output.splitlines()]
        self.assertGreater(len(samples), 0, error)
        sample = samples[0]
        self.assertGreater(sample["timestamp"], 0)
        for field, length in (("accel", 3), ("gyro", 3), ("mag", 3),
                              ("quaternion", 4), ("euler_angles", 3)):
            self.assertEqual(len(sample[field]), length, field)
            self.assertTrue(all(math.isfinite(value) for value in sample[field]), field)
        self.assertTrue(math.isfinite(sample["temp"]))

    def test_brightness_command(self):
        result = subprocess.run(self.command("backlight"), check=True,
                                capture_output=True, text=True, timeout=5)
        self.assertIn("backlight0", json.loads(result.stdout))
        subprocess.run(self.command("backlight", "backlight0", "set", "0.5"),
                       check=True, capture_output=True, timeout=5)
        result = subprocess.run(self.command("backlight", "backlight0", "get"),
                                check=True, capture_output=True, text=True, timeout=5)
        self.assertAlmostEqual(json.loads(result.stdout)["brightness"], 0.5, places=2)


if __name__ == "__main__":
    unittest.main()
