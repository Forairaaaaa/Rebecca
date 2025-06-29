// SPDX-License-Identifier: GPL-2.0
#include <linux/backlight.h>
#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/module.h>
#include <linux/of.h>
#include <linux/regulator/consumer.h>

#include <drm/drm_mipi_dsi.h>
#include <drm/drm_modes.h>
#include <drm/drm_panel.h>

struct lg3p92_panel {
	struct drm_panel panel;
	struct mipi_dsi_device *dsi;
	struct gpio_desc *reset_gpio;
    uint8_t prepared;
};

static inline struct lg3p92_panel *to_lg3p92_panel(struct drm_panel *panel)
{
	return container_of(panel, struct lg3p92_panel, panel);
}

static void lg3p92_panel_reset(struct lg3p92_panel *ctx)
{
	printk(KERN_INFO "lg3p92_panel_reset\n");

    // TODO
	gpiod_set_value_cansleep(ctx->reset_gpio, 1);
	msleep(200);
	gpiod_set_value_cansleep(ctx->reset_gpio, 0);
	msleep(120);

	// gpiod_set_value_cansleep(ctx->reset_gpio, 0);
	// msleep(200);
	// gpiod_set_value_cansleep(ctx->reset_gpio, 1);
	// msleep(120);
}

// static void lg3p92_panel_on(struct mipi_dsi_multi_context *dsi_ctx)
// {
// 	printk(KERN_INFO "lg3p92_panel_on\n");

// 	// 11
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x5A, 0x0B);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x5C, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x80);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x00, 0x4F);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x08, 0xDF);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x09, 0xEE);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0A, 0xED);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0B, 0xFC);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0C, 0xF0);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0D, 0xFB);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0E, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0F, 0x01);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x14, 0x3F);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x15, 0xB3);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x16, 0x74);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x17, 0xFF);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x18, 0xAD);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x19, 0x3F);

// 	// 22
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x1A, 0xC9);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x1B, 0x07);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x10);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x00, 0x9C);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x01, 0x57);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x02, 0x05);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x03, 0x14);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x04, 0x10);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x05, 0x01);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x06, 0x6C);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x07, 0xA7);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x08, 0x02);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x09, 0x06);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0A, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0B, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0C, 0x8D);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0D, 0x07);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x12);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x84, 0x00);


// 	// 33
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x12);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x83, 0x10);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x40);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0D, 0x18);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xC2, 0x08);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x35, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x11, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x29, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x5A, 0x0B);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x5C, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x80);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x00, 0x4F);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x08, 0xDF);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x09, 0xEE);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0A, 0xED);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0B, 0xFC);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0C, 0xF0);

// 	// 44
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0D, 0xFB);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0E, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0F, 0x01);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x14, 0x3F);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x15, 0xB3);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x16, 0x74);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x17, 0xFF);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x18, 0xAD);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x19, 0x3F);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x1A, 0xC9);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x1B, 0x07);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x10);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x00, 0x9C);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x01, 0x57);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x02, 0x05);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x03, 0x14);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x04, 0x10);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x05, 0x01);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x06, 0x6C);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x07, 0xA7);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x08, 0x02);

// 	// 55
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x09, 0x06);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0A, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0B, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0C, 0x8D);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x0D, 0x07);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x12);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x84, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x83, 0x10);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x5C, 0x0F);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x80);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x00, 0x4F);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x05, 0x1C);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x12);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x84, 0x00);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);

// 	// 66
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x2A, 0x00, 0x00, 0x04, 0x37);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x2B, 0x00, 0x00, 0x04, 0xD7);
// 	mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x51, 0x07, 0xFF);


//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x11);
//     mipi_dsi_msleep(dsi_ctx, 200);

//     mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x29);  // display on
//     mipi_dsi_msleep(dsi_ctx, 120);
// }

static void lg3p92_panel_on(struct mipi_dsi_multi_context *dsi_ctx)
{
	printk(KERN_INFO "lg3p92_panel_on\n");
	

	// mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xFE, 0x00);
    // mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0xC2, 0x08);
    // mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x35, 0x00);

    // mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x51, 0x07, 0xFF);
    // mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x11);  // exit sleep
    // // mipi_dsi_msleep(dsi_ctx, 120);
	// msleep(120);

    // mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x29);  // display on
    // // mipi_dsi_msleep(dsi_ctx, 80);
	// msleep(80);

	struct mipi_dsi_device *dsi = dsi_ctx->dsi;
	u8 data1[] = { 0x00 };
	u8 data2[] = { 0x08 };
	u8 data3[] = { 0x00 };
	u8 data4[] = { 0x07, 0xFF };

	printk(KERN_INFO "lg3p92_panel_on asda ss asda sd\n");

	mipi_dsi_dcs_write(dsi, 0xFE, data1, sizeof(data1));
	mipi_dsi_dcs_write(dsi, 0xC2, data2, sizeof(data2));
	mipi_dsi_dcs_write(dsi, 0x35, data3, sizeof(data3));
	mipi_dsi_dcs_write(dsi, 0x51, data4, sizeof(data4));

	mipi_dsi_dcs_write(dsi, 0x11, NULL, 0);
	msleep(120);

	mipi_dsi_dcs_write(dsi, 0x29, NULL, 0);
	msleep(80);
}

static void lg3p92_panel_off(struct mipi_dsi_multi_context *dsi_ctx)
{
	printk(KERN_INFO "lg3p92_panel_off\n");

    // TODO
    // mipi_dsi_dcs_set_display_off_multi(dsi_ctx);

	// mipi_dsi_msleep(dsi_ctx, 60);

	// mipi_dsi_dcs_enter_sleep_mode_multi(dsi_ctx);

    // mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x28);
    // mipi_dsi_dcs_write_seq_multi(dsi_ctx, 0x10);
}

static int lg3p92_panel_prepare(struct drm_panel *panel)
{
	struct lg3p92_panel *ctx = to_lg3p92_panel(panel);
    struct mipi_dsi_multi_context dsi_ctx =	{.dsi = ctx->dsi};

	if (ctx->prepared)
		return 0;

	lg3p92_panel_reset(ctx);

	lg3p92_panel_on(&dsi_ctx);

	ctx->prepared = true;
	return 0;
}

static int lg3p92_panel_unprepare(struct drm_panel *panel)
{
	struct lg3p92_panel *ctx = to_lg3p92_panel(panel);
    struct mipi_dsi_multi_context dsi_ctx =	{.dsi = ctx->dsi};

	if (!ctx->prepared)
		return 0;

	lg3p92_panel_off(&dsi_ctx);

	ctx->prepared = false;
	return 0;
}

// static const struct drm_display_mode panel_mode = {
// 	// .clock = 88000, // kHz
// 	.clock = 83333,
// 	// .clock = 50000,
// 	// .clock = 148500,
// 	.hdisplay = 1080,
// 	.hsync_start = 1080 + 40,
// 	.hsync_end = 1080 + 40 + 4,
// 	.htotal = 1080 + 40 + 4 + 40,

// 	.vdisplay = 1240,
// 	.vsync_start = 1240 + 16,
// 	.vsync_end = 1240 + 16 + 4,
// 	.vtotal = 1240 + 16 + 4 + 20,

// 	// .flags = DRM_MODE_FLAG_NHSYNC | DRM_MODE_FLAG_NVSYNC,

// 	.width_mm = 65,
// 	.height_mm = 75,

//     .type = DRM_MODE_TYPE_DRIVER | DRM_MODE_TYPE_PREFERRED,
// };

// static const struct drm_display_mode panel_mode = {
// 	.clock = 87300, // kHz (1268 x 1148 x 60 / 1000)

// 	.hdisplay = 1080,
// 	.hsync_start = 1080 + 36,        // hbp
// 	.hsync_end   = 1080 + 36 + 4,    // hbp + hsa
// 	.htotal      = 1080 + 36 + 4 + 28, // hbp + hsa + hfp

// 	.vdisplay = 1240,
// 	.vsync_start = 1240 + 8,         // vbp
// 	.vsync_end   = 1240 + 8 + 4,     // vbp + vsa
// 	.vtotal      = 1240 + 8 + 4 + 16, // vbp + vsa + vfp

// 	.width_mm = 65,   // 根据实际物理尺寸调整
// 	.height_mm = 75,  // 根据实际物理尺寸调整

// 	.type = DRM_MODE_TYPE_DRIVER | DRM_MODE_TYPE_PREFERRED,
// };

static const struct drm_display_mode panel_mode = {
    // .clock = 74200, // 单位: kHz，先估一个默认值 74.2MHz，后面详细说明如何计算
	.clock = 83333,
    .hdisplay = 1080,
    .hsync_start = 1080 + 28,              // HFP
    .hsync_end   = 1080 + 28 + 4,          // HSW
    .htotal      = 1080 + 28 + 4 + 36,     // HFP + HSW + HBP

    .vdisplay = 1240,
    .vsync_start = 1240 + 16,              // VFP
    .vsync_end   = 1240 + 16 + 4,          // VSW
    .vtotal      = 1240 + 16 + 4 + 8,      // VFP + VSW + VBP

    // .flags = DRM_MODE_FLAG_NHSYNC | DRM_MODE_FLAG_NVSYNC, // 同步信号为低电平有效
    // .type = DRM_MODE_TYPE_DRIVER | DRM_MODE_TYPE_PREFERRED,
    .width_mm = 65,   // 你可根据屏幕物理尺寸填写真实值
    .height_mm = 75, // 同上
};


static int lg3p92_panel_get_modes(struct drm_panel *panel,
			     struct drm_connector *connector)
{
	struct drm_display_mode *mode;

	mode = drm_mode_duplicate(connector->dev, &panel_mode);
	if (!mode)
		return -ENOMEM;

	drm_mode_set_name(mode);
	mode->type = DRM_MODE_TYPE_DRIVER | DRM_MODE_TYPE_PREFERRED;

	connector->display_info.width_mm = mode->width_mm;
	connector->display_info.height_mm = mode->height_mm;

	drm_mode_probed_add(connector, mode);
	return 1;
}

static const struct drm_panel_funcs lg3p92_panel_funcs = {
	.prepare = lg3p92_panel_prepare,
	.unprepare = lg3p92_panel_unprepare,
	.get_modes = lg3p92_panel_get_modes,
};

static int lg3p92_panel_bl_update_status(struct backlight_device *bl)
{
	struct mipi_dsi_device *dsi = bl_get_data(bl);
	u16 brightness = backlight_get_brightness(bl);

	brightness = 127;

	int ret;

	printk(KERN_INFO "lg3p92_panel_bl_update_status: %d\n", brightness);

	// dsi->mode_flags &= ~MIPI_DSI_MODE_LPM;

	ret = mipi_dsi_dcs_set_display_brightness(dsi, brightness);
	if (ret < 0)
		return ret;

	// dsi->mode_flags |= MIPI_DSI_MODE_LPM;

	return 0;
}

static int lg3p92_panel_bl_get_brightness(struct backlight_device *bl)
{
	printk(KERN_INFO "lg3p92_panel_bl_get_brightness\n");

	struct mipi_dsi_device *dsi = bl_get_data(bl);
	u16 brightness = bl->props.brightness;
	int ret;

	// dsi->mode_flags &= ~MIPI_DSI_MODE_LPM;

	ret = mipi_dsi_dcs_get_display_brightness(dsi, &brightness);
	if (ret < 0)
		return ret;

	// dsi->mode_flags |= MIPI_DSI_MODE_LPM;

	return brightness & 0xff;
}

static const struct backlight_ops lg3p92_panel_bl_ops = {
	.update_status = lg3p92_panel_bl_update_status,
	.get_brightness = lg3p92_panel_bl_get_brightness,
};

static struct backlight_device *
lg3p92_panel_create_backlight(struct mipi_dsi_device *dsi)
{
	struct device *dev = &dsi->dev;
	const struct backlight_properties props = {
		.type = BACKLIGHT_RAW,
		.brightness = 255,
		.max_brightness = 255,
	};

	return devm_backlight_device_register(dev, dev_name(dev), dev, dsi,
					      &lg3p92_panel_bl_ops, &props);
}

// static int lg3p92_panel_probe(struct mipi_dsi_device *dsi)
// {
// 	printk(KERN_INFO "lg3p92_panel_probe\n");

// 	struct device *dev = &dsi->dev;
// 	struct lg3p92_panel *ctx;
// 	int ret;

// 	ctx = devm_kzalloc(dev, sizeof(*ctx), GFP_KERNEL);
// 	if (!ctx)
// 		return -ENOMEM;

// 	ctx->reset_gpio = devm_gpiod_get_optional(dev, "reset", GPIOD_OUT_LOW);
// 	if (IS_ERR(ctx->reset_gpio))
// 		return dev_err_probe(dev, PTR_ERR(ctx->reset_gpio), "Failed to get reset GPIO\n");

// 	// ctx->dsi = dsi;
// 	// mipi_dsi_set_drvdata(dsi, ctx);

// 	// // dsi->lanes = 4;
// 	// dsi->lanes = 1;
// 	// dsi->format = MIPI_DSI_FMT_RGB888;
// 	// // dsi->mode_flags = MIPI_DSI_MODE_VIDEO | MIPI_DSI_MODE_VIDEO_BURST |
// 	// //                   MIPI_DSI_MODE_LPM | MIPI_DSI_MODE_NO_EOT_PACKET;
// 	// // dsi->mode_flags = MIPI_DSI_MODE_VIDEO_HSE | MIPI_DSI_MODE_VIDEO | MIPI_DSI_CLOCK_NON_CONTINUOUS;
// 	// dsi->mode_flags = MIPI_DSI_MODE_VIDEO_HSE | MIPI_DSI_MODE_VIDEO |
// 	// 	      MIPI_DSI_MODE_LPM | MIPI_DSI_CLOCK_NON_CONTINUOUS;

// 	drm_panel_init(&ctx->panel, dev, &lg3p92_panel_funcs, DRM_MODE_CONNECTOR_DSI);

// 	ctx->panel.backlight = lg3p92_panel_create_backlight(dsi);
// 	if (IS_ERR(ctx->panel.backlight)) {
// 		ret = PTR_ERR(ctx->panel.backlight);
// 		dev_err(dev, "Failed to create backlight: %d\n", ret);
// 		return ret;
// 	}

// 	drm_panel_add(&ctx->panel);

// 	// ret = mipi_dsi_attach(dsi);
// 	// if (ret < 0) {
// 	// 	drm_panel_remove(&ctx->panel);
// 	// 	return dev_err_probe(dev, ret, "Failed to attach to DSI host\n");
// 	// }

// 	ctx->dsi = dsi;

// 	dsi->mode_flags = MIPI_DSI_MODE_VIDEO_HSE | MIPI_DSI_MODE_VIDEO | MIPI_DSI_MODE_LPM | MIPI_DSI_CLOCK_NON_CONTINUOUS;
// 	dsi->format = MIPI_DSI_FMT_RGB888;
// 	dsi->lanes = 4;

// 	ret = devm_mipi_dsi_attach(dev, dsi);

// 	if (ret)
// 		dev_err(dev, "failed to attach dsi to host: %d\n", ret);

// 	return 0;
// }

static int lg3p92_panel_probe(struct mipi_dsi_device *dsi)
{
	struct lg3p92_panel *ctx;
	int ret;

	dev_info(&dsi->dev, "dsi panel: %s\n",
		 (char *)of_get_property(dsi->dev.of_node, "compatible", NULL));

	ctx = devm_kzalloc(&dsi->dev, sizeof(*ctx), GFP_KERNEL);
	if (!ctx)
		return -ENOMEM;
	mipi_dsi_set_drvdata(dsi, ctx);
	ctx->dsi = dsi;
	// ctx->desc = of_device_get_match_data(&dsi->dev);

	ctx->panel.prepare_prev_first = true;
	drm_panel_init(&ctx->panel, &dsi->dev, &lg3p92_panel_funcs,
		       DRM_MODE_CONNECTOR_DSI);

	ctx->reset_gpio = devm_gpiod_get_optional(&dsi->dev, "reset", GPIOD_OUT_LOW);
	if (IS_ERR(ctx->reset_gpio))
		return dev_err_probe(&dsi->dev, PTR_ERR(ctx->reset_gpio),
				     "Couldn't get our reset GPIO\n");

	// ret = of_drm_get_panel_orientation(dsi->dev.of_node, &ctx->orientation);
	// if (ret) {
	// 	dev_err(&dsi->dev, "%pOF: failed to get orientation: %d\n",
	// 		dsi->dev.of_node, ret);
	// 	return ret;
	// }

	// ret = drm_panel_of_backlight(&ctx->panel);
	// if (ret)
	// 	return ret;

	drm_panel_add(&ctx->panel);

	dsi->mode_flags = MIPI_DSI_MODE_VIDEO_HSE | MIPI_DSI_MODE_VIDEO |
		      		      MIPI_DSI_MODE_LPM | MIPI_DSI_CLOCK_NON_CONTINUOUS;
	dsi->format = MIPI_DSI_FMT_RGB888;
	dsi->lanes = 4;
	dev_info(&dsi->dev, "lanes: %d\n", dsi->lanes);

	ret = mipi_dsi_attach(dsi);
	if (ret)
		drm_panel_remove(&ctx->panel);

	return ret;
}

static void lg3p92_panel_remove(struct mipi_dsi_device *dsi)
{
	struct lg3p92_panel *ctx = mipi_dsi_get_drvdata(dsi);

	mipi_dsi_detach(dsi);
	drm_panel_remove(&ctx->panel);
}

static const struct of_device_id lg3p92_of_match[] = {
	{ .compatible = "lg,lg3p92" },
	{ /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, lg3p92_of_match);

static struct mipi_dsi_driver lg3p92_panel_driver = {
	.driver = {
		.name = "panel-lg3p92",
		.of_match_table = lg3p92_of_match,
	},
	.probe = lg3p92_panel_probe,
	.remove = lg3p92_panel_remove,
};
module_mipi_dsi_driver(lg3p92_panel_driver);

MODULE_AUTHOR("Dashabi");
MODULE_DESCRIPTION("LG 3.92 AMOLED panel driver");
MODULE_LICENSE("GPL");
