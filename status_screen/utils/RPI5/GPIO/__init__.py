# RPI.GPIO for raspberry pi 5
# https://gpiozero.readthedocs.io/en/latest/migrating_from_rpigpio.html


from gpiozero.pins.lgpio import LGPIOFactory
from gpiozero import DigitalOutputDevice, DigitalInputDevice, PWMOutputDevice


class GPIO:
    LOW = False
    HIGH = True
    IN = 0
    OUT = 1

    _factory = LGPIOFactory(chip=0)
    _pins = {}

    @staticmethod
    def setup(pin, mode, **kwargs):
        if pin is None:
            return
        if pin in GPIO._pins:
            return
        if mode == GPIO.OUT:
            initial = kwargs.get("initial", GPIO.LOW)
            GPIO._pins[pin] = DigitalOutputDevice(
                pin, pin_factory=GPIO._factory, initial_value=initial)
        elif mode == GPIO.IN:
            pull_up = kwargs.get("pull_up", False)
            GPIO._pins[pin] = DigitalInputDevice(
                pin, pin_factory=GPIO._factory, pull_up=pull_up)

    @staticmethod
    def output(pin, value):
        dev = GPIO._pins.get(pin)
        if isinstance(dev, DigitalOutputDevice):
            dev.on() if value else dev.off()

    @staticmethod
    def input(pin):
        dev = GPIO._pins.get(pin)
        if dev is None:
            return None
        return bool(dev.value)

    @staticmethod
    def cleanup(*args, **kwargs):
        pass

    class PWM:
        def __init__(self, pin, frequency):
            self.pin = pin
            self.frequency = frequency
            self._dev: PWMOutputDevice | None = None
            self._running = False

        def start(self, duty_cycle):
            if self._running:
                return
            self._dev = PWMOutputDevice(self.pin, pin_factory=GPIO._factory,
                                        frequency=self.frequency, initial_value=duty_cycle/100.0)
            self._running = True

        def ChangeDutyCycle(self, duty_cycle):
            if self._dev is None:
                return
            if not self._running:
                raise RuntimeError("PWM not started yet")
            self._dev.value = duty_cycle / 100.0

        def ChangeFrequency(self, frequency):
            if self._dev is None:
                return
            if not self._running:
                raise RuntimeError("PWM not started yet")
            self._dev.frequency = frequency
            self.frequency = frequency

        def stop(self):
            if self._running and self._dev:
                self._dev.off()
                self._dev.close()
            self._dev = None
            self._running = False
