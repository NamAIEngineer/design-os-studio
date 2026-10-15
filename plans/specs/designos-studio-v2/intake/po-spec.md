---
id: po-spec
type: intake
title: "PO Intake — DESIGN:OS Studio Agentic Product OS"
status: Approved
updated: 2026-10-15
owner_role: Product Owner
---

## 1. Bối cảnh và Bài toán (Problem Statement)

Các nhóm thiết kế và phát triển sản phẩm AI-native đang bị phân mảnh giữa 3 công cụ tách rời:
1. **Figma Canvas** để vẽ UI và lưu file `.fig` nhị phân.
2. **Spec Driven Reader (`product-os-spec-driven`)** để quản lý PRD, User Stories, Acceptance Criteria, UI Contract và Gate duyệt.
3. **CLI Agent Terminal (`claude` / `codex` / `gemini`)** để viết code, sinh giao diện và chạy kiểm thử.

**DESIGN:OS Studio** hợp nhất cả 3 bề mặt trong một ứng dụng Rust/egui tốc độ 120fps duy nhất: mở/lưu trực tiếp `.fig` v106 lossless, đọc và duyệt gói Spec Format v2 ngay trong không gian làm việc, và điều khiển Agent qua PTY Terminal tích hợp.

## 2. Mục tiêu đầu ra (Desired Outcomes)

- **1:1 Figma Binary & CSS Grid**: Mở và ghi file `.fig` (Kiwi container v106) bảo toàn 100% unknown fields, hỗ trợ Auto-Layout 2D CSS Grid (`LayoutDirection::Grid`) với track sizing `fr` / `px` / `auto` và cell spanning.
- **Native Spec Driven v2 Reader**: Đọc trực tiếp gói `spec.manifest.json` + `config/pipeline.yaml` + `prd/PRD.md` + `us/US-*.md` + `ux/ui-contract.md` + `review/gates/*.md` theo 5 Role Views chuẩn (`PO View`, `Dev View`, `QA & Coverage`, `Wireframes & UI Contract`, `Gates & Canvas`) kèm nút **Copy cho AI** định dạng chuẩn trích dẫn dòng nguồn.
- **Embedded PTY Agent Terminal**: Khung terminal thực thi trực tiếp `claude`, `codex`, `gemini` hoặc `$SHELL` và cầu nối hai chiều với Canvas & Spec Reader.
