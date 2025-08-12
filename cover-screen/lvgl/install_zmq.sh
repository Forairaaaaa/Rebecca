#!/bin/bash

set -e

echo "Updating apt repositories..."
sudo apt update

echo "Installing libzmq3-dev..."
sudo apt install -y libzmq3-dev cmake build-essential git

# 安装 cppzmq
echo "Cloning cppzmq..."
git clone https://github.com/zeromq/cppzmq.git

cd cppzmq
mkdir -p build
cd build

echo "Building cppzmq..."
cmake -DCPPZMQ_BUILD_TESTS=OFF ..
make -j6
sudo make install
cd ../..

echo "Cleaning up..."
rm -rf cppzmq

echo "zmq and cppzmq installed successfully!"
