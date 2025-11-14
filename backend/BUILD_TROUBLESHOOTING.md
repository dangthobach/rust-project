# 🔧 Build Troubleshooting Guide

## ❌ Error: "Access is denied" khi build

### Nguyên nhân:
File `crm-backend.exe` đang được sử dụng bởi một process khác.

### ✅ Giải pháp:

#### 1. **Kiểm tra process đang chạy**
```powershell
# Tìm process đang chạy
Get-Process | Where-Object {$_.ProcessName -like "*crm-backend*"}

# Kill process nếu có
Stop-Process -Name "crm-backend" -Force
```

#### 2. **Đóng tất cả terminal/IDE đang chạy backend**
- Đóng tất cả terminal windows đang chạy `cargo run`
- Đóng IDE nếu đang debug/run
- Restart terminal và thử lại

#### 3. **Xóa thủ công file .exe**
```powershell
cd backend
# Xóa file .exe nếu bị lock
Remove-Item -Path "target\debug\crm-backend.exe" -Force -ErrorAction SilentlyContinue
cargo build
```

#### 4. **Xóa toàn bộ target folder (nếu cần)**
```powershell
cd backend
# Xóa toàn bộ build artifacts
Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
cargo build
```

#### 5. **Kiểm tra Antivirus/Windows Defender**
- Tạm thời disable Windows Defender real-time protection
- Hoặc thêm `backend\target` vào exclusion list

#### 6. **Chạy PowerShell/CMD với quyền Administrator**
```powershell
# Right-click PowerShell -> Run as Administrator
cd D:\project\rust-system\backend
cargo build
```

#### 7. **Sử dụng cargo clean trước khi build**
```powershell
cd backend
cargo clean
cargo build
```

### 🎯 Giải pháp nhanh nhất:

```powershell
# 1. Kill process nếu đang chạy
Get-Process | Where-Object {$_.ProcessName -like "*crm-backend*"} | Stop-Process -Force

# 2. Xóa file .exe
Remove-Item -Path "backend\target\debug\crm-backend.exe" -Force -ErrorAction SilentlyContinue

# 3. Build lại
cd backend
cargo build
```

### 📝 Lưu ý:
- **133 warnings** là bình thường - đó là các `#[allow(dead_code)]` warnings cho code được dùng gián tiếp
- Lỗi "Access is denied" KHÔNG phải lỗi code, chỉ là file đang bị lock
- Nếu vẫn không được, restart máy tính

