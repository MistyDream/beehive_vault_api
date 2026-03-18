DELETE FROM metrics_catalog WHERE key IN (
    'pettm', 'peg', 'ev2ebitda', 'ps', 'pb', 'pfcf',
    'rvn_growth_3y', 'ebitda_growth_3y', 'cashflow_growth_3y', 'earning_growth_3y', 'book_growth_3y',
    'roic', 'oprt_margain', 'FCFmargin', 'roa', 'net_margain',
    'interest_coverage', 'debt2ebitda', 'quick_ratio', 'cash2debt', 'equity2asset',
    'ForwardDividendYield', 'buyback_yield', 'dividend_growth_3y', 'payout', 'shareholder_yield'
);

-- Restore original category CHECK constraint
ALTER TABLE metrics_catalog DROP CONSTRAINT IF EXISTS metrics_catalog_category_check;
ALTER TABLE metrics_catalog ADD CONSTRAINT metrics_catalog_category_check CHECK (
    category IN ('valuation', 'profitability', 'growth', 'solidity', 'shareholder_return', 'tactical')
);
