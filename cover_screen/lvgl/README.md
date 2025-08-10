直接在 lvgl flush 回调给 socket 推 buffer 就行

python ./fetch_repos.py

mkdir build

cd build && cmake ..

make -j8

./cover-screen-client
