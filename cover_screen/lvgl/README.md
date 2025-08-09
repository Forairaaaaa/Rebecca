python ./fetch_repos.py

mkdir build

cd build && cmake ..

make -j8

./cover-screen-client
