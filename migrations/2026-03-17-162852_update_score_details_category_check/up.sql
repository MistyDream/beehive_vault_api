ALTER TABLE score_details DROP CONSTRAINT IF EXISTS score_details_category_check;
ALTER TABLE score_details ADD CONSTRAINT score_details_category_check
    CHECK (category IN ('valuation', 'profitability', 'growth', 'financial_health', 'investor_return'));
