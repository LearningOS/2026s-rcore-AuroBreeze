#!/bin/bash
# 使用方法: source ~/qemu-env.sh

# 你的编译输出目录
QEMU_BUILD_DIR="/home/AuroBreeze/Downloads/qemu-7.0.0/qemu-7.0.0/qemu-7.0.0/build"

# 检查目录是否存在防呆
if [ ! -d "$QEMU_BUILD_DIR" ]; then
    echo "❌ 错误：找不到构建目录 $QEMU_BUILD_DIR"
    # 兼容 source 和 ./ 两种执行方式的退出
    return 1 2>/dev/null || exit 1
fi

# 1. 备份原有的环境变量（方便未来如果需要写退出脚本时使用）
export OLD_PATH="$PATH"
export OLD_PS1="$PS1"

# 2. 注入新的环境变量
export PATH="$QEMU_BUILD_DIR:$PATH"
export MY_QEMU_DIR="$QEMU_BUILD_DIR"

# 3. 修改终端提示符，增加 (qemu-7.0.0) 前缀
export PS1="(qemu-7.0.0) $PS1"

# 4. 打印欢迎和状态信息
echo "==================================================="
echo "✅ 成功激活 AuroBreeze 的自定义 QEMU 环境！"
echo "📂 当前目录已优先指向: $QEMU_BUILD_DIR"
echo "🛠️  你可以直接运行 qemu-system-riscv64, qemu-img 等"
echo "🚪 提示: 关闭当前终端标签页即可退出该隔离环境"
echo "==================================================="
