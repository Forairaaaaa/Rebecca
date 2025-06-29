obj-m := panel-lg-3p92.o

KDIR := /lib/modules/$(shell uname -r)/build
PWD := $(shell pwd)

default:
	$(MAKE) -C $(KDIR) M=$(PWD) modules
	dtc -@ -I dts -O dtb -o panel-lg-3p92.dtbo panel-lg-3p92-overlay.dts
	
clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
	rm -f panel-lg-3p92.dtbo

install:
	cp panel-lg-3p92.ko /lib/modules/$(shell uname -r)/kernel/drivers/gpu/drm/panel
	cp panel-lg-3p92.dtbo /boot/firmware/overlays/
	
uninstall:
	rm -f /lib/modules/$(shell uname -r)/kernel/drivers/gpu/drm/panel/panel-lg-3p92.ko
	rm -f /boot/firmware/overlays/panel-lg-3p92.dtbo
