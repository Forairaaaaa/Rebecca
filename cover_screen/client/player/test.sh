#!/bin/bash

PLAYER=./target/release/cover-screen-player

SCREEN=screen0

TEST_CASES=(
    "$SCREEN"
    "$SCREEN -u https://c-ssl.dtstatic.com/uploads/item/201910/01/20191001164555_ldwlb.thumb.1000_0.png"
    "$SCREEN -u https://img0.baidu.com/it/u=2256422267,1818901425&fm=253&fmt=auto&app=138&f=JPEG?w=800&h=800"
    "$SCREEN -u https://ww1.sinaimg.cn/mw690/001QzeLXly1hw11qvmzo6j60yu0yuafk02.jpg"
    "$SCREEN -u https://bkimg.cdn.bcebos.com/pic/faedab64034f78f0220d8e0c79310a55b2191cce"
)

if [ -n "$1" ]; then
    SCREEN=$1
fi

cargo build --release

for test_case in "${TEST_CASES[@]}"; do
    echo "--------------------------------"
    echo "test case: $test_case"
    $PLAYER $test_case
    echo "--------------------------------"
    sleep 2
done
