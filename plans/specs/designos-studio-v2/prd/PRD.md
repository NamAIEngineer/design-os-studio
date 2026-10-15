---
id: prd-designos-v2
type: prd
title: "PRD — DESIGN:OS Studio: Figma 1:1, Spec Driven v2 Reader & Agent Terminal"
status: Approved
updated: 2026-10-15
primary: true
---

- Owner, revision, template version: po@designos.local · rev 2.0 · template v2

## 1. Problem và bối cảnh

Kỹ sư sản phẩm và nhà thiết kế hệ thống cần một môi trường làm việc hợp nhất nơi bản vẽ vector `.fig`, tài liệu đặc tả Spec Format v2 (`PRD`, `US`, `AC`, `UI Contract`, `Gates`) và tác tử lập trình AI (`claude`, `codex`) cùng chia sẻ một nguồn sự thật duy nhất.

## 4. Yêu cầu và quy tắc

### 4.1 Lõi Canvas Figma 1:1 & 2D CSS Grid

| ID | Requirement | Source | US |
|---|---|---|---|
| REQ-01 | Đọc và ghi trực tiếp định dạng nhị phân `.fig` (Kiwi v106) bảo toàn lossless các trường mở rộng và hỗ trợ bố cục Auto-Layout 2D CSS Grid (`fr`, `px`, `auto`, `col_span`, `row_span`). | contract BR-01 (D1) | US-01 |
| REQ-02 | Cho phép áp dụng trực tiếp bản thiết kế khai báo `.spec.json` (`NodeSpec`) lên Canvas hiện tại hoặc xuất Frame đang chọn ra `.spec.json` / `.fig` trong 1 cú nhấp. | contract BR-01, BR-05 | US-01 |

### 4.2 Bộ đọc PRODUCT:OS Spec Driven v2 & Role Views

| ID | Requirement | Source | US |
|---|---|---|---|
| REQ-03 | Tự động phát hiện và nạp cả gói Spec Format v2 (`spec.manifest.json`, `config/pipeline.yaml`, `prd/PRD.md`, `us/US-*.md`, `ux/ui-contract.md`, `review/gates/*.md`) lẫn gói legacy (`brainstorm.md`, `usecases.md`, `plan.md`). | contract BR-02, BR-03 (D2, D3) | US-02 |
| REQ-04 | Hiển thị 5 Role Views chuẩn (`PO View`, `Dev View (US & AC)`, `QA & Coverage`, `Wireframes & UI Contract`, `Gates & Canvas`) cùng nút `Copy cho AI` trích dẫn chính xác dòng nguồn. | contract BR-02, BR-04 (D4) | US-02 |

### 4.3 Embedded PTY Agent Terminal & Live Canvas Bridge

| ID | Requirement | Source | US |
|---|---|---|---|
| REQ-05 | Tích hợp cửa sổ dòng lệnh PTY (`claude`, `codex`, `gemini`, `designos`) ngay trong Studio và nhận trực tiếp nội dung `Copy cho AI` từ Spec Reader để thực thi tác vụ tự động. | contract BR-04, BR-05 (D5) | US-03 |
