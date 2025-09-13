## 安装QEMU

```shell
sudo apt update
sudo apt install qemu-system qemu-utils
```

## Windows XP

### 安装

首先下载系统镜像，我在这里找的精简版：[这里](https://www.bilibili.com/video/BV1184y1F7B2)

[标准版](https://archive.org/details/windows-xp-all-sp-msdn-iso-files-en-de-ru-tr-x86-x64)要密钥，而且安装很慢

下载完改名为 `WinXP.iso`

安装模拟器

```shell
sudo apt install qemu-system-x86
```

创建虚拟硬盘

```shell
qemu-img create -f raw ~/xp.img 10G
```

第一次用光盘启动，安装系统

```shell
qemu-system-i386 \
  -m 512 -smp 1 -cpu qemu32 \
  -machine pc \
  -accel tcg,thread=multi \
  -drive file=${HOME}/xp.img,if=ide,format=raw,cache=unsafe,aio=threads \
  -cdrom ./WinXP.iso -boot d \
  -net none \
  -usb -device usb-tablet
```

后面直接正常启动

```shell
qemu-system-i386 \
  -m 512 -smp 1 -cpu qemu32 \
  -machine pc \
  -accel tcg,thread=multi \
  -drive file=${HOME}/xp.img,if=ide,format=raw,cache=unsafe,aio=threads \
  -boot c \
  -net user \
  -usb -device usb-tablet
```

### 游戏

下载[自带游戏](https://archive.org/details/windows-xp-games)

```shell
wget https://archive.org/download/windows-xp-games/windows%20xp%20games.zip && unzip windows\ xp\ games.zip
```

挂载个虚拟盘，把游戏带进去就可以玩了～

```shell
mkdir share && cp -r windows\ xp\ games share
```

```shell
qemu-system-i386 \
  -m 512 -smp 1 -cpu qemu32 \
  -machine pc \
  -accel tcg,thread=multi \
  -drive file=${HOME}/xp.img,if=ide,format=raw,cache=unsafe,aio=threads \
  -boot c \
  -net user \
  -usb -device usb-tablet \
  -drive file=fat:rw:./share,format=raw,if=ide,index=1,media=disk
```

