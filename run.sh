#!/bin/bash

# 1. Nạp môi trường Rust
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
else
    echo "Không tìm thấy môi trường Rust. Vui lòng cài đặt Rust trước."
    exit 1
fi

# 2. Biên dịch dự án
echo "======================================"
echo " [1/4] Đang biên dịch glass-wm..."
echo "======================================"
cargo build

if [ $? -ne 0 ]; then
    echo "❌ Biên dịch thất bại! Hãy kiểm tra lỗi code ở trên."
    exit 1
fi

# 3. Tạo thư mục cấu hình mặc định
CONFIG_DIR="$HOME/.config/glass-wm"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "[2/4] Khởi tạo thư mục cấu hình tại $CONFIG_DIR..."
    mkdir -p "$CONFIG_DIR"
fi

# 4. Chạy Compositor trong nền (Background)
echo "======================================"
echo " [3/4] Đang khởi động glass-wm..."
echo "======================================"
export RUST_LOG=info

# Ghi đè log cũ
echo "" > compositor.log

# Khởi chạy compositor trong nền
./target/debug/glass-wm > compositor.log 2>&1 &
COMPOSITOR_PID=$!

# Đợi 2 giây để Compositor khởi động và tạo socket
sleep 2

# Kiểm tra xem tiến trình có hoạt động không
if ! kill -0 $COMPOSITOR_PID >/dev/null 2>&1; then
    echo "❌ Compositor thất bại khi khởi chạy! Xem chi tiết lỗi tại 'compositor.log':"
    cat compositor.log
    exit 1
fi

echo "✅ Compositor đang chạy với PID: $COMPOSITOR_PID"
echo "👉 Bạn có thể xem log thời gian thực bằng lệnh: tail -f compositor.log"
echo ""

# 5. Phát hiện socket Wayland đang lắng nghe từ log file
SOCKET_NAME=$(grep -o "wayland-[0-9]" compositor.log | tail -n 1)
if [ -z "$SOCKET_NAME" ]; then
    SOCKET_NAME="wayland-1" # Fallback
fi

echo "======================================"
echo " [4/4] Khởi chạy ứng dụng thử nghiệm (Socket: $SOCKET_NAME)..."
echo "======================================"

# Khởi chạy ứng dụng khách Wayland lồng bên trong
if command -v alacritty >/dev/null 2>&1; then
    echo "🚀 Đang mở Alacritty..."
    WAYLAND_DISPLAY=$SOCKET_NAME alacritty &
elif command -v kitty >/dev/null 2>&1; then
    echo "🚀 Đang mở Kitty..."
    WAYLAND_DISPLAY=$SOCKET_NAME kitty &
elif command -v weston-terminal >/dev/null 2>&1; then
    echo "🚀 Đang mở Weston Terminal..."
    WAYLAND_DISPLAY=$SOCKET_NAME weston-terminal &
else
    echo "⚠️  Không tìm thấy alacritty, kitty hoặc weston-terminal."
    echo "💡 Bạn có thể tự mở bất kỳ ứng dụng Wayland nào bằng cách chạy lệnh sau ở terminal khác:"
    echo "   WAYLAND_DISPLAY=$SOCKET_NAME <tên-ứng-dụng>"
fi

# Chờ người dùng kết thúc test
echo ""
echo "--------------------------------------------------"
echo "Ấn phím [ENTER] để dừng compositor và tắt ứng dụng..."
echo "--------------------------------------------------"
read

# Dọn dẹp tiến trình
kill $COMPOSITOR_PID
echo "🧹 Đã tắt glass-wm thành công."
