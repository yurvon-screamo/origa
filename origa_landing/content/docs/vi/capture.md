---
title: "Chụp nhận diện: OCR và nhận dạng giọng nói trong Origa"
slug: /docs/capture
locale: vi
meta_title: "Chụp nhận diện trong Origa — OCR và nhận dạng giọng nói trên máy"
meta_description: "OCR và nhận dạng giọng nói trên máy của Origa hoạt động thế nào, khi nào mô hình được tải, định dạng hỗ trợ, và giới hạn đã biết. Mọi xử lý chạy cục bộ."
target_keywords: ["ứng dụng ocr tiếng nhật", "nhận dạng giọng nói tiếng nhật", "stt tiếng nhật ngoại tuyến", "nhận dạng văn bản tiếng nhật", "ocr học tiếng nhật"]
lastmod: 2026-07-23
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# Chụp nhận diện: OCR và nhận dạng giọng nói

Origa đọc được tiếng Nhật từ hình ảnh và chép lại từ âm thanh. Cả hai tính năng chạy trên máy của bạn. Quá trình xử lý không bao giờ gửi ảnh hay bản ghi của bạn lên máy chủ. Trang này gồm cách dùng và giới hạn của chúng.

## Nhận dạng ký tự quang học (OCR)

OCR trích văn bản tiếng Nhật từ hình ảnh. Dùng khi bạn có ảnh thực đơn, trang sách, biển hiệu, ảnh chụp màn hình, hay nguồn trực quan nào khác của văn bản tiếng Nhật.

**Cách dùng:**

1. Mở màn hình từ và chọn thẻ **Hình ảnh**.
2. Kéo thả hoặc dán một ảnh, hoặc chọn tệp từ máy.
3. Origa chạy OCR và hiện văn bản nhận ra.
4. Xem lại các từ trích được, chọn phần bạn muốn, và thêm thành thẻ.

**Định dạng hỗ trợ:** PNG, JPEG, WebP. Có giới hạn kích thước tệp tối đa.

**Khi nào mô hình được tải:** Mô hình OCR không đi kèm trình cài đặt. Nó tải về ở lần đầu bạn dùng tính năng, kèm thanh tiến trình. Sau lần tải đầu, OCR chạy ngoại tuyến.

**OCR xử lý tốt:** văn bản in, ảnh chụp màn hình sạch, tiếng Nhật đánh máy.

**OCR gặp khó:** font trang trí đậm, ảnh tương phản thấp, chữ dọc đặt trong bố cục lạ, ký tự rất nhỏ. Chữ viết tay không được hỗ trợ.

## Nhận dạng giọng nói (STT)

STT chép tiếng Nhật nói từ một tệp âm thanh. Dùng khi bạn có bản ghi lời nói: một đoạn podcast, một câu trong phim, một ghi âm thoại.

**Cách dùng:**

1. Mở màn hình từ và chọn thẻ **Âm thanh**.
2. Tải lên một tệp âm thanh.
3. Origa chép lại trên máy và hiện văn bản nhận ra.
4. Xem lại và thêm các từ bạn muốn.

**Định dạng hỗ trợ:** chỉ WAV. Các định dạng phổ biến khác (MP3, M4A, OGG) hiện không nhận. Chuyển đổi tệp trước khi tải lên.

**Khi nào mô hình được tải:** Như OCR, mô hình STT không đi kèm. Nó tải về ở lần dùng đầu, rồi chạy ngoại tuyến.

**STT xử lý tốt:** âm thanh một người nói rõ ràng, tốc độ chậm đến vừa.

**STT gặp khó:** nhiều người nói chồng nhau, ồn nền nặng, nói rất nhanh, giọng vùng mạnh. Kết quả nhận dạng là gần đúng; xem lại bản ra trước khi thêm từ.

## Riêng tư

Cả OCR và STT chạy hoàn toàn trên máy của bạn. Hình ảnh hay âm thanh bạn đưa vào được xử lý cục bộ và không bao giờ được tải lên. Lưu lượng mạng duy nhất là lần tải mô hình một lần từ CDN của Origa.

## Khi nào dùng chụp nhận diện

Chụp nhận diện không phải cách chính để thêm từ vựng. Gõ nhanh hơn cho từ bạn đã biết. Chụp nhận diện tỏ sáng khi:

- Bạn đang đọc vật liệu in (sách, biển hiệu) và muốn nhặt từ lạ mà không phải gõ.
- Bạn có ảnh chụp màn hình từ trình đọc truyện hoặc phụ đề và muốn biến nó thành thẻ.
- Bạn nghe được một từ trong nội dung âm thanh và muốn tra nó mà không phải gõ những gì bạn nghe (chừng chừng).

## Liên quan

- [Từ vựng](/vi/docs/vocabulary)
- [Bắt đầu](/vi/docs/getting-started)
- [Giới hạn](/vi/docs/limitations)
