# Origa　「オリガ」

[🇬🇧 English](./README.md) | [🇷🇺 Русский](./README.ru.md) | [🇰🇷 한국어](./README.ko.md) | 🇻🇳 Tiếng Việt

---

**Học tiếng Nhật không qua trung gian tiếng Anh.**

オリガ là ứng dụng học tiếng Nhật toàn diện và ôn luyện chuyên sâu cho kỳ thi JLPT.

Thuật toán lặp lại ngắt quãng (FSRS), OCR tích hợp, nhận dạng văn bản và giọng nói — toàn bộ xử lý AI chạy cục bộ trên thiết bị của bạn. Chỉ cần kết nối internet cho lần đăng nhập đầu tiên và tải nội dung ban đầu.

![Bảng điều khiển chính của Origa](origa_landing/public/images/en.hero.png)

---

## 🎯 Nguyên tắc

* **Học từ nội dung của bạn** — bạn chọn những gì muốn học. Ứng dụng thích ứng với những gì bạn đã biết và những gì bạn đang đọc, xem hoặc nghe ngay lúc này.
* **Thuật toán thông minh** — hệ thống lặp lại ngắt quãng FSRS (giống Anki) tối ưu hóa khoảng cách ôn tập cho từng từ.
* **Quyền riêng tư** — mọi mô hình AI chạy cục bộ trên thiết bị. Ảnh, âm thanh và hoạt động học được xử lý ngay trên máy, không tải lên dịch vụ bên ngoài.
* **Ưu tiên ngoại tuyến** — đầy đủ chức năng không cần internet sau lần thiết lập đầu tiên.
* **Đa nền tảng** — Web, Windows, Linux, macOS, Android.
* **Học bằng tiếng mẹ đẻ** — giao diện và từ điển bằng tiếng Nga, tiếng Anh, tiếng Hàn và tiếng Việt (tiếng Indonesia và tiếng Tây Ban Nha trong kế hoạch).
* **Phân tích JLPT** — theo dõi cấp độ hiện tại và dự báo tiến độ học.

![Tổng quan về Origa](origa_landing/public/images/en.all_in_one.png)

---

## ✨ Tính năng

### Từ vựng

* Từ điển tích hợp bằng tiếng mẹ đẻ của bạn.
* Tạo thẻ siêu nhanh — chỉ cần gõ một từ hoặc câu tiếng Nhật.
* Tự động nhận dạng và trích xuất từ vựng từ văn bản, ảnh và âm thanh.
* Nhập sẵn bộ từ từ các ứng dụng phổ biến và giáo trình kinh điển.
* Ngân hàng âm thanh tích hợp với phát âm chuẩn (dựa trên NHK và các nguồn đáng tin cậy khác).

### Hán tự

* Tự động tạo furigana cho toàn bộ nội dung học.
* Ẩn thông minh: hán tự đã học không còn hiển thị furigana để rèn kỹ năng đọc.
* Luyện viết hán tự với thứ tự nét chuẩn.
* Từ điển hán tự tích hợp ánh xạ chặt chẽ theo cấp độ JLPT N5–N1.
* Bài kiểm tra đọc hán tự tương tác.

### Ngữ pháp

* Tài liệu ngữ pháp tích hợp có cấu trúc, bao trùm cấp độ JLPT N5–N1.
* Học theo ngữ cảnh: quy tắc ngữ pháp được giải thích bằng những từ bạn *đã* học.
* Bài kiểm tra thực hành để củng cố các mẫu câu ngữ pháp.

### Câu và nghe

* Kho hơn 200.000 câu từ nội dung tiếng Nhật bản ngữ (visual novel, anime) với lồng tiếng gốc.
* Tự động chọn câu cho bài học dựa trên vốn từ hiện tại (cách tiếp cận N+1).
* Luyện nghe và nhận hiểu lời nói.
* Đắm mình vào tiếng Nhật hội thoại và đời thường.

![Giao diện học của Origa](origa_landing/public/images/en.learn.png)

---

## 📥 Tải xuống

| Nền tảng | Trạng thái | Định dạng |
| :--- | :--- | :--- |
| **Windows** | ✅ Sẵn sàng | `.exe`, `.msi` |
| **Linux** | ✅ Sẵn sàng | `.deb`, `.AppImage`, `.rpm` |
| **macOS** | ✅ Sẵn sàng | `.dmg`, `.app` |
| **Android** | ✅ Sẵn sàng | `.apk` |

> Tất cả các phiên bản đều hỗ trợ chế độ ngoại tuyến.

Phiên bản web cũng có sẵn.

---

## 🌍 Ngôn ngữ giao diện

| Ngôn ngữ | Trạng thái |
| :--- | :--- |
| **Tiếng Việt, tiếng Anh, tiếng Nga, tiếng Hàn** | ✅ Khả dụng |
| **Tiếng Indonesia, tiếng Tây Ban Nha** | 📋 Trong kế hoạch |

---

## 🏗️ Kiến trúc và công nghệ

Dự án được xây dựng trên nền tảng công nghệ hiện đại, mang lại hiệu năng của ứng dụng native cùng sự linh hoạt của giao diện web.

* **Lõi và backend**: **Rust** — an toàn và xử lý dữ liệu tốc độ cao.
* **Lớp desktop**: **Tauri v2** — ứng dụng native cho Windows, macOS và Linux.
* **Frontend**: **Leptos** — framework UI phản ứng trên Rust (WebAssembly) cho phản hồi giao diện tức thì.
* **Di động**: bản build **Android** native qua Tauri Mobile.

---

## 🚀 Lộ trình

* Nền tảng di động: phát hành iOS.
* Tính năng xã hội: thi đua giữa người dùng.
* Loại bài tập mới: đọc văn bản, manga, câu theo ngữ cảnh, âm thanh, video.
* Bản địa hóa: thêm tiếng Indonesia và tiếng Tây Ban Nha.

---

## 📊 So sánh với các ứng dụng khác

*Origa so với các công cụ phổ biến và cách dùng chúng cùng nhau.*

### Anki

Ứng dụng lặp lại ngắt quãng mạnh mẽ và linh hoạt cho mọi loại tài liệu.

* **Khi dùng Anki:** bạn cần tự do tùy biến tuyệt đối, tự làm thẻ HTML/CSS, hoặc học nhiều môn ngoài tiếng Nhật.
* **Lợi thế của Origa:** tiết kiệm thời gian tạo thẻ, nạp nội dung của bạn nhanh chóng, từ vựng + hán tự + ngữ pháp trong cùng một hệ sinh thái.

### ReWord

Ứng dụng ghi nhớ từ vựng bằng flashcard.

* **Khi dùng ReWord:** khởi động nhanh và ghi nhớ máy móc các danh sách từ cơ bản không ngữ cảnh.
* **Lợi thế của Origa:** từ vựng gắn với nội dung, ngữ pháp và âm thanh bản ngữ của bạn — hiểu ngôn ngữ sâu hơn.

### Bunpro

Trình luyện ngữ pháp chuyên biệt (Grammar SRS).

* **Khi dùng Bunpro:** bạn muốn tập trung hoàn toàn vào luyện các quy tắc ngữ pháp.
* **Lợi thế của Origa:** ví dụ ngữ pháp được xây từ *những từ bạn đã học* — từ vựng và ngữ pháp vận hành như một khối thống nhất.

### WaniKani

Dịch vụ phổ biến để học hán tự và từ vựng bằng phép ghi nhớ (mnemonics).

* **Khi dùng WaniKani:** cách học hán tự theo radical với trật tự cố định từ đầu phù hợp với bạn.
* **Lợi thế của Origa:** bạn học đúng những hán tự và từ gặp hôm nay — trong manga, một bài báo hay ở chỗ làm.

### Duolingo

Ứng dụng hấp dẫn, dần dần đưa bạn vào môi trường ngôn ngữ.

* **Kết hợp với Origa:** dùng Duolingo để khởi đầu nhẹ nhàng và có nền tảng từ vựng, hán tự ban đầu. Origa sẽ củng cố nội dung đó, lấp khoảng trống ngữ pháp và chuyển kiến thức từ hình thức game sang sử dụng thực tế.

### Migii

Trình mô phỏng ôn thi JLPT nghiêm túc.

* **Kết hợp với Origa:** dùng Migii để luyện giải đề thi tính giờ. Origa là nền tảng giúp bạn thu thập và củng cố toàn bộ kiến thức cần cho kỳ thi.

---

## 📄 Giấy phép

Dự án được phân phối theo BSL 1.1 (Business Source License 1.1).

Điều đó có nghĩa là:

* ✅ Bạn có thể tự do sử dụng, nghiên cứu và sửa đổi mã cho mục đích cá nhân.
* ✅ Bạn có thể tự build ứng dụng cho mình.
* ❌ Cấm sử dụng mã để cung cấp dịch vụ thương mại công khai (SaaS) hoặc bán lại nếu chưa có thỏa thuận rõ ràng từ tác giả.

Sau một thời gian nhất định (hoặc khi điều kiện được đáp ứng), giấy phép có thể chuyển sang dạng mở hơn (ví dụ: Apache 2.0 hoặc MIT).

Xem chi tiết trong tệp LICENSE. Bản dịch trên chỉ mang tính tham khảo; giá trị pháp lý thuộc về bản gốc trong LICENSE.

## 📬 Liên hệ

* GitHub Issues: báo lỗi và đề xuất tính năng.
* Discussions: câu hỏi chung và thảo luận ý tưởng.
* Được tạo bằng tình yêu với tiếng Nhật và công nghệ. 🇯🇵💻
