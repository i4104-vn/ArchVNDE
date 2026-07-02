# Quy trình làm việc với Git (Git Workflow)

Tài liệu hướng dẫn ngắn gọn các bước làm việc với nhánh, commit, và đồng bộ code.

---

### Bước 1: Cập nhật nhánh `main` trước khi tạo nhánh mới
```bash
git checkout main
git pull origin main
```

### Bước 2: Tạo nhánh mới (để fix hoặc làm tính năng mới)
Đặt tên nhánh theo chuẩn: `fix/<tên-lỗi>` hoặc `feature/<tên-tính-năng>`.
```bash
git checkout -b fix/control-center
```

### Bước 3: Lưu thay đổi và Tạo Commit
1. Xem các tệp đã thay đổi:
   ```bash
   git status
   ```
2. Thêm các tệp muốn commit:
   ```bash
   git add <đường-dẫn-file>
   # Hoặc thêm tất cả thay đổi:
   git add .
   ```
3. Quyết định Tạo Commit Mới hoặc Gộp Commit (Amend):
   - **Trường hợp 1: Tạo Commit Mới** -> Dành cho việc **thêm tính năng mới** hoặc **code thêm phần mới hoàn toàn**:
     ```bash
     git commit -m "feat(panel): add new battery percentage label"
     ```
   - **Trường hợp 2: Gộp Commit (Amend Commit) - *Không tạo commit mới*** -> Dành cho việc **sửa lại code, sửa lỗi (bug fixes), tinh chỉnh hoặc tối ưu hóa** phần code đã có trên nhánh để giữ lịch sử commit sạch sẽ:
     ```bash
     git add <đường-dẫn-file-sửa-đổi>
     # Gộp thay đổi và giữ nguyên nội dung commit cũ:
     git commit --amend --no-edit
     # Gộp thay đổi và thay đổi nội dung commit cũ:
     git commit --amend -m "nội dung commit mới"
     ```

### Bước 5: Đẩy nhánh lên Remote (GitHub) - Sau phần này chỉ thực hiện khi người dùng yêu cầu
```bash
git push origin <tên-nhánh-của-bạn>
# Ví dụ:
git push origin fix/control-center
# Lưu ý: Nếu đã push nhánh lên remote trước đó và sau đó thực hiện `--amend`, bạn phải push đè lên:
git push origin <tên-nhánh-của-bạn> --force-with-lease
```

### Bước 6: Gộp nhánh vào `main` ở Local và đẩy `main` lên GitHub
1. Chuyển về nhánh `main`:
   ```bash
   git checkout main
   ```
2. Gộp nhánh vừa làm (ví dụ `fix/control-center`) vào nhánh `main` ở local:
   ```bash
   git merge fix/control-center
   ```
3. Đẩy nhánh `main` mới nhất lên GitHub:
   ```bash
   git push origin main
   ```
4. (Tùy chọn) Xóa nhánh local vừa gộp xong để dọn dẹp:
   ```bash
   git branch -d fix/control-center
   ```

### Bước 7: Cập nhật lại các nhánh lớn khác (sau khi main đã thay đổi)
Để cập nhật các nhánh phát triển khác (như `feature/theme`, `fix/panel`, v.v.) theo nhánh `main` mới nhất:
1. Chuyển sang nhánh đó:
   ```bash
   git checkout <tên-nhánh-lớn>
   ```
2. Cập nhật nhánh đó bằng cách kéo thay đổi hoặc rebase từ `main`:
   ```bash
   # Cách 1: Gộp code từ main vào nhánh hiện tại (Tạo commit merge)
   git merge main
   
   # Cách 2: Đặt các commit của nhánh lên trên main mới nhất (Lịch sử sạch hơn)
   git rebase main
   ```
3. Đẩy nhánh đã cập nhật lên GitHub:
   ```bash
   git push origin <tên-nhánh-lớn> --force-with-lease
   ```
