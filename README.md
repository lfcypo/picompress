# picompress

一个用 Rust 编写的图片压缩 CLI 支持 PNG JPEG WebP 等常见格式 默认在尽量保留画面的前提下压缩体积

## 安装

```bash
cargo install --path .
```

## 使用

```bash
# 默认中等压缩等级 输出到原图旁 photo.compressed.jpg
picompress photo.jpg

# 指定质量档位
picompress photo.jpg --quality high

# 指定输出最大体积
picompress photo.jpg --max-size 200KB

# 指定输出路径与格式
picompress photo.png -o photo.webp
```

## 参数

- `INPUT` 待压缩图片路径
- `-q, --quality <LEVEL>` 质量档位 可选 `lowest` `low` `medium` `high` `highest`
- `--max-size <SIZE>` 输出最大体积 支持 `200KB` `1MB` `1.5MB` 等写法
- `-o, --output <OUTPUT>` 输出文件或目录 省略时在输入图片同目录生成 `*.compressed.<ext>`
- `-h, --help` 查看帮助
- `-V, --version` 查看版本

`--quality` 与 `--max-size` 互斥 两者都不给时使用 `medium`

## 压缩策略

- JPEG 使用质量参数编码 并保留 ICC 与 EXIF 元数据
- WebP 使用 libwebp 有损编码
- PNG 中高质量档位使用无损优化 较低质量档位退化为调色板量化以换取更小体积
- 通过 `--max-size` 指定体积时 会先逐步降低质量 仍不满足时再按需缩小分辨率
- 如果压缩结果反而更大 会保留原始文件内容

## 支持格式

输入支持 `image` crate 可解码的常见格式 输出支持 PNG JPEG WebP

## 许可证

本项目使用 MIT 许可证
