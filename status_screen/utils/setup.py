from setuptools import setup, find_packages

setup(
    name='local_utils',
    version='0.1',
    packages=find_packages(),
    install_requires=[
        'gpiozero==2.0.1',
        'lgpio==0.2.2.0',
    ],
)
