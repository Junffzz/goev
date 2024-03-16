
## NexaRift XR 桌面端
**NexaRift XR Link**（视界融渡） 借助串流技术将虚拟和现实的"视界"无缝"融渡"在一起。“Nexa”是一个创新的词汇，可以联想到“下一个”（Next）和“连接”（Connect），而“Rift”直接指向VR技术，整体名称强调软件在连接和同步VR体验方面的能力。

## 编译
```shell
cargo clean
# 编译ffmpeg依赖（todo:cargo vcpkg方案废弃）
### 设置vcpkg的根路径
cd app/native/
cargo vcpkg --verbose build --target=/Users/zhaojunfeng/code_runtimes/rust/devrust/NextRiftXR_desktop

# 指定运行时路径
cargo run --package enter --bin enter --target-dir=/Users/zhaojunfeng/code_runtimes/rust/NextRiftXR_desktop
```
### 运行环境变量
```shell
COREAUDIO_SDK_PATH=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/;
FFMPEG_PKG_CONFIG_PATH=/Users/zhaojunfeng/code_runtimes/rust/NextRiftXR_desktop/vcpkg/installed/arm64-osx/lib/pkgconfig;

# 用于指定运行时路径
CARGO_TARGET_PATH=/Users/zhaojunfeng/code_runtimes/rust

# 更新rsmpeg
cargo update -p rsmpeg
```