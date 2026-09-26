# Rebecca

![done mp4_20250916_234539 802](https://github.com/user-attachments/assets/22227d79-40d6-4819-8e60-59795eafba4e)

视频链接：[链接](https://www.bilibili.com/video/BV1dNp4zVE2r)

## 月饼盒

一些玩法的尝试，详情可以参考各自的 README

```shell
git clone https://github.com/Forairaaaaa/Rebecca
```

```shell
.
├── camera
│   └── camera                   # 相机 app
├── hal
│   ├── cli-tool
│   │   ├── kava                 # 副屏控制便捷工具
│   │   └── rebecca-hal          # HAL API 便捷工具
│   ├── godot-plugin             # 给 Godot 项目用的 HAL 插件
│   └── service                  # HAL 服务
├── imu
│   └── pose-tracking            # Godot 姿态跟踪
├── screen
│   ├── cover
│   │   ├── hotop_like           # 副屏上的 htop
│   │   ├── lvgl                 # 副屏上跑 lvgl
│   │   └── web                  # 副屏上渲染 web canvas
│   └── jerry-tv                 # 全部屏幕随机循环播放猫和老鼠
├── steam                        # Steam Link 串流
└── vintage                      # 古早系统模拟器
```

## 内核和驱动

> **目前驱动是以64位官方镜像为基础开发的**

内核源码： [linux](https://github.com/Forairaaaaa/linux/tree/rpi-6.12.y-rebecca)

驱动开发仓库：[rebecca_drivers](https://github.com/Forairaaaaa/rebecca_drivers)，多谢[🧊🍅哥](https://github.com/IcingTomato)猛猛调驱动

### 内核编译和更新：

相关细节可以看[树莓派文档](https://www.raspberrypi.com/documentation/computers/linux_kernel.html#kernel)

下载 kernel 源码：

```shell
git clone --depth 1 -b rpi-6.12.y-rebecca https://github.com/Forairaaaaa/linux.git
```

安装工具链：

```shell
sudo apt install bc bison flex libssl-dev make
```

编译参数配置：

```shell
cd linux
KERNEL=kernel_2712
make rebecca_defconfig
```

编译：

```shell
make -j6 Image.gz modules dtbs
```

安装内核：

```shell
./install.sh
```

### 调节主屏亮度

先安装并启动 [HAL 服务](hal/service/README.md)，再安装 [HAL CLI](hal/cli-tool/rebecca-hal/README.md)。列出亮度设备：

```bash
rebecca-hal backlight
```

查看设备信息，确认主屏对应的设备 ID。下面以 `backlight0` 为例，实际以查询结果为准：

```bash
rebecca-hal backlight backlight0 info
```

```bash
rebecca-hal backlight backlight0 set 0.5
```

亮度范围为 0～1，`0.5` 表示 50%。读回当前设置：

```bash
rebecca-hal backlight backlight0 get
```

## 硬件

立创开源链接：[链接](https://oshwhub.com/eedadada/rebecca)

主控是[树莓派5](https://www.raspberrypi.com/products/raspberry-pi-5/)

### 屏幕驱动板

- 屏幕驱动
- 自定义按钮
- MPU6500

感谢 [@Cjiio](https://oshwhub.com/ccrs/g1392fh101gg-003-qu-dong-ban) 和 [@萨纳兰的黄昏](https://oshwhub.com/planevina/tai-shan-pai-amoled-ping-zhuan-jie-ban) 的屏幕驱动项目，参考了很多~

### 中间转接板

- 两个 SPI 副屏接口
- 两个 I2C 扩展接口
- ES8311 Codec，NS4150 功放 + 喇叭接口，模拟 MIC
- 环境光传感器

注意事项：

- 两个 I2C 接口是用来连线到两边侧翼的磁吸接口的，还没实际试过
- 模拟 MIC 没调试出来，没有声音，还不确定是软件问题还是电路问题
- 环境光传感器位置不理想，用不透明材料做外壳的话会挡住

### 部分零件链接

| :)                                          | (:                                                           |
| :------------------------------------------ | ------------------------------------------------------------ |
| UPS 电源                                    | [链接](https://item.taobao.com/item.htm?_u=g2bdtj0fe87e&id=782837596364&spm=a1z09.2.0.0.ddaa2e8d5YZtxH&sku_properties=1627207%3A9104969) |
| 侧边 SPI LCD 副屏                           | [链接](https://item.taobao.com/item.htm?_u=g2bdtj0f4c60&id=636002776097&spm=a1z09.2.0.0.ddaa2e8d5YZtxH) |
| 3520五磁喇叭[150MM1.25插头]                 | [链接](https://item.taobao.com/item.htm?_u=g2bdtj0f32dd&id=863554404251&spm=a1z09.2.0.0.ddaa2e8d5YZtxH) |
| 屏幕驱动到中间板排线 8P SH1.0               | [链接](https://item.taobao.com/item.htm?spm=a1z09.2.0.0.ddaa2e8d5YZtxH&id=745193272628&_u=g2bdtj0f2817) |
| 屏幕排线 22pin芯线同向50毫米                | [链接](https://item.taobao.com/item.htm?_u=g2bdtj0f8ed0&id=702853160953&spm=a1z09.2.0.0.ddaa2e8d5YZtxH) |
| 针脚加长的 2x20P 排母，用来增高树莓派的排针 | 之前不知道买什么送的，搜一下应该有                           |

## 结构

Fusion和拓竹工程可以在 [release](https://github.com/Forairaaaaa/Rebecca/releases) 下载

我视频里用的 PLA 哑光，长时间使用建议用更耐高温的，底部散热出气还是比较热的

### 零件链接

| :)                            | (:                                                           |
| :---------------------------- | ------------------------------------------------------------ |
| 十字圆头螺丝 M2.5*10          | [链接](https://detail.tmall.com/item.htm?_u=g2bdtj0ff887&id=16908083014&spm=a1z09.2.0.0.ddaa2e8d5YZtxH) |
| 单头六角柱 M2.5*10+6          | [链接](https://detail.tmall.com/item.htm?id=625159170166&spm=a1z09.2.0.0.ddaa2e8d5YZtxH&_u=g2bdtj0f8690) |
| 平头螺丝 M2.5*8               | [链接](https://detail.tmall.com/item.htm?_u=g2bdtj0fa5c4&id=19815788248&spm=a1z09.2.0.0.ddaa2e8d5YZtxH) |
| 防滑垫                        | [链接](https://item.taobao.com/item.htm?spm=a1z09.2.0.0.ddaa2e8d5YZtxH&id=633230700359&_u=g2bdtj0f8b00) |
| MagSafe磁吸环                 | [链接](https://detail.tmall.com/item.htm?_u=g2bdtj0f9b42&id=681312383366&spm=a1z09.2.0.0.ddaa2e8d5YZtxH) |
| 侧板磁吸磁铁 直径3mm 厚度 2mm | [链接](https://item.taobao.com/item.htm?spm=a1z09.2.0.0.ddaa2e8d5YZtxH&id=710543909089&_u=g2bdtj0fe1c5) |
| 固定屏幕的双面胶 1毫米宽      | [链接](https://detail.tmall.com/item.htm?id=653868724810&spm=a1z09.2.0.0.ddaa2e8d5YZtxH&_u=g2bdtj0fb094) |







