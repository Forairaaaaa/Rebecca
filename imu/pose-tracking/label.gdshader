shader_type canvas_item;

uniform sampler2D screen_tex : hint_screen_texture, filter_linear_mipmap;

// ========== HSL 工具 ==========
vec3 rgb2hsl(vec3 c) {
    float maxc = max(c.r, max(c.g, c.b));
    float minc = min(c.r, min(c.g, c.b));
    float l = (maxc + minc) * 0.5;
    float h = 0.0;
    float s = 0.0;

    if (maxc != minc) {
        float d = maxc - minc;
        s = l > 0.5 ? d / (2.0 - maxc - minc) : d / (maxc + minc);

        if (maxc == c.r) {
            h = (c.g - c.b) / d + (c.g < c.b ? 6.0 : 0.0);
        } else if (maxc == c.g) {
            h = (c.b - c.r) / d + 2.0;
        } else {
            h = (c.r - c.g) / d + 4.0;
        }
        h /= 6.0;
    }
    return vec3(h, s, l);
}

float hue2rgb(float p, float q, float t) {
    if (t < 0.0) t += 1.0;
    if (t > 1.0) t -= 1.0;
    if (t < 1.0/6.0) return p + (q - p) * 6.0 * t;
    if (t < 1.0/2.0) return q;
    if (t < 2.0/3.0) return p + (q - p) * (2.0/3.0 - t) * 6.0;
    return p;
}

vec3 hsl2rgb(vec3 c) {
    float r, g, b;
    if (c.y == 0.0) {
        r = g = b = c.z;
    } else {
        float q = c.z < 0.5 ? c.z * (1.0 + c.y) : c.z + c.y - c.z * c.y;
        float p = 2.0 * c.z - q;
        r = hue2rgb(p, q, c.x + 1.0/3.0);
        g = hue2rgb(p, q, c.x);
        b = hue2rgb(p, q, c.x - 1.0/3.0);
    }
    return vec3(r, g, b);
}

void vertex() {
	// Called for every vertex the material is visible on.
}

void fragment() {
	// 取背景像素
    vec4 bg = texture(screen_tex, SCREEN_UV);

    // 转 HSL，Hue + 180°（互补色）
    vec3 hsl = rgb2hsl(bg.rgb);
    hsl.x = mod(hsl.x + 0.5, 1.0); // Hue + 180°

    vec3 comp = hsl2rgb(hsl);

    // Label 的字体贴图
    vec4 font = texture(TEXTURE, UV);

    // 用字体 alpha 混合
    COLOR = vec4(comp, font.a);
}

//void light() {
//	// Called for every pixel for every light affecting the CanvasItem.
//	// Uncomment to replace the default light processing function with this one.
//}
