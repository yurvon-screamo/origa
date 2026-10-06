---
title: "Các giới hạn đã biết của Origa"
slug: /docs/limitations
locale: vi
meta_title: "Các giới hạn đã biết của Origa — Ranh giới nói thẳng"
meta_description: "Origa còn thiếu ở đâu: STT chỉ nhận WAV, OCR chưa chính xác trên chữ trang trí, không có chế độ khách, đồng bộ cần mạng, và các ranh giới hiện tại khác."
target_keywords: ["giới hạn origa", "giới hạn ứng dụng học tiếng nhật", "giới hạn ocr tiếng nhật", "stt tiếng nhật chỉ wav"]
lastmod: 2026-09-23
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# Các giới hạn đã biết

Trang này liệt kê ranh giới hiện tại của Origa. Giới hạn không phải lỗi. Chúng mô tả điều ứng dụng chưa làm hôm nay, để bạn quyết định nó có vừa quy trình của bạn không.

## Chuyển âm thanh chỉ nhận WAV

Tính năng nhận dạng giọng nói nhận tệp `.wav`. Các định dạng phổ biến khác (MP3, M4A, OGG) hiện chưa hỗ trợ. Chuyển đổi âm thanh trước khi tải lên. Xem toàn bộ luồng trong [Chụp nhận diện](/vi/docs/capture).

## OCR là gần đúng trên đầu vào khó

Nhận dạng ký tự quang học hoạt động tốt trên văn bản in và ảnh chụp màn hình sạch. Nó gặp khó với font trang trí đậm, ảnh tương phản thấp hoặc ngược sáng, chữ dọc trong bố cục lạ, ký tự rất nhỏ, và chữ viết tay (không hỗ trợ).

Luôn xem lại các từ nhận ra trước khi thêm thành thẻ.

## Không có chế độ khách

Bạn phải đăng nhập để dùng Origa. Tài khoản bắt buộc vì tiến trình được đồng bộ giữa các thiết bị. Không có chế độ chỉ-ngoại-tuyến hay ẩn danh.

## Lần chạy đầu cần mạng

Lần đầu đăng nhập, Origa tải từ điển và nội dung. Lần đầu dùng OCR hoặc nhận dạng giọng nói, mô hình tương ứng được tải. Sau những lần tải một-lần này, ứng dụng chạy ngoại tuyến. Xem bảng phân tích đầy đủ trong [Bắt đầu](/vi/docs/getting-started).

## Đồng bộ cần mạng

Tiến trình của bạn lưu cục bộ và đồng bộ lên máy chủ. Nếu bạn ngoại tuyến, tiến trình xếp hàng đợi và đồng bộ khi bạn kết nối lại. Bạn không thể đẩy tiến trình sang thiết bị khác mà không có kết nối mạng.

## Luyện viết dựa trên animation

Tính năng viết hán tự trình bày thứ tự nét đúng dưới dạng animation để bạn viết theo. Nó hiện chưa nhận dạng viết tay tự do.

## Đánh giá hai nút

Đánh giá sau mỗi thẻ là nhị phân: **Không biết** hoặc **Biết** trong lượt ôn thường, **Nhớ** hoặc **Không nhớ** lúc luyện làm quen. Không có lựa chọn trung gian (không có mức "khó" hay "dễ"). Bộ lập lịch FSRS dùng hai tín hiệu này để đặt khoảng cách.

## Từ đếm: chưa có trang danh sách, lô đầu tiên dài

Từ đếm (本, 枚, 匹 …) được luyện như thẻ, với một trang từ đếm riêng liệt kê toàn bộ và thẻ chi tiết của từng hậu tố. Ôn tập là kiểu "biết / không biết" cổ điển — bảng đọc đầy đủ nằm ở slide làm quen và trang từ đếm, không nằm trong lần ôn. Đánh dấu "đã biết" bỏ qua cả luyện nghĩa lẫn kết hợp. Tài khoản tạo trước khi có từ đếm sẽ nhận chúng qua một migration một lần ở lần chạy đầu tiên sau khi cập nhật.

## Văn bản pháp lý chỉ có tiếng Anh và tiếng Nga

Giao diện và nội dung học (từ vựng, câu, ngữ pháp, hán tự) có tiếng Việt, tiếng Anh, tiếng Nga và tiếng Hàn. Các trang tài liệu này cũng có cả bốn ngôn ngữ. Văn bản pháp lý (chính sách riêng tư, điều khoản sử dụng) hiện chỉ có tiếng Anh và tiếng Nga.

## Liên quan

- [Chụp nhận diện (OCR và giọng nói)](/vi/docs/capture)
- [Bắt đầu](/vi/docs/getting-started)
