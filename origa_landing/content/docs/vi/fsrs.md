---
title: "Origa quyết định thế nào thứ sẽ hiển thị"
slug: /docs/fsrs
locale: vi
meta_title: "Origa quyết định thế nào thứ sẽ hiển thị — FSRS bằng lời thường"
meta_description: "Đường cong quên lãng, lặp lại ngắt quãng, và FSRS: vì sao Origa giới hạn thẻ mới mỗi ngày, vì sao hai nút đánh giá thay vì bốn, và lịch ôn tập đến từ đâu."
target_keywords: ["fsrs tiếng nhật", "lặp lại ngắt quãng tiếng nhật", "vì sao thẻ ghi nhớ hết"]
lastmod: 2026-08-19
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# Origa quyết định thế nào thứ sẽ hiển thị

Bạn trả lời một thẻ. Origa quyết định khi nào hiển thị lại. Trang này nói về cách quyết định đó được đưa ra: đường cong quên lãng là gì, lặp lại ngắt quãng là gì, và vì sao giới hạn thẻ mới mỗi ngày là một phần của phương pháp chứ không phải khuyết điểm.

## Đường cong quên lãng

Học một từ hôm nay, đến mai bạn gần như chắc chắn còn nhớ. Một tuần nữa thì mong manh. Một tháng nữa thì nhiều khả năng không. Ebbinghaus đo hình mẫu này từ thế kỷ 19, và dạng của nó được nhiều thí nghiệm xác nhận sau đó. Sự kiện chính là: trí nhớ suy giảm theo cách dự đoán được, nghĩa là khoảnh khắc "gần quên" có thể tính trước.

## Lặp lại ngắt quãng

Ý tưởng là không hiển thị thẻ mỗi ngày, mà đúng lúc bạn sắp quên nó. Một lượt ôn ở khoảnh khắc đó là tín hiệu mạnh nhất cho trí nhớ, và mỗi lần gợi nhớ thành công khiến khoảng cách kế dài hơn: một ngày, rồi một tuần, rồi một tháng. Nên thay vì lật đều cả bộ bài, bạn ôn ít và đúng hạn.

## FSRS

FSRS (Free Spaced Repetition Scheduler) là thuật toán lên lịch ôn tập hiện đại, tiêu chuẩn ngành cho ứng dụng SRS từ 2023, và mặc định trong Anki từ 2024. Với mỗi câu trả lời, nó cập nhật mô hình trí nhớ của bạn cho thẻ đó: bạn thuộc đến đâu và quên nhanh bao nhiêu. Khoảng cách đến lần hiển thị kế xuất phát từ mô hình. Origa dùng FSRS làm bộ lập lịch duy nhất: nó quyết định hôm nay hiển thị gì và một tháng nữa hiển thị gì.

## Vì sao hai nút thay vì bốn

Trong Anki bạn chọn một trong bốn nút sau mỗi câu trả lời: "lại", "khó", "tốt", "dễ". Origa hỏi đúng một điều: **bạn có biết thẻ đó hay không**. Đây là một quyết định có chủ đích, không phải giản lược để giản lược. Các mức trung gian trên thực tế bị dùng vô nghĩa: trong Anki "khó" là nút bị lạm dụng nhất, và bấm nó thay cho "lại" làm lệch tham số của mô hình. Với tiếng Nhật, câu trả lời phần nhiều là nhị phân: hoặc bạn gợi nhớ ra âm đọc và nghĩa, hoặc không. Ít thời gian chọn nút, nhiều thời gian cho ngôn ngữ.

## Vì sao có giới hạn thẻ mới mỗi ngày

Một mẻ thẻ mới lớn hôm nay biến thành nhiều lượt ôn hơn nhiều trong hai tuần kế, mà FSRS sẽ lên lịch từ chính những thẻ mới đó. Giới hạn thẻ mới giữ lượng ôn hằng ngày trong phạm vi bạn thực sự làm được, thay để nợ chất chồng. Một bộ bài quá tải quay lại thành tuyết lở: thẻ nhận "trả trước" về lại thường hơn, và lượng lớn nhanh hơn tốc độ bạn theo kịp. Do đó mới có các mức tốc độ: bạn đổi được, nhưng mỗi mức kế có cái giá riêng của nó tính theo phút mỗi ngày.

## Tốc độ và cỡ bài học

Số thẻ mới mỗi ngày do tốc độ trong thiết lập hồ sơ quyết định: sáu mức, từ một nhóm thẻ mới nhỏ mỗi ngày đến một lượng dồn dập. Thẻ mới phát theo nhóm đủ bảy (nhóm làm quen, xem [cách bài học hoạt động](/vi/docs/lesson)), nên mỗi tốc độ là một số nguyên nhóm mỗi ngày.

Một bài học được dựng từ thẻ mới trong ngày và các lượt ôn đã đến hạn, tối đa 22 thẻ. Khi cả thẻ mới lẫn lượt ôn đều cạn, Origa nói "Không còn thẻ để học." Đó không phải lỗi và không phải giới hạn tài khoản: mai FSRS sẽ lên lịch ôn mới, và bài học lại xuất hiện. Nội dung bạn cứ sai mãi sẽ quay lại trước: một thẻ bạn trượt đi trượt lại sẽ xuất hiện trong những bài học gần nhất cho tới khi ghi nhớ được — ôn lại cái đã xem luôn ưu tiên hơn nội dung mới.

## Liên quan

- [Bài học](/vi/docs/lesson)
- [Bắt đầu](/vi/docs/getting-started)
