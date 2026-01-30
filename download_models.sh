#!/bin/bash

# 下载模型文件的脚本

echo "开始下载模型文件..."

# 创建模型目录
mkdir -p models

echo "注意：此脚本提供模型下载指导，实际下载需根据您的具体需求进行。"

echo ""
echo "1. Whisper模型下载："
echo "   - 访问 https://huggingface.co/ggerganov/whisper.cpp"
echo "   - 下载 ggml-tiny.bin 或 ggml-base.bin"
echo "   - 重命名为 models/whisper-tiny.bin 或 models/whisper-base.bin"

echo ""
echo "2. LLM模型下载："
echo "   - 访问 https://huggingface.co/models (搜索Qwen2.5-0.5B或Llama-3.2-1B)"
echo "   - 下载GGUF格式的量化模型"
echo "   - 放置到 models/ 目录下"

echo ""
echo "3. Silero VAD模型："
echo "   - 访问 https://github.com/snakers4/silero-vad"
echo "   - 下载所需的模型文件"
echo "   - 放置到 models/ 目录下"

echo ""
echo "模型文件下载完成后，更新代码中的模型路径。"

# 创建一个示例配置文件
cat > models/config.json << EOF
{
  "asr_model_path": "./models/whisper-tiny.bin",
  "mt_model_path": "./models/qwen2.5-0.5b.bin",
  "vad_model_path": "./models/silero_vad.jit"
}
EOF

echo ""
echo "配置文件已创建: models/config.json"