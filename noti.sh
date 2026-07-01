#!/bin/bash

# ArchVNDE Notification Test Script
# This script sends test notifications using notify-send with various app names and icons.

echo "================================================="
# Test 1: Firefox notification
echo "Sending Firefox notification..."
notify-send -a "Firefox" -i "firefox" "Firefox" "Trình duyệt Firefox đã khởi động thành công."
sleep 3

# Test 2: Discord notification
echo "Sending Discord notification..."
notify-send -a "Discord" -i "discord" "Discord" "Bạn có tin nhắn mới từ Antigravity."
sleep 3

# Test 3: Spotify notification
echo "Sending Spotify notification..."
notify-send -a "Spotify" -i "spotify" "Spotify" "Đang phát: Starboy - The Weeknd"
sleep 3

# Test 4: System notification
echo "Sending System notification..."
notify-send -a "System Settings" -i "preferences-system" "Cập nhật hệ thống" "Đã tải xuống bản cập nhật mới."
sleep 3

# Test 5: Telegram notification
echo "Sending Telegram notification..."
notify-send -a "Telegram" -i "telegram" "Telegram" "Tin nhắn nhóm mới từ nhóm phát triển."
echo "================================================="
echo "All test notifications sent!"
