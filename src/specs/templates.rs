//! Template engine for spec documents
//!
//! This module provides templates for generating initial content
//! for requirements, design, and implementation documents.

use super::SpecDocumentType;
use chrono::Local;
use std::collections::HashMap;

/// Template engine for generating spec documents
pub struct TemplateEngine {
    /// Template variables
    variables: HashMap<String, String>,
}

impl TemplateEngine {
    /// Create a new template engine
    #[must_use]
    pub fn new() -> Self {
        let mut variables = HashMap::new();
        variables.insert(
            "date".to_string(),
            Local::now().format("%Y-%m-%d").to_string(),
        );

        Self { variables }
    }

    /// Set a template variable
    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Generate document from template
    #[must_use]
    pub fn generate(&self, template: &SpecTemplate) -> String {
        let mut content = template.content();

        // Replace variables
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{key}}}}}");
            content = content.replace(&placeholder, value);
        }

        content
    }
}

/// Specification document template
pub enum SpecTemplate {
    /// Requirements definition template
    Requirements { title: String, description: String },

    /// Technical design template
    Design {
        title: String,
        requirements_summary: String,
    },

    /// Implementation plan template
    Tasks {
        title: String,
        design_summary: String,
    },
}

impl SpecTemplate {
    /// Create a template for a document type
    #[must_use]
    pub fn for_document_type(
        doc_type: SpecDocumentType,
        title: String,
        context: Option<String>,
    ) -> Self {
        match doc_type {
            SpecDocumentType::Requirements => Self::Requirements {
                title,
                description: context.unwrap_or_default(),
            },
            SpecDocumentType::Design => Self::Design {
                title,
                requirements_summary: context.unwrap_or_default(),
            },
            SpecDocumentType::Tasks => Self::Tasks {
                title,
                design_summary: context.unwrap_or_default(),
            },
        }
    }

    /// Get template content
    #[must_use]
    pub fn content(&self) -> String {
        match self {
            Self::Requirements { title, description } => {
                format!(
                    r"# 要件定義書 / Requirements Definition

**タイトル / Title**: {title}
**作成日 / Date**: {{{{date}}}}
**バージョン / Version**: 0.1.0

## 1. 概要 / Overview

{description}

## 2. 背景と目的 / Background and Purpose

### 2.1 背景 / Background
<!-- プロジェクトの背景、なぜこの機能が必要なのかを記述 -->

### 2.2 目的 / Purpose
<!-- この仕様で達成したい具体的な目標を記述 -->

### 2.3 成功基準 / Success Criteria
<!-- 成功と判断できる具体的な基準を記述 -->

## 3. スコープ / Scope

### 3.1 対象範囲 / In Scope
<!-- この仕様に含まれる機能や要件 -->

### 3.2 対象外 / Out of Scope
<!-- この仕様に含まれない機能や要件 -->

## 4. 機能要件 / Functional Requirements

### 4.1 ユーザーストーリー / User Stories
<!-- As a [role], I want [feature] so that [benefit] 形式で記述 -->

### 4.2 機能一覧 / Feature List
<!-- 実装すべき機能のリスト -->

### 4.3 画面/インターフェース要件 / UI/Interface Requirements
<!-- ユーザーインターフェースやAPIインターフェースの要件 -->

## 5. 非機能要件 / Non-Functional Requirements

### 5.1 パフォーマンス要件 / Performance Requirements
<!-- 応答時間、処理速度、同時接続数などの要件 -->

### 5.2 セキュリティ要件 / Security Requirements
<!-- 認証、認可、データ保護などの要件 -->

### 5.3 信頼性要件 / Reliability Requirements
<!-- 可用性、エラー処理、データ整合性などの要件 -->

### 5.4 保守性要件 / Maintainability Requirements
<!-- コードの保守性、拡張性、ドキュメントなどの要件 -->

## 6. 制約事項 / Constraints

### 6.1 技術的制約 / Technical Constraints
<!-- 使用すべき技術、プラットフォーム、言語などの制約 -->

### 6.2 リソース制約 / Resource Constraints
<!-- 時間、予算、人員などの制約 -->

## 7. 前提条件 / Assumptions
<!-- この要件定義で前提としている条件 -->

## 8. リスク / Risks
<!-- 想定されるリスクとその対策 -->

## 9. 用語集 / Glossary
<!-- プロジェクト固有の用語の定義 -->

## 10. 参考資料 / References
<!-- 参考にした資料やドキュメントのリンク -->
"
                )
            },

            Self::Design {
                title,
                requirements_summary,
            } => {
                format!(
                    r"# 技術設計書 / Technical Design Document

**タイトル / Title**: {title}
**作成日 / Date**: {{{{date}}}}
**バージョン / Version**: 0.1.0

## 1. 概要 / Overview

### 1.1 要件サマリー / Requirements Summary
{requirements_summary}

### 1.2 設計方針 / Design Principles
<!-- この設計で重視する原則や方針 -->

## 2. アーキテクチャ / Architecture

### 2.1 全体構成図 / System Architecture
<!-- システム全体のアーキテクチャ図 -->
```
[Component A] --> [Component B]
      |               |
      v               v
[Component C] --> [Database]
```

### 2.2 コンポーネント設計 / Component Design
<!-- 各コンポーネントの責務と相互作用 -->

### 2.3 データフロー / Data Flow
<!-- データの流れと処理の順序 -->

## 3. 詳細設計 / Detailed Design

### 3.1 モジュール構成 / Module Structure
```
src/
├── module_a/
│   ├── mod.rs
│   └── ...
├── module_b/
│   ├── mod.rs
│   └── ...
└── ...
```

### 3.2 インターフェース定義 / Interface Definition
<!-- API、関数、クラスなどのインターフェース -->

### 3.3 データモデル / Data Model
<!-- データ構造、スキーマ、エンティティの定義 -->

## 4. 実装詳細 / Implementation Details

### 4.1 主要アルゴリズム / Key Algorithms
<!-- 重要なアルゴリズムやロジックの説明 -->

### 4.2 エラー処理 / Error Handling
<!-- エラーの種類と処理方法 -->

### 4.3 ロギングとモニタリング / Logging and Monitoring
<!-- ログ出力とモニタリングの設計 -->

## 5. セキュリティ設計 / Security Design

### 5.1 認証・認可 / Authentication & Authorization
<!-- 認証と認可の仕組み -->

### 5.2 データ保護 / Data Protection
<!-- データの暗号化、アクセス制御など -->

## 6. パフォーマンス設計 / Performance Design

### 6.1 最適化戦略 / Optimization Strategy
<!-- パフォーマンス向上のための設計上の工夫 -->

### 6.2 キャッシング / Caching
<!-- キャッシュの使用方法と戦略 -->

## 7. 拡張性 / Extensibility

### 7.1 プラグインシステム / Plugin System
<!-- 拡張ポイントとプラグインの仕組み -->

### 7.2 将来の拡張計画 / Future Extensions
<!-- 将来的に追加可能な機能や拡張 -->

## 8. テスト戦略 / Testing Strategy

### 8.1 単体テスト / Unit Testing
<!-- 単体テストの方針とカバレッジ目標 -->

### 8.2 統合テスト / Integration Testing
<!-- 統合テストの方針と重点項目 -->

## 9. デプロイメント / Deployment

### 9.1 デプロイ構成 / Deployment Configuration
<!-- デプロイ環境と構成 -->

### 9.2 設定管理 / Configuration Management
<!-- 環境ごとの設定管理方法 -->

## 10. 技術的決定事項 / Technical Decisions

### 10.1 技術選定 / Technology Choices
<!-- 使用する技術とその選定理由 -->

### 10.2 トレードオフ / Trade-offs
<!-- 設計上のトレードオフと判断理由 -->
"
                )
            },

            Self::Tasks {
                title,
                design_summary,
            } => {
                format!(
                    r"# 実装計画書 / Implementation Plan

**タイトル / Title**: {title}
**作成日 / Date**: {{{{date}}}}
**バージョン / Version**: 0.1.0

## 1. 概要 / Overview

### 1.1 設計サマリー / Design Summary
{design_summary}

### 1.2 実装方針 / Implementation Strategy
<!-- 実装の進め方と優先順位 -->

## 2. マイルストーン / Milestones

### Phase 1: 基盤実装 / Foundation (推定: X日)
- [ ] 開発環境のセットアップ
- [ ] 基本的なプロジェクト構造の作成
- [ ] コア機能の骨組み実装

### Phase 2: 機能実装 / Feature Implementation (推定: Y日)
- [ ] 主要機能の実装
- [ ] UI/APIの実装
- [ ] データ永続化の実装

### Phase 3: 品質向上 / Quality Enhancement (推定: Z日)
- [ ] テストの実装
- [ ] ドキュメントの作成
- [ ] パフォーマンス最適化

## 3. タスク詳細 / Detailed Tasks

### 3.1 セットアップタスク / Setup Tasks
| タスク | 説明 | 見積もり | 担当者 | 状態 |
|--------|------|----------|--------|------|
| T001 | プロジェクト初期化 | 1h | - | Todo |
| T002 | 依存関係のインストール | 0.5h | - | Todo |

### 3.2 開発タスク / Development Tasks
| タスク | 説明 | 見積もり | 前提タスク | 状態 |
|--------|------|----------|------------|------|
| T101 | モジュールAの実装 | 4h | T001 | Todo |
| T102 | モジュールBの実装 | 6h | T001 | Todo |
| T103 | モジュール統合 | 2h | T101, T102 | Todo |

### 3.3 テストタスク / Testing Tasks
| タスク | 説明 | 見積もり | 前提タスク | 状態 |
|--------|------|----------|------------|------|
| T201 | 単体テスト作成 | 3h | T101, T102 | Todo |
| T202 | 統合テスト作成 | 4h | T103 | Todo |

### 3.4 ドキュメントタスク / Documentation Tasks
| タスク | 説明 | 見積もり | 前提タスク | 状態 |
|--------|------|----------|------------|------|
| T301 | APIドキュメント作成 | 2h | T103 | Todo |
| T302 | ユーザーガイド作成 | 3h | T103 | Todo |

## 4. リスクと対策 / Risks and Mitigations

### 4.1 技術的リスク / Technical Risks
| リスク | 影響度 | 発生確率 | 対策 |
|--------|--------|----------|------|
| 外部APIの仕様変更 | 高 | 中 | APIバージョンの固定、モック作成 |

### 4.2 スケジュールリスク / Schedule Risks
| リスク | 影響度 | 発生確率 | 対策 |
|--------|--------|----------|------|
| 見積もりの甘さ | 中 | 高 | バッファ時間の確保 |

## 5. 依存関係 / Dependencies

### 5.1 外部依存 / External Dependencies
- ライブラリA (バージョン x.y.z)
- APIサービスB

### 5.2 内部依存 / Internal Dependencies
- 既存モジュールC
- 共通ライブラリD

## 6. 完了条件 / Definition of Done

### 6.1 コード完了条件 / Code Completion Criteria
- [ ] すべての機能が実装されている
- [ ] コードレビューが完了している
- [ ] リファクタリングが完了している

### 6.2 品質完了条件 / Quality Completion Criteria
- [ ] すべてのテストがパスしている
- [ ] テストカバレッジが80%以上
- [ ] パフォーマンス基準を満たしている

### 6.3 ドキュメント完了条件 / Documentation Completion Criteria
- [ ] APIドキュメントが完成している
- [ ] ユーザーガイドが完成している
- [ ] コメントが適切に記載されている

## 7. 見積もりサマリー / Estimation Summary

- **総見積もり時間 / Total Estimated Time**: XX時間
- **バッファ / Buffer**: XX時間 (20%)
- **推定完了日 / Estimated Completion Date**: YYYY-MM-DD

## 8. 備考 / Notes
<!-- その他の注意事項や申し送り事項 -->
"
                )
            },
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_variables() {
        let mut engine = TemplateEngine::new();
        engine.set_variable("project", "TestProject");

        let template = SpecTemplate::Requirements {
            title: "Test {{project}}".to_string(),
            description: "Description for {{project}}".to_string(),
        };

        let content = engine.generate(&template);
        assert!(content.contains("Test TestProject"));
        assert!(content.contains("Description for TestProject"));
    }
}
