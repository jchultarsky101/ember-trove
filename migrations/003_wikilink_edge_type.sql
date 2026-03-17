-- Add wiki_link edge type for automatic edges created from [[wiki-link]] syntax.
-- PostgreSQL 16 supports ALTER TYPE ... ADD VALUE inside a transaction.
ALTER TYPE edge_type ADD VALUE IF NOT EXISTS 'wiki_link';
