use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum ResizeMode {
    Stretch,   // 拉伸
    Letterbox, // 等比缩放，居中显示
    Fill,      // 等比缩放，填满屏幕（可能裁剪）
}
